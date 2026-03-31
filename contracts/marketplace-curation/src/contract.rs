use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::*;
use crate::state::*;

const CONTRACT_NAME: &str = "crates.io:marketplace-curation";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ════════════════════════════════════════════════════════════════════
// Instantiate
// ════════════════════════════════════════════════════════════════════

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admin: info.sender.clone(),
        bond_denom: msg.bond_denom,
        min_curation_bond: msg.min_curation_bond,
        listing_fee: msg.listing_fee,
        curation_fee_bps: msg.curation_fee_bps,
        challenge_deposit: msg.challenge_deposit,
        slash_pct_bps: msg.slash_pct_bps,
        challenge_reward_bps: msg.challenge_reward_bps,
        activation_delay_s: msg.activation_delay_s,
        unbonding_period_s: msg.unbonding_period_s,
        top_up_window_s: msg.top_up_window_s,
        min_quality_score: msg.min_quality_score,
        max_collections_per_curator: msg.max_collections_per_curator,
    };

    CONFIG.save(deps.storage, &config)?;
    COLLECTION_SEQ.save(deps.storage, &0u64)?;
    CHALLENGE_SEQ.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender))
}

// ════════════════════════════════════════════════════════════════════
// Execute
// ════════════════════════════════════════════════════════════════════

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateCollection {
            name,
            description,
            criteria,
        } => exec_create_collection(deps, env, info, name, description, criteria),
        ExecuteMsg::ActivateCollection { collection_id } => {
            exec_activate_collection(deps, env, info, collection_id)
        }
        ExecuteMsg::AddBatch {
            collection_id,
            batch_denom,
        } => exec_add_batch(deps, env, info, collection_id, batch_denom),
        ExecuteMsg::RemoveBatch {
            collection_id,
            batch_denom,
        } => exec_remove_batch(deps, info, collection_id, batch_denom),
        ExecuteMsg::ChallengeInclusion {
            collection_id,
            batch_denom,
            reason,
        } => exec_challenge_inclusion(deps, env, info, collection_id, batch_denom, reason),
        ExecuteMsg::ResolveChallenge {
            challenge_id,
            outcome,
        } => exec_resolve_challenge(deps, env, info, challenge_id, outcome),
        ExecuteMsg::TopUpBond { collection_id } => {
            exec_top_up_bond(deps, env, info, collection_id)
        }
        ExecuteMsg::CloseCollection { collection_id } => {
            exec_close_collection(deps, env, info, collection_id)
        }
        ExecuteMsg::ClaimRefund { collection_id } => {
            exec_claim_refund(deps, env, info, collection_id)
        }
        ExecuteMsg::RecordTrade {
            collection_id,
            trade_amount,
        } => exec_record_trade(deps, info, collection_id, trade_amount),
        ExecuteMsg::SubmitQualityScore {
            batch_denom,
            score,
            confidence,
            factors,
        } => exec_submit_quality_score(deps, env, info, batch_denom, score, confidence, factors),
        ExecuteMsg::ForceCloseSuspended { collection_id } => {
            exec_force_close_suspended(deps, env, collection_id)
        }
        ExecuteMsg::UpdateConfig {
            min_curation_bond,
            listing_fee,
            curation_fee_bps,
            challenge_deposit,
            slash_pct_bps,
            challenge_reward_bps,
            activation_delay_s,
            unbonding_period_s,
            top_up_window_s,
            min_quality_score,
            max_collections_per_curator,
        } => exec_update_config(
            deps,
            info,
            min_curation_bond,
            listing_fee,
            curation_fee_bps,
            challenge_deposit,
            slash_pct_bps,
            challenge_reward_bps,
            activation_delay_s,
            unbonding_period_s,
            top_up_window_s,
            min_quality_score,
            max_collections_per_curator,
        ),
    }
}

// ── Create collection ───────────────────────────────────────────────

fn exec_create_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    description: String,
    criteria: CurationCriteria,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check max collections per curator
    let count = CURATOR_COLLECTION_COUNT
        .may_load(deps.storage, &info.sender)?
        .unwrap_or(0);
    if count >= config.max_collections_per_curator {
        return Err(ContractError::MaxCollectionsReached {
            max: config.max_collections_per_curator,
        });
    }

    // Validate bond funds
    let bond_sent = get_sent_amount(&info, &config.bond_denom)?;
    if bond_sent < config.min_curation_bond {
        return Err(ContractError::InsufficientBond {
            sent: bond_sent.u128(),
            min: config.min_curation_bond.u128(),
        });
    }

    // Create collection
    let id = COLLECTION_SEQ.load(deps.storage)? + 1;
    COLLECTION_SEQ.save(deps.storage, &id)?;

    let collection = Collection {
        id,
        curator: info.sender.clone(),
        name: name.clone(),
        description,
        criteria,
        bond_amount: bond_sent,
        bond_remaining: bond_sent,
        status: CollectionStatus::Proposed,
        members: vec![],
        trade_volume: Uint128::zero(),
        total_rewards: Uint128::zero(),
        created_at_s: env.block.time.seconds(),
        activated_at_s: None,
        suspended_at_s: None,
        close_initiated_at_s: None,
    };

    COLLECTIONS.save(deps.storage, id, &collection)?;
    CURATOR_COLLECTION_COUNT.save(deps.storage, &info.sender, &(count + 1))?;

    Ok(Response::new()
        .add_attribute("method", "create_collection")
        .add_attribute("collection_id", id.to_string())
        .add_attribute("curator", info.sender)
        .add_attribute("name", name)
        .add_attribute("bond", bond_sent))
}

// ── Activate collection ─────────────────────────────────────────────

fn exec_activate_collection(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    collection_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut col = load_collection(deps.as_ref(), collection_id)?;

    if col.status != CollectionStatus::Proposed {
        return Err(ContractError::CollectionNotProposed {});
    }

    let elapsed = env.block.time.seconds() - col.created_at_s;
    if elapsed < config.activation_delay_s {
        return Err(ContractError::ActivationDelayNotElapsed {});
    }

    // Check no active challenge
    if ACTIVE_CHALLENGE
        .may_load(deps.storage, collection_id)?
        .is_some()
    {
        return Err(ContractError::PendingChallenge {});
    }

    col.status = CollectionStatus::Active;
    col.activated_at_s = Some(env.block.time.seconds());
    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    Ok(Response::new()
        .add_attribute("method", "activate_collection")
        .add_attribute("collection_id", collection_id.to_string()))
}

// ── Add batch ───────────────────────────────────────────────────────

fn exec_add_batch(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_id: u64,
    batch_denom: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut col = load_collection(deps.as_ref(), collection_id)?;

    // Only curator can add
    if info.sender != col.curator {
        return Err(ContractError::Unauthorized {});
    }
    if col.status != CollectionStatus::Active {
        return Err(ContractError::CollectionNotActive {});
    }

    // Check not already in collection
    if col.members.contains(&batch_denom) {
        return Err(ContractError::BatchAlreadyInCollection {
            batch_denom: batch_denom.clone(),
            collection_id,
        });
    }

    // Check quality score meets minimum
    let qs = QUALITY_SCORES
        .may_load(deps.storage, &batch_denom)?
        .ok_or(ContractError::ScoreNotFound {
            batch_denom: batch_denom.clone(),
        })?;
    if qs.score < config.min_quality_score {
        return Err(ContractError::QualityScoreTooLow {
            score: qs.score,
            min: config.min_quality_score,
        });
    }

    // Check listing fee
    if !config.listing_fee.is_zero() {
        let fee_sent = get_sent_amount(&info, &config.bond_denom)?;
        if fee_sent < config.listing_fee {
            return Err(ContractError::InsufficientBond {
                sent: fee_sent.u128(),
                min: config.listing_fee.u128(),
            });
        }
    }

    col.members.push(batch_denom.clone());
    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    Ok(Response::new()
        .add_attribute("method", "add_batch")
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("batch_denom", batch_denom))
}

// ── Remove batch ────────────────────────────────────────────────────

fn exec_remove_batch(
    deps: DepsMut,
    info: MessageInfo,
    collection_id: u64,
    batch_denom: String,
) -> Result<Response, ContractError> {
    let mut col = load_collection(deps.as_ref(), collection_id)?;

    if info.sender != col.curator {
        return Err(ContractError::Unauthorized {});
    }
    if col.status != CollectionStatus::Active {
        return Err(ContractError::CollectionNotActive {});
    }

    let pos = col
        .members
        .iter()
        .position(|m| m == &batch_denom)
        .ok_or(ContractError::BatchNotInCollection {
            batch_denom: batch_denom.clone(),
            collection_id,
        })?;
    col.members.remove(pos);
    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    Ok(Response::new()
        .add_attribute("method", "remove_batch")
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("batch_denom", batch_denom))
}

// ── Challenge inclusion ─────────────────────────────────────────────

fn exec_challenge_inclusion(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u64,
    batch_denom: String,
    reason: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut col = load_collection(deps.as_ref(), collection_id)?;

    if col.status != CollectionStatus::Active {
        return Err(ContractError::CollectionNotActive {});
    }

    // No self-challenge
    if info.sender == col.curator {
        return Err(ContractError::SelfChallenge {});
    }

    // No double challenge
    if ACTIVE_CHALLENGE
        .may_load(deps.storage, collection_id)?
        .is_some()
    {
        return Err(ContractError::PendingChallenge {});
    }

    // Check batch is in collection
    if !col.members.contains(&batch_denom) {
        return Err(ContractError::BatchNotInCollection {
            batch_denom: batch_denom.clone(),
            collection_id,
        });
    }

    // Check deposit
    let deposit_sent = get_sent_amount(&info, &config.bond_denom)?;
    if deposit_sent < config.challenge_deposit {
        return Err(ContractError::InsufficientChallengeDeposit {
            sent: deposit_sent.u128(),
            required: config.challenge_deposit.u128(),
        });
    }

    let challenge_id = CHALLENGE_SEQ.load(deps.storage)? + 1;
    CHALLENGE_SEQ.save(deps.storage, &challenge_id)?;

    let challenge = Challenge {
        id: challenge_id,
        collection_id,
        challenger: info.sender.clone(),
        batch_denom: batch_denom.clone(),
        reason: reason.clone(),
        deposit: deposit_sent,
        outcome: None,
        challenged_at_s: env.block.time.seconds(),
        resolved_at_s: None,
    };

    CHALLENGES.save(deps.storage, challenge_id, &challenge)?;
    ACTIVE_CHALLENGE.save(deps.storage, collection_id, &challenge_id)?;

    col.status = CollectionStatus::UnderReview;
    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    Ok(Response::new()
        .add_attribute("method", "challenge_inclusion")
        .add_attribute("challenge_id", challenge_id.to_string())
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("batch_denom", batch_denom)
        .add_attribute("challenger", info.sender))
}

// ── Resolve challenge ───────────────────────────────────────────────

fn exec_resolve_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    challenge_id: u64,
    outcome: ChallengeOutcome,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::OnlyAdminCanResolve {});
    }

    let mut challenge = CHALLENGES
        .may_load(deps.storage, challenge_id)?
        .ok_or(ContractError::ChallengeNotFound { id: challenge_id })?;
    let mut col = load_collection(deps.as_ref(), challenge.collection_id)?;

    if col.status != CollectionStatus::UnderReview {
        return Err(ContractError::CollectionNotUnderReview {});
    }

    challenge.outcome = Some(outcome.clone());
    challenge.resolved_at_s = Some(env.block.time.seconds());

    let mut msgs: Vec<BankMsg> = vec![];

    match outcome {
        ChallengeOutcome::CuratorWins => {
            // Challenger loses deposit — send to community pool (here: contract admin as proxy)
            msgs.push(BankMsg::Send {
                to_address: config.admin.to_string(),
                amount: vec![Coin {
                    denom: config.bond_denom.clone(),
                    amount: challenge.deposit,
                }],
            });
            col.status = CollectionStatus::Active;
        }
        ChallengeOutcome::ChallengerWins => {
            // Slash curator bond
            let slash_amount = col
                .bond_remaining
                .multiply_ratio(config.slash_pct_bps, 10_000u64);
            let slash_amount = slash_amount.min(col.bond_remaining);
            col.bond_remaining = col.bond_remaining.checked_sub(slash_amount).unwrap();

            // Challenger gets reward share of slash
            let challenger_reward =
                slash_amount.multiply_ratio(config.challenge_reward_bps, 10_000u64);
            // Rest goes to community pool (admin as proxy)
            let community_share = slash_amount.checked_sub(challenger_reward).unwrap();

            if !challenger_reward.is_zero() {
                msgs.push(BankMsg::Send {
                    to_address: challenge.challenger.to_string(),
                    amount: vec![Coin {
                        denom: config.bond_denom.clone(),
                        amount: challenger_reward,
                    }],
                });
            }
            // Return challenger's deposit
            if !challenge.deposit.is_zero() {
                msgs.push(BankMsg::Send {
                    to_address: challenge.challenger.to_string(),
                    amount: vec![Coin {
                        denom: config.bond_denom.clone(),
                        amount: challenge.deposit,
                    }],
                });
            }
            if !community_share.is_zero() {
                msgs.push(BankMsg::Send {
                    to_address: config.admin.to_string(),
                    amount: vec![Coin {
                        denom: config.bond_denom.clone(),
                        amount: community_share,
                    }],
                });
            }

            // Remove challenged batch
            if let Some(pos) = col.members.iter().position(|m| m == &challenge.batch_denom) {
                col.members.remove(pos);
            }

            // Check if bond is below minimum
            if col.bond_remaining < config.min_curation_bond {
                col.status = CollectionStatus::Suspended;
                col.suspended_at_s = Some(env.block.time.seconds());
            } else {
                col.status = CollectionStatus::Active;
            }
        }
    }

    CHALLENGES.save(deps.storage, challenge_id, &challenge)?;
    ACTIVE_CHALLENGE.remove(deps.storage, challenge.collection_id);
    COLLECTIONS.save(deps.storage, challenge.collection_id, &col)?;

    let mut resp = Response::new()
        .add_attribute("method", "resolve_challenge")
        .add_attribute("challenge_id", challenge_id.to_string())
        .add_attribute(
            "outcome",
            match &challenge.outcome {
                Some(ChallengeOutcome::CuratorWins) => "curator_wins",
                Some(ChallengeOutcome::ChallengerWins) => "challenger_wins",
                None => "none",
            },
        );

    for m in msgs {
        resp = resp.add_message(m);
    }

    Ok(resp)
}

// ── Top up bond ─────────────────────────────────────────────────────

fn exec_top_up_bond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut col = load_collection(deps.as_ref(), collection_id)?;

    if info.sender != col.curator {
        return Err(ContractError::Unauthorized {});
    }
    if col.status != CollectionStatus::Suspended {
        return Err(ContractError::CollectionNotSuspended {});
    }

    // Check top-up window hasn't expired
    let suspended_at = col.suspended_at_s.unwrap_or(0);
    if env.block.time.seconds() > suspended_at + config.top_up_window_s {
        return Err(ContractError::TopUpWindowExpired {});
    }

    let top_up = get_sent_amount(&info, &config.bond_denom)?;
    col.bond_remaining = col.bond_remaining.checked_add(top_up)?;

    if col.bond_remaining < config.min_curation_bond {
        return Err(ContractError::BondBelowMinAfterTopUp {});
    }

    col.status = CollectionStatus::Active;
    col.suspended_at_s = None;
    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    Ok(Response::new()
        .add_attribute("method", "top_up_bond")
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("top_up", top_up)
        .add_attribute("bond_remaining", col.bond_remaining))
}

// ── Close collection ────────────────────────────────────────────────

fn exec_close_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u64,
) -> Result<Response, ContractError> {
    let mut col = load_collection(deps.as_ref(), collection_id)?;

    if info.sender != col.curator {
        return Err(ContractError::Unauthorized {});
    }
    if col.status != CollectionStatus::Active {
        return Err(ContractError::CollectionNotActive {});
    }

    // No pending challenges
    if ACTIVE_CHALLENGE
        .may_load(deps.storage, collection_id)?
        .is_some()
    {
        return Err(ContractError::PendingChallenge {});
    }

    col.status = CollectionStatus::Closed;
    col.close_initiated_at_s = Some(env.block.time.seconds());
    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    Ok(Response::new()
        .add_attribute("method", "close_collection")
        .add_attribute("collection_id", collection_id.to_string()))
}

// ── Claim refund ────────────────────────────────────────────────────

fn exec_claim_refund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut col = load_collection(deps.as_ref(), collection_id)?;

    if info.sender != col.curator {
        return Err(ContractError::Unauthorized {});
    }
    if col.status != CollectionStatus::Closed {
        // Allow claim after close
        return Err(ContractError::CollectionNotActive {}); // reuse — it must be Closed
    }

    let close_at = col.close_initiated_at_s.unwrap_or(0);
    if env.block.time.seconds() < close_at + config.unbonding_period_s {
        return Err(ContractError::UnbondingNotElapsed {});
    }

    let refund = col.bond_remaining;
    col.bond_remaining = Uint128::zero();

    // Decrement curator count
    let count = CURATOR_COLLECTION_COUNT
        .may_load(deps.storage, &col.curator)?
        .unwrap_or(1);
    CURATOR_COLLECTION_COUNT.save(deps.storage, &col.curator, &count.saturating_sub(1))?;

    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    let mut resp = Response::new()
        .add_attribute("method", "claim_refund")
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("refund", refund);

    if !refund.is_zero() {
        resp = resp.add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: config.bond_denom,
                amount: refund,
            }],
        });
    }

    Ok(resp)
}

// ── Record trade ────────────────────────────────────────────────────

fn exec_record_trade(
    deps: DepsMut,
    info: MessageInfo,
    collection_id: u64,
    trade_amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let mut col = load_collection(deps.as_ref(), collection_id)?;
    if col.status != CollectionStatus::Active {
        return Err(ContractError::CollectionNotActive {});
    }

    let curation_fee = trade_amount.multiply_ratio(config.curation_fee_bps, 10_000u64);
    col.trade_volume = col.trade_volume.checked_add(trade_amount)?;
    col.total_rewards = col.total_rewards.checked_add(curation_fee)?;
    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    let mut resp = Response::new()
        .add_attribute("method", "record_trade")
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("trade_amount", trade_amount)
        .add_attribute("curation_fee", curation_fee);

    // Send curation fee to curator
    if !curation_fee.is_zero() {
        resp = resp.add_message(BankMsg::Send {
            to_address: col.curator.to_string(),
            amount: vec![Coin {
                denom: config.bond_denom,
                amount: curation_fee,
            }],
        });
    }

    Ok(resp)
}

// ── Submit quality score ────────────────────────────────────────────

fn exec_submit_quality_score(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    batch_denom: String,
    score: u64,
    confidence: u64,
    factors: QualityFactors,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::OnlyAdminCanScore {});
    }

    let qs = QualityScore {
        batch_denom: batch_denom.clone(),
        score,
        confidence,
        factors,
        scored_at_s: env.block.time.seconds(),
    };

    QUALITY_SCORES.save(deps.storage, &batch_denom, &qs)?;

    // Append to history
    let mut history = QUALITY_HISTORY
        .may_load(deps.storage, &batch_denom)?
        .unwrap_or_default();
    history.push(qs);
    QUALITY_HISTORY.save(deps.storage, &batch_denom, &history)?;

    Ok(Response::new()
        .add_attribute("method", "submit_quality_score")
        .add_attribute("batch_denom", batch_denom)
        .add_attribute("score", score.to_string())
        .add_attribute("confidence", confidence.to_string()))
}

// ── Force close suspended ───────────────────────────────────────────

fn exec_force_close_suspended(
    deps: DepsMut,
    env: Env,
    collection_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut col = load_collection(deps.as_ref(), collection_id)?;

    if col.status != CollectionStatus::Suspended {
        return Err(ContractError::CollectionNotSuspended {});
    }

    let suspended_at = col.suspended_at_s.unwrap_or(0);
    if env.block.time.seconds() <= suspended_at + config.top_up_window_s {
        return Err(ContractError::TopUpWindowNotExpired {});
    }

    col.status = CollectionStatus::Closed;
    col.close_initiated_at_s = Some(env.block.time.seconds());
    COLLECTIONS.save(deps.storage, collection_id, &col)?;

    // Decrement curator count
    let count = CURATOR_COLLECTION_COUNT
        .may_load(deps.storage, &col.curator)?
        .unwrap_or(1);
    CURATOR_COLLECTION_COUNT.save(deps.storage, &col.curator, &count.saturating_sub(1))?;

    // Refund remaining bond immediately (no unbonding for forced close)
    let refund = col.bond_remaining;
    let mut resp = Response::new()
        .add_attribute("method", "force_close_suspended")
        .add_attribute("collection_id", collection_id.to_string())
        .add_attribute("refund", refund);

    if !refund.is_zero() {
        resp = resp.add_message(BankMsg::Send {
            to_address: col.curator.to_string(),
            amount: vec![Coin {
                denom: config.bond_denom,
                amount: refund,
            }],
        });
    }

    Ok(resp)
}

// ── Update config ───────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn exec_update_config(
    deps: DepsMut,
    info: MessageInfo,
    min_curation_bond: Option<Uint128>,
    listing_fee: Option<Uint128>,
    curation_fee_bps: Option<u64>,
    challenge_deposit: Option<Uint128>,
    slash_pct_bps: Option<u64>,
    challenge_reward_bps: Option<u64>,
    activation_delay_s: Option<u64>,
    unbonding_period_s: Option<u64>,
    top_up_window_s: Option<u64>,
    min_quality_score: Option<u64>,
    max_collections_per_curator: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(v) = min_curation_bond {
        config.min_curation_bond = v;
    }
    if let Some(v) = listing_fee {
        config.listing_fee = v;
    }
    if let Some(v) = curation_fee_bps {
        config.curation_fee_bps = v;
    }
    if let Some(v) = challenge_deposit {
        config.challenge_deposit = v;
    }
    if let Some(v) = slash_pct_bps {
        config.slash_pct_bps = v;
    }
    if let Some(v) = challenge_reward_bps {
        config.challenge_reward_bps = v;
    }
    if let Some(v) = activation_delay_s {
        config.activation_delay_s = v;
    }
    if let Some(v) = unbonding_period_s {
        config.unbonding_period_s = v;
    }
    if let Some(v) = top_up_window_s {
        config.top_up_window_s = v;
    }
    if let Some(v) = min_quality_score {
        config.min_quality_score = v;
    }
    if let Some(v) = max_collections_per_curator {
        config.max_collections_per_curator = v;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "update_config"))
}

// ════════════════════════════════════════════════════════════════════
// Query
// ════════════════════════════════════════════════════════════════════

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Collection { collection_id } => {
            to_json_binary(&query_collection(deps, collection_id)?)
        }
        QueryMsg::Collections {
            curator,
            status,
            start_after,
            limit,
        } => to_json_binary(&query_collections(deps, curator, status, start_after, limit)?),
        QueryMsg::QualityScore { batch_denom } => {
            to_json_binary(&query_quality_score(deps, batch_denom)?)
        }
        QueryMsg::QualityHistory { batch_denom } => {
            to_json_binary(&query_quality_history(deps, batch_denom)?)
        }
        QueryMsg::Challenge { challenge_id } => {
            to_json_binary(&query_challenge(deps, challenge_id)?)
        }
        QueryMsg::ActiveChallenge { collection_id } => {
            to_json_binary(&query_active_challenge(deps, collection_id)?)
        }
        QueryMsg::CuratorStats { curator } => to_json_binary(&query_curator_stats(deps, curator)?),
        QueryMsg::ListingScore { batch_denom } => {
            to_json_binary(&query_listing_score(deps, batch_denom)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        bond_denom: config.bond_denom,
        min_curation_bond: config.min_curation_bond,
        listing_fee: config.listing_fee,
        curation_fee_bps: config.curation_fee_bps,
        challenge_deposit: config.challenge_deposit,
        slash_pct_bps: config.slash_pct_bps,
        challenge_reward_bps: config.challenge_reward_bps,
        activation_delay_s: config.activation_delay_s,
        unbonding_period_s: config.unbonding_period_s,
        top_up_window_s: config.top_up_window_s,
        min_quality_score: config.min_quality_score,
        max_collections_per_curator: config.max_collections_per_curator,
    })
}

fn query_collection(deps: Deps, collection_id: u64) -> StdResult<CollectionResponse> {
    let col = COLLECTIONS.load(deps.storage, collection_id)?;
    Ok(collection_to_response(&col))
}

fn query_collections(
    deps: Deps,
    curator: Option<String>,
    status: Option<String>,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<CollectionsResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.unwrap_or(0);

    let collections: Vec<CollectionResponse> = COLLECTIONS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|r| r.ok())
        .filter(|(id, _)| *id > start)
        .filter(|(_, col)| {
            if let Some(ref c) = curator {
                col.curator.as_str() == c
            } else {
                true
            }
        })
        .filter(|(_, col)| {
            if let Some(ref s) = status {
                status_to_string(&col.status) == *s
            } else {
                true
            }
        })
        .take(limit)
        .map(|(_, col)| collection_to_response(&col))
        .collect();

    Ok(CollectionsResponse { collections })
}

fn query_quality_score(deps: Deps, batch_denom: String) -> StdResult<QualityScoreResponse> {
    let qs = QUALITY_SCORES.load(deps.storage, &batch_denom)?;
    Ok(QualityScoreResponse {
        batch_denom: qs.batch_denom,
        score: qs.score,
        confidence: qs.confidence,
        factors: qs.factors,
        scored_at_s: qs.scored_at_s,
    })
}

fn query_quality_history(deps: Deps, batch_denom: String) -> StdResult<QualityHistoryResponse> {
    let history = QUALITY_HISTORY
        .may_load(deps.storage, &batch_denom)?
        .unwrap_or_default();
    Ok(QualityHistoryResponse {
        batch_denom: batch_denom.clone(),
        scores: history
            .into_iter()
            .map(|qs| QualityScoreResponse {
                batch_denom: qs.batch_denom,
                score: qs.score,
                confidence: qs.confidence,
                factors: qs.factors,
                scored_at_s: qs.scored_at_s,
            })
            .collect(),
    })
}

fn query_challenge(deps: Deps, challenge_id: u64) -> StdResult<ChallengeResponse> {
    let ch = CHALLENGES.load(deps.storage, challenge_id)?;
    Ok(challenge_to_response(&ch))
}

fn query_active_challenge(
    deps: Deps,
    collection_id: u64,
) -> StdResult<Option<ChallengeResponse>> {
    let maybe_id = ACTIVE_CHALLENGE.may_load(deps.storage, collection_id)?;
    match maybe_id {
        None => Ok(None),
        Some(id) => {
            let ch = CHALLENGES.load(deps.storage, id)?;
            Ok(Some(challenge_to_response(&ch)))
        }
    }
}

fn query_curator_stats(deps: Deps, curator: String) -> StdResult<CuratorStatsResponse> {
    let curator_addr = deps.api.addr_validate(&curator)?;
    let count = CURATOR_COLLECTION_COUNT
        .may_load(deps.storage, &curator_addr)?
        .unwrap_or(0);

    let mut total_bond = Uint128::zero();
    let mut total_rewards = Uint128::zero();

    // Iterate collections to sum up bond and rewards
    for item in COLLECTIONS.range(deps.storage, None, None, Order::Ascending) {
        let (_, col) = item?;
        if col.curator == curator_addr {
            total_bond = total_bond.checked_add(col.bond_remaining)?;
            total_rewards = total_rewards.checked_add(col.total_rewards)?;
        }
    }

    Ok(CuratorStatsResponse {
        curator,
        collection_count: count,
        total_bond,
        total_rewards,
    })
}

fn query_listing_score(deps: Deps, batch_denom: String) -> StdResult<ListingScoreResponse> {
    let qs = QUALITY_SCORES.may_load(deps.storage, &batch_denom)?;

    // Count collections containing this batch
    let mut collection_count = 0u64;
    for item in COLLECTIONS.range(deps.storage, None, None, Order::Ascending) {
        let (_, col) = item?;
        if col.members.contains(&batch_denom) && col.status == CollectionStatus::Active {
            collection_count += 1;
        }
    }

    // Featured: score >= 800, no active challenges on any containing collection, in at least one collection
    let featured = match &qs {
        Some(q) => q.score >= 800 && collection_count > 0,
        None => false,
    };

    Ok(ListingScoreResponse {
        batch_denom,
        quality_score: qs.as_ref().map(|q| q.score),
        confidence: qs.as_ref().map(|q| q.confidence),
        collection_count,
        featured,
    })
}

// ════════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════════

fn load_collection(deps: Deps, id: u64) -> Result<Collection, ContractError> {
    COLLECTIONS
        .may_load(deps.storage, id)?
        .ok_or(ContractError::CollectionNotFound { id })
}

fn get_sent_amount(info: &MessageInfo, denom: &str) -> Result<Uint128, ContractError> {
    if info.funds.is_empty() {
        return Err(ContractError::NoFundsSent {});
    }
    let coin = info
        .funds
        .iter()
        .find(|c| c.denom == denom)
        .ok_or(ContractError::WrongDenom {
            sent: info.funds[0].denom.clone(),
            expected: denom.to_string(),
        })?;
    Ok(coin.amount)
}

fn status_to_string(status: &CollectionStatus) -> String {
    match status {
        CollectionStatus::Proposed => "PROPOSED".to_string(),
        CollectionStatus::Active => "ACTIVE".to_string(),
        CollectionStatus::UnderReview => "UNDER_REVIEW".to_string(),
        CollectionStatus::Suspended => "SUSPENDED".to_string(),
        CollectionStatus::Closed => "CLOSED".to_string(),
    }
}

fn collection_to_response(col: &Collection) -> CollectionResponse {
    CollectionResponse {
        id: col.id,
        curator: col.curator.to_string(),
        name: col.name.clone(),
        description: col.description.clone(),
        criteria: col.criteria.clone(),
        bond_amount: col.bond_amount,
        bond_remaining: col.bond_remaining,
        status: status_to_string(&col.status),
        members: col.members.clone(),
        trade_volume: col.trade_volume,
        total_rewards: col.total_rewards,
        created_at_s: col.created_at_s,
        activated_at_s: col.activated_at_s,
    }
}

fn challenge_to_response(ch: &Challenge) -> ChallengeResponse {
    ChallengeResponse {
        id: ch.id,
        collection_id: ch.collection_id,
        challenger: ch.challenger.to_string(),
        batch_denom: ch.batch_denom.clone(),
        reason: ch.reason.clone(),
        deposit: ch.deposit,
        outcome: ch.outcome.as_ref().map(|o| match o {
            ChallengeOutcome::CuratorWins => "CURATOR_WINS".to_string(),
            ChallengeOutcome::ChallengerWins => "CHALLENGER_WINS".to_string(),
        }),
        challenged_at_s: ch.challenged_at_s,
        resolved_at_s: ch.resolved_at_s,
    }
}

// ════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{from_json, Coin, Timestamp};

    const DENOM: &str = "uregen";
    const MIN_BOND: u128 = 1_000_000_000; // 1000 REGEN
    const LISTING_FEE: u128 = 10_000_000; // 10 REGEN
    const CHALLENGE_DEPOSIT: u128 = 100_000_000; // 100 REGEN
    const ACTIVATION_DELAY: u64 = 172_800; // 48h
    const UNBONDING_PERIOD: u64 = 1_209_600; // 14 days
    const TOP_UP_WINDOW: u64 = 604_800; // 7 days

    fn default_instantiate_msg() -> InstantiateMsg {
        InstantiateMsg {
            bond_denom: DENOM.to_string(),
            min_curation_bond: Uint128::new(MIN_BOND),
            listing_fee: Uint128::new(LISTING_FEE),
            curation_fee_bps: 50,       // 0.5%
            challenge_deposit: Uint128::new(CHALLENGE_DEPOSIT),
            slash_pct_bps: 2000,        // 20%
            challenge_reward_bps: 5000, // 50%
            activation_delay_s: ACTIVATION_DELAY,
            unbonding_period_s: UNBONDING_PERIOD,
            top_up_window_s: TOP_UP_WINDOW,
            min_quality_score: 300,
            max_collections_per_curator: 5,
        }
    }

    fn setup() -> (
        cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        Env,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let admin = deps.api.addr_make("admin");
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), env.clone(), info, default_instantiate_msg()).unwrap();
        (deps, env)
    }

    fn default_criteria() -> CurationCriteria {
        CurationCriteria {
            min_project_reputation: None,
            min_class_reputation: None,
            allowed_credit_types: vec![],
            min_vintage_year: None,
            max_vintage_year: None,
        }
    }

    fn default_quality_factors() -> QualityFactors {
        QualityFactors {
            project_reputation: 800,
            class_reputation: 750,
            vintage_freshness: 900,
            verification_recency: 850,
            seller_reputation: 700,
            price_fairness: 950,
            additionality_confidence: 800,
        }
    }

    fn submit_score(
        deps: &mut cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        env: &Env,
        batch_denom: &str,
        score: u64,
    ) {
        let admin = deps.api.addr_make("admin");
        let info = message_info(&admin, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::SubmitQualityScore {
                batch_denom: batch_denom.to_string(),
                score,
                confidence: 1000,
                factors: default_quality_factors(),
            },
        )
        .unwrap();
    }

    fn env_at(seconds: u64) -> Env {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(seconds);
        env
    }

    // ── Instantiation tests ─────────────────────────────────────────

    #[test]
    fn test_instantiate() {
        let (deps, _env) = setup();
        let admin = deps.api.addr_make("admin");
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(res).unwrap();
        assert_eq!(config.admin, admin.to_string());
        assert_eq!(config.bond_denom, DENOM);
        assert_eq!(config.min_curation_bond, Uint128::new(MIN_BOND));
        assert_eq!(config.curation_fee_bps, 50);
        assert_eq!(config.slash_pct_bps, 2000);
        assert_eq!(config.min_quality_score, 300);
    }

    // ── Collection lifecycle tests ──────────────────────────────────

    #[test]
    fn test_create_collection() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let info = message_info(
            &curator,
            &[Coin::new(MIN_BOND, DENOM)],
        );

        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateCollection {
                name: "Carbon Premium".to_string(),
                description: "High-quality carbon credits".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();

        assert!(res
            .attributes
            .iter()
            .any(|a| a.key == "collection_id" && a.value == "1"));

        // Query it
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.curator, curator.to_string());
        assert_eq!(col.status, "PROPOSED");
        assert_eq!(col.bond_amount, Uint128::new(MIN_BOND));
    }

    #[test]
    fn test_create_collection_insufficient_bond() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let info = message_info(
            &curator,
            &[Coin::new(MIN_BOND - 1, DENOM)],
        );

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::InsufficientBond { .. }));
    }

    #[test]
    fn test_activate_collection() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let info = message_info(
            &curator,
            &[Coin::new(MIN_BOND, DENOM)],
        );

        // Create
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();

        // Try activate too early
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::ActivationDelayNotElapsed {}));

        // Activate after delay
        let later_env = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later_env,
            info,
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.status, "ACTIVE");
    }

    #[test]
    fn test_max_collections_per_curator() {
        let (mut deps, env) = setup();

        // Update config to allow only 2
        let admin = deps.api.addr_make("admin");
        let info = message_info(&admin, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateConfig {
                max_collections_per_curator: Some(2),
                min_curation_bond: None,
                listing_fee: None,
                curation_fee_bps: None,
                challenge_deposit: None,
                slash_pct_bps: None,
                challenge_reward_bps: None,
                activation_delay_s: None,
                unbonding_period_s: None,
                top_up_window_s: None,
                min_quality_score: None,
            },
        )
        .unwrap();

        let curator = deps.api.addr_make("curator1");
        let info = message_info(
            &curator,
            &[Coin::new(MIN_BOND, DENOM)],
        );

        // Create 2 collections
        for _ in 0..2 {
            execute(
                deps.as_mut(),
                env.clone(),
                info.clone(),
                ExecuteMsg::CreateCollection {
                    name: "Test".to_string(),
                    description: "".to_string(),
                    criteria: default_criteria(),
                },
            )
            .unwrap();
        }

        // Third should fail
        let err = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CreateCollection {
                name: "Third".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::MaxCollectionsReached { max: 2 }));
    }

    // ── Batch management tests ──────────────────────────────────────

    #[test]
    fn test_add_and_remove_batch() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let info = message_info(
            &curator,
            &[Coin::new(MIN_BOND, DENOM)],
        );

        // Create and activate
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();

        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        // Submit quality score for batch
        submit_score(&mut deps, &later, "C01-001", 500);

        // Add batch (with listing fee)
        let add_info = message_info(
            &curator,
            &[Coin::new(LISTING_FEE, DENOM)],
        );
        execute(
            deps.as_mut(),
            later.clone(),
            add_info,
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();

        // Query collection — batch should be there
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.members, vec!["C01-001"]);

        // Remove batch
        execute(
            deps.as_mut(),
            later,
            message_info(&curator, &[]),
            ExecuteMsg::RemoveBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert!(col.members.is_empty());
    }

    #[test]
    fn test_add_batch_quality_too_low() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");

        // Create and activate
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();

        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        // Submit low score
        submit_score(&mut deps, &later, "C01-BAD", 200);

        let err = execute(
            deps.as_mut(),
            later,
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-BAD".to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(
            err,
            ContractError::QualityScoreTooLow {
                score: 200,
                min: 300
            }
        ));
    }

    #[test]
    fn test_add_batch_duplicate() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");

        // Create and activate
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        submit_score(&mut deps, &later, "C01-001", 500);

        // Add once
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();

        // Add again — should fail
        let err = execute(
            deps.as_mut(),
            later,
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(
            err,
            ContractError::BatchAlreadyInCollection { .. }
        ));
    }

    #[test]
    fn test_add_batch_unauthorized() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let other = deps.api.addr_make("other");

        // Create and activate
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        submit_score(&mut deps, &later, "C01-001", 500);

        // Other user tries to add
        let err = execute(
            deps.as_mut(),
            later,
            message_info(&other, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    // ── Quality score tests ─────────────────────────────────────────

    #[test]
    fn test_submit_and_query_quality_score() {
        let (mut deps, env) = setup();
        let admin = deps.api.addr_make("admin");
        let info = message_info(&admin, &[]);

        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::SubmitQualityScore {
                batch_denom: "C01-001".to_string(),
                score: 818,
                confidence: 1000,
                factors: default_quality_factors(),
            },
        )
        .unwrap();

        // Query latest
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::QualityScore {
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();
        let qs: QualityScoreResponse = from_json(res).unwrap();
        assert_eq!(qs.score, 818);
        assert_eq!(qs.confidence, 1000);
        assert_eq!(qs.factors.project_reputation, 800);

        // Submit a second score — history should have 2 entries
        let admin = deps.api.addr_make("admin");
        let info2 = message_info(&admin, &[]);
        let env2 = env_at(env.block.time.seconds() + 86400);
        execute(
            deps.as_mut(),
            env2,
            info2,
            ExecuteMsg::SubmitQualityScore {
                batch_denom: "C01-001".to_string(),
                score: 790,
                confidence: 857,
                factors: QualityFactors {
                    project_reputation: 750,
                    class_reputation: 700,
                    vintage_freshness: 850,
                    verification_recency: 800,
                    seller_reputation: 650,
                    price_fairness: 900,
                    additionality_confidence: 750,
                },
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::QualityHistory {
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();
        let hist: QualityHistoryResponse = from_json(res).unwrap();
        assert_eq!(hist.scores.len(), 2);
        assert_eq!(hist.scores[0].score, 818);
        assert_eq!(hist.scores[1].score, 790);
    }

    #[test]
    fn test_submit_score_non_admin() {
        let (mut deps, env) = setup();
        let other = deps.api.addr_make("other");
        let info = message_info(&other, &[]);

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::SubmitQualityScore {
                batch_denom: "C01-001".to_string(),
                score: 500,
                confidence: 500,
                factors: default_quality_factors(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::OnlyAdminCanScore {}));
    }

    // ── Challenge tests ─────────────────────────────────────────────

    #[test]
    fn test_challenge_curator_wins() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let challenger = deps.api.addr_make("challenger1");
        let admin = deps.api.addr_make("admin");

        // Create, activate, add batch
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();

        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        submit_score(&mut deps, &later, "C01-001", 500);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();

        // Challenge
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(
                &challenger,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
                reason: "Low quality".to_string(),
            },
        )
        .unwrap();

        // Collection should be UNDER_REVIEW
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.status, "UNDER_REVIEW");

        // Resolve — curator wins
        let resolve_res = execute(
            deps.as_mut(),
            later,
            message_info(&admin, &[]),
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                outcome: ChallengeOutcome::CuratorWins,
            },
        )
        .unwrap();

        // Challenger deposit goes to admin (community pool proxy)
        assert_eq!(resolve_res.messages.len(), 1);
        let msg = &resolve_res.messages[0].msg;
        match msg {
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(*to_address, admin.to_string());
                assert_eq!(amount[0].amount, Uint128::new(CHALLENGE_DEPOSIT));
            }
            _ => panic!("Expected BankMsg::Send"),
        }

        // Collection back to ACTIVE, batch still there
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.status, "ACTIVE");
        assert!(col.members.contains(&"C01-001".to_string()));
    }

    #[test]
    fn test_challenge_challenger_wins() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let challenger = deps.api.addr_make("challenger1");
        let admin = deps.api.addr_make("admin");

        // Setup: create, activate, add batch
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();

        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        submit_score(&mut deps, &later, "C01-001", 500);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();

        // Challenge
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(
                &challenger,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
                reason: "Fraudulent".to_string(),
            },
        )
        .unwrap();

        // Resolve — challenger wins
        let resolve_res = execute(
            deps.as_mut(),
            later,
            message_info(&admin, &[]),
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                outcome: ChallengeOutcome::ChallengerWins,
            },
        )
        .unwrap();

        // Should have 3 bank messages:
        // 1. Challenger reward (50% of 20% of 1000 REGEN = 100 REGEN)
        // 2. Challenger deposit returned (100 REGEN)
        // 3. Community pool share (50% of 20% of 1000 REGEN = 100 REGEN)
        assert_eq!(resolve_res.messages.len(), 3);

        // Slash = 20% of 1_000_000_000 = 200_000_000
        // Challenger reward = 50% of 200_000_000 = 100_000_000
        // Community share = 50% of 200_000_000 = 100_000_000
        let challenger_reward = &resolve_res.messages[0].msg;
        match challenger_reward {
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(*to_address, challenger.to_string());
                assert_eq!(amount[0].amount, Uint128::new(100_000_000));
            }
            _ => panic!("Expected BankMsg::Send for challenger reward"),
        }

        // Batch removed, bond remaining reduced
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert!(!col.members.contains(&"C01-001".to_string()));
        // Bond remaining = 1_000_000_000 - 200_000_000 = 800_000_000
        assert_eq!(col.bond_remaining, Uint128::new(800_000_000));
        // 800M >= 1000M min? No — should be SUSPENDED
        assert_eq!(col.status, "SUSPENDED");
    }

    #[test]
    fn test_self_challenge_rejected() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");

        // Create, activate, add batch
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        submit_score(&mut deps, &later, "C01-001", 500);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();

        // Curator tries to challenge own collection
        let err = execute(
            deps.as_mut(),
            later,
            message_info(
                &curator,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
                reason: "Self-challenge".to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::SelfChallenge {}));
    }

    // ── Suspension & recovery tests ─────────────────────────────────

    #[test]
    fn test_suspension_top_up_recovery() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let challenger = deps.api.addr_make("challenger1");
        let admin = deps.api.addr_make("admin");

        // Create with exact minimum bond, activate, add batch, challenge + lose
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();
        submit_score(&mut deps, &later, "C01-001", 500);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();

        // Challenge and challenger wins -> suspended
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(
                &challenger,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
                reason: "Bad".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&admin, &[]),
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                outcome: ChallengeOutcome::ChallengerWins,
            },
        )
        .unwrap();

        // Confirm suspended
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.status, "SUSPENDED");

        // Top up to recover — need to bring bond back above minimum
        // Bond remaining is 800M, need at least 1000M, so send 200M+
        let top_up_env = env_at(later.block.time.seconds() + 3600); // 1 hour later, within window
        execute(
            deps.as_mut(),
            top_up_env,
            message_info(
                &curator,
                &[Coin::new(200_000_000u128, DENOM)],
            ),
            ExecuteMsg::TopUpBond { collection_id: 1 },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.status, "ACTIVE");
        assert_eq!(col.bond_remaining, Uint128::new(1_000_000_000));
    }

    #[test]
    fn test_top_up_window_expired() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let challenger = deps.api.addr_make("challenger1");
        let admin = deps.api.addr_make("admin");

        // Create, activate, add batch, challenge + lose (suspended)
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();
        submit_score(&mut deps, &later, "C01-001", 500);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(
                &challenger,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
                reason: "Bad".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&admin, &[]),
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                outcome: ChallengeOutcome::ChallengerWins,
            },
        )
        .unwrap();

        // Try top up after window expired
        let expired_env = env_at(later.block.time.seconds() + TOP_UP_WINDOW + 1);
        let err = execute(
            deps.as_mut(),
            expired_env,
            message_info(
                &curator,
                &[Coin::new(200_000_000u128, DENOM)],
            ),
            ExecuteMsg::TopUpBond { collection_id: 1 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::TopUpWindowExpired {}));
    }

    // ── Force close suspended tests ─────────────────────────────────

    #[test]
    fn test_force_close_suspended() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let challenger = deps.api.addr_make("challenger1");
        let admin = deps.api.addr_make("admin");
        let anyone = deps.api.addr_make("anyone");

        // Create, activate, add, challenge, lose -> suspended
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();
        submit_score(&mut deps, &later, "C01-001", 500);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(
                &challenger,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
                reason: "Bad".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&admin, &[]),
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                outcome: ChallengeOutcome::ChallengerWins,
            },
        )
        .unwrap();

        // Too early
        let too_early = env_at(later.block.time.seconds() + TOP_UP_WINDOW - 1);
        let err = execute(
            deps.as_mut(),
            too_early,
            message_info(&anyone, &[]),
            ExecuteMsg::ForceCloseSuspended { collection_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::TopUpWindowNotExpired {}));

        // After window
        let after_window = env_at(later.block.time.seconds() + TOP_UP_WINDOW + 1);
        let res = execute(
            deps.as_mut(),
            after_window,
            message_info(&anyone, &[]),
            ExecuteMsg::ForceCloseSuspended { collection_id: 1 },
        )
        .unwrap();

        // Should refund remaining bond to curator
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(*to_address, curator.to_string());
                assert_eq!(amount[0].amount, Uint128::new(800_000_000));
            }
            _ => panic!("Expected BankMsg::Send"),
        }

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.status, "CLOSED");
    }

    // ── Close and refund tests ──────────────────────────────────────

    #[test]
    fn test_close_and_claim_refund() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");

        // Create and activate
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        // Close
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::CloseCollection { collection_id: 1 },
        )
        .unwrap();

        // Try claim too early
        let too_early = env_at(later.block.time.seconds() + UNBONDING_PERIOD - 1);
        let err = execute(
            deps.as_mut(),
            too_early,
            message_info(&curator, &[]),
            ExecuteMsg::ClaimRefund { collection_id: 1 },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::UnbondingNotElapsed {}));

        // Claim after unbonding
        let after_unbonding = env_at(later.block.time.seconds() + UNBONDING_PERIOD + 1);
        let res = execute(
            deps.as_mut(),
            after_unbonding,
            message_info(&curator, &[]),
            ExecuteMsg::ClaimRefund { collection_id: 1 },
        )
        .unwrap();

        // Should refund full bond
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(*to_address, curator.to_string());
                assert_eq!(amount[0].amount, Uint128::new(MIN_BOND));
            }
            _ => panic!("Expected BankMsg::Send"),
        }

        // Curator collection count should be decremented
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CuratorStats {
                curator: curator.to_string(),
            },
        )
        .unwrap();
        let stats: CuratorStatsResponse = from_json(res).unwrap();
        assert_eq!(stats.collection_count, 0);
    }

    // ── Trade recording tests ───────────────────────────────────────

    #[test]
    fn test_record_trade() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let admin = deps.api.addr_make("admin");

        // Create and activate
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        // Record a trade of 10,000 REGEN (10_000_000_000 uregen)
        let trade_amount = Uint128::new(10_000_000_000u128);
        let res = execute(
            deps.as_mut(),
            later,
            message_info(&admin, &[]),
            ExecuteMsg::RecordTrade {
                collection_id: 1,
                trade_amount,
            },
        )
        .unwrap();

        // Curation fee = 0.5% of 10B = 50M
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(*to_address, curator.to_string());
                assert_eq!(amount[0].amount, Uint128::new(50_000_000));
            }
            _ => panic!("Expected BankMsg::Send"),
        }

        // Check collection stats
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.trade_volume, trade_amount);
        assert_eq!(col.total_rewards, Uint128::new(50_000_000));
    }

    // ── Listing score / featured tests ──────────────────────────────

    #[test]
    fn test_listing_score_featured() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");

        // Submit high quality score
        submit_score(&mut deps, &env, "C01-PREMIUM", 850);

        // Before any collection — not featured
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListingScore {
                batch_denom: "C01-PREMIUM".to_string(),
            },
        )
        .unwrap();
        let ls: ListingScoreResponse = from_json(res).unwrap();
        assert_eq!(ls.quality_score, Some(850));
        assert_eq!(ls.collection_count, 0);
        assert!(!ls.featured); // not in any collection

        // Create, activate, add batch
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Premium".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-PREMIUM".to_string(),
            },
        )
        .unwrap();

        // Now featured
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListingScore {
                batch_denom: "C01-PREMIUM".to_string(),
            },
        )
        .unwrap();
        let ls: ListingScoreResponse = from_json(res).unwrap();
        assert_eq!(ls.quality_score, Some(850));
        assert_eq!(ls.collection_count, 1);
        assert!(ls.featured);
    }

    #[test]
    fn test_listing_score_not_featured_low_score() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");

        submit_score(&mut deps, &env, "C01-LOW", 400);

        // Create, activate, add batch
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Budget".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-LOW".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ListingScore {
                batch_denom: "C01-LOW".to_string(),
            },
        )
        .unwrap();
        let ls: ListingScoreResponse = from_json(res).unwrap();
        assert_eq!(ls.quality_score, Some(400));
        assert_eq!(ls.collection_count, 1);
        assert!(!ls.featured); // score < 800
    }

    // ── Query tests ─────────────────────────────────────────────────

    #[test]
    fn test_query_collections_filtered() {
        let (mut deps, env) = setup();
        let curator1 = deps.api.addr_make("curator1");
        let curator2 = deps.api.addr_make("curator2");

        // Create 2 collections from different curators
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator1, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "C1 Collection".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator2, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "C2 Collection".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();

        // Activate first
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later,
            message_info(&curator1, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        // Query all
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collections {
                curator: None,
                status: None,
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let cols: CollectionsResponse = from_json(res).unwrap();
        assert_eq!(cols.collections.len(), 2);

        // Query by curator
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collections {
                curator: Some(curator1.to_string()),
                status: None,
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let cols: CollectionsResponse = from_json(res).unwrap();
        assert_eq!(cols.collections.len(), 1);
        assert_eq!(cols.collections[0].name, "C1 Collection");

        // Query by status
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collections {
                curator: None,
                status: Some("ACTIVE".to_string()),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let cols: CollectionsResponse = from_json(res).unwrap();
        assert_eq!(cols.collections.len(), 1);
        assert_eq!(cols.collections[0].name, "C1 Collection");
    }

    #[test]
    fn test_query_curator_stats() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let admin = deps.api.addr_make("admin");

        // Create collection
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();

        // Activate and record trade
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later,
            message_info(&admin, &[]),
            ExecuteMsg::RecordTrade {
                collection_id: 1,
                trade_amount: Uint128::new(1_000_000_000),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CuratorStats {
                curator: curator.to_string(),
            },
        )
        .unwrap();
        let stats: CuratorStatsResponse = from_json(res).unwrap();
        assert_eq!(stats.collection_count, 1);
        assert_eq!(stats.total_bond, Uint128::new(MIN_BOND));
        assert_eq!(stats.total_rewards, Uint128::new(5_000_000)); // 0.5% of 1B
    }

    #[test]
    fn test_query_active_challenge() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let challenger = deps.api.addr_make("challenger1");

        // Setup
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();
        submit_score(&mut deps, &later, "C01-001", 500);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
            },
        )
        .unwrap();

        // No active challenge initially
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ActiveChallenge { collection_id: 1 },
        )
        .unwrap();
        let ac: Option<ChallengeResponse> = from_json(res).unwrap();
        assert!(ac.is_none());

        // Create challenge
        execute(
            deps.as_mut(),
            later,
            message_info(
                &challenger,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-001".to_string(),
                reason: "Suspect".to_string(),
            },
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::ActiveChallenge { collection_id: 1 },
        )
        .unwrap();
        let ac: Option<ChallengeResponse> = from_json(res).unwrap();
        assert!(ac.is_some());
        let ch = ac.unwrap();
        assert_eq!(ch.challenger, challenger.to_string());
        assert_eq!(ch.batch_denom, "C01-001");
    }

    // ── Update config tests ─────────────────────────────────────────

    #[test]
    fn test_update_config() {
        let (mut deps, env) = setup();
        let admin = deps.api.addr_make("admin");

        execute(
            deps.as_mut(),
            env,
            message_info(&admin, &[]),
            ExecuteMsg::UpdateConfig {
                min_curation_bond: Some(Uint128::new(2_000_000_000)),
                listing_fee: None,
                curation_fee_bps: Some(100), // 1%
                challenge_deposit: None,
                slash_pct_bps: None,
                challenge_reward_bps: None,
                activation_delay_s: None,
                unbonding_period_s: None,
                top_up_window_s: None,
                min_quality_score: Some(500),
                max_collections_per_curator: None,
            },
        )
        .unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(res).unwrap();
        assert_eq!(config.min_curation_bond, Uint128::new(2_000_000_000));
        assert_eq!(config.curation_fee_bps, 100);
        assert_eq!(config.min_quality_score, 500);
        // Unchanged values
        assert_eq!(config.listing_fee, Uint128::new(LISTING_FEE));
    }

    #[test]
    fn test_update_config_unauthorized() {
        let (mut deps, env) = setup();
        let other = deps.api.addr_make("other");

        let err = execute(
            deps.as_mut(),
            env,
            message_info(&other, &[]),
            ExecuteMsg::UpdateConfig {
                min_curation_bond: Some(Uint128::new(1)),
                listing_fee: None,
                curation_fee_bps: None,
                challenge_deposit: None,
                slash_pct_bps: None,
                challenge_reward_bps: None,
                activation_delay_s: None,
                unbonding_period_s: None,
                top_up_window_s: None,
                min_quality_score: None,
                max_collections_per_curator: None,
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized {}));
    }

    // ── Full lifecycle acceptance test ───────────────────────────────

    #[test]
    fn test_full_collection_lifecycle() {
        // Acceptance test 1: Curator bonds -> creates collection -> adds batches
        // -> earns trade fees -> closes -> bond refunded
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let admin = deps.api.addr_make("admin");

        // 1. Create collection
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(
                &curator,
                &[Coin::new(2 * MIN_BOND, DENOM)],
            ),
            ExecuteMsg::CreateCollection {
                name: "Premium Carbon".to_string(),
                description: "Top-tier carbon credits".to_string(),
                criteria: CurationCriteria {
                    min_project_reputation: Some(500),
                    min_class_reputation: Some(400),
                    allowed_credit_types: vec!["C".to_string()],
                    min_vintage_year: Some(2024),
                    max_vintage_year: None,
                },
            },
        )
        .unwrap();

        // 2. Activate
        let t1 = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            t1.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();

        // 3. Submit scores and add batches
        submit_score(&mut deps, &t1, "C01-ALPHA", 818);
        submit_score(&mut deps, &t1, "C01-BETA", 648);

        execute(
            deps.as_mut(),
            t1.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-ALPHA".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            t1.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-BETA".to_string(),
            },
        )
        .unwrap();

        // 4. Record trades
        execute(
            deps.as_mut(),
            t1.clone(),
            message_info(&admin, &[]),
            ExecuteMsg::RecordTrade {
                collection_id: 1,
                trade_amount: Uint128::new(5_000_000_000),
            },
        )
        .unwrap();

        // 5. Verify state
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Collection { collection_id: 1 },
        )
        .unwrap();
        let col: CollectionResponse = from_json(res).unwrap();
        assert_eq!(col.status, "ACTIVE");
        assert_eq!(col.members.len(), 2);
        assert_eq!(col.trade_volume, Uint128::new(5_000_000_000));
        assert_eq!(col.total_rewards, Uint128::new(25_000_000)); // 0.5% of 5B

        // 6. Close
        execute(
            deps.as_mut(),
            t1.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::CloseCollection { collection_id: 1 },
        )
        .unwrap();

        // 7. Claim refund after unbonding
        let t2 = env_at(t1.block.time.seconds() + UNBONDING_PERIOD + 1);
        let res = execute(
            deps.as_mut(),
            t2,
            message_info(&curator, &[]),
            ExecuteMsg::ClaimRefund { collection_id: 1 },
        )
        .unwrap();

        // Full bond refunded
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            cosmwasm_std::CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(*to_address, curator.to_string());
                assert_eq!(amount[0].amount, Uint128::new(2 * MIN_BOND));
            }
            _ => panic!("Expected BankMsg::Send"),
        }
    }

    // ── Price fairness reference test ───────────────────────────────

    #[test]
    fn test_price_fairness_calculation() {
        // Acceptance test 8 from spec: at median -> 1000; 50% above -> 0
        // This validates the reference impl logic is correctly applicable

        // At median: deviation = 0, fairness = max(0, (1.0 - 0) * 1000) = 1000
        let at_median = compute_price_fairness(100, 100);
        assert_eq!(at_median, 1000);

        // 25% above: deviation = 0.25, fairness = max(0, (1.0 - 0.5) * 1000) = 500
        let above_25 = compute_price_fairness(125, 100);
        assert_eq!(above_25, 500);

        // 50% above: deviation = 0.5, fairness = max(0, (1.0 - 1.0) * 1000) = 0
        let above_50 = compute_price_fairness(150, 100);
        assert_eq!(above_50, 0);

        // 50% below: deviation = 0.5, fairness = 0
        let below_50 = compute_price_fairness(50, 100);
        assert_eq!(below_50, 0);

        // Invalid median
        let bad_median = compute_price_fairness(100, 0);
        assert_eq!(bad_median, 0);
    }

    /// Helper: mirrors the JS reference implementation price fairness formula
    fn compute_price_fairness(listing_price: u64, median_price: u64) -> u64 {
        if median_price == 0 {
            return 0;
        }
        let deviation =
            (listing_price as f64 - median_price as f64).abs() / median_price as f64;
        let raw = (1.0 - deviation * 2.0) * 1000.0;
        if raw < 0.0 {
            0
        } else {
            raw.round() as u64
        }
    }

    // ── Quality score matches reference impl test vectors ───────────

    #[test]
    fn test_quality_score_matches_reference_vectors() {
        // Validate against the test vectors from the JS reference implementation

        struct TestCase {
            factors: [u64; 7], // project, class, vintage, verification, seller, price, additionality
            expected_score: u64,
        }

        let cases = vec![
            TestCase {
                factors: [800, 750, 900, 850, 700, 950, 800],
                expected_score: 818,
            },
            TestCase {
                factors: [600, 700, 700, 600, 500, 800, 650],
                expected_score: 648,
            },
            TestCase {
                factors: [400, 500, 950, 900, 300, 600, 500],
                expected_score: 593,
            },
            TestCase {
                factors: [900, 850, 500, 400, 800, 1000, 900],
                expected_score: 755,
            },
            TestCase {
                factors: [200, 300, 800, 200, 100, 400, 300],
                expected_score: 325,
            },
        ];

        for (i, tc) in cases.iter().enumerate() {
            let score = compute_quality_score(
                tc.factors[0],
                tc.factors[1],
                tc.factors[2],
                tc.factors[3],
                tc.factors[4],
                tc.factors[5],
                tc.factors[6],
            );
            assert_eq!(
                score, tc.expected_score,
                "Test vector {}: expected {}, got {}",
                i, tc.expected_score, score
            );
        }
    }

    /// Helper: mirrors the JS reference implementation scoring formula
    fn compute_quality_score(
        project_rep: u64,
        class_rep: u64,
        vintage: u64,
        verification: u64,
        seller_rep: u64,
        price: u64,
        additionality: u64,
    ) -> u64 {
        let score = 0.25 * project_rep as f64
            + 0.20 * class_rep as f64
            + 0.15 * vintage as f64
            + 0.15 * verification as f64
            + 0.10 * seller_rep as f64
            + 0.10 * price as f64
            + 0.05 * additionality as f64;
        score.round() as u64
    }

    // ── Wrong denom test ────────────────────────────────────────────

    #[test]
    fn test_wrong_denom_rejected() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");

        let err = execute(
            deps.as_mut(),
            env,
            message_info(
                &curator,
                &[Coin::new(MIN_BOND, "uatom")],
            ),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::WrongDenom { .. }));
    }

    #[test]
    fn test_no_funds_sent() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");

        let err = execute(
            deps.as_mut(),
            env,
            message_info(&curator, &[]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::NoFundsSent {}));
    }

    // ── Double challenge rejected ───────────────────────────────────

    #[test]
    fn test_double_challenge_rejected() {
        let (mut deps, env) = setup();
        let curator = deps.api.addr_make("curator1");
        let challenger1 = deps.api.addr_make("challenger1");
        let challenger2 = deps.api.addr_make("challenger2");

        // Setup: create, activate, add 2 batches
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&curator, &[Coin::new(MIN_BOND, DENOM)]),
            ExecuteMsg::CreateCollection {
                name: "Test".to_string(),
                description: "".to_string(),
                criteria: default_criteria(),
            },
        )
        .unwrap();
        let later = env_at(env.block.time.seconds() + ACTIVATION_DELAY + 1);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[]),
            ExecuteMsg::ActivateCollection { collection_id: 1 },
        )
        .unwrap();
        submit_score(&mut deps, &later, "C01-A", 500);
        submit_score(&mut deps, &later, "C01-B", 500);
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-A".to_string(),
            },
        )
        .unwrap();
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(&curator, &[Coin::new(LISTING_FEE, DENOM)]),
            ExecuteMsg::AddBatch {
                collection_id: 1,
                batch_denom: "C01-B".to_string(),
            },
        )
        .unwrap();

        // First challenge succeeds
        execute(
            deps.as_mut(),
            later.clone(),
            message_info(
                &challenger1,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-A".to_string(),
                reason: "Bad".to_string(),
            },
        )
        .unwrap();

        // Second challenge fails — already under review
        let err = execute(
            deps.as_mut(),
            later,
            message_info(
                &challenger2,
                &[Coin::new(CHALLENGE_DEPOSIT, DENOM)],
            ),
            ExecuteMsg::ChallengeInclusion {
                collection_id: 1,
                batch_denom: "C01-B".to_string(),
                reason: "Also bad".to_string(),
            },
        )
        .unwrap_err();

        // Collection is UNDER_REVIEW, not ACTIVE
        assert!(matches!(err, ContractError::CollectionNotActive {}));
    }
}
