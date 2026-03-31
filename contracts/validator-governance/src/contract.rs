use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    CompositionResponse, ConfigResponse, ExecuteMsg, FactorsResponse, InstantiateMsg,
    PerformanceScoreResponse, ProposalListResponse, ProposalResponse, ProposalStatusFilter,
    QueryMsg, ValidatorResponse, ValidatorSetResponse, VotesResponse,
};
use crate::state::{
    Config, PerformanceScores, Proposal, ProposalStatus, ProposalType, Validator,
    ValidatorCategory, ValidatorStatus, Vote, VoteOption, CONFIG, NEXT_PROPOSAL_ID, PROPOSALS,
    VALIDATORS, VOTES,
};

const CONTRACT_NAME: &str = "crates.io:regen-validator-governance";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Instantiate ──────────────────────────────────────────────────────

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = deps.api.addr_validate(&msg.admin)?;

    let config = Config {
        admin,
        max_validators: msg.max_validators.unwrap_or(21),
        min_validators: msg.min_validators.unwrap_or(15),
        min_per_category: msg.min_per_category.unwrap_or(5),
        term_length_seconds: msg.term_length_seconds.unwrap_or(31_536_000), // 12 months
        probation_period_seconds: msg.probation_period_seconds.unwrap_or(2_592_000), // 30 days
        min_uptime_bps: msg.min_uptime_bps.unwrap_or(9950),                // 99.50%
        performance_threshold_bps: msg.performance_threshold_bps.unwrap_or(7000), // 70%
        performance_bonus_bps: msg.performance_bonus_bps.unwrap_or(1000),  // 10%
        voting_period_seconds: msg.voting_period_seconds.unwrap_or(604_800), // 7 days
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    NEXT_PROPOSAL_ID.save(deps.storage, &1u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", config.admin.as_str()))
}

// ── Execute ──────────────────────────────────────────────────────────

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ApplyValidator { moniker, category } => {
            execute_apply_validator(deps, env, info, moniker, category)
        }
        ExecuteMsg::ApproveValidator { address } => {
            execute_approve_validator(deps, info, address)
        }
        ExecuteMsg::ActivateValidator { address } => {
            execute_activate_validator(deps, env, info, address)
        }
        ExecuteMsg::PutOnProbation { address, reason } => {
            execute_put_on_probation(deps, env, info, address, reason)
        }
        ExecuteMsg::RestoreValidator { address } => {
            execute_restore_validator(deps, info, address)
        }
        ExecuteMsg::RemoveValidator { address, reason } => {
            execute_remove_validator(deps, info, address, reason)
        }
        ExecuteMsg::Reapply { address } => execute_reapply(deps, info, address),
        ExecuteMsg::UpdateScores {
            address,
            uptime_bps,
            governance_participation_bps,
            ecosystem_contribution_bps,
        } => execute_update_scores(
            deps,
            info,
            address,
            uptime_bps,
            governance_participation_bps,
            ecosystem_contribution_bps,
        ),
        ExecuteMsg::CreateProposal {
            title,
            description,
            proposal_type,
        } => execute_create_proposal(deps, env, info, title, description, proposal_type),
        ExecuteMsg::CastVote { proposal_id, vote } => {
            execute_cast_vote(deps, env, info, proposal_id, vote)
        }
        ExecuteMsg::ExecuteProposal { proposal_id } => {
            execute_execute_proposal(deps, env, info, proposal_id)
        }
        ExecuteMsg::UpdateConfig {
            max_validators,
            min_validators,
            min_per_category,
            term_length_seconds,
            probation_period_seconds,
            min_uptime_bps,
            performance_threshold_bps,
            performance_bonus_bps,
            voting_period_seconds,
        } => execute_update_config(
            deps,
            info,
            max_validators,
            min_validators,
            min_per_category,
            term_length_seconds,
            probation_period_seconds,
            min_uptime_bps,
            performance_threshold_bps,
            performance_bonus_bps,
            voting_period_seconds,
        ),
    }
}

fn execute_apply_validator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    moniker: String,
    category: ValidatorCategory,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Check if validator already exists
    if VALIDATORS.has(deps.storage, &info.sender) {
        return Err(ContractError::ValidatorAlreadyExists {
            address: info.sender.to_string(),
        });
    }

    // Check total count against max
    let total = count_validators_by_status(deps.as_ref(), None)?;
    if total >= config.max_validators {
        return Err(ContractError::ValidatorSetFull {
            current: total,
            max: config.max_validators,
        });
    }

    let validator = Validator {
        address: info.sender.clone(),
        moniker: moniker.clone(),
        category,
        status: ValidatorStatus::Candidate,
        term_start: 0,
        term_end: 0,
        performance: PerformanceScores {
            uptime_bps: None,
            governance_participation_bps: None,
            ecosystem_contribution_bps: None,
        },
        probation_start: 0,
        removal_reason: String::new(),
    };

    VALIDATORS.save(deps.storage, &info.sender, &validator)?;

    Ok(Response::new()
        .add_attribute("action", "apply_validator")
        .add_attribute("address", info.sender.as_str())
        .add_attribute("moniker", moniker)
        .add_attribute("timestamp", env.block.time.seconds().to_string()))
}

fn execute_approve_validator(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let addr = deps.api.addr_validate(&address)?;
    let mut validator = VALIDATORS
        .load(deps.storage, &addr)
        .map_err(|_| ContractError::ValidatorNotFound {
            address: address.clone(),
        })?;

    if !matches!(validator.status, ValidatorStatus::Candidate) {
        return Err(ContractError::InvalidStatusTransition {
            from: validator.status.to_string(),
            to: "approved".to_string(),
        });
    }

    validator.status = ValidatorStatus::Approved;
    VALIDATORS.save(deps.storage, &addr, &validator)?;

    Ok(Response::new()
        .add_attribute("action", "approve_validator")
        .add_attribute("address", address))
}

fn execute_activate_validator(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let addr = deps.api.addr_validate(&address)?;
    let mut validator = VALIDATORS
        .load(deps.storage, &addr)
        .map_err(|_| ContractError::ValidatorNotFound {
            address: address.clone(),
        })?;

    if !matches!(validator.status, ValidatorStatus::Approved) {
        return Err(ContractError::InvalidStatusTransition {
            from: validator.status.to_string(),
            to: "active".to_string(),
        });
    }

    // Check we don't exceed max active validators
    let active_count = count_validators_by_status(deps.as_ref(), Some(ValidatorStatus::Active))?;
    if active_count >= config.max_validators {
        return Err(ContractError::ValidatorSetFull {
            current: active_count,
            max: config.max_validators,
        });
    }

    let now = env.block.time.seconds();
    validator.status = ValidatorStatus::Active;
    validator.term_start = now;
    validator.term_end = now + config.term_length_seconds;
    VALIDATORS.save(deps.storage, &addr, &validator)?;

    Ok(Response::new()
        .add_attribute("action", "activate_validator")
        .add_attribute("address", address)
        .add_attribute("term_start", now.to_string())
        .add_attribute("term_end", validator.term_end.to_string()))
}

fn execute_put_on_probation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    reason: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let addr = deps.api.addr_validate(&address)?;
    let mut validator = VALIDATORS
        .load(deps.storage, &addr)
        .map_err(|_| ContractError::ValidatorNotFound {
            address: address.clone(),
        })?;

    if !matches!(validator.status, ValidatorStatus::Active) {
        return Err(ContractError::InvalidStatusTransition {
            from: validator.status.to_string(),
            to: "probation".to_string(),
        });
    }

    let now = env.block.time.seconds();
    validator.status = ValidatorStatus::Probation;
    validator.probation_start = now;
    VALIDATORS.save(deps.storage, &addr, &validator)?;

    Ok(Response::new()
        .add_attribute("action", "put_on_probation")
        .add_attribute("address", address)
        .add_attribute("reason", reason)
        .add_attribute("probation_start", now.to_string()))
}

fn execute_restore_validator(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let addr = deps.api.addr_validate(&address)?;
    let mut validator = VALIDATORS
        .load(deps.storage, &addr)
        .map_err(|_| ContractError::ValidatorNotFound {
            address: address.clone(),
        })?;

    if !matches!(validator.status, ValidatorStatus::Probation) {
        return Err(ContractError::InvalidStatusTransition {
            from: validator.status.to_string(),
            to: "active".to_string(),
        });
    }

    validator.status = ValidatorStatus::Active;
    validator.probation_start = 0;
    VALIDATORS.save(deps.storage, &addr, &validator)?;

    Ok(Response::new()
        .add_attribute("action", "restore_validator")
        .add_attribute("address", address))
}

fn execute_remove_validator(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    reason: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let addr = deps.api.addr_validate(&address)?;
    let mut validator = VALIDATORS
        .load(deps.storage, &addr)
        .map_err(|_| ContractError::ValidatorNotFound {
            address: address.clone(),
        })?;

    if matches!(validator.status, ValidatorStatus::Removed) {
        return Err(ContractError::InvalidStatusTransition {
            from: "removed".to_string(),
            to: "removed".to_string(),
        });
    }

    // Check composition guarantee: don't drop a category below minimum
    if matches!(
        validator.status,
        ValidatorStatus::Active | ValidatorStatus::Probation
    ) {
        let category_count =
            count_active_validators_by_category(deps.as_ref(), &validator.category)?;
        if category_count <= config.min_per_category {
            return Err(ContractError::CompositionViolation {
                category: validator.category.to_string(),
                min: config.min_per_category,
            });
        }

        // Check minimum active validators
        let active_count =
            count_validators_by_status(deps.as_ref(), Some(ValidatorStatus::Active))?;
        let probation_count =
            count_validators_by_status(deps.as_ref(), Some(ValidatorStatus::Probation))?;
        let working_count = active_count + probation_count;
        if working_count <= config.min_validators {
            return Err(ContractError::MinimumValidatorsReached {
                min: config.min_validators,
            });
        }
    }

    validator.status = ValidatorStatus::Removed;
    validator.removal_reason = reason.clone();
    VALIDATORS.save(deps.storage, &addr, &validator)?;

    Ok(Response::new()
        .add_attribute("action", "remove_validator")
        .add_attribute("address", address)
        .add_attribute("reason", reason))
}

fn execute_reapply(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let addr = deps.api.addr_validate(&address)?;

    // Only admin or the validator themselves can reapply
    if info.sender != config.admin && info.sender != addr {
        return Err(ContractError::Unauthorized);
    }

    let mut validator = VALIDATORS
        .load(deps.storage, &addr)
        .map_err(|_| ContractError::ValidatorNotFound {
            address: address.clone(),
        })?;

    if !matches!(validator.status, ValidatorStatus::TermExpired) {
        return Err(ContractError::InvalidStatusTransition {
            from: validator.status.to_string(),
            to: "candidate".to_string(),
        });
    }

    validator.status = ValidatorStatus::Candidate;
    validator.term_start = 0;
    validator.term_end = 0;
    validator.probation_start = 0;
    VALIDATORS.save(deps.storage, &addr, &validator)?;

    Ok(Response::new()
        .add_attribute("action", "reapply")
        .add_attribute("address", address))
}

fn execute_update_scores(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    uptime_bps: Option<u16>,
    governance_participation_bps: Option<u16>,
    ecosystem_contribution_bps: Option<u16>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    // Validate score ranges
    for score in [uptime_bps, governance_participation_bps, ecosystem_contribution_bps]
        .iter()
        .flatten()
    {
        if *score > 10000 {
            return Err(ContractError::InvalidScore);
        }
    }

    let addr = deps.api.addr_validate(&address)?;
    let mut validator = VALIDATORS
        .load(deps.storage, &addr)
        .map_err(|_| ContractError::ValidatorNotFound {
            address: address.clone(),
        })?;

    if let Some(v) = uptime_bps {
        validator.performance.uptime_bps = Some(v);
    }
    if let Some(v) = governance_participation_bps {
        validator.performance.governance_participation_bps = Some(v);
    }
    if let Some(v) = ecosystem_contribution_bps {
        validator.performance.ecosystem_contribution_bps = Some(v);
    }

    VALIDATORS.save(deps.storage, &addr, &validator)?;

    Ok(Response::new()
        .add_attribute("action", "update_scores")
        .add_attribute("address", address))
}

fn execute_create_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    description: String,
    proposal_type: ProposalType,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only active validators can create proposals
    let validator = VALIDATORS
        .load(deps.storage, &info.sender)
        .map_err(|_| ContractError::NotActiveValidator {
            address: info.sender.to_string(),
        })?;

    if !matches!(validator.status, ValidatorStatus::Active) {
        return Err(ContractError::NotActiveValidator {
            address: info.sender.to_string(),
        });
    }

    let id = NEXT_PROPOSAL_ID.load(deps.storage)?;
    let now = env.block.time.seconds();

    let proposal = Proposal {
        id,
        proposer: info.sender,
        title: title.clone(),
        description,
        proposal_type,
        status: ProposalStatus::Active,
        start_time: now,
        end_time: now + config.voting_period_seconds,
        yes_votes: 0,
        no_votes: 0,
        abstain_votes: 0,
    };

    PROPOSALS.save(deps.storage, id, &proposal)?;
    NEXT_PROPOSAL_ID.save(deps.storage, &(id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "create_proposal")
        .add_attribute("proposal_id", id.to_string())
        .add_attribute("title", title))
}

fn execute_cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote_option: VoteOption,
) -> Result<Response, ContractError> {
    // Must be an active validator
    let validator = VALIDATORS
        .load(deps.storage, &info.sender)
        .map_err(|_| ContractError::NotActiveValidator {
            address: info.sender.to_string(),
        })?;

    if !matches!(validator.status, ValidatorStatus::Active) {
        return Err(ContractError::NotActiveValidator {
            address: info.sender.to_string(),
        });
    }

    let mut proposal = PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound { id: proposal_id })?;

    if !matches!(proposal.status, ProposalStatus::Active) {
        return Err(ContractError::ProposalNotActive { id: proposal_id });
    }

    let now = env.block.time.seconds();
    if now > proposal.end_time {
        return Err(ContractError::VotingPeriodExpired { id: proposal_id });
    }

    // Check if already voted
    if VOTES.has(deps.storage, (proposal_id, &info.sender)) {
        return Err(ContractError::AlreadyVoted { id: proposal_id });
    }

    // Weight is the voter's composite performance score (bps, minimum 1)
    let weight = compute_composite_score(&validator.performance).max(1) as u64;

    match vote_option {
        VoteOption::Yes => proposal.yes_votes += weight,
        VoteOption::No => proposal.no_votes += weight,
        VoteOption::Abstain => proposal.abstain_votes += weight,
    }

    let vote = Vote {
        voter: info.sender.clone(),
        option: vote_option,
        weight,
    };

    VOTES.save(deps.storage, (proposal_id, &info.sender), &vote)?;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "cast_vote")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("voter", info.sender.as_str())
        .add_attribute("weight", weight.to_string()))
}

fn execute_execute_proposal(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let mut proposal = PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalNotFound { id: proposal_id })?;

    if !matches!(proposal.status, ProposalStatus::Active) {
        return Err(ContractError::ProposalNotActive { id: proposal_id });
    }

    let now = env.block.time.seconds();
    if now <= proposal.end_time {
        return Err(ContractError::VotingPeriodNotExpired { id: proposal_id });
    }

    // Simple majority: yes > no (abstain doesn't count)
    let passed = proposal.yes_votes > proposal.no_votes;

    if !passed {
        proposal.status = ProposalStatus::Rejected;
        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;
        return Ok(Response::new()
            .add_attribute("action", "execute_proposal")
            .add_attribute("proposal_id", proposal_id.to_string())
            .add_attribute("result", "rejected"));
    }

    // Execute the proposal action
    let config = CONFIG.load(deps.storage)?;
    match &proposal.proposal_type {
        ProposalType::AddValidator {
            address,
            moniker,
            category,
        } => {
            let addr = deps.api.addr_validate(address)?;
            if VALIDATORS.has(deps.storage, &addr) {
                let mut v = VALIDATORS.load(deps.storage, &addr)?;
                v.status = ValidatorStatus::Approved;
                v.moniker = moniker.clone();
                v.category = category.clone();
                v.removal_reason = String::new();
                VALIDATORS.save(deps.storage, &addr, &v)?;
            } else {
                let v = Validator {
                    address: addr.clone(),
                    moniker: moniker.clone(),
                    category: category.clone(),
                    status: ValidatorStatus::Approved,
                    term_start: 0,
                    term_end: 0,
                    performance: PerformanceScores {
                        uptime_bps: None,
                        governance_participation_bps: None,
                        ecosystem_contribution_bps: None,
                    },
                    probation_start: 0,
                    removal_reason: String::new(),
                };
                VALIDATORS.save(deps.storage, &addr, &v)?;
            }
        }
        ProposalType::RemoveValidator { address, reason } => {
            let addr = deps.api.addr_validate(address)?;
            if let Ok(mut v) = VALIDATORS.load(deps.storage, &addr) {
                if matches!(
                    v.status,
                    ValidatorStatus::Active | ValidatorStatus::Probation
                ) {
                    let cat_count =
                        count_active_validators_by_category(deps.as_ref(), &v.category)?;
                    if cat_count <= config.min_per_category {
                        proposal.status = ProposalStatus::Rejected;
                        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;
                        return Ok(Response::new()
                            .add_attribute("action", "execute_proposal")
                            .add_attribute("proposal_id", proposal_id.to_string())
                            .add_attribute("result", "rejected_composition_violation"));
                    }
                }
                v.status = ValidatorStatus::Removed;
                v.removal_reason = reason.clone();
                VALIDATORS.save(deps.storage, &addr, &v)?;
            }
        }
        ProposalType::UpdateConfig {
            max_validators,
            min_validators,
            min_per_category,
            term_length_seconds,
            probation_period_seconds,
            min_uptime_bps,
            performance_threshold_bps,
            performance_bonus_bps,
            voting_period_seconds,
        } => {
            let mut c = CONFIG.load(deps.storage)?;
            if let Some(v) = max_validators {
                c.max_validators = *v;
            }
            if let Some(v) = min_validators {
                c.min_validators = *v;
            }
            if let Some(v) = min_per_category {
                c.min_per_category = *v;
            }
            if let Some(v) = term_length_seconds {
                c.term_length_seconds = *v;
            }
            if let Some(v) = probation_period_seconds {
                c.probation_period_seconds = *v;
            }
            if let Some(v) = min_uptime_bps {
                c.min_uptime_bps = *v;
            }
            if let Some(v) = performance_threshold_bps {
                c.performance_threshold_bps = *v;
            }
            if let Some(v) = performance_bonus_bps {
                c.performance_bonus_bps = *v;
            }
            if let Some(v) = voting_period_seconds {
                c.voting_period_seconds = *v;
            }
            CONFIG.save(deps.storage, &c)?;
        }
    }

    proposal.status = ProposalStatus::Executed;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "execute_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("result", "executed"))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    max_validators: Option<u32>,
    min_validators: Option<u32>,
    min_per_category: Option<u32>,
    term_length_seconds: Option<u64>,
    probation_period_seconds: Option<u64>,
    min_uptime_bps: Option<u16>,
    performance_threshold_bps: Option<u16>,
    performance_bonus_bps: Option<u16>,
    voting_period_seconds: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    if let Some(v) = max_validators {
        config.max_validators = v;
    }
    if let Some(v) = min_validators {
        config.min_validators = v;
    }
    if let Some(v) = min_per_category {
        config.min_per_category = v;
    }
    if let Some(v) = term_length_seconds {
        config.term_length_seconds = v;
    }
    if let Some(v) = probation_period_seconds {
        config.probation_period_seconds = v;
    }
    if let Some(v) = min_uptime_bps {
        config.min_uptime_bps = v;
    }
    if let Some(v) = performance_threshold_bps {
        config.performance_threshold_bps = v;
    }
    if let Some(v) = performance_bonus_bps {
        config.performance_bonus_bps = v;
    }
    if let Some(v) = voting_period_seconds {
        if v == 0 {
            return Err(ContractError::InvalidVotingPeriod);
        }
        config.voting_period_seconds = v;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

// ── Query ────────────────────────────────────────────────────────────

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::ValidatorSet { status, category } => {
            to_json_binary(&query_validator_set(deps, status, category)?)
        }
        QueryMsg::Validator { address } => to_json_binary(&query_validator(deps, address)?),
        QueryMsg::PerformanceScore { address } => {
            to_json_binary(&query_performance_score(deps, address)?)
        }
        QueryMsg::Proposals { status } => to_json_binary(&query_proposals(deps, status)?),
        QueryMsg::Proposal { id } => to_json_binary(&query_proposal(deps, id)?),
        QueryMsg::Votes { proposal_id } => to_json_binary(&query_votes(deps, proposal_id)?),
        QueryMsg::Composition {} => to_json_binary(&query_composition(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

fn query_validator_set(
    deps: Deps,
    status_filter: Option<ValidatorStatus>,
    category_filter: Option<ValidatorCategory>,
) -> StdResult<ValidatorSetResponse> {
    let validators: Vec<Validator> = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .map(|(_, v)| v)
        .filter(|v| {
            let status_match = status_filter.as_ref().map_or(true, |s| v.status == *s);
            let category_match = category_filter.as_ref().map_or(true, |c| v.category == *c);
            status_match && category_match
        })
        .collect();

    let total = validators.len() as u32;
    Ok(ValidatorSetResponse { validators, total })
}

fn query_validator(deps: Deps, address: String) -> StdResult<ValidatorResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let validator = VALIDATORS.load(deps.storage, &addr)?;
    let config = CONFIG.load(deps.storage)?;

    let (composite_score, confidence) = compute_score_and_confidence(&validator.performance);
    let flags = compute_flags(composite_score, &validator.performance, &config);

    Ok(ValidatorResponse {
        validator,
        composite_score,
        confidence,
        flags,
    })
}

fn query_performance_score(deps: Deps, address: String) -> StdResult<PerformanceScoreResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let validator = VALIDATORS.load(deps.storage, &addr)?;
    let config = CONFIG.load(deps.storage)?;

    let (composite_score, confidence) = compute_score_and_confidence(&validator.performance);
    let flags = compute_flags(composite_score, &validator.performance, &config);

    Ok(PerformanceScoreResponse {
        address,
        composite_score,
        confidence,
        factors: FactorsResponse {
            uptime_bps: validator.performance.uptime_bps,
            governance_participation_bps: validator.performance.governance_participation_bps,
            ecosystem_contribution_bps: validator.performance.ecosystem_contribution_bps,
        },
        flags,
    })
}

fn query_proposals(
    deps: Deps,
    status_filter: Option<ProposalStatusFilter>,
) -> StdResult<ProposalListResponse> {
    let proposals: Vec<Proposal> = PROPOSALS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .map(|(_, p)| p)
        .filter(|p| {
            status_filter.as_ref().map_or(true, |filter| match filter {
                ProposalStatusFilter::Active => matches!(p.status, ProposalStatus::Active),
                ProposalStatusFilter::Passed => matches!(p.status, ProposalStatus::Passed),
                ProposalStatusFilter::Rejected => matches!(p.status, ProposalStatus::Rejected),
                ProposalStatusFilter::Executed => matches!(p.status, ProposalStatus::Executed),
            })
        })
        .collect();

    Ok(ProposalListResponse { proposals })
}

fn query_proposal(deps: Deps, id: u64) -> StdResult<ProposalResponse> {
    let proposal = PROPOSALS.load(deps.storage, id)?;

    let votes: Vec<Vote> = VOTES
        .prefix(id)
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .map(|(_, v)| v)
        .collect();

    Ok(ProposalResponse { proposal, votes })
}

fn query_votes(deps: Deps, proposal_id: u64) -> StdResult<VotesResponse> {
    let votes: Vec<Vote> = VOTES
        .prefix(proposal_id)
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .map(|(_, v)| v)
        .collect();

    Ok(VotesResponse { votes })
}

fn query_composition(deps: Deps) -> StdResult<CompositionResponse> {
    let mut infra = 0u32;
    let mut refi = 0u32;
    let mut eco = 0u32;

    let _ = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .for_each(|(_, v)| {
            if matches!(
                v.status,
                ValidatorStatus::Active | ValidatorStatus::Probation
            ) {
                match v.category {
                    ValidatorCategory::InfrastructureBuilders => infra += 1,
                    ValidatorCategory::TrustedRefiPartners => refi += 1,
                    ValidatorCategory::EcologicalDataStewards => eco += 1,
                }
            }
        });

    Ok(CompositionResponse {
        infrastructure_builders: infra,
        trusted_refi_partners: refi,
        ecological_data_stewards: eco,
        total_active: infra + refi + eco,
    })
}

// ── Helpers ──────────────────────────────────────────────────────────

/// Compute composite performance score per SPEC section 5.1
/// Returns score in basis points (0-10000)
/// Weights: uptime=0.4, governance=0.3, ecosystem=0.3
pub fn compute_composite_score(perf: &PerformanceScores) -> u16 {
    let (score, _) = compute_score_and_confidence(perf);
    score
}

/// Compute both composite score and confidence
/// Score uses integer arithmetic: weight_bps * factor_bps / 10000
fn compute_score_and_confidence(perf: &PerformanceScores) -> (u16, u16) {
    let factors: [(Option<u16>, u32); 3] = [
        (perf.uptime_bps, 4000),                  // 0.4 as bps
        (perf.governance_participation_bps, 3000), // 0.3 as bps
        (perf.ecosystem_contribution_bps, 3000),   // 0.3 as bps
    ];

    let mut weighted_sum: u64 = 0;
    let mut total_weight: u64 = 0;
    let mut available_count = 0u8;

    for (value, weight) in &factors {
        if let Some(v) = value {
            weighted_sum += (*v as u64) * (*weight as u64);
            total_weight += *weight as u64;
            available_count += 1;
        }
    }

    let score = if total_weight > 0 {
        (weighted_sum / total_weight) as u16
    } else {
        0
    };

    let confidence = match available_count {
        3 => 10000, // 1.0
        2 => 6700,  // 0.67
        1 => 3300,  // 0.33
        _ => 0,     // 0.0
    };

    (score, confidence)
}

/// Compute performance flags per SPEC section 5.3
fn compute_flags(score: u16, perf: &PerformanceScores, config: &Config) -> Vec<String> {
    let mut flags = Vec::new();
    let has_data = perf.uptime_bps.is_some()
        || perf.governance_participation_bps.is_some()
        || perf.ecosystem_contribution_bps.is_some();

    if score < config.performance_threshold_bps && has_data {
        flags.push("below_performance_threshold".to_string());
    }
    if let Some(uptime) = perf.uptime_bps {
        if uptime < config.min_uptime_bps {
            flags.push("below_uptime_minimum".to_string());
        }
    }
    if flags.contains(&"below_performance_threshold".to_string())
        || flags.contains(&"below_uptime_minimum".to_string())
    {
        flags.push("probation_recommended".to_string());
    }
    flags
}

/// Count validators matching a specific status (or all if None)
fn count_validators_by_status(deps: Deps, status: Option<ValidatorStatus>) -> StdResult<u32> {
    let count = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .filter(|(_, v)| status.as_ref().map_or(true, |s| v.status == *s))
        .count() as u32;
    Ok(count)
}

/// Count active+probation validators in a specific category
fn count_active_validators_by_category(
    deps: Deps,
    category: &ValidatorCategory,
) -> StdResult<u32> {
    let count = VALIDATORS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .filter(|(_, v)| {
            v.category == *category
                && matches!(
                    v.status,
                    ValidatorStatus::Active | ValidatorStatus::Probation
                )
        })
        .count() as u32;
    Ok(count)
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi};
    use cosmwasm_std::{Addr, Timestamp};

    const ADMIN: &str = "admin";
    const VALIDATOR1: &str = "val1";
    const VALIDATOR2: &str = "val2";
    const VALIDATOR3: &str = "val3";

    fn addr(name: &str) -> Addr {
        MockApi::default().addr_make(name)
    }

    fn mock_info(sender: &str) -> MessageInfo {
        MessageInfo {
            sender: addr(sender),
            funds: vec![],
        }
    }

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            admin: addr(ADMIN).to_string(),
            max_validators: Some(21),
            min_validators: Some(3),
            min_per_category: Some(1),
            term_length_seconds: Some(31_536_000),
            probation_period_seconds: Some(2_592_000),
            min_uptime_bps: Some(9950),
            performance_threshold_bps: Some(7000),
            performance_bonus_bps: Some(1000),
            voting_period_seconds: Some(604_800),
        };
        let info = mock_info(ADMIN);
        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    fn apply_validator(deps: DepsMut, env: &Env, name: &str, category: ValidatorCategory) {
        let info = mock_info(name);
        execute(
            deps,
            env.clone(),
            info,
            ExecuteMsg::ApplyValidator {
                moniker: format!("Validator {}", name),
                category,
            },
        )
        .unwrap();
    }

    fn approve_and_activate(deps: DepsMut, env: &Env, name: &str) {
        let info = mock_info(ADMIN);
        execute(
            deps,
            env.clone(),
            info,
            ExecuteMsg::ApproveValidator {
                address: addr(name).to_string(),
            },
        )
        .unwrap();
    }

    fn activate_validator(deps: DepsMut, env: &Env, name: &str) {
        let info = mock_info(ADMIN);
        execute(
            deps,
            env.clone(),
            info,
            ExecuteMsg::ActivateValidator {
                address: addr(name).to_string(),
            },
        )
        .unwrap();
    }

    // ── Instantiation tests ──────────────────────────────────────────

    #[test]
    fn test_instantiate_defaults() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admin: addr(ADMIN).to_string(),
            max_validators: None,
            min_validators: None,
            min_per_category: None,
            term_length_seconds: None,
            probation_period_seconds: None,
            min_uptime_bps: None,
            performance_threshold_bps: None,
            performance_bonus_bps: None,
            voting_period_seconds: None,
        };
        let info = mock_info(ADMIN);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 2);

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.max_validators, 21);
        assert_eq!(config.min_validators, 15);
        assert_eq!(config.min_per_category, 5);
        assert_eq!(config.term_length_seconds, 31_536_000);
        assert_eq!(config.min_uptime_bps, 9950);
        assert_eq!(config.performance_threshold_bps, 7000);
        assert_eq!(config.performance_bonus_bps, 1000);
        assert_eq!(config.voting_period_seconds, 604_800);
    }

    // ── Validator lifecycle tests ────────────────────────────────────

    #[test]
    fn test_apply_validator() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let info = mock_info(VALIDATOR1);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::ApplyValidator {
                moniker: "Test Validator".to_string(),
                category: ValidatorCategory::InfrastructureBuilders,
            },
        )
        .unwrap();

        assert_eq!(res.attributes[0].value, "apply_validator");

        let val_addr = addr(VALIDATOR1);
        let v = VALIDATORS.load(&deps.storage, &val_addr).unwrap();
        assert_eq!(v.moniker, "Test Validator");
        assert!(matches!(v.status, ValidatorStatus::Candidate));
        assert!(matches!(
            v.category,
            ValidatorCategory::InfrastructureBuilders
        ));
    }

    #[test]
    fn test_duplicate_application_fails() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        let info = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::ApplyValidator {
                moniker: "V1".to_string(),
                category: ValidatorCategory::InfrastructureBuilders,
            },
        )
        .unwrap();

        let info2 = mock_info(VALIDATOR1);
        let err = execute(
            deps.as_mut(),
            mock_env(),
            info2,
            ExecuteMsg::ApplyValidator {
                moniker: "V1 Again".to_string(),
                category: ValidatorCategory::InfrastructureBuilders,
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::ValidatorAlreadyExists { .. }));
    }

    #[test]
    fn test_full_lifecycle_candidate_to_active() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let val_addr = addr(VALIDATOR1);
        let v = VALIDATORS.load(&deps.storage, &val_addr).unwrap();
        assert!(matches!(v.status, ValidatorStatus::Active));
        assert!(v.term_start > 0);
        assert!(v.term_end > v.term_start);
    }

    #[test]
    fn test_approve_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );

        let info = mock_info(VALIDATOR2);
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ApproveValidator {
                address: addr(VALIDATOR1).to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_probation_and_restore() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::PutOnProbation {
                address: addr(VALIDATOR1).to_string(),
                reason: "Low uptime".to_string(),
            },
        )
        .unwrap();

        let val_addr = addr(VALIDATOR1);
        let v = VALIDATORS.load(&deps.storage, &val_addr).unwrap();
        assert!(matches!(v.status, ValidatorStatus::Probation));
        assert!(v.probation_start > 0);

        let info2 = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env,
            info2,
            ExecuteMsg::RestoreValidator {
                address: addr(VALIDATOR1).to_string(),
            },
        )
        .unwrap();

        let v2 = VALIDATORS.load(&deps.storage, &val_addr).unwrap();
        assert!(matches!(v2.status, ValidatorStatus::Active));
        assert_eq!(v2.probation_start, 0);
    }

    #[test]
    fn test_remove_validator() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        for (name, cat) in [
            (VALIDATOR1, ValidatorCategory::InfrastructureBuilders),
            (VALIDATOR2, ValidatorCategory::TrustedRefiPartners),
            (VALIDATOR3, ValidatorCategory::EcologicalDataStewards),
            ("val4", ValidatorCategory::InfrastructureBuilders),
        ] {
            apply_validator(deps.as_mut(), &env, name, cat);
            approve_and_activate(deps.as_mut(), &env, name);
            activate_validator(deps.as_mut(), &env, name);
        }

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::RemoveValidator {
                address: addr("val4").to_string(),
                reason: "Inactive".to_string(),
            },
        )
        .unwrap();

        let val4_addr = addr("val4");
        let v = VALIDATORS.load(&deps.storage, &val4_addr).unwrap();
        assert!(matches!(v.status, ValidatorStatus::Removed));
        assert_eq!(v.removal_reason, "Inactive");
    }

    #[test]
    fn test_cannot_remove_below_min_validators() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        // Set up exactly min_validators (3) active, same category so
        // composition check doesn't fire first
        for name in ["v1", "v2", "v3"] {
            apply_validator(
                deps.as_mut(),
                &env,
                name,
                ValidatorCategory::InfrastructureBuilders,
            );
            approve_and_activate(deps.as_mut(), &env, name);
            activate_validator(deps.as_mut(), &env, name);
        }

        let info = mock_info(ADMIN);
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::RemoveValidator {
                address: addr("v1").to_string(),
                reason: "test".to_string(),
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::MinimumValidatorsReached { .. }));
    }

    // ── Score tests ──────────────────────────────────────────────────

    #[test]
    fn test_update_and_query_scores() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR1).to_string(),
                uptime_bps: Some(9980),
                governance_participation_bps: Some(8500),
                ecosystem_contribution_bps: Some(7000),
            },
        )
        .unwrap();

        let res: PerformanceScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                env,
                QueryMsg::PerformanceScore {
                    address: addr(VALIDATOR1).to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // (9980*4000 + 8500*3000 + 7000*3000) / 10000 = 8642
        assert_eq!(res.composite_score, 8642);
        assert_eq!(res.confidence, 10000);
        assert!(res.flags.is_empty());
    }

    #[test]
    fn test_score_with_missing_factors() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR1).to_string(),
                uptime_bps: Some(9900),
                governance_participation_bps: None,
                ecosystem_contribution_bps: None,
            },
        )
        .unwrap();

        let res: PerformanceScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                env,
                QueryMsg::PerformanceScore {
                    address: addr(VALIDATOR1).to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // Re-normalized: 9900 * 4000 / 4000 = 9900
        assert_eq!(res.composite_score, 9900);
        assert_eq!(res.confidence, 3300);
        assert!(res.flags.contains(&"below_uptime_minimum".to_string()));
        assert!(res.flags.contains(&"probation_recommended".to_string()));
    }

    #[test]
    fn test_below_performance_threshold_flags() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR1).to_string(),
                uptime_bps: Some(5000),
                governance_participation_bps: Some(4000),
                ecosystem_contribution_bps: Some(3000),
            },
        )
        .unwrap();

        let res: PerformanceScoreResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                env,
                QueryMsg::PerformanceScore {
                    address: addr(VALIDATOR1).to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();

        // (5000*4000 + 4000*3000 + 3000*3000) / 10000 = 4100
        assert_eq!(res.composite_score, 4100);
        assert!(res.flags.contains(&"below_performance_threshold".to_string()));
        assert!(res.flags.contains(&"below_uptime_minimum".to_string()));
        assert!(res.flags.contains(&"probation_recommended".to_string()));
    }

    #[test]
    fn test_invalid_score_rejected() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let info = mock_info(ADMIN);
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR1).to_string(),
                uptime_bps: Some(15000),
                governance_participation_bps: None,
                ecosystem_contribution_bps: None,
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::InvalidScore));
    }

    // ── Governance proposal tests ────────────────────────────────────

    #[test]
    fn test_create_and_vote_on_proposal() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR2,
            ValidatorCategory::TrustedRefiPartners,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR2);
        activate_validator(deps.as_mut(), &env, VALIDATOR2);

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR1).to_string(),
                uptime_bps: Some(9990),
                governance_participation_bps: Some(9000),
                ecosystem_contribution_bps: Some(8000),
            },
        )
        .unwrap();

        let info2 = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info2,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR2).to_string(),
                uptime_bps: Some(9950),
                governance_participation_bps: Some(7000),
                ecosystem_contribution_bps: Some(6000),
            },
        )
        .unwrap();

        let v1_info = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            env.clone(),
            v1_info,
            ExecuteMsg::CreateProposal {
                title: "Add new validator".to_string(),
                description: "Add val3 as ecological steward".to_string(),
                proposal_type: ProposalType::AddValidator {
                    address: addr(VALIDATOR3).to_string(),
                    moniker: "Eco Steward".to_string(),
                    category: ValidatorCategory::EcologicalDataStewards,
                },
            },
        )
        .unwrap();

        let v1_vote = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            env.clone(),
            v1_vote,
            ExecuteMsg::CastVote {
                proposal_id: 1,
                vote: VoteOption::Yes,
            },
        )
        .unwrap();

        let v2_vote = mock_info(VALIDATOR2);
        execute(
            deps.as_mut(),
            env.clone(),
            v2_vote,
            ExecuteMsg::CastVote {
                proposal_id: 1,
                vote: VoteOption::Yes,
            },
        )
        .unwrap();

        let res: ProposalResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), env, QueryMsg::Proposal { id: 1 }).unwrap(),
        )
        .unwrap();

        assert!(matches!(res.proposal.status, ProposalStatus::Active));
        assert!(res.proposal.yes_votes > 0);
        assert_eq!(res.proposal.no_votes, 0);
        assert_eq!(res.votes.len(), 2);
    }

    #[test]
    fn test_execute_passed_proposal() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let mut env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR1).to_string(),
                uptime_bps: Some(9990),
                governance_participation_bps: Some(9000),
                ecosystem_contribution_bps: Some(8000),
            },
        )
        .unwrap();

        let v1_info = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            env.clone(),
            v1_info,
            ExecuteMsg::CreateProposal {
                title: "Update config".to_string(),
                description: "Increase max validators".to_string(),
                proposal_type: ProposalType::UpdateConfig {
                    max_validators: Some(25),
                    min_validators: None,
                    min_per_category: None,
                    term_length_seconds: None,
                    probation_period_seconds: None,
                    min_uptime_bps: None,
                    performance_threshold_bps: None,
                    performance_bonus_bps: None,
                    voting_period_seconds: None,
                },
            },
        )
        .unwrap();

        let v1_vote = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            env.clone(),
            v1_vote,
            ExecuteMsg::CastVote {
                proposal_id: 1,
                vote: VoteOption::Yes,
            },
        )
        .unwrap();

        // Advance time past voting period
        env.block.time = Timestamp::from_seconds(env.block.time.seconds() + 604_801);

        let anyone = mock_info("anyone");
        let res = execute(
            deps.as_mut(),
            env,
            anyone,
            ExecuteMsg::ExecuteProposal { proposal_id: 1 },
        )
        .unwrap();

        assert_eq!(res.attributes[2].value, "executed");

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.max_validators, 25);
    }

    #[test]
    fn test_cannot_vote_twice() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR1).to_string(),
                uptime_bps: Some(9990),
                governance_participation_bps: Some(9000),
                ecosystem_contribution_bps: Some(8000),
            },
        )
        .unwrap();

        let v1_info = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            env.clone(),
            v1_info,
            ExecuteMsg::CreateProposal {
                title: "Test".to_string(),
                description: "Test".to_string(),
                proposal_type: ProposalType::UpdateConfig {
                    max_validators: Some(25),
                    min_validators: None,
                    min_per_category: None,
                    term_length_seconds: None,
                    probation_period_seconds: None,
                    min_uptime_bps: None,
                    performance_threshold_bps: None,
                    performance_bonus_bps: None,
                    voting_period_seconds: None,
                },
            },
        )
        .unwrap();

        let v1_vote = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            env.clone(),
            v1_vote,
            ExecuteMsg::CastVote {
                proposal_id: 1,
                vote: VoteOption::Yes,
            },
        )
        .unwrap();

        let v1_vote2 = mock_info(VALIDATOR1);
        let err = execute(
            deps.as_mut(),
            env,
            v1_vote2,
            ExecuteMsg::CastVote {
                proposal_id: 1,
                vote: VoteOption::No,
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::AlreadyVoted { .. }));
    }

    #[test]
    fn test_cannot_execute_before_voting_period() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateScores {
                address: addr(VALIDATOR1).to_string(),
                uptime_bps: Some(9990),
                governance_participation_bps: Some(9000),
                ecosystem_contribution_bps: Some(8000),
            },
        )
        .unwrap();

        let v1_info = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            env.clone(),
            v1_info,
            ExecuteMsg::CreateProposal {
                title: "Test".to_string(),
                description: "Test".to_string(),
                proposal_type: ProposalType::UpdateConfig {
                    max_validators: Some(25),
                    min_validators: None,
                    min_per_category: None,
                    term_length_seconds: None,
                    probation_period_seconds: None,
                    min_uptime_bps: None,
                    performance_threshold_bps: None,
                    performance_bonus_bps: None,
                    voting_period_seconds: None,
                },
            },
        )
        .unwrap();

        let anyone = mock_info("anyone");
        let err = execute(
            deps.as_mut(),
            env,
            anyone,
            ExecuteMsg::ExecuteProposal { proposal_id: 1 },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::VotingPeriodNotExpired { .. }));
    }

    #[test]
    fn test_non_active_validator_cannot_propose() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );

        let v1_info = mock_info(VALIDATOR1);
        let err = execute(
            deps.as_mut(),
            env,
            v1_info,
            ExecuteMsg::CreateProposal {
                title: "Test".to_string(),
                description: "Test".to_string(),
                proposal_type: ProposalType::UpdateConfig {
                    max_validators: Some(25),
                    min_validators: None,
                    min_per_category: None,
                    term_length_seconds: None,
                    probation_period_seconds: None,
                    min_uptime_bps: None,
                    performance_threshold_bps: None,
                    performance_bonus_bps: None,
                    voting_period_seconds: None,
                },
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::NotActiveValidator { .. }));
    }

    // ── Query tests ──────────────────────────────────────────────────

    #[test]
    fn test_query_composition() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        for (name, cat) in [
            (VALIDATOR1, ValidatorCategory::InfrastructureBuilders),
            (VALIDATOR2, ValidatorCategory::TrustedRefiPartners),
            (VALIDATOR3, ValidatorCategory::EcologicalDataStewards),
        ] {
            apply_validator(deps.as_mut(), &env, name, cat);
            approve_and_activate(deps.as_mut(), &env, name);
            activate_validator(deps.as_mut(), &env, name);
        }

        let res: CompositionResponse = cosmwasm_std::from_json(
            query(deps.as_ref(), env, QueryMsg::Composition {}).unwrap(),
        )
        .unwrap();

        assert_eq!(res.infrastructure_builders, 1);
        assert_eq!(res.trusted_refi_partners, 1);
        assert_eq!(res.ecological_data_stewards, 1);
        assert_eq!(res.total_active, 3);
    }

    #[test]
    fn test_query_validator_set_with_filter() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR2,
            ValidatorCategory::TrustedRefiPartners,
        );

        let res: ValidatorSetResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::ValidatorSet {
                    status: None,
                    category: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.total, 2);

        let res: ValidatorSetResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::ValidatorSet {
                    status: Some(ValidatorStatus::Active),
                    category: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.total, 1);

        let res: ValidatorSetResponse = cosmwasm_std::from_json(
            query(
                deps.as_ref(),
                env,
                QueryMsg::ValidatorSet {
                    status: Some(ValidatorStatus::Candidate),
                    category: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.total, 1);
    }

    // ── Config update tests ──────────────────────────────────────────

    #[test]
    fn test_update_config() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        let info = mock_info(ADMIN);
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::UpdateConfig {
                max_validators: Some(30),
                min_validators: None,
                min_per_category: None,
                term_length_seconds: None,
                probation_period_seconds: None,
                min_uptime_bps: None,
                performance_threshold_bps: None,
                performance_bonus_bps: None,
                voting_period_seconds: Some(86400),
            },
        )
        .unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.max_validators, 30);
        assert_eq!(config.voting_period_seconds, 86400);
        assert_eq!(config.min_validators, 3);
    }

    #[test]
    fn test_update_config_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        let non_admin = mock_info(VALIDATOR1);
        let err = execute(
            deps.as_mut(),
            env,
            non_admin,
            ExecuteMsg::UpdateConfig {
                max_validators: Some(30),
                min_validators: None,
                min_per_category: None,
                term_length_seconds: None,
                probation_period_seconds: None,
                min_uptime_bps: None,
                performance_threshold_bps: None,
                performance_bonus_bps: None,
                voting_period_seconds: None,
            },
        )
        .unwrap_err();

        assert!(matches!(err, ContractError::Unauthorized));
    }

    // ── Reapply test ─────────────────────────────────────────────────

    #[test]
    fn test_reapply_after_term_expired() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();

        apply_validator(
            deps.as_mut(),
            &env,
            VALIDATOR1,
            ValidatorCategory::InfrastructureBuilders,
        );
        approve_and_activate(deps.as_mut(), &env, VALIDATOR1);
        activate_validator(deps.as_mut(), &env, VALIDATOR1);

        let val_addr = addr(VALIDATOR1);
        let mut v = VALIDATORS.load(&deps.storage, &val_addr).unwrap();
        v.status = ValidatorStatus::TermExpired;
        VALIDATORS.save(deps.as_mut().storage, &val_addr, &v).unwrap();

        let v1_info = mock_info(VALIDATOR1);
        execute(
            deps.as_mut(),
            env,
            v1_info,
            ExecuteMsg::Reapply {
                address: addr(VALIDATOR1).to_string(),
            },
        )
        .unwrap();

        let v2 = VALIDATORS.load(&deps.storage, &val_addr).unwrap();
        assert!(matches!(v2.status, ValidatorStatus::Candidate));
    }

    // ── Composite score calculation tests ────────────────────────────

    #[test]
    fn test_composite_score_all_factors() {
        let perf = PerformanceScores {
            uptime_bps: Some(9980),
            governance_participation_bps: Some(8500),
            ecosystem_contribution_bps: Some(7000),
        };
        let (score, confidence) = compute_score_and_confidence(&perf);
        assert_eq!(score, 8642);
        assert_eq!(confidence, 10000);
    }

    #[test]
    fn test_composite_score_no_factors() {
        let perf = PerformanceScores {
            uptime_bps: None,
            governance_participation_bps: None,
            ecosystem_contribution_bps: None,
        };
        let (score, confidence) = compute_score_and_confidence(&perf);
        assert_eq!(score, 0);
        assert_eq!(confidence, 0);
    }

    #[test]
    fn test_composite_score_two_factors() {
        let perf = PerformanceScores {
            uptime_bps: Some(10000),
            governance_participation_bps: Some(5000),
            ecosystem_contribution_bps: None,
        };
        let (score, confidence) = compute_score_and_confidence(&perf);
        // (10000*4000 + 5000*3000) / 7000 = 55_000_000 / 7000 = 7857
        assert_eq!(score, 7857);
        assert_eq!(confidence, 6700);
    }
}
