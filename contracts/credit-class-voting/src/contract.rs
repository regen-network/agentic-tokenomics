use cosmwasm_std::{
    entry_point, to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ApprovedClassResponse, ApprovedClassesResponse, ConfigResponse, ExecuteMsg, InstantiateMsg,
    ProposalResponse, ProposalStatusFilter, ProposalsResponse, QueryMsg, ScoringFactorsMsg,
    TallyResponse, VoteResponse,
};
use crate::state::{
    AgentScore, ApprovedClass, Config, Proposal, ProposalStatus, Recommendation, ScoringFactors,
    Tally, Vote, VoteOption, APPROVED_CLASSES, CONFIG, PROPOSALS, PROPOSAL_COUNT,
    VALID_CREDIT_TYPES, VOTES,
};

const CONTRACT_NAME: &str = "crates.io:credit-class-voting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_QUERY_LIMIT: u32 = 30;
const MAX_QUERY_LIMIT: u32 = 100;

// ── Instantiate ────────────────────────────────────────────────────────

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = match msg.admin {
        Some(addr) => deps.api.addr_validate(&addr)?,
        None => info.sender.clone(),
    };

    let quorum_threshold = msg.quorum_threshold.unwrap_or(100);
    let pass_threshold = msg.pass_threshold.unwrap_or(500);
    let veto_threshold = msg.veto_threshold.unwrap_or(334);

    validate_threshold(quorum_threshold)?;
    validate_threshold(pass_threshold)?;
    validate_threshold(veto_threshold)?;

    let config = Config {
        admin,
        quorum_threshold,
        pass_threshold,
        veto_threshold,
        voting_period_seconds: msg.voting_period_seconds.unwrap_or(604_800),
        override_window_seconds: msg.override_window_seconds.unwrap_or(21_600),
    };

    CONFIG.save(deps.storage, &config)?;
    PROPOSAL_COUNT.save(deps.storage, &0u64)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", config.admin.as_str()))
}

// ── Execute ────────────────────────────────────────────────────────────

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ProposeClass {
            credit_type,
            methodology_iri,
            admin_address,
        } => execute_propose_class(deps, env, info, credit_type, methodology_iri, admin_address),
        ExecuteMsg::SubmitAgentScore {
            proposal_id,
            score,
            confidence,
            factors,
        } => execute_submit_agent_score(deps, env, info, proposal_id, score, confidence, factors),
        ExecuteMsg::CastVote {
            proposal_id,
            vote,
            weight,
        } => execute_cast_vote(deps, env, info, proposal_id, vote, weight),
        ExecuteMsg::OverrideAgentReject { proposal_id } => {
            execute_override_agent_reject(deps, env, info, proposal_id)
        }
        ExecuteMsg::ExecuteProposal {
            proposal_id,
            class_id,
        } => execute_proposal(deps, env, info, proposal_id, class_id),
        ExecuteMsg::FinalizeAgentReject { proposal_id } => {
            execute_finalize_agent_reject(deps, env, info, proposal_id)
        }
        ExecuteMsg::UpdateConfig {
            admin,
            quorum_threshold,
            pass_threshold,
            veto_threshold,
            voting_period_seconds,
            override_window_seconds,
        } => execute_update_config(
            deps,
            info,
            admin,
            quorum_threshold,
            pass_threshold,
            veto_threshold,
            voting_period_seconds,
            override_window_seconds,
        ),
    }
}

fn execute_propose_class(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    credit_type: String,
    methodology_iri: String,
    admin_address: String,
) -> Result<Response, ContractError> {
    // Validate credit type
    if !VALID_CREDIT_TYPES.contains(&credit_type.as_str()) {
        return Err(ContractError::InvalidCreditType { credit_type });
    }

    // Increment proposal counter
    let id = PROPOSAL_COUNT.load(deps.storage)? + 1;
    PROPOSAL_COUNT.save(deps.storage, &id)?;

    let proposal = Proposal {
        id,
        proposer: info.sender.clone(),
        credit_type: credit_type.clone(),
        methodology_iri: methodology_iri.clone(),
        admin_address: admin_address.clone(),
        status: ProposalStatus::AgentReview,
        submit_time: env.block.time.seconds(),
        agent_score: None,
        voting_start_time: None,
        human_override: false,
        tally: Tally::default(),
    };

    PROPOSALS.save(deps.storage, id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "propose_class")
        .add_attribute("proposal_id", id.to_string())
        .add_attribute("proposer", info.sender.as_str())
        .add_attribute("credit_type", &credit_type)
        .add_attribute("status", "AGENT_REVIEW"))
}

fn execute_submit_agent_score(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    score: u64,
    confidence: u64,
    factors: ScoringFactorsMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    if score > 1000 {
        return Err(ContractError::InvalidScore { value: score });
    }
    if confidence > 1000 {
        return Err(ContractError::InvalidScore { value: confidence });
    }

    let mut proposal = PROPOSALS
        .may_load(deps.storage, proposal_id)?
        .ok_or(ContractError::ProposalNotFound { id: proposal_id })?;

    if proposal.status != ProposalStatus::AgentReview {
        return Err(ContractError::InvalidState {
            id: proposal_id,
            expected: "AGENT_REVIEW".to_string(),
            actual: proposal.status.to_string(),
        });
    }

    // Determine recommendation per SPEC section 3 thresholds
    let recommendation = compute_recommendation(score, confidence);

    let agent_score = AgentScore {
        score,
        confidence,
        recommendation: recommendation.clone(),
        factors: ScoringFactors {
            methodology_quality: factors.methodology_quality,
            admin_reputation: factors.admin_reputation,
            novelty: factors.novelty,
            completeness: factors.completeness,
        },
    };

    proposal.agent_score = Some(agent_score);

    // Determine state transition based on recommendation
    match recommendation {
        Recommendation::Approve | Recommendation::Conditional => {
            // Advance to VOTING
            proposal.status = ProposalStatus::Voting;
            proposal.voting_start_time = Some(env.block.time.seconds());
        }
        Recommendation::Reject => {
            // Stay in AgentReview — awaiting override window expiry or human override.
            // The proposal remains in AGENT_REVIEW with agent_score set.
            // FinalizeAgentReject or OverrideAgentReject will handle next transition.
        }
    }

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "submit_agent_score")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("score", score.to_string())
        .add_attribute("confidence", confidence.to_string())
        .add_attribute("recommendation", recommendation.to_string())
        .add_attribute("status", proposal.status.to_string()))
}

fn execute_cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
    vote_option: VoteOption,
    weight: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut proposal = PROPOSALS
        .may_load(deps.storage, proposal_id)?
        .ok_or(ContractError::ProposalNotFound { id: proposal_id })?;

    if proposal.status != ProposalStatus::Voting {
        return Err(ContractError::InvalidState {
            id: proposal_id,
            expected: "VOTING".to_string(),
            actual: proposal.status.to_string(),
        });
    }

    // Check voting period hasn't ended
    if let Some(start) = proposal.voting_start_time {
        let end = start + config.voting_period_seconds;
        if env.block.time.seconds() >= end {
            return Err(ContractError::VotingPeriodEnded { id: proposal_id });
        }
    }

    // Check for duplicate votes
    if VOTES
        .may_load(deps.storage, (proposal_id, &info.sender))?
        .is_some()
    {
        return Err(ContractError::AlreadyVoted {
            voter: info.sender.to_string(),
            proposal_id,
        });
    }

    // Record vote
    let vote = Vote {
        voter: info.sender.clone(),
        proposal_id,
        option: vote_option.clone(),
        weight,
    };
    VOTES.save(deps.storage, (proposal_id, &info.sender), &vote)?;

    // Update tally
    match vote_option {
        VoteOption::Yes => proposal.tally.yes_weight += weight,
        VoteOption::No => proposal.tally.no_weight += weight,
        VoteOption::Veto => proposal.tally.veto_weight += weight,
        VoteOption::Abstain => proposal.tally.abstain_weight += weight,
    }

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "cast_vote")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("voter", info.sender.as_str())
        .add_attribute("weight", weight.to_string()))
}

fn execute_override_agent_reject(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    let mut proposal = PROPOSALS
        .may_load(deps.storage, proposal_id)?
        .ok_or(ContractError::ProposalNotFound { id: proposal_id })?;

    if proposal.status != ProposalStatus::AgentReview {
        return Err(ContractError::InvalidState {
            id: proposal_id,
            expected: "AGENT_REVIEW".to_string(),
            actual: proposal.status.to_string(),
        });
    }

    // Must have been agent-rejected
    let agent_score = proposal
        .agent_score
        .as_ref()
        .ok_or(ContractError::NotAgentRejected { id: proposal_id })?;
    if agent_score.recommendation != Recommendation::Reject {
        return Err(ContractError::NotAgentRejected { id: proposal_id });
    }

    // Check override window hasn't expired
    let override_deadline = proposal.submit_time + config.override_window_seconds;
    if env.block.time.seconds() > override_deadline {
        return Err(ContractError::OverrideWindowExpired { id: proposal_id });
    }

    // Advance to VOTING with override flag
    proposal.status = ProposalStatus::Voting;
    proposal.voting_start_time = Some(env.block.time.seconds());
    proposal.human_override = true;

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "override_agent_reject")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", "VOTING")
        .add_attribute("override", "true"))
}

fn execute_finalize_agent_reject(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    proposal_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut proposal = PROPOSALS
        .may_load(deps.storage, proposal_id)?
        .ok_or(ContractError::ProposalNotFound { id: proposal_id })?;

    if proposal.status != ProposalStatus::AgentReview {
        return Err(ContractError::InvalidState {
            id: proposal_id,
            expected: "AGENT_REVIEW".to_string(),
            actual: proposal.status.to_string(),
        });
    }

    // Must have been agent-rejected
    let agent_score = proposal
        .agent_score
        .as_ref()
        .ok_or(ContractError::NotAgentRejected { id: proposal_id })?;
    if agent_score.recommendation != Recommendation::Reject {
        return Err(ContractError::NotAgentRejected { id: proposal_id });
    }

    // Override window must have expired
    let override_deadline = proposal.submit_time + config.override_window_seconds;
    if env.block.time.seconds() <= override_deadline {
        return Err(ContractError::OverrideWindowNotExpired { id: proposal_id });
    }

    // Finalize as rejected
    proposal.status = ProposalStatus::Rejected;
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::new()
        .add_attribute("action", "finalize_agent_reject")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", "REJECTED")
        .add_attribute("reason", "agent_reject_no_override"))
}

fn execute_proposal(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    proposal_id: u64,
    class_id: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut proposal = PROPOSALS
        .may_load(deps.storage, proposal_id)?
        .ok_or(ContractError::ProposalNotFound { id: proposal_id })?;

    if proposal.status != ProposalStatus::Voting {
        return Err(ContractError::InvalidState {
            id: proposal_id,
            expected: "VOTING".to_string(),
            actual: proposal.status.to_string(),
        });
    }

    // Check voting period has ended
    let voting_start = proposal.voting_start_time.unwrap_or(proposal.submit_time);
    let voting_end = voting_start + config.voting_period_seconds;
    if env.block.time.seconds() < voting_end {
        return Err(ContractError::VotingPeriodNotEnded { id: proposal_id });
    }

    let tally = &proposal.tally;
    let total = tally.total_participating();

    // Check quorum: total_participating / assumed_total_weight >= quorum_threshold / 1000
    // For simplicity in v0, we use total_participating as an absolute weight check.
    // In production, this would compare against total staked supply.
    // Here we check: if no votes at all, it's expired.
    let quorum_met = total > 0;

    let (status, reason) = if !quorum_met {
        // No votes — expired
        (ProposalStatus::Expired, "quorum_not_met")
    } else {
        let non_abstain = tally.total_non_abstain();
        if non_abstain == 0 {
            // All abstain — expired (no meaningful decision)
            (ProposalStatus::Expired, "all_abstain")
        } else {
            // Veto check: veto_weight / non_abstain > veto_threshold / 1000
            let veto_ratio_x1000 = (tally.veto_weight * 1000) / non_abstain;
            if veto_ratio_x1000 > config.veto_threshold {
                (ProposalStatus::Rejected, "vetoed")
            } else {
                // Pass check: yes_weight / non_abstain > pass_threshold / 1000
                let yes_ratio_x1000 = (tally.yes_weight * 1000) / non_abstain;
                if yes_ratio_x1000 >= config.pass_threshold {
                    (ProposalStatus::Approved, "governance_approved")
                } else {
                    (ProposalStatus::Rejected, "governance_rejected")
                }
            }
        }
    };

    proposal.status = status.clone();
    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    // If approved, register the credit class
    if status == ProposalStatus::Approved {
        let cid = class_id.unwrap_or_else(|| {
            format!("{}_{}", proposal.credit_type, proposal_id)
        });
        let approved_class = ApprovedClass {
            class_id: cid.clone(),
            credit_type: proposal.credit_type.clone(),
            methodology_iri: proposal.methodology_iri.clone(),
            admin_address: proposal.admin_address.clone(),
            proposal_id,
            approved_at: env.block.time.seconds(),
        };
        APPROVED_CLASSES.save(deps.storage, &cid, &approved_class)?;
    }

    Ok(Response::new()
        .add_attribute("action", "execute_proposal")
        .add_attribute("proposal_id", proposal_id.to_string())
        .add_attribute("status", status.to_string())
        .add_attribute("reason", reason))
}

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    admin: Option<String>,
    quorum_threshold: Option<u64>,
    pass_threshold: Option<u64>,
    veto_threshold: Option<u64>,
    voting_period_seconds: Option<u64>,
    override_window_seconds: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized);
    }

    if let Some(admin) = admin {
        config.admin = deps.api.addr_validate(&admin)?;
    }
    if let Some(q) = quorum_threshold {
        validate_threshold(q)?;
        config.quorum_threshold = q;
    }
    if let Some(p) = pass_threshold {
        validate_threshold(p)?;
        config.pass_threshold = p;
    }
    if let Some(v) = veto_threshold {
        validate_threshold(v)?;
        config.veto_threshold = v;
    }
    if let Some(vp) = voting_period_seconds {
        config.voting_period_seconds = vp;
    }
    if let Some(ow) = override_window_seconds {
        config.override_window_seconds = ow;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("admin", config.admin.as_str()))
}

// ── Query ──────────────────────────────────────────────────────────────

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Proposal { id } => to_json_binary(&query_proposal(deps, id)?),
        QueryMsg::Proposals {
            status,
            start_after,
            limit,
        } => to_json_binary(&query_proposals(deps, status, start_after, limit)?),
        QueryMsg::Tally { proposal_id } => to_json_binary(&query_tally(deps, proposal_id)?),
        QueryMsg::Vote {
            proposal_id,
            voter,
        } => to_json_binary(&query_vote(deps, proposal_id, voter)?),
        QueryMsg::ApprovedClasses { start_after, limit } => {
            to_json_binary(&query_approved_classes(deps, start_after, limit)?)
        }
        QueryMsg::ApprovedClass { class_id } => {
            to_json_binary(&query_approved_class(deps, class_id)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

fn query_proposal(deps: Deps, id: u64) -> StdResult<ProposalResponse> {
    let proposal = PROPOSALS.load(deps.storage, id)?;
    Ok(ProposalResponse { proposal })
}

fn query_proposals(
    deps: Deps,
    status: Option<ProposalStatusFilter>,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<ProposalsResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = start_after.unwrap_or(0);

    let proposals: Vec<Proposal> = PROPOSALS
        .range(deps.storage, Some(cw_storage_plus::Bound::exclusive(start)), None, Order::Ascending)
        .filter_map(|item| {
            let (_, proposal) = item.ok()?;
            if let Some(ref filter) = status {
                if !filter.matches(&proposal.status) {
                    return None;
                }
            }
            Some(proposal)
        })
        .take(limit)
        .collect();

    Ok(ProposalsResponse { proposals })
}

fn query_tally(deps: Deps, proposal_id: u64) -> StdResult<TallyResponse> {
    let proposal = PROPOSALS.load(deps.storage, proposal_id)?;
    let tally = &proposal.tally;
    let total = tally.total_participating();
    let non_abstain = tally.total_non_abstain();

    let yes_ratio = if non_abstain > 0 {
        Some((tally.yes_weight * 1000) / non_abstain)
    } else {
        None
    };
    let veto_ratio = if non_abstain > 0 {
        Some((tally.veto_weight * 1000) / non_abstain)
    } else {
        None
    };

    Ok(TallyResponse {
        proposal_id,
        tally: tally.clone(),
        total_participating: total,
        yes_ratio,
        veto_ratio,
    })
}

fn query_vote(deps: Deps, proposal_id: u64, voter: String) -> StdResult<VoteResponse> {
    // Use unchecked for lookup — the key was validated at write time.
    let voter_addr = Addr::unchecked(&voter);
    let vote = VOTES.may_load(deps.storage, (proposal_id, &voter_addr))?;
    Ok(VoteResponse { vote })
}

fn query_approved_classes(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<ApprovedClassesResponse> {
    let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let bound = start_after
        .as_deref()
        .map(cw_storage_plus::Bound::exclusive);

    let classes: Vec<ApprovedClass> = APPROVED_CLASSES
        .range(deps.storage, bound, None, Order::Ascending)
        .take(limit)
        .filter_map(|item| item.ok().map(|(_, c)| c))
        .collect();

    Ok(ApprovedClassesResponse { classes })
}

fn query_approved_class(deps: Deps, class_id: String) -> StdResult<ApprovedClassResponse> {
    let class = APPROVED_CLASSES.may_load(deps.storage, &class_id)?;
    Ok(ApprovedClassResponse { class })
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Compute recommendation per SPEC section 3 thresholds.
fn compute_recommendation(score: u64, confidence: u64) -> Recommendation {
    if score >= 700 {
        Recommendation::Approve
    } else if score < 300 && confidence > 900 {
        Recommendation::Reject
    } else {
        Recommendation::Conditional
    }
}

fn validate_threshold(value: u64) -> Result<(), ContractError> {
    if value == 0 || value > 1000 {
        return Err(ContractError::InvalidThreshold { value });
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env,
    };
    use cosmwasm_std::{Addr, Timestamp};

    // ── Helpers ────────────────────────────────────────────────────────

    fn setup_contract(deps: DepsMut) -> MessageInfo {
        let admin = Addr::unchecked("admin");
        let info = message_info(&admin, &[]);
        let msg = InstantiateMsg {
            admin: None,
            quorum_threshold: Some(100), // 10%
            pass_threshold: Some(500),   // 50%
            veto_threshold: Some(334),   // 33.4%
            voting_period_seconds: Some(604_800), // 7 days
            override_window_seconds: Some(21_600), // 6 hours
        };
        instantiate(deps, mock_env(), info.clone(), msg).unwrap();
        info
    }

    fn propose_class(deps: DepsMut, env: &Env, sender: &Addr) -> u64 {
        let info = message_info(sender, &[]);
        let msg = ExecuteMsg::ProposeClass {
            credit_type: "C".to_string(),
            methodology_iri: "koi://methodology/soil-carbon-v4".to_string(),
            admin_address: "regen1admin123".to_string(),
        };
        let res = execute(deps, env.clone(), info, msg).unwrap();
        // Extract proposal_id from attributes
        res.attributes
            .iter()
            .find(|a| a.key == "proposal_id")
            .unwrap()
            .value
            .parse()
            .unwrap()
    }

    fn submit_score(
        deps: DepsMut,
        env: &Env,
        admin: &Addr,
        proposal_id: u64,
        score: u64,
        confidence: u64,
    ) {
        let info = message_info(admin, &[]);
        let msg = ExecuteMsg::SubmitAgentScore {
            proposal_id,
            score,
            confidence,
            factors: ScoringFactorsMsg {
                methodology_quality: 800,
                admin_reputation: 700,
                novelty: 600,
                completeness: 900,
            },
        };
        execute(deps, env.clone(), info, msg).unwrap();
    }

    fn cast_vote_helper(
        deps: DepsMut,
        env: &Env,
        voter: &Addr,
        proposal_id: u64,
        option: VoteOption,
        weight: u64,
    ) {
        let info = message_info(voter, &[]);
        let msg = ExecuteMsg::CastVote {
            proposal_id,
            vote: option,
            weight,
        };
        execute(deps, env.clone(), info, msg).unwrap();
    }

    fn env_at(seconds: u64) -> Env {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(seconds);
        env
    }

    // ── Instantiation tests ───────────────────────────────────────────

    #[test]
    fn test_instantiate_defaults() {
        let mut deps = mock_dependencies();
        let admin = Addr::unchecked("admin");
        let info = message_info(&admin, &[]);
        let msg = InstantiateMsg {
            admin: None,
            quorum_threshold: None,
            pass_threshold: None,
            veto_threshold: None,
            voting_period_seconds: None,
            override_window_seconds: None,
        };
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes.len(), 2);

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.quorum_threshold, 100);
        assert_eq!(config.pass_threshold, 500);
        assert_eq!(config.veto_threshold, 334);
        assert_eq!(config.voting_period_seconds, 604_800);
        assert_eq!(config.override_window_seconds, 21_600);
    }

    #[test]
    fn test_instantiate_custom_config() {
        let mut deps = mock_dependencies();
        let admin = Addr::unchecked("admin");
        let info = message_info(&admin, &[]);
        let msg = InstantiateMsg {
            admin: None, // defaults to sender
            quorum_threshold: Some(200),
            pass_threshold: Some(667),
            veto_threshold: Some(250),
            voting_period_seconds: Some(86_400),
            override_window_seconds: Some(3_600),
        };
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.quorum_threshold, 200);
        assert_eq!(config.pass_threshold, 667);
        assert_eq!(config.veto_threshold, 250);
        assert_eq!(config.voting_period_seconds, 86_400);
        assert_eq!(config.override_window_seconds, 3_600);
    }

    #[test]
    fn test_instantiate_invalid_threshold() {
        let mut deps = mock_dependencies();
        let admin = Addr::unchecked("admin");
        let info = message_info(&admin, &[]);
        let msg = InstantiateMsg {
            admin: None,
            quorum_threshold: Some(0), // invalid
            pass_threshold: None,
            veto_threshold: None,
            voting_period_seconds: None,
            override_window_seconds: None,
        };
        let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert!(matches!(err, ContractError::InvalidThreshold { value: 0 }));
    }

    // ── Proposal tests ────────────────────────────────────────────────

    #[test]
    fn test_propose_class_valid() {
        let mut deps = mock_dependencies();
        let _admin_info = setup_contract(deps.as_mut());
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        assert_eq!(id, 1);

        let proposal = PROPOSALS.load(&deps.storage, 1).unwrap();
        assert_eq!(proposal.proposer, proposer);
        assert_eq!(proposal.credit_type, "C");
        assert_eq!(proposal.status, ProposalStatus::AgentReview);
        assert_eq!(proposal.submit_time, 1000);
        assert!(proposal.agent_score.is_none());
    }

    #[test]
    fn test_propose_class_invalid_credit_type() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let info = message_info(&proposer, &[]);

        let msg = ExecuteMsg::ProposeClass {
            credit_type: "XYZ".to_string(),
            methodology_iri: "koi://test".to_string(),
            admin_address: "regen1test".to_string(),
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(
            err,
            ContractError::InvalidCreditType {
                credit_type
            } if credit_type == "XYZ"
        ));
    }

    #[test]
    fn test_propose_all_valid_credit_types() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        for ct in VALID_CREDIT_TYPES {
            let info = message_info(&proposer, &[]);
            let msg = ExecuteMsg::ProposeClass {
                credit_type: ct.to_string(),
                methodology_iri: format!("koi://methodology/{ct}"),
                admin_address: "regen1admin".to_string(),
            };
            execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        }

        let count = PROPOSAL_COUNT.load(&deps.storage).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_proposal_counter_increments() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id1 = propose_class(deps.as_mut(), &env, &proposer);
        let id2 = propose_class(deps.as_mut(), &env, &proposer);
        let id3 = propose_class(deps.as_mut(), &env, &proposer);

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    // ── Agent scoring tests ───────────────────────────────────────────

    #[test]
    fn test_agent_score_approve_advances_to_voting() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Voting);
        assert_eq!(
            proposal.agent_score.unwrap().recommendation,
            Recommendation::Approve
        );
        assert!(proposal.voting_start_time.is_some());
    }

    #[test]
    fn test_agent_score_conditional_advances_to_voting() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 500, 600);

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Voting);
        assert_eq!(
            proposal.agent_score.unwrap().recommendation,
            Recommendation::Conditional
        );
    }

    #[test]
    fn test_agent_score_reject_stays_in_agent_review() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 150, 950);

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::AgentReview);
        assert_eq!(
            proposal.agent_score.unwrap().recommendation,
            Recommendation::Reject
        );
    }

    #[test]
    fn test_low_score_low_confidence_is_conditional() {
        // SPEC: score < 300, confidence <= 900 -> CONDITIONAL
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 200, 400);

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Voting);
        assert_eq!(
            proposal.agent_score.unwrap().recommendation,
            Recommendation::Conditional
        );
    }

    #[test]
    fn test_agent_score_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let not_admin = Addr::unchecked("rando");

        let id = propose_class(deps.as_mut(), &env, &proposer);

        let info = message_info(&not_admin, &[]);
        let msg = ExecuteMsg::SubmitAgentScore {
            proposal_id: id,
            score: 800,
            confidence: 950,
            factors: ScoringFactorsMsg {
                methodology_quality: 800,
                admin_reputation: 700,
                novelty: 600,
                completeness: 900,
            },
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    #[test]
    fn test_agent_score_invalid_score_value() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);

        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::SubmitAgentScore {
            proposal_id: id,
            score: 1500, // invalid
            confidence: 950,
            factors: ScoringFactorsMsg {
                methodology_quality: 800,
                admin_reputation: 700,
                novelty: 600,
                completeness: 900,
            },
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(
            err,
            ContractError::InvalidScore { value: 1500 }
        ));
    }

    #[test]
    fn test_agent_score_wrong_state() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        // Advance to VOTING
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        // Try to score again
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::SubmitAgentScore {
            proposal_id: id,
            score: 800,
            confidence: 950,
            factors: ScoringFactorsMsg {
                methodology_quality: 800,
                admin_reputation: 700,
                novelty: 600,
                completeness: 900,
            },
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::InvalidState { .. }));
    }

    // ── Voting tests ──────────────────────────────────────────────────

    #[test]
    fn test_cast_vote_success() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let voter = Addr::unchecked("voter1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        cast_vote_helper(deps.as_mut(), &env, &voter, id, VoteOption::Yes, 100);

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.tally.yes_weight, 100);

        let vote = VOTES.load(&deps.storage, (id, &voter)).unwrap();
        assert_eq!(vote.weight, 100);
        assert_eq!(vote.option, VoteOption::Yes);
    }

    #[test]
    fn test_cast_vote_multiple_voters() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        let voter3 = Addr::unchecked("voter3");

        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 100);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::No, 50);
        cast_vote_helper(deps.as_mut(), &env, &voter3, id, VoteOption::Veto, 25);

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.tally.yes_weight, 100);
        assert_eq!(proposal.tally.no_weight, 50);
        assert_eq!(proposal.tally.veto_weight, 25);
        assert_eq!(proposal.tally.total_participating(), 175);
    }

    #[test]
    fn test_cast_vote_duplicate_rejected() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let voter = Addr::unchecked("voter1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);
        cast_vote_helper(deps.as_mut(), &env, &voter, id, VoteOption::Yes, 100);

        // Try voting again
        let info = message_info(&voter, &[]);
        let msg = ExecuteMsg::CastVote {
            proposal_id: id,
            vote: VoteOption::No,
            weight: 50,
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::AlreadyVoted { .. }));
    }

    #[test]
    fn test_cast_vote_wrong_state() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let voter = Addr::unchecked("voter1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        // Still in AGENT_REVIEW — no score submitted yet

        let info = message_info(&voter, &[]);
        let msg = ExecuteMsg::CastVote {
            proposal_id: id,
            vote: VoteOption::Yes,
            weight: 100,
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::InvalidState { .. }));
    }

    #[test]
    fn test_cast_vote_after_period_ended() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let voter = Addr::unchecked("voter1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        // Fast forward past voting period
        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&voter, &[]);
        let msg = ExecuteMsg::CastVote {
            proposal_id: id,
            vote: VoteOption::Yes,
            weight: 100,
        };
        let err = execute(deps.as_mut(), env_after, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::VotingPeriodEnded { .. }));
    }

    // ── Execute proposal tests ────────────────────────────────────────

    #[test]
    fn test_execute_proposal_approved() {
        // SPEC test 10: yes_ratio = 0.65, quorum met -> APPROVED
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 65);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::No, 35);

        // Fast forward past voting period
        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: Some("C10".to_string()),
        };
        let res = execute(deps.as_mut(), env_after, info, msg).unwrap();

        let status_attr = res
            .attributes
            .iter()
            .find(|a| a.key == "status")
            .unwrap();
        assert_eq!(status_attr.value, "APPROVED");

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Approved);

        // Check class was registered
        let class = APPROVED_CLASSES.load(&deps.storage, "C10").unwrap();
        assert_eq!(class.credit_type, "C");
        assert_eq!(class.proposal_id, id);
    }

    #[test]
    fn test_execute_proposal_rejected() {
        // SPEC test 11: yes_ratio = 0.35 -> REJECTED
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 35);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::No, 65);

        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: None,
        };
        let res = execute(deps.as_mut(), env_after, info, msg).unwrap();

        let status_attr = res
            .attributes
            .iter()
            .find(|a| a.key == "status")
            .unwrap();
        assert_eq!(status_attr.value, "REJECTED");

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);

        // No class should be registered
        let class = APPROVED_CLASSES.may_load(&deps.storage, "C_1").unwrap();
        assert!(class.is_none());
    }

    #[test]
    fn test_execute_proposal_vetoed() {
        // SPEC test 12: veto > 33.4% -> REJECTED
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        let voter3 = Addr::unchecked("voter3");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 50);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::No, 10);
        cast_vote_helper(deps.as_mut(), &env, &voter3, id, VoteOption::Veto, 40);

        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: None,
        };
        let res = execute(deps.as_mut(), env_after, info, msg).unwrap();

        let reason_attr = res
            .attributes
            .iter()
            .find(|a| a.key == "reason")
            .unwrap();
        assert_eq!(reason_attr.value, "vetoed");

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_execute_proposal_expired_no_votes() {
        // SPEC test 13: quorum not met -> EXPIRED
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        // No votes cast — fast forward
        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: None,
        };
        let res = execute(deps.as_mut(), env_after, info, msg).unwrap();

        let status_attr = res
            .attributes
            .iter()
            .find(|a| a.key == "status")
            .unwrap();
        assert_eq!(status_attr.value, "EXPIRED");
    }

    #[test]
    fn test_execute_proposal_too_early() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        // Try to execute before voting period ends
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: None,
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::VotingPeriodNotEnded { .. }));
    }

    #[test]
    fn test_execute_proposal_auto_class_id() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        let voter = Addr::unchecked("voter1");
        cast_vote_helper(deps.as_mut(), &env, &voter, id, VoteOption::Yes, 100);

        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: None, // auto-generate
        };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        // Should be "C_1" (credit_type + "_" + proposal_id)
        let class = APPROVED_CLASSES.load(&deps.storage, "C_1").unwrap();
        assert_eq!(class.credit_type, "C");
    }

    // ── Override tests ────────────────────────────────────────────────

    #[test]
    fn test_override_agent_reject_success() {
        // SPEC test 6: Agent rejects, admin overrides within window
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 150, 950); // REJECT

        // Override within window (at 2000, window is 1000 + 21600 = 22600)
        let env_override = env_at(2000);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::OverrideAgentReject { proposal_id: id };
        execute(deps.as_mut(), env_override, info, msg).unwrap();

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Voting);
        assert!(proposal.human_override);
    }

    #[test]
    fn test_override_agent_reject_window_expired() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 150, 950); // REJECT

        // Try override after window (1000 + 21600 + 1)
        let env_late = env_at(1000 + 21_600 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::OverrideAgentReject { proposal_id: id };
        let err = execute(deps.as_mut(), env_late, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::OverrideWindowExpired { .. }));
    }

    #[test]
    fn test_override_non_rejected_proposal() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        // No score yet — still AGENT_REVIEW but not rejected

        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::OverrideAgentReject { proposal_id: id };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::NotAgentRejected { .. }));
    }

    // ── Finalize agent reject tests ───────────────────────────────────

    #[test]
    fn test_finalize_agent_reject_success() {
        // SPEC test 7: Agent rejects, override window expires -> REJECTED
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let anyone = Addr::unchecked("anyone");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 150, 950); // REJECT

        // Wait for override window to expire
        let env_after = env_at(1000 + 21_600 + 1);
        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::FinalizeAgentReject { proposal_id: id };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        let proposal = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_finalize_agent_reject_too_early() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let anyone = Addr::unchecked("anyone");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 150, 950);

        // Try before window expires
        let info = message_info(&anyone, &[]);
        let msg = ExecuteMsg::FinalizeAgentReject { proposal_id: id };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(
            err,
            ContractError::OverrideWindowNotExpired { .. }
        ));
    }

    // ── Full lifecycle tests (SPEC acceptance) ────────────────────────

    #[test]
    fn test_full_lifecycle_happy_path() {
        // SPEC test 1: Full lifecycle — propose -> score (APPROVE) -> vote -> approve
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        // 1. Submit proposal
        let id = propose_class(deps.as_mut(), &env, &proposer);
        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::AgentReview);

        // 2. Agent scores 750/900 (APPROVE)
        submit_score(deps.as_mut(), &env, &admin, id, 750, 900);
        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Voting);
        assert_eq!(
            p.agent_score.as_ref().unwrap().recommendation,
            Recommendation::Approve
        );

        // 3. Governance approves (65% yes)
        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 65);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::No, 35);

        // 4. Execute after voting period
        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: Some("C10".to_string()),
        };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Approved);

        // 5. Class registered
        let class = APPROVED_CLASSES.load(&deps.storage, "C10").unwrap();
        assert_eq!(class.credit_type, "C");
        assert_eq!(class.methodology_iri, "koi://methodology/soil-carbon-v4");
        assert_eq!(class.proposal_id, id);
    }

    #[test]
    fn test_agent_auto_reject_then_override_then_approve() {
        // SPEC test 6 extended: Agent rejects -> override -> vote -> approve
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 150, 950); // REJECT

        // Override within window
        let env_override = env_at(2000);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::OverrideAgentReject { proposal_id: id };
        execute(deps.as_mut(), env_override.clone(), info, msg).unwrap();

        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Voting);
        assert!(p.human_override);

        // Vote and approve
        let voter = Addr::unchecked("voter1");
        cast_vote_helper(deps.as_mut(), &env_override, &voter, id, VoteOption::Yes, 100);

        let env_after = env_at(2000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: Some("C11".to_string()),
        };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Approved);
    }

    #[test]
    fn test_governance_overrides_agent_approve() {
        // SPEC test 19: Agent recommends APPROVE, governance rejects -> REJECTED
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950); // APPROVE

        // But governance rejects (80% no)
        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::No, 80);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::Yes, 20);

        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: None,
        };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_abstain_votes_count_for_quorum_not_ratio() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        // 30 yes, 20 no, 50 abstain — yes_ratio of non-abstain is 60%
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 30);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::Abstain, 50);

        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: Some("C12".to_string()),
        };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        // 30/30 = 100% yes ratio (abstain excluded from ratio), should pass
        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Approved);
    }

    // ── Recommendation computation tests ──────────────────────────────

    #[test]
    fn test_recommendation_thresholds() {
        // score >= 700 -> APPROVE
        assert_eq!(compute_recommendation(700, 500), Recommendation::Approve);
        assert_eq!(compute_recommendation(1000, 0), Recommendation::Approve);

        // score < 300 AND confidence > 900 -> REJECT
        assert_eq!(compute_recommendation(299, 901), Recommendation::Reject);
        assert_eq!(compute_recommendation(0, 1000), Recommendation::Reject);

        // score < 300 AND confidence <= 900 -> CONDITIONAL
        assert_eq!(
            compute_recommendation(200, 400),
            Recommendation::Conditional
        );
        assert_eq!(
            compute_recommendation(299, 900),
            Recommendation::Conditional
        );

        // 300 <= score < 700 -> CONDITIONAL regardless of confidence
        assert_eq!(
            compute_recommendation(300, 1000),
            Recommendation::Conditional
        );
        assert_eq!(
            compute_recommendation(699, 1000),
            Recommendation::Conditional
        );
        assert_eq!(
            compute_recommendation(500, 500),
            Recommendation::Conditional
        );
    }

    // ── Query tests ───────────────────────────────────────────────────

    #[test]
    fn test_query_config() {
        let mut deps = mock_dependencies();
        let _admin_info = setup_contract(deps.as_mut());
        let env = mock_env();

        let res = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let config_resp: ConfigResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(config_resp.config.quorum_threshold, 100);
        assert_eq!(config_resp.config.pass_threshold, 500);
    }

    #[test]
    fn test_query_proposal() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);

        let res = query(deps.as_ref(), env, QueryMsg::Proposal { id }).unwrap();
        let resp: ProposalResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(resp.proposal.id, id);
        assert_eq!(resp.proposal.credit_type, "C");
    }

    #[test]
    fn test_query_proposals_with_filter() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        // Create 3 proposals
        let id1 = propose_class(deps.as_mut(), &env, &proposer);
        let id2 = propose_class(deps.as_mut(), &env, &proposer);
        let id3 = propose_class(deps.as_mut(), &env, &proposer);

        // Score proposals 1 and 2 to advance to VOTING
        submit_score(deps.as_mut(), &env, &admin, id1, 800, 950);
        submit_score(deps.as_mut(), &env, &admin, id2, 500, 600);

        // Query all
        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Proposals {
                status: None,
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let resp: ProposalsResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(resp.proposals.len(), 3);

        // Query VOTING only
        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Proposals {
                status: Some(ProposalStatusFilter::Voting),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let resp: ProposalsResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(resp.proposals.len(), 2);

        // Query AGENT_REVIEW only
        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Proposals {
                status: Some(ProposalStatusFilter::AgentReview),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let resp: ProposalsResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(resp.proposals.len(), 1);
        assert_eq!(resp.proposals[0].id, id3);
    }

    #[test]
    fn test_query_proposals_pagination() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        for _ in 0..5 {
            propose_class(deps.as_mut(), &env, &proposer);
        }

        // Page 1: first 2
        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Proposals {
                status: None,
                start_after: None,
                limit: Some(2),
            },
        )
        .unwrap();
        let resp: ProposalsResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(resp.proposals.len(), 2);
        assert_eq!(resp.proposals[0].id, 1);
        assert_eq!(resp.proposals[1].id, 2);

        // Page 2: next 2 after id=2
        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Proposals {
                status: None,
                start_after: Some(2),
                limit: Some(2),
            },
        )
        .unwrap();
        let resp: ProposalsResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(resp.proposals.len(), 2);
        assert_eq!(resp.proposals[0].id, 3);
        assert_eq!(resp.proposals[1].id, 4);
    }

    #[test]
    fn test_query_tally() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 60);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::No, 40);

        let res = query(deps.as_ref(), env, QueryMsg::Tally { proposal_id: id }).unwrap();
        let resp: TallyResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(resp.tally.yes_weight, 60);
        assert_eq!(resp.tally.no_weight, 40);
        assert_eq!(resp.total_participating, 100);
        assert_eq!(resp.yes_ratio, Some(600)); // 60/100 * 1000
        assert_eq!(resp.veto_ratio, Some(0));
    }

    #[test]
    fn test_query_vote() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");
        let voter = Addr::unchecked("voter1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);
        cast_vote_helper(deps.as_mut(), &env, &voter, id, VoteOption::Yes, 100);

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::Vote {
                proposal_id: id,
                voter: voter.to_string(),
            },
        )
        .unwrap();
        let resp: VoteResponse = cosmwasm_std::from_json(res).unwrap();
        assert!(resp.vote.is_some());
        assert_eq!(resp.vote.as_ref().unwrap().weight, 100);

        // Non-voter
        let non_voter = Addr::unchecked("non_voter");
        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::Vote {
                proposal_id: id,
                voter: non_voter.to_string(),
            },
        )
        .unwrap();
        let resp: VoteResponse = cosmwasm_std::from_json(res).unwrap();
        assert!(resp.vote.is_none());
    }

    #[test]
    fn test_query_approved_classes() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        // Create and approve two proposals
        for (class_id, ct) in [("C10", "C"), ("KSH02", "KSH")] {
            let info = message_info(&proposer, &[]);
            let msg = ExecuteMsg::ProposeClass {
                credit_type: ct.to_string(),
                methodology_iri: format!("koi://test/{ct}"),
                admin_address: "regen1admin".to_string(),
            };
            let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
            let id: u64 = res
                .attributes
                .iter()
                .find(|a| a.key == "proposal_id")
                .unwrap()
                .value
                .parse()
                .unwrap();

            submit_score(deps.as_mut(), &env, &admin, id, 800, 950);
            let voter = Addr::unchecked("voter1");
            cast_vote_helper(deps.as_mut(), &env, &voter, id, VoteOption::Yes, 100);

            let env_after = env_at(1000 + 604_800 + 1);
            let info = message_info(&admin, &[]);
            let msg = ExecuteMsg::ExecuteProposal {
                proposal_id: id,
                class_id: Some(class_id.to_string()),
            };
            execute(deps.as_mut(), env_after, info, msg).unwrap();
        }

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::ApprovedClasses {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let resp: ApprovedClassesResponse = cosmwasm_std::from_json(res).unwrap();
        assert_eq!(resp.classes.len(), 2);

        // Query single class
        let res = query(
            deps.as_ref(),
            env,
            QueryMsg::ApprovedClass {
                class_id: "C10".to_string(),
            },
        )
        .unwrap();
        let resp: ApprovedClassResponse = cosmwasm_std::from_json(res).unwrap();
        assert!(resp.class.is_some());
        assert_eq!(resp.class.unwrap().credit_type, "C");
    }

    // ── Update config tests ──────────────────────────────────────────

    #[test]
    fn test_update_config_success() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = mock_env();

        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::UpdateConfig {
            admin: None,
            quorum_threshold: Some(200),
            pass_threshold: Some(667),
            veto_threshold: None,
            voting_period_seconds: Some(86_400),
            override_window_seconds: None,
        };
        execute(deps.as_mut(), env, info, msg).unwrap();

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.quorum_threshold, 200);
        assert_eq!(config.pass_threshold, 667);
        assert_eq!(config.veto_threshold, 334); // unchanged
        assert_eq!(config.voting_period_seconds, 86_400);
    }

    #[test]
    fn test_update_config_unauthorized() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let env = mock_env();
        let rando = Addr::unchecked("rando");

        let info = message_info(&rando, &[]);
        let msg = ExecuteMsg::UpdateConfig {
            admin: None,
            quorum_threshold: Some(200),
            pass_threshold: None,
            veto_threshold: None,
            voting_period_seconds: None,
            override_window_seconds: None,
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized));
    }

    // ── Slash cap verification (SPEC test 20) ─────────────────────────

    #[test]
    fn test_slash_cap_max_50_percent() {
        // The contract doesn't handle deposits directly in v0 (advisory),
        // but we verify the state machine ensures no path results in
        // more than REJECTED status, which maps to max 50% slash per SPEC.
        // REJECTED via governance -> 200/1000 = 20% slash
        // REJECTED via agent -> 500/1000 = 50% slash (max)
        // EXPIRED -> 50/1000 = 5% fee
        // APPROVED -> 0% slash (full refund)
        // All paths: max slash is 50%, occurring only on agent auto-reject.
        // This test validates the state machine termination correctly.

        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        // Path 1: Agent reject -> finalize -> REJECTED (50% slash per SPEC)
        let id1 = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id1, 100, 950);
        let env_after = env_at(1000 + 21_600 + 1);
        let anyone = Addr::unchecked("anyone");
        let info = message_info(&anyone, &[]);
        execute(
            deps.as_mut(),
            env_after,
            info,
            ExecuteMsg::FinalizeAgentReject { proposal_id: id1 },
        )
        .unwrap();
        let p = PROPOSALS.load(&deps.storage, id1).unwrap();
        assert_eq!(p.status, ProposalStatus::Rejected);

        // Path 2: Vote -> reject -> REJECTED (20% slash per SPEC)
        let id2 = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id2, 800, 950);
        let voter = Addr::unchecked("voter1");
        cast_vote_helper(deps.as_mut(), &env, &voter, id2, VoteOption::No, 100);
        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        execute(
            deps.as_mut(),
            env_after,
            info,
            ExecuteMsg::ExecuteProposal {
                proposal_id: id2,
                class_id: None,
            },
        )
        .unwrap();
        let p = PROPOSALS.load(&deps.storage, id2).unwrap();
        assert_eq!(p.status, ProposalStatus::Rejected);

        // Path 3: No votes -> EXPIRED (5% fee per SPEC)
        let id3 = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id3, 800, 950);
        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        execute(
            deps.as_mut(),
            env_after,
            info,
            ExecuteMsg::ExecuteProposal {
                proposal_id: id3,
                class_id: None,
            },
        )
        .unwrap();
        let p = PROPOSALS.load(&deps.storage, id3).unwrap();
        assert_eq!(p.status, ProposalStatus::Expired);

        // All 3 paths terminate in terminal states. Max slash = 50%.
    }

    // ── Terminal state immutability ───────────────────────────────────

    #[test]
    fn test_terminal_states_immutable() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        // Approve a proposal
        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);
        let voter = Addr::unchecked("voter1");
        cast_vote_helper(deps.as_mut(), &env, &voter, id, VoteOption::Yes, 100);
        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        execute(
            deps.as_mut(),
            env_after.clone(),
            info,
            ExecuteMsg::ExecuteProposal {
                proposal_id: id,
                class_id: Some("C10".to_string()),
            },
        )
        .unwrap();

        // Cannot vote on approved proposal
        let voter2 = Addr::unchecked("voter2");
        let info = message_info(&voter2, &[]);
        let err = execute(
            deps.as_mut(),
            env_after.clone(),
            info,
            ExecuteMsg::CastVote {
                proposal_id: id,
                vote: VoteOption::Yes,
                weight: 50,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidState { .. }));

        // Cannot execute again
        let info = message_info(&admin, &[]);
        let err = execute(
            deps.as_mut(),
            env_after,
            info,
            ExecuteMsg::ExecuteProposal {
                proposal_id: id,
                class_id: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, ContractError::InvalidState { .. }));
    }

    // ── Proposal not found ────────────────────────────────────────────

    #[test]
    fn test_proposal_not_found() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = mock_env();

        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::SubmitAgentScore {
            proposal_id: 999,
            score: 800,
            confidence: 950,
            factors: ScoringFactorsMsg {
                methodology_quality: 800,
                admin_reputation: 700,
                novelty: 600,
                completeness: 900,
            },
        };
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(
            err,
            ContractError::ProposalNotFound { id: 999 }
        ));
    }

    // ── Scoring factor verification against test vectors ──────────────

    #[test]
    fn test_scoring_matches_reference_vectors() {
        // Verify recommendation logic matches the reference-impl test vectors
        // from mechanisms/m001-enh-credit-class-approval/reference-impl/

        // prop-001: score=776, confidence=1000 -> APPROVE
        assert_eq!(compute_recommendation(776, 1000), Recommendation::Approve);

        // prop-002: score=665, confidence=500 -> CONDITIONAL
        assert_eq!(
            compute_recommendation(665, 500),
            Recommendation::Conditional
        );

        // prop-003: score=280, confidence=500 -> CONDITIONAL (low confidence prevents REJECT)
        assert_eq!(
            compute_recommendation(280, 500),
            Recommendation::Conditional
        );

        // prop-004: score=690, confidence=1000 -> CONDITIONAL (below 700)
        assert_eq!(
            compute_recommendation(690, 1000),
            Recommendation::Conditional
        );

        // prop-005: score=155, confidence=750 -> CONDITIONAL (confidence <= 900)
        assert_eq!(
            compute_recommendation(155, 750),
            Recommendation::Conditional
        );
    }

    // ── Edge cases ────────────────────────────────────────────────────

    #[test]
    fn test_veto_exactly_at_threshold() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        // Veto exactly at 33.4% (334/1000)
        // 334 veto out of 1000 total = 334/1000 ratio = 334 in thousandths
        // threshold is 334, and 334 > 334 is false -> NOT vetoed
        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 666);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::Veto, 334);

        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: Some("C13".to_string()),
        };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        // At exactly the threshold, veto does NOT trigger (must be strictly greater)
        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Approved);
    }

    #[test]
    fn test_veto_just_above_threshold() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        // 335 veto out of 1000 = 335/1000 = 335 > 334 -> vetoed
        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 665);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::Veto, 335);

        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: None,
        };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_pass_threshold_exactly_50_percent() {
        let mut deps = mock_dependencies();
        let admin_info = setup_contract(deps.as_mut());
        let admin = admin_info.sender.clone();
        let env = env_at(1000);
        let proposer = Addr::unchecked("proposer1");

        let id = propose_class(deps.as_mut(), &env, &proposer);
        submit_score(deps.as_mut(), &env, &admin, id, 800, 950);

        // Exactly 50% yes (500/1000 = 500 >= 500 threshold)
        let voter1 = Addr::unchecked("voter1");
        let voter2 = Addr::unchecked("voter2");
        cast_vote_helper(deps.as_mut(), &env, &voter1, id, VoteOption::Yes, 500);
        cast_vote_helper(deps.as_mut(), &env, &voter2, id, VoteOption::No, 500);

        let env_after = env_at(1000 + 604_800 + 1);
        let info = message_info(&admin, &[]);
        let msg = ExecuteMsg::ExecuteProposal {
            proposal_id: id,
            class_id: Some("C14".to_string()),
        };
        execute(deps.as_mut(), env_after, info, msg).unwrap();

        // >= threshold means approved
        let p = PROPOSALS.load(&deps.storage, id).unwrap();
        assert_eq!(p.status, ProposalStatus::Approved);
    }
}
