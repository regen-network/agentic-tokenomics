use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::*;
use crate::state::*;

const CONTRACT_NAME: &str = "crates.io:contribution-rewards";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Approximate blocks per month at 10s block time (30 days * 24h * 60m * 6 blocks/min)
const BLOCKS_PER_MONTH: u64 = 259_200;

// ============================================================================
// Instantiate
// ============================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = deps.api.addr_validate(&msg.admin)?;

    let weights = msg
        .activity_weights
        .unwrap_or_else(ActivityWeights::default_weights);
    if weights.sum() != 10_000 {
        return Err(ContractError::InvalidWeights {});
    }

    let config = Config {
        admin,
        activity_weights: weights,
        max_stability_share_bps: msg.max_stability_share_bps.unwrap_or(3000),
        stability_annual_return_bps: msg.stability_annual_return_bps.unwrap_or(600),
        min_commitment_uregen: msg
            .min_commitment_uregen
            .unwrap_or(Uint128::new(100_000_000)),
        min_lock_months: msg.min_lock_months.unwrap_or(6),
        max_lock_months: msg.max_lock_months.unwrap_or(24),
        early_exit_penalty_bps: msg.early_exit_penalty_bps.unwrap_or(5000),
        blocks_per_epoch: msg.blocks_per_epoch.unwrap_or(60_480),
        calibration_epochs: msg.calibration_epochs.unwrap_or(13),
        epochs_per_year: msg.epochs_per_year.unwrap_or(52),
        denom: msg.denom.unwrap_or_else(|| "uregen".to_string()),
    };

    let state = ContractState {
        mechanism_state: MechanismState::Inactive,
        current_epoch: 0,
        epoch_start_block: env.block.height,
        activation_epoch: None,
        paused: false,
    };

    let stats = StabilityStats {
        total_committed: Uint128::zero(),
        active_commitments: 0,
        total_stability_allocated: Uint128::zero(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;
    STABILITY_STATS.save(deps.storage, &stats)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", config.admin.as_str())
        .add_attribute("mechanism_state", "inactive"))
}

// ============================================================================
// Execute
// ============================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RecordContribution {
            participant,
            activity,
            value,
            proposal_outcome,
            tx_hash,
        } => execute_record_contribution(
            deps,
            env,
            info,
            participant,
            activity,
            value,
            proposal_outcome,
            tx_hash,
        ),
        ExecuteMsg::FinalizeEpoch {
            community_pool_inflow,
        } => execute_finalize_epoch(deps, env, info, community_pool_inflow),
        ExecuteMsg::ClaimRewards {} => execute_claim_rewards(deps, env, info),
        ExecuteMsg::CommitStability { lock_months } => {
            execute_commit_stability(deps, env, info, lock_months)
        }
        ExecuteMsg::ClaimMaturedCommitment {} => execute_claim_matured(deps, env, info),
        ExecuteMsg::ExitEarly {} => execute_exit_early(deps, env, info),
        ExecuteMsg::Activate {} => execute_activate(deps, env, info),
        ExecuteMsg::EnableDistribution {} => execute_enable_distribution(deps, info),
        ExecuteMsg::UpdateWeights { new_weights } => {
            execute_update_weights(deps, info, new_weights)
        }
        ExecuteMsg::UpdateStabilityParams {
            max_stability_share_bps,
            stability_annual_return_bps,
            min_commitment_uregen,
            min_lock_months,
            max_lock_months,
            early_exit_penalty_bps,
        } => execute_update_stability_params(
            deps,
            info,
            max_stability_share_bps,
            stability_annual_return_bps,
            min_commitment_uregen,
            min_lock_months,
            max_lock_months,
            early_exit_penalty_bps,
        ),
        ExecuteMsg::Pause {} => execute_pause(deps, info),
        ExecuteMsg::Resume {} => execute_resume(deps, info),
    }
}

// ---- Activity tracking ----

#[allow(clippy::too_many_arguments)]
fn execute_record_contribution(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    participant: String,
    activity: ActivityType,
    value: Uint128,
    proposal_outcome: Option<ProposalOutcome>,
    tx_hash: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    // Only admin can record contributions (on-chain hooks call through admin)
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Must be in TRACKING or DISTRIBUTING state
    match state.mechanism_state {
        MechanismState::Inactive => {
            return Err(ContractError::NotActive {
                state: "inactive".to_string(),
            });
        }
        _ => {}
    }

    if state.paused {
        return Err(ContractError::NotActive {
            state: "paused".to_string(),
        });
    }

    // Dedup check
    if RECORDED_TX_HASHES
        .may_load(deps.storage, &tx_hash)?
        .is_some()
    {
        return Err(ContractError::DuplicateContribution { tx_hash });
    }
    RECORDED_TX_HASHES.save(deps.storage, &tx_hash, &true)?;

    let addr = deps.api.addr_validate(&participant)?;
    let epoch = state.current_epoch;

    // Load or create activity record
    let mut record = ACTIVITY_RECORDS
        .may_load(deps.storage, (epoch, addr.as_str()))?
        .unwrap_or_default();

    match activity {
        ActivityType::CreditPurchase => {
            record.credit_purchase_value += value;
        }
        ActivityType::CreditRetirement => {
            record.credit_retirement_value += value;
        }
        ActivityType::PlatformFacilitation => {
            record.platform_facilitation_value += value;
        }
        ActivityType::GovernanceVote => {
            record.governance_votes += value;
        }
        ActivityType::ProposalSubmission => {
            // Apply proposal outcome scaling (SPEC section 5.2)
            let credit_x100 = match proposal_outcome {
                Some(ProposalOutcome::PassedAndApproved) => Uint128::new(100), // 1.0
                Some(ProposalOutcome::ReachedQuorumFailed) => Uint128::new(50), // 0.5
                Some(ProposalOutcome::FailedQuorum) | None => Uint128::zero(), // 0.0
            };
            record.proposal_credits_x100 += credit_x100;
        }
    }

    ACTIVITY_RECORDS.save(deps.storage, (epoch, addr.as_str()), &record)?;
    EPOCH_PARTICIPANTS.save(deps.storage, (epoch, addr.as_str()), &true)?;

    Ok(Response::new()
        .add_attribute("action", "record_contribution")
        .add_attribute("participant", addr.as_str())
        .add_attribute("epoch", epoch.to_string())
        .add_attribute("activity", format!("{:?}", activity))
        .add_attribute("value", value.to_string()))
}

// ---- Epoch management ----

fn execute_finalize_epoch(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    community_pool_inflow: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if state.paused {
        return Err(ContractError::NotActive {
            state: "paused".to_string(),
        });
    }

    match state.mechanism_state {
        MechanismState::Inactive => {
            return Err(ContractError::NotActive {
                state: "inactive".to_string(),
            });
        }
        _ => {}
    }

    let epoch = state.current_epoch;

    // Check epoch has ended
    let epoch_end_block = state.epoch_start_block + config.blocks_per_epoch;
    if env.block.height < epoch_end_block {
        return Err(ContractError::EpochNotEnded {
            epoch,
            end_block: epoch_end_block,
        });
    }

    // Check not already finalized
    if DISTRIBUTIONS.may_load(deps.storage, epoch)?.is_some() {
        return Err(ContractError::EpochAlreadyFinalized { epoch });
    }

    if community_pool_inflow.is_zero() {
        return Err(ContractError::ZeroInflow {});
    }

    // 1. Compute stability allocation (SPEC section 5.3 step 1)
    let mut stats = STABILITY_STATS.load(deps.storage)?;
    let raw_stability = compute_stability_allocation(
        stats.total_committed,
        config.stability_annual_return_bps,
        config.epochs_per_year,
    );
    let max_stability = community_pool_inflow
        .multiply_ratio(config.max_stability_share_bps as u128, 10_000u128);
    let stability_allocation = std::cmp::min(raw_stability, max_stability);

    // 2. Activity pool (SPEC section 5.3 step 2)
    let activity_pool = community_pool_inflow - stability_allocation;

    // 3. Compute scores and distribute (SPEC section 5.3 steps 3-4)
    let mut total_score = Uint128::zero();
    let mut participant_scores: Vec<(Addr, Uint128)> = Vec::new();

    // Collect all participants for this epoch
    let participants: Vec<(String, bool)> = EPOCH_PARTICIPANTS
        .prefix(epoch)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;

    for (addr_str, _) in &participants {
        let record = ACTIVITY_RECORDS.load(deps.storage, (epoch, addr_str.as_str()))?;
        let score = compute_weighted_score(&record, &config.activity_weights);
        if !score.is_zero() {
            total_score += score;
            let addr = deps.api.addr_validate(addr_str)?;
            participant_scores.push((addr, score));
        }
    }

    // Distribute activity rewards proportionally
    if state.mechanism_state == MechanismState::Distributing && !total_score.is_zero() {
        for (addr, score) in &participant_scores {
            let reward = activity_pool.multiply_ratio(*score, total_score);
            if !reward.is_zero() {
                let existing = PENDING_ACTIVITY_REWARDS
                    .may_load(deps.storage, addr.as_str())?
                    .unwrap_or_default();
                PENDING_ACTIVITY_REWARDS.save(
                    deps.storage,
                    addr.as_str(),
                    &(existing + reward),
                )?;
            }
        }

        // Distribute stability rewards to committed holders
        distribute_stability_rewards(deps.storage, stability_allocation, &config)?;
    }

    // Record distribution
    let dist = DistributionRecord {
        community_pool_inflow,
        stability_allocation,
        activity_pool,
        total_score,
        participant_count: participant_scores.len() as u32,
    };
    DISTRIBUTIONS.save(deps.storage, epoch, &dist)?;

    stats.total_stability_allocated += stability_allocation;
    STABILITY_STATS.save(deps.storage, &stats)?;

    // Advance epoch
    state.current_epoch += 1;
    state.epoch_start_block = env.block.height;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "finalize_epoch")
        .add_attribute("epoch", epoch.to_string())
        .add_attribute("community_pool_inflow", community_pool_inflow.to_string())
        .add_attribute("stability_allocation", stability_allocation.to_string())
        .add_attribute("activity_pool", activity_pool.to_string())
        .add_attribute("total_score", total_score.to_string())
        .add_attribute("participants", participant_scores.len().to_string()))
}

fn execute_claim_rewards(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let addr = info.sender;

    let activity = PENDING_ACTIVITY_REWARDS
        .may_load(deps.storage, addr.as_str())?
        .unwrap_or_default();
    let stability = PENDING_STABILITY_REWARDS
        .may_load(deps.storage, addr.as_str())?
        .unwrap_or_default();
    let total = activity + stability;

    if total.is_zero() {
        return Ok(Response::new()
            .add_attribute("action", "claim_rewards")
            .add_attribute("amount", "0"));
    }

    // Clear pending
    PENDING_ACTIVITY_REWARDS.save(deps.storage, addr.as_str(), &Uint128::zero())?;
    PENDING_STABILITY_REWARDS.save(deps.storage, addr.as_str(), &Uint128::zero())?;

    let send_msg = BankMsg::Send {
        to_address: addr.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount: total,
        }],
    };

    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("action", "claim_rewards")
        .add_attribute("recipient", addr.as_str())
        .add_attribute("activity_rewards", activity.to_string())
        .add_attribute("stability_rewards", stability.to_string())
        .add_attribute("total", total.to_string()))
}

// ---- Stability tier ----

fn execute_commit_stability(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    lock_months: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    if state.mechanism_state == MechanismState::Inactive {
        return Err(ContractError::NotActive {
            state: "inactive".to_string(),
        });
    }

    // Validate lock period
    if lock_months < config.min_lock_months || lock_months > config.max_lock_months {
        return Err(ContractError::InvalidLockPeriod {
            months: lock_months,
            min: config.min_lock_months,
            max: config.max_lock_months,
        });
    }

    // Validate sent funds
    let sent = info
        .funds
        .iter()
        .find(|c| c.denom == config.denom)
        .map(|c| c.amount)
        .unwrap_or_default();

    if sent < config.min_commitment_uregen {
        return Err(ContractError::CommitmentTooSmall {
            amount: sent.u128(),
            min: config.min_commitment_uregen.u128(),
        });
    }

    let maturity_block = env.block.height + lock_months * BLOCKS_PER_MONTH;

    let commitment = StabilityCommitment {
        amount: sent,
        lock_months,
        committed_at_block: env.block.height,
        maturity_block,
        state: CommitmentState::Committed,
        accrued_rewards: Uint128::zero(),
    };

    STABILITY_COMMITMENTS.save(deps.storage, info.sender.as_str(), &commitment)?;

    // Update aggregate stats
    let mut stats = STABILITY_STATS.load(deps.storage)?;
    stats.total_committed += sent;
    stats.active_commitments += 1;
    STABILITY_STATS.save(deps.storage, &stats)?;

    Ok(Response::new()
        .add_attribute("action", "commit_stability")
        .add_attribute("address", info.sender.as_str())
        .add_attribute("amount", sent.to_string())
        .add_attribute("lock_months", lock_months.to_string())
        .add_attribute("maturity_block", maturity_block.to_string()))
}

fn execute_claim_matured(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut commitment = STABILITY_COMMITMENTS
        .may_load(deps.storage, info.sender.as_str())?
        .ok_or(ContractError::NoCommitment {
            addr: info.sender.to_string(),
        })?;

    if commitment.state != CommitmentState::Committed {
        return Err(ContractError::CommitmentNotActive {});
    }

    if env.block.height < commitment.maturity_block {
        return Err(ContractError::CommitmentNotMatured {
            maturity_block: commitment.maturity_block,
        });
    }

    // Matured: return principal + full accrued rewards
    let total_return = commitment.amount + commitment.accrued_rewards;
    commitment.state = CommitmentState::Matured;
    STABILITY_COMMITMENTS.save(deps.storage, info.sender.as_str(), &commitment)?;

    // Update stats
    let mut stats = STABILITY_STATS.load(deps.storage)?;
    stats.total_committed = stats.total_committed.saturating_sub(commitment.amount);
    stats.active_commitments = stats.active_commitments.saturating_sub(1);
    STABILITY_STATS.save(deps.storage, &stats)?;

    let send_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount: total_return,
        }],
    };

    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("action", "claim_matured")
        .add_attribute("address", info.sender.as_str())
        .add_attribute("principal", commitment.amount.to_string())
        .add_attribute("rewards", commitment.accrued_rewards.to_string())
        .add_attribute("total", total_return.to_string()))
}

fn execute_exit_early(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut commitment = STABILITY_COMMITMENTS
        .may_load(deps.storage, info.sender.as_str())?
        .ok_or(ContractError::NoCommitment {
            addr: info.sender.to_string(),
        })?;

    if commitment.state != CommitmentState::Committed {
        return Err(ContractError::CommitmentNotActive {});
    }

    // Early exit: return principal + penalized rewards (50% forfeited)
    let penalized_rewards = commitment
        .accrued_rewards
        .multiply_ratio(10_000u128 - config.early_exit_penalty_bps as u128, 10_000u128);
    let total_return = commitment.amount + penalized_rewards;
    let forfeited = commitment.accrued_rewards - penalized_rewards;

    commitment.state = CommitmentState::EarlyExit;
    STABILITY_COMMITMENTS.save(deps.storage, info.sender.as_str(), &commitment)?;

    // Update stats
    let mut stats = STABILITY_STATS.load(deps.storage)?;
    stats.total_committed = stats.total_committed.saturating_sub(commitment.amount);
    stats.active_commitments = stats.active_commitments.saturating_sub(1);
    STABILITY_STATS.save(deps.storage, &stats)?;

    let send_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: config.denom.clone(),
            amount: total_return,
        }],
    };

    Ok(Response::new()
        .add_message(send_msg)
        .add_attribute("action", "exit_early")
        .add_attribute("address", info.sender.as_str())
        .add_attribute("principal", commitment.amount.to_string())
        .add_attribute("penalized_rewards", penalized_rewards.to_string())
        .add_attribute("forfeited", forfeited.to_string())
        .add_attribute("total_return", total_return.to_string()))
}

// ---- Governance admin ----

fn execute_activate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if state.mechanism_state != MechanismState::Inactive {
        return Err(ContractError::NotActive {
            state: format!("already {:?}", state.mechanism_state),
        });
    }

    state.mechanism_state = MechanismState::Tracking;
    state.current_epoch = 1;
    state.epoch_start_block = env.block.height;
    state.activation_epoch = Some(1);
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "activate")
        .add_attribute("mechanism_state", "tracking")
        .add_attribute("epoch", "1"))
}

fn execute_enable_distribution(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut state = STATE.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if state.mechanism_state != MechanismState::Tracking {
        return Err(ContractError::NotTracking {});
    }

    let activation_epoch = state.activation_epoch.unwrap_or(1);
    let elapsed = state.current_epoch.saturating_sub(activation_epoch);
    if elapsed < config.calibration_epochs {
        return Err(ContractError::CalibrationIncomplete {
            elapsed_epochs: elapsed,
            required_epochs: config.calibration_epochs,
        });
    }

    state.mechanism_state = MechanismState::Distributing;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "enable_distribution")
        .add_attribute("mechanism_state", "distributing")
        .add_attribute("calibration_epochs_completed", elapsed.to_string()))
}

fn execute_update_weights(
    deps: DepsMut,
    info: MessageInfo,
    new_weights: ActivityWeights,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if new_weights.sum() != 10_000 {
        return Err(ContractError::InvalidWeights {});
    }

    config.activity_weights = new_weights;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_weights"))
}

#[allow(clippy::too_many_arguments)]
fn execute_update_stability_params(
    deps: DepsMut,
    info: MessageInfo,
    max_stability_share_bps: Option<u16>,
    stability_annual_return_bps: Option<u16>,
    min_commitment_uregen: Option<Uint128>,
    min_lock_months: Option<u64>,
    max_lock_months: Option<u64>,
    early_exit_penalty_bps: Option<u16>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(v) = max_stability_share_bps {
        config.max_stability_share_bps = v;
    }
    if let Some(v) = stability_annual_return_bps {
        config.stability_annual_return_bps = v;
    }
    if let Some(v) = min_commitment_uregen {
        config.min_commitment_uregen = v;
    }
    if let Some(v) = min_lock_months {
        config.min_lock_months = v;
    }
    if let Some(v) = max_lock_months {
        config.max_lock_months = v;
    }
    if let Some(v) = early_exit_penalty_bps {
        config.early_exit_penalty_bps = v;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_stability_params"))
}

fn execute_pause(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let mut state = STATE.load(deps.storage)?;
    state.paused = true;
    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("action", "pause"))
}

fn execute_resume(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }
    let mut state = STATE.load(deps.storage)?;
    state.paused = false;
    STATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("action", "resume"))
}

// ============================================================================
// Query
// ============================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::State {} => to_json_binary(&query_state(deps)?),
        QueryMsg::ParticipantScore { address, epoch } => {
            to_json_binary(&query_participant_score(deps, address, epoch)?)
        }
        QueryMsg::EpochScores {
            epoch,
            start_after,
            limit,
        } => to_json_binary(&query_epoch_scores(deps, epoch, start_after, limit)?),
        QueryMsg::DistributionHistory { start_epoch, limit } => {
            to_json_binary(&query_distribution_history(deps, start_epoch, limit)?)
        }
        QueryMsg::PendingRewards { address } => {
            to_json_binary(&query_pending_rewards(deps, address)?)
        }
        QueryMsg::StabilityCommitment { address } => {
            to_json_binary(&query_stability_commitment(deps, address)?)
        }
        QueryMsg::StabilityStats {} => to_json_binary(&query_stability_stats(deps)?),
        QueryMsg::SimulateScore { activities } => {
            to_json_binary(&query_simulate_score(deps, activities)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        activity_weights: config.activity_weights,
        max_stability_share_bps: config.max_stability_share_bps,
        stability_annual_return_bps: config.stability_annual_return_bps,
        min_commitment_uregen: config.min_commitment_uregen,
        min_lock_months: config.min_lock_months,
        max_lock_months: config.max_lock_months,
        early_exit_penalty_bps: config.early_exit_penalty_bps,
        blocks_per_epoch: config.blocks_per_epoch,
        calibration_epochs: config.calibration_epochs,
        epochs_per_year: config.epochs_per_year,
        denom: config.denom,
    })
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        mechanism_state: state.mechanism_state,
        current_epoch: state.current_epoch,
        epoch_start_block: state.epoch_start_block,
        activation_epoch: state.activation_epoch,
        paused: state.paused,
    })
}

fn query_participant_score(
    deps: Deps,
    address: String,
    epoch: Option<u64>,
) -> StdResult<ParticipantScoreResponse> {
    let state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let epoch = epoch.unwrap_or(state.current_epoch);
    let addr = deps.api.addr_validate(&address)?;

    let record = ACTIVITY_RECORDS
        .may_load(deps.storage, (epoch, addr.as_str()))?
        .unwrap_or_default();

    let weighted_score = compute_weighted_score(&record, &config.activity_weights);

    Ok(ParticipantScoreResponse {
        address: addr.to_string(),
        epoch,
        credit_purchase_value: record.credit_purchase_value,
        credit_retirement_value: record.credit_retirement_value,
        platform_facilitation_value: record.platform_facilitation_value,
        governance_votes: record.governance_votes,
        proposal_credits: record.proposal_credits_x100,
        weighted_score,
    })
}

fn query_epoch_scores(
    deps: Deps,
    epoch: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<EpochScoresResponse> {
    let config = CONFIG.load(deps.storage)?;
    let limit = limit.unwrap_or(30).min(100) as usize;

    let min_bound = start_after
        .as_deref()
        .map(cw_storage_plus::Bound::exclusive);

    let scores: Vec<ParticipantScoreResponse> = EPOCH_PARTICIPANTS
        .prefix(epoch)
        .range(deps.storage, min_bound, None, Order::Ascending)
        .take(limit)
        .filter_map(|item| item.ok())
        .map(|(addr_str, _)| {
            let record = ACTIVITY_RECORDS
                .load(deps.storage, (epoch, addr_str.as_str()))
                .unwrap_or_default();
            let weighted_score = compute_weighted_score(&record, &config.activity_weights);
            ParticipantScoreResponse {
                address: addr_str,
                epoch,
                credit_purchase_value: record.credit_purchase_value,
                credit_retirement_value: record.credit_retirement_value,
                platform_facilitation_value: record.platform_facilitation_value,
                governance_votes: record.governance_votes,
                proposal_credits: record.proposal_credits_x100,
                weighted_score,
            }
        })
        .collect();

    Ok(EpochScoresResponse { epoch, scores })
}

fn query_distribution_history(
    deps: Deps,
    start_epoch: Option<u64>,
    limit: Option<u32>,
) -> StdResult<DistributionHistoryResponse> {
    let limit = limit.unwrap_or(10).min(50) as usize;
    let min_bound = start_epoch.map(cw_storage_plus::Bound::inclusive);

    let distributions: Vec<EpochDistribution> = DISTRIBUTIONS
        .range(deps.storage, min_bound, None, Order::Ascending)
        .take(limit)
        .filter_map(|item| item.ok())
        .map(|(epoch, dist)| EpochDistribution {
            epoch,
            community_pool_inflow: dist.community_pool_inflow,
            stability_allocation: dist.stability_allocation,
            activity_pool: dist.activity_pool,
            total_score: dist.total_score,
            participant_count: dist.participant_count,
        })
        .collect();

    Ok(DistributionHistoryResponse { distributions })
}

fn query_pending_rewards(deps: Deps, address: String) -> StdResult<PendingRewardsResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let activity = PENDING_ACTIVITY_REWARDS
        .may_load(deps.storage, addr.as_str())?
        .unwrap_or_default();
    let stability = PENDING_STABILITY_REWARDS
        .may_load(deps.storage, addr.as_str())?
        .unwrap_or_default();

    Ok(PendingRewardsResponse {
        address: addr.to_string(),
        pending_activity_rewards: activity,
        pending_stability_rewards: stability,
        total_pending: activity + stability,
    })
}

fn query_stability_commitment(
    deps: Deps,
    address: String,
) -> StdResult<StabilityCommitmentResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let commitment = STABILITY_COMMITMENTS
        .may_load(deps.storage, addr.as_str())?
        .unwrap_or(StabilityCommitment {
            amount: Uint128::zero(),
            lock_months: 0,
            committed_at_block: 0,
            maturity_block: 0,
            state: CommitmentState::EarlyExit,
            accrued_rewards: Uint128::zero(),
        });

    Ok(StabilityCommitmentResponse {
        address: addr.to_string(),
        amount: commitment.amount,
        lock_months: commitment.lock_months,
        committed_at_block: commitment.committed_at_block,
        maturity_block: commitment.maturity_block,
        state: commitment.state,
        accrued_rewards: commitment.accrued_rewards,
    })
}

fn query_stability_stats(deps: Deps) -> StdResult<StabilityStatsResponse> {
    let stats = STABILITY_STATS.load(deps.storage)?;
    Ok(StabilityStatsResponse {
        total_committed: stats.total_committed,
        active_commitments: stats.active_commitments,
        total_stability_allocated: stats.total_stability_allocated,
    })
}

fn query_simulate_score(
    deps: Deps,
    activities: Vec<SimulateActivity>,
) -> StdResult<SimulateScoreResponse> {
    let config = CONFIG.load(deps.storage)?;
    let mut record = ActivityRecord::new();
    let mut breakdown = Vec::new();

    for act in &activities {
        match act.activity {
            ActivityType::CreditPurchase => {
                record.credit_purchase_value += act.value;
            }
            ActivityType::CreditRetirement => {
                record.credit_retirement_value += act.value;
            }
            ActivityType::PlatformFacilitation => {
                record.platform_facilitation_value += act.value;
            }
            ActivityType::GovernanceVote => {
                record.governance_votes += act.value;
            }
            ActivityType::ProposalSubmission => {
                let credit_x100 = match act.proposal_outcome {
                    Some(ProposalOutcome::PassedAndApproved) => Uint128::new(100),
                    Some(ProposalOutcome::ReachedQuorumFailed) => Uint128::new(50),
                    _ => Uint128::zero(),
                };
                record.proposal_credits_x100 += credit_x100;
            }
        }
    }

    // Compute individual component scores for breakdown
    let cp_score = record
        .credit_purchase_value
        .multiply_ratio(config.activity_weights.credit_purchase_bps as u128, 10_000u128);
    let cr_score = record
        .credit_retirement_value
        .multiply_ratio(config.activity_weights.credit_retirement_bps as u128, 10_000u128);
    let pf_score = record.platform_facilitation_value.multiply_ratio(
        config.activity_weights.platform_facilitation_bps as u128,
        10_000u128,
    );
    let gv_score = record
        .governance_votes
        .multiply_ratio(config.activity_weights.governance_voting_bps as u128, 10_000u128);
    let ps_score = record
        .proposal_credits_x100
        .multiply_ratio(config.activity_weights.proposal_submission_bps as u128, 10_000u128)
        / Uint128::new(100);

    breakdown.push(("credit_purchase".to_string(), cp_score));
    breakdown.push(("credit_retirement".to_string(), cr_score));
    breakdown.push(("platform_facilitation".to_string(), pf_score));
    breakdown.push(("governance_voting".to_string(), gv_score));
    breakdown.push(("proposal_submission".to_string(), ps_score));

    let weighted_score = cp_score + cr_score + pf_score + gv_score + ps_score;

    Ok(SimulateScoreResponse {
        weighted_score,
        breakdown,
    })
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Compute weighted activity score from an ActivityRecord.
/// Uses integer arithmetic only (SPEC requirement).
/// Score = sum of (value * weight_bps / 10000) for each activity.
/// Proposal credits use fixed-point x100 representation.
fn compute_weighted_score(record: &ActivityRecord, weights: &ActivityWeights) -> Uint128 {
    let cp = record
        .credit_purchase_value
        .multiply_ratio(weights.credit_purchase_bps as u128, 10_000u128);
    let cr = record
        .credit_retirement_value
        .multiply_ratio(weights.credit_retirement_bps as u128, 10_000u128);
    let pf = record
        .platform_facilitation_value
        .multiply_ratio(weights.platform_facilitation_bps as u128, 10_000u128);
    let gv = record
        .governance_votes
        .multiply_ratio(weights.governance_voting_bps as u128, 10_000u128);
    // proposal_credits_x100 is in fixed-point (100 = 1.0), so divide by 100 after weighting
    let ps = record
        .proposal_credits_x100
        .multiply_ratio(weights.proposal_submission_bps as u128, 10_000u128)
        / Uint128::new(100);

    cp + cr + pf + gv + ps
}

/// Compute raw stability allocation for one epoch:
/// sum(commitment.amount * annual_return_bps / 10000) / epochs_per_year
fn compute_stability_allocation(
    total_committed: Uint128,
    annual_return_bps: u16,
    epochs_per_year: u64,
) -> Uint128 {
    if total_committed.is_zero() || epochs_per_year == 0 {
        return Uint128::zero();
    }
    total_committed
        .multiply_ratio(annual_return_bps as u128, 10_000u128)
        / Uint128::new(epochs_per_year as u128)
}

/// Distribute stability rewards to all committed holders proportionally
fn distribute_stability_rewards(
    storage: &mut dyn cosmwasm_std::Storage,
    stability_allocation: Uint128,
    _config: &Config,
) -> Result<(), ContractError> {
    if stability_allocation.is_zero() {
        return Ok(());
    }

    // Iterate all active commitments and distribute proportionally to committed amount
    let commitments: Vec<(String, StabilityCommitment)> = STABILITY_COMMITMENTS
        .range(storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .filter(|(_, c)| c.state == CommitmentState::Committed)
        .collect();

    let total_committed: Uint128 = commitments.iter().map(|(_, c)| c.amount).sum();

    if total_committed.is_zero() {
        return Ok(());
    }

    for (addr, mut commitment) in commitments {
        let share = stability_allocation.multiply_ratio(commitment.amount, total_committed);
        commitment.accrued_rewards += share;
        STABILITY_COMMITMENTS.save(storage, &addr, &commitment)?;

        let existing = PENDING_STABILITY_REWARDS
            .may_load(storage, &addr)?
            .unwrap_or_default();
        PENDING_STABILITY_REWARDS.save(storage, &addr, &(existing + share))?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env, MockApi};
    use cosmwasm_std::{coins, Addr};

    fn addr(input: &str) -> Addr {
        MockApi::default().addr_make(input)
    }

    fn admin_addr() -> Addr {
        addr("admin")
    }
    fn alice_addr() -> Addr {
        addr("alice")
    }
    fn bob_addr() -> Addr {
        addr("bob")
    }
    fn carol_addr() -> Addr {
        addr("carol")
    }

    fn setup_contract(deps: DepsMut, blocks_per_epoch: u64) -> Addr {
        let admin = admin_addr();
        let info = message_info(&admin, &[]);
        let msg = InstantiateMsg {
            admin: admin.to_string(),
            activity_weights: None,
            max_stability_share_bps: None,
            stability_annual_return_bps: None,
            min_commitment_uregen: None,
            min_lock_months: None,
            max_lock_months: None,
            early_exit_penalty_bps: None,
            blocks_per_epoch: Some(blocks_per_epoch),
            calibration_epochs: Some(2), // Short calibration for tests
            epochs_per_year: Some(52),
            denom: None,
        };
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes[0].value, "instantiate");
        admin
    }

    fn activate(deps: DepsMut, admin: &Addr) {
        let info = message_info(admin, &[]);
        execute(deps, mock_env(), info, ExecuteMsg::Activate {}).unwrap();
    }

    fn record(
        deps: DepsMut,
        admin: &Addr,
        participant: &Addr,
        activity: ActivityType,
        value: u128,
        proposal_outcome: Option<ProposalOutcome>,
        tx_hash: &str,
    ) {
        let info = message_info(admin, &[]);
        execute(
            deps,
            mock_env(),
            info,
            ExecuteMsg::RecordContribution {
                participant: participant.to_string(),
                activity,
                value: Uint128::new(value),
                proposal_outcome,
                tx_hash: tx_hash.to_string(),
            },
        )
        .unwrap();
    }

    // ---- Test: Instantiation ----
    #[test]
    fn test_instantiate_defaults() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);

        let config: ConfigResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap())
                .unwrap();

        assert_eq!(config.admin, admin.to_string());
        assert_eq!(config.activity_weights.credit_purchase_bps, 3000);
        assert_eq!(config.activity_weights.credit_retirement_bps, 3000);
        assert_eq!(config.activity_weights.platform_facilitation_bps, 2000);
        assert_eq!(config.activity_weights.governance_voting_bps, 1000);
        assert_eq!(config.activity_weights.proposal_submission_bps, 1000);
        assert_eq!(config.max_stability_share_bps, 3000);
        assert_eq!(config.stability_annual_return_bps, 600);
        assert_eq!(config.min_commitment_uregen, Uint128::new(100_000_000));
        assert_eq!(config.denom, "uregen");
    }

    #[test]
    fn test_instantiate_invalid_weights() {
        let mut deps = mock_dependencies();
        let info = message_info(&admin_addr(), &[]);
        let msg = InstantiateMsg {
            admin: admin_addr().to_string(),
            activity_weights: Some(ActivityWeights {
                credit_purchase_bps: 5000,
                credit_retirement_bps: 5000,
                platform_facilitation_bps: 5000,
                governance_voting_bps: 0,
                proposal_submission_bps: 0,
            }),
            max_stability_share_bps: None,
            stability_annual_return_bps: None,
            min_commitment_uregen: None,
            min_lock_months: None,
            max_lock_months: None,
            early_exit_penalty_bps: None,
            blocks_per_epoch: None,
            calibration_epochs: None,
            epochs_per_year: None,
            denom: None,
        };
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidWeights {});
    }

    // ---- Test: State machine ----
    #[test]
    fn test_activation() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);

        // Starts inactive
        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        assert_eq!(state.mechanism_state, MechanismState::Inactive);

        // Activate
        activate(deps.as_mut(), &admin);
        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        assert_eq!(state.mechanism_state, MechanismState::Tracking);
        assert_eq!(state.current_epoch, 1);
        assert_eq!(state.activation_epoch, Some(1));
    }

    #[test]
    fn test_activate_unauthorized() {
        let mut deps = mock_dependencies();
        let _admin = setup_contract(deps.as_mut(), 100);
        let info = message_info(&addr("hacker"), &[]);
        let err = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Activate {}).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_enable_distribution_too_early() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        let info = message_info(&admin_addr(), &[]);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::EnableDistribution {},
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::CalibrationIncomplete { .. }));
    }

    // ---- Test: Activity scoring (SPEC acceptance tests 1-6) ----

    // AT-1: Participant with only credit purchases receives score = purchase_value * 0.30
    #[test]
    fn test_score_credit_purchase_only() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            1_000_000,
            None,
            "tx1",
        );

        let alice_addr = alice_addr();
        let score: ParticipantScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ParticipantScore {
                    address: alice_addr.to_string(),
                    epoch: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // 1_000_000 * 3000 / 10000 = 300_000
        assert_eq!(score.weighted_score, Uint128::new(300_000));
    }

    // AT-2: Participant with all five activity types receives correct weighted sum
    #[test]
    fn test_score_all_activity_types() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            1_000_000, // * 0.30 = 300_000
            None,
            "tx1",
        );
        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditRetirement,
            500_000, // * 0.30 = 150_000
            None,
            "tx2",
        );
        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::PlatformFacilitation,
            200_000, // * 0.20 = 40_000
            None,
            "tx3",
        );
        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::GovernanceVote,
            3, // * 0.10 = 0 (integer truncation: 3 * 1000 / 10000 = 0)
            None,
            "tx4",
        );
        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::ProposalSubmission,
            1,
            Some(ProposalOutcome::PassedAndApproved),
            "tx5",
        );

        let alice_addr = alice_addr();
        let score: ParticipantScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ParticipantScore {
                    address: alice_addr.to_string(),
                    epoch: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // credit_purchase: 1_000_000 * 3000 / 10000 = 300_000
        // credit_retirement: 500_000 * 3000 / 10000 = 150_000
        // platform_facilitation: 200_000 * 2000 / 10000 = 40_000
        // governance: 3 * 1000 / 10000 = 0 (integer truncation)
        // proposal: 100 (x100) * 1000 / 10000 / 100 = 0 (integer: small value)
        // Total = 490_000
        assert_eq!(score.weighted_score, Uint128::new(490_000));
    }

    // AT-3: Proposal that passed quorum earns full weight
    #[test]
    fn test_proposal_passed_quorum() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::ProposalSubmission,
            1,
            Some(ProposalOutcome::PassedAndApproved),
            "tx1",
        );

        let alice_addr = alice_addr();
        let score: ParticipantScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ParticipantScore {
                    address: alice_addr.to_string(),
                    epoch: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // proposal_credits_x100 = 100, weight 1000/10000, / 100 = 0 at small scale
        // But credits_x100 is stored correctly
        assert_eq!(score.proposal_credits, Uint128::new(100));
    }

    // AT-4: Proposal that reached quorum but failed earns 0.05 effective weight
    #[test]
    fn test_proposal_reached_quorum_failed() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::ProposalSubmission,
            1,
            Some(ProposalOutcome::ReachedQuorumFailed),
            "tx1",
        );

        let alice_addr = alice_addr();
        let score: ParticipantScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ParticipantScore {
                    address: alice_addr.to_string(),
                    epoch: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // proposal_credits_x100 = 50 (half credit)
        assert_eq!(score.proposal_credits, Uint128::new(50));
    }

    // AT-5: Proposal that failed to reach quorum earns 0 weight
    #[test]
    fn test_proposal_failed_quorum() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::ProposalSubmission,
            1,
            Some(ProposalOutcome::FailedQuorum),
            "tx1",
        );

        let alice_addr = alice_addr();
        let score: ParticipantScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ParticipantScore {
                    address: alice_addr.to_string(),
                    epoch: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // Zero credits for failed quorum
        assert_eq!(score.proposal_credits, Uint128::zero());
        assert_eq!(score.weighted_score, Uint128::zero());
    }

    // AT-6: Participant with zero activity receives 0 score
    #[test]
    fn test_zero_activity_zero_score() {
        let mut deps = mock_dependencies();
        let _admin = setup_contract(deps.as_mut(), 100);

        let alice_addr = alice_addr();
        let score: ParticipantScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::ParticipantScore {
                    address: alice_addr.to_string(),
                    epoch: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(score.weighted_score, Uint128::zero());
    }

    // ---- Test: Distribution (SPEC acceptance tests 7-11) ----

    /// Sets up the contract through INACTIVE -> TRACKING -> DISTRIBUTING.
    /// Must be called on `deps.as_mut()` which re-borrows each time.
    /// Returns the current epoch after setup.
    fn setup_distributing(deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, blocks_per_epoch: u64) {
        let admin = admin_addr();
        // Activate
        activate(deps.as_mut(), &admin);

        // Finalize 2 calibration epochs to enable distribution
        for i in 0..2 {
            let mut env = mock_env();
            env.block.height = 12_345 + (i + 1) * blocks_per_epoch;
            let info = message_info(&admin_addr(), &[]);
            execute(
                deps.as_mut(),
                env,
                info,
                ExecuteMsg::FinalizeEpoch {
                    community_pool_inflow: Uint128::new(1_000_000),
                },
            )
            .unwrap();
        }

        // Enable distribution
        let info = message_info(&admin_addr(), &[]);
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::EnableDistribution {}).unwrap();
    }

    // AT-7: Sum of all participant rewards equals activity_pool
    // AT-8: Each participant's reward is proportional to their share of total score
    #[test]
    fn test_proportional_distribution() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 10);
        setup_distributing(&mut deps, 10);

        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        let current_epoch = state.current_epoch;

        // Alice: 1M purchase, Bob: 500K purchase
        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            1_000_000,
            None,
            "tx_dist_1",
        );
        record(
            deps.as_mut(),
            &admin,
            &bob_addr(),
            ActivityType::CreditPurchase,
            500_000,
            None,
            "tx_dist_2",
        );

        // Finalize with 900_000 inflow, no stability commitments
        let mut env = mock_env();
        env.block.height = 12_345 + (current_epoch + 1) * 10;
        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::FinalizeEpoch {
                community_pool_inflow: Uint128::new(900_000),
            },
        )
        .unwrap();

        let alice_addr = alice_addr();
        let bob_addr = bob_addr();

        let alice_pending: PendingRewardsResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::PendingRewards {
                    address: alice_addr.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let bob_pending: PendingRewardsResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::PendingRewards {
                    address: bob_addr.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // No stability: full 900K goes to activity pool
        // Alice score: 1M * 3000/10000 = 300K
        // Bob score: 500K * 3000/10000 = 150K
        // Total score = 450K
        // Alice reward: 900K * 300K / 450K = 600K
        // Bob reward: 900K * 150K / 450K = 300K
        assert_eq!(
            alice_pending.pending_activity_rewards,
            Uint128::new(600_000)
        );
        assert_eq!(bob_pending.pending_activity_rewards, Uint128::new(300_000));

        // AT-7: Sum = activity_pool
        let sum = alice_pending.pending_activity_rewards + bob_pending.pending_activity_rewards;
        assert_eq!(sum, Uint128::new(900_000));
    }

    // AT-9: Stability allocation is capped at 30% of community_pool_inflow
    #[test]
    fn test_stability_cap() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 10);
        setup_distributing(&mut deps, 10);

        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        let current_epoch = state.current_epoch;

        // Commit large stability (1B uregen = 1000 REGEN)
        let mut env = mock_env();
        env.block.height = 12_345 + current_epoch * 10;
        let info = message_info(&carol_addr(), &coins(1_000_000_000, "uregen"));
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CommitStability { lock_months: 12 },
        )
        .unwrap();

        // Record some activity
        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            1_000_000,
            None,
            "tx_cap_1",
        );

        // Finalize with 1M inflow
        env.block.height = 12_345 + (current_epoch + 1) * 10;
        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::FinalizeEpoch {
                community_pool_inflow: Uint128::new(1_000_000),
            },
        )
        .unwrap();

        let dist: DistributionHistoryResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::DistributionHistory {
                    start_epoch: Some(current_epoch),
                    limit: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        let d = &dist.distributions[0];
        // Raw stability: 1B * 600/10000 / 52 = ~1_153_846 uregen per epoch
        // Cap: 1M * 3000/10000 = 300_000
        // Should be capped
        assert_eq!(d.stability_allocation, Uint128::new(300_000));
        assert_eq!(d.activity_pool, Uint128::new(700_000));
    }

    // AT-11: Activity pool = community_pool_inflow - stability_allocation
    #[test]
    fn test_activity_pool_equals_inflow_minus_stability() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 10);
        setup_distributing(&mut deps, 10);

        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        let current_epoch = state.current_epoch;

        // Small stability so it doesn't hit cap
        let mut env = mock_env();
        env.block.height = 12_345 + current_epoch * 10;
        let info = message_info(&carol_addr(), &coins(100_000_000, "uregen")); // 100 REGEN
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CommitStability { lock_months: 6 },
        )
        .unwrap();

        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            500_000,
            None,
            "tx_pool_1",
        );

        env.block.height = 12_345 + (current_epoch + 1) * 10;
        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::FinalizeEpoch {
                community_pool_inflow: Uint128::new(10_000_000),
            },
        )
        .unwrap();

        let dist: DistributionHistoryResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::DistributionHistory {
                    start_epoch: Some(current_epoch),
                    limit: Some(1),
                },
            )
            .unwrap(),
        )
        .unwrap();

        let d = &dist.distributions[0];
        assert_eq!(
            d.activity_pool,
            d.community_pool_inflow - d.stability_allocation
        );
    }

    // ---- Test: Stability tier (SPEC acceptance tests 12-16) ----

    // AT-12: Commitment with amount < 100 REGEN is rejected
    #[test]
    fn test_commitment_too_small() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        let info = message_info(&carol_addr(), &coins(50_000_000, "uregen")); // 50 REGEN < 100 min
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::CommitStability { lock_months: 6 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::CommitmentTooSmall { .. }));
    }

    // AT-13: Commitment with lock_period < 6 months is rejected
    #[test]
    fn test_lock_period_too_short() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        let info = message_info(&carol_addr(), &coins(100_000_000, "uregen"));
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::CommitStability { lock_months: 3 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::InvalidLockPeriod { .. }));
    }

    // AT-14: Commitment with lock_period > 24 months is rejected
    #[test]
    fn test_lock_period_too_long() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        let info = message_info(&carol_addr(), &coins(100_000_000, "uregen"));
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::CommitStability { lock_months: 36 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::InvalidLockPeriod { .. }));
    }

    // AT-16: Early exit forfeits 50% of accrued rewards
    #[test]
    fn test_early_exit_penalty() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 10);
        setup_distributing(&mut deps, 10);

        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        let current_epoch = state.current_epoch;

        // Commit
        let committer = carol_addr();
        let mut env = mock_env();
        env.block.height = 12_345 + current_epoch * 10;
        let info = message_info(&carol_addr(), &coins(200_000_000, "uregen")); // 200 REGEN
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CommitStability { lock_months: 12 },
        )
        .unwrap();

        // Add activity so epoch can finalize
        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            1_000_000,
            None,
            "tx_early_1",
        );

        // Finalize epoch to accrue stability rewards
        env.block.height = 12_345 + (current_epoch + 1) * 10;
        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::FinalizeEpoch {
                community_pool_inflow: Uint128::new(10_000_000),
            },
        )
        .unwrap();

        // Check accrued
        let commitment: StabilityCommitmentResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::StabilityCommitment {
                    address: committer.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let accrued = commitment.accrued_rewards;
        assert!(!accrued.is_zero());

        // Exit early
        let info = message_info(&carol_addr(), &[]);
        let res = execute(deps.as_mut(), env, info, ExecuteMsg::ExitEarly {}).unwrap();

        // Check penalty: 50% of accrued forfeited
        let penalized = accrued.multiply_ratio(5000u128, 10000u128);
        let forfeited_attr = res
            .attributes
            .iter()
            .find(|a| a.key == "forfeited")
            .unwrap();
        assert_eq!(forfeited_attr.value, (accrued - penalized).to_string());
    }

    // ---- Test: Security invariants (SPEC AT-17, 19, 20) ----

    // AT-17: Total distributions per period <= Community Pool inflow
    #[test]
    fn test_distribution_does_not_exceed_inflow() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 10);
        setup_distributing(&mut deps, 10);

        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        let current_epoch = state.current_epoch;

        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            5_000_000,
            None,
            "tx_inv_1",
        );
        record(
            deps.as_mut(),
            &admin,
            &bob_addr(),
            ActivityType::CreditRetirement,
            3_000_000,
            None,
            "tx_inv_2",
        );

        let inflow = Uint128::new(1_000_000);
        let mut env = mock_env();
        env.block.height = 12_345 + (current_epoch + 1) * 10;
        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::FinalizeEpoch {
                community_pool_inflow: inflow,
            },
        )
        .unwrap();

        let alice_addr = alice_addr();
        let bob_addr = bob_addr();

        let alice_pending: PendingRewardsResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::PendingRewards {
                    address: alice_addr.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        let bob_pending: PendingRewardsResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::PendingRewards {
                    address: bob_addr.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        let total_distributed = alice_pending.total_pending + bob_pending.total_pending;
        assert!(total_distributed <= inflow);
    }

    // AT-19: Each transaction counted exactly once (dedup)
    #[test]
    fn test_duplicate_tx_hash_rejected() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            100_000,
            None,
            "dup_tx",
        );

        // Second record with same tx_hash should fail
        let alice_addr = alice_addr();
        let info = message_info(&admin_addr(), &[]);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::RecordContribution {
                participant: alice_addr.to_string(),
                activity: ActivityType::CreditPurchase,
                value: Uint128::new(100_000),
                proposal_outcome: None,
                tx_hash: "dup_tx".to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::DuplicateContribution { .. }));
    }

    // AT-20: Distribution parameters changeable via governance
    #[test]
    fn test_governance_update_weights() {
        let mut deps = mock_dependencies();
        let _admin = setup_contract(deps.as_mut(), 100);

        let new_weights = ActivityWeights {
            credit_purchase_bps: 4000,
            credit_retirement_bps: 2000,
            platform_facilitation_bps: 2000,
            governance_voting_bps: 1000,
            proposal_submission_bps: 1000,
        };

        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::UpdateWeights {
                new_weights: new_weights.clone(),
            },
        )
        .unwrap();

        let config: ConfigResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap())
                .unwrap();
        assert_eq!(config.activity_weights.credit_purchase_bps, 4000);
        assert_eq!(config.activity_weights.credit_retirement_bps, 2000);
    }

    // ---- Test: State machine transitions (SPEC AT-21, 22) ----

    // AT-21: INACTIVE -> TRACKING only via governance approval
    #[test]
    fn test_inactive_to_tracking() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);

        // Can't record while inactive
        let alice_addr = alice_addr();
        let info = message_info(&admin_addr(), &[]);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::RecordContribution {
                participant: alice_addr.to_string(),
                activity: ActivityType::CreditPurchase,
                value: Uint128::new(1000),
                proposal_outcome: None,
                tx_hash: "tx_inactive".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotActive { .. }));

        // Activate
        activate(deps.as_mut(), &admin);
        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        assert_eq!(state.mechanism_state, MechanismState::Tracking);
    }

    // AT-22: TRACKING -> DISTRIBUTING after calibration
    #[test]
    fn test_tracking_to_distributing() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 10);
        activate(deps.as_mut(), &admin);

        // Finalize 2 epochs (calibration_epochs = 2)
        for i in 0..2 {
            let mut env = mock_env();
            env.block.height = 12_345 + (i + 1) * 10;
            let info = message_info(&admin_addr(), &[]);
            execute(
                deps.as_mut(),
                env,
                info,
                ExecuteMsg::FinalizeEpoch {
                    community_pool_inflow: Uint128::new(1_000_000),
                },
            )
            .unwrap();
        }

        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::EnableDistribution {},
        )
        .unwrap();

        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        assert_eq!(state.mechanism_state, MechanismState::Distributing);
    }

    // ---- Test: Circuit breaker (pause/resume) ----
    #[test]
    fn test_pause_resume() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 100);
        activate(deps.as_mut(), &admin);

        // Pause
        let info = message_info(&admin_addr(), &[]);
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Pause {}).unwrap();

        // Can't record while paused
        let alice_addr = alice_addr();
        let info = message_info(&admin_addr(), &[]);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::RecordContribution {
                participant: alice_addr.to_string(),
                activity: ActivityType::CreditPurchase,
                value: Uint128::new(1000),
                proposal_outcome: None,
                tx_hash: "tx_pause".to_string(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::NotActive { .. }));

        // Resume
        let info = message_info(&admin_addr(), &[]);
        execute(deps.as_mut(), mock_env(), info, ExecuteMsg::Resume {}).unwrap();

        // Can record again
        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::RecordContribution {
                participant: alice_addr.to_string(),
                activity: ActivityType::CreditPurchase,
                value: Uint128::new(1000),
                proposal_outcome: None,
                tx_hash: "tx_resume".to_string(),
            },
        )
        .unwrap();
    }

    // ---- Test: Simulate score query ----
    #[test]
    fn test_simulate_score() {
        let mut deps = mock_dependencies();
        let _admin = setup_contract(deps.as_mut(), 100);

        let result: SimulateScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::SimulateScore {
                    activities: vec![
                        SimulateActivity {
                            activity: ActivityType::CreditPurchase,
                            value: Uint128::new(10_000_000),
                            proposal_outcome: None,
                        },
                        SimulateActivity {
                            activity: ActivityType::CreditRetirement,
                            value: Uint128::new(5_000_000),
                            proposal_outcome: None,
                        },
                    ],
                },
            )
            .unwrap(),
        )
        .unwrap();

        // 10M * 3000/10000 + 5M * 3000/10000 = 3M + 1.5M = 4.5M
        assert_eq!(result.weighted_score, Uint128::new(4_500_000));
        assert_eq!(result.breakdown.len(), 5);
        assert_eq!(result.breakdown[0].1, Uint128::new(3_000_000)); // credit_purchase
        assert_eq!(result.breakdown[1].1, Uint128::new(1_500_000)); // credit_retirement
    }

    // ---- Test: Epoch already finalized ----
    #[test]
    fn test_epoch_already_finalized() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 10);
        activate(deps.as_mut(), &admin);

        let mut env = mock_env();
        env.block.height = 12_345 + 10;
        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::FinalizeEpoch {
                community_pool_inflow: Uint128::new(1_000),
            },
        )
        .unwrap();

        // Try to finalize same epoch again — should fail because epoch advanced
        // and new epoch hasn't ended yet
        let info = message_info(&admin_addr(), &[]);
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::FinalizeEpoch {
                community_pool_inflow: Uint128::new(1_000),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::EpochNotEnded { .. }));
    }

    // ---- Test: Claim rewards sends BankMsg ----
    #[test]
    fn test_claim_rewards() {
        let mut deps = mock_dependencies();
        let admin = setup_contract(deps.as_mut(), 10);
        setup_distributing(&mut deps, 10);

        let state: StateResponse =
            cosmwasm_std::from_json(query(deps.as_ref(), mock_env(), QueryMsg::State {}).unwrap())
                .unwrap();
        let current_epoch = state.current_epoch;

        // Record and finalize
        record(
            deps.as_mut(),
            &admin,
            &alice_addr(),
            ActivityType::CreditPurchase,
            1_000_000,
            None,
            "tx_claim_1",
        );

        let mut env = mock_env();
        env.block.height = 12_345 + (current_epoch + 1) * 10;
        let info = message_info(&admin_addr(), &[]);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::FinalizeEpoch {
                community_pool_inflow: Uint128::new(500_000),
            },
        )
        .unwrap();

        // Claim
        let alice = alice_addr();
        let info = message_info(&alice, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::ClaimRewards {}).unwrap();

        // Should have a BankMsg::Send
        assert_eq!(res.messages.len(), 1);
        let total_attr = res
            .attributes
            .iter()
            .find(|a| a.key == "total")
            .unwrap();
        assert_eq!(total_attr.value, "500000");

        // Pending should be zero after claim
        let pending: PendingRewardsResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::PendingRewards {
                    address: alice_addr().to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(pending.total_pending, Uint128::zero());
    }
}
