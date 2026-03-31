use cosmwasm_std::{
    entry_point, to_json_binary, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    AllAttestationTypesResponse, AttestationResponse, AttestationTypeInput,
    AttestationTypeResponse, AttestationsResponse, BondInfoResponse, ChallengeResolutionInput,
    ChallengeResponse, ChallengesResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
};
use crate::state::{
    Attestation, AttestationStatus, AttestationType, Challenge, ChallengeResolution,
    ChallengeStatus, Config, ACTIVATION_DELAY_SECS, ATTESTATIONS, ATTESTATION_CHALLENGE,
    ATTESTATION_TYPES, BASIS_POINTS_DIVISOR, CHALLENGES, CONFIG, NEXT_ATTESTATION_ID,
    NEXT_CHALLENGE_ID,
};

const CONTRACT_NAME: &str = "crates.io:attestation-bonding";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ============================================================================
// Instantiate
// ============================================================================

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let arbiter_dao = deps.api.addr_validate(&msg.arbiter_dao)?;

    let config = Config {
        admin: info.sender.clone(),
        arbiter_dao,
        min_challenge_deposit_ratio: msg.min_challenge_deposit_ratio,
        arbiter_fee_ratio: msg.arbiter_fee_ratio,
        bond_denom: msg.bond_denom,
    };
    CONFIG.save(deps.storage, &config)?;

    NEXT_ATTESTATION_ID.save(deps.storage, &1u64)?;
    NEXT_CHALLENGE_ID.save(deps.storage, &1u64)?;

    // Register initial attestation types
    for at in msg.attestation_types {
        let att_type = AttestationType {
            name: at.name.clone(),
            min_bond: at.min_bond,
            lock_period_days: at.lock_period_days,
            challenge_window_days: at.challenge_window_days,
        };
        ATTESTATION_TYPES.save(deps.storage, &at.name, &att_type)?;
    }

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("contract", CONTRACT_NAME))
}

// ============================================================================
// Execute
// ============================================================================

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateAttestation {
            attestation_type,
            attestation_iri,
            beneficiary,
        } => execute_create_attestation(deps, env, info, attestation_type, attestation_iri, beneficiary),
        ExecuteMsg::ActivateAttestation { attestation_id } => {
            execute_activate_attestation(deps, env, attestation_id)
        }
        ExecuteMsg::ChallengeAttestation {
            attestation_id,
            evidence_iri,
        } => execute_challenge_attestation(deps, env, info, attestation_id, evidence_iri),
        ExecuteMsg::ResolveChallenge {
            challenge_id,
            resolution,
        } => execute_resolve_challenge(deps, env, info, challenge_id, resolution),
        ExecuteMsg::ReleaseBond { attestation_id } => {
            execute_release_bond(deps, env, info, attestation_id)
        }
        ExecuteMsg::UpdateConfig {
            arbiter_dao,
            min_challenge_deposit_ratio,
            arbiter_fee_ratio,
        } => execute_update_config(deps, info, arbiter_dao, min_challenge_deposit_ratio, arbiter_fee_ratio),
        ExecuteMsg::AddAttestationType { attestation_type } => {
            execute_add_attestation_type(deps, info, attestation_type)
        }
    }
}

// ---------------------------------------------------------------------------
// CreateAttestation
// ---------------------------------------------------------------------------

fn execute_create_attestation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    attestation_type: String,
    attestation_iri: String,
    beneficiary: Option<String>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Load attestation type config
    let att_type = ATTESTATION_TYPES
        .load(deps.storage, &attestation_type)
        .map_err(|_| ContractError::UnknownAttestationType {})?;

    // Validate bond amount — must send at least min_bond in the required denom
    let bond = info
        .funds
        .iter()
        .find(|c| c.denom == config.bond_denom)
        .ok_or(ContractError::NoBondProvided {})?;

    if bond.amount < att_type.min_bond {
        return Err(ContractError::InsufficientBond {
            required: att_type.min_bond,
            provided: bond.amount,
        });
    }

    // Calculate timestamps
    let lock_expires_at = env.block.time.plus_days(att_type.lock_period_days);
    let challenge_window_closes_at = env.block.time.plus_days(att_type.challenge_window_days);
    let activation_eligible_at = env.block.time.plus_seconds(ACTIVATION_DELAY_SECS);

    // Validate beneficiary if provided
    let beneficiary_addr = beneficiary
        .map(|b| deps.api.addr_validate(&b))
        .transpose()?;

    // Allocate ID and save
    let id = NEXT_ATTESTATION_ID.load(deps.storage)?;
    let attestation = Attestation {
        id,
        attester: info.sender.clone(),
        attestation_type: attestation_type.clone(),
        attestation_iri: attestation_iri.clone(),
        beneficiary: beneficiary_addr,
        bond: bond.clone(),
        status: AttestationStatus::Bonded,
        bonded_at: env.block.time,
        activated_at: None,
        lock_expires_at,
        challenge_window_closes_at,
        activation_eligible_at,
    };

    ATTESTATIONS.save(deps.storage, id, &attestation)?;
    NEXT_ATTESTATION_ID.save(deps.storage, &(id + 1))?;

    Ok(Response::new()
        .add_attribute("action", "create_attestation")
        .add_attribute("attestation_id", id.to_string())
        .add_attribute("attester", info.sender)
        .add_attribute("attestation_type", attestation_type)
        .add_attribute("bond_amount", bond.amount))
}

// ---------------------------------------------------------------------------
// ActivateAttestation — permissionless crank after 48h delay
// ---------------------------------------------------------------------------

fn execute_activate_attestation(
    deps: DepsMut,
    env: Env,
    attestation_id: u64,
) -> Result<Response, ContractError> {
    let mut attestation = ATTESTATIONS
        .load(deps.storage, attestation_id)
        .map_err(|_| ContractError::AttestationNotFound {})?;

    // Must be in Bonded state
    if attestation.status != AttestationStatus::Bonded {
        return Err(ContractError::AttestationNotReleasable {});
    }

    // 48h activation delay must have passed
    if env.block.time < attestation.activation_eligible_at {
        return Err(ContractError::LockPeriodNotExpired {});
    }

    attestation.status = AttestationStatus::Active;
    attestation.activated_at = Some(env.block.time);
    ATTESTATIONS.save(deps.storage, attestation_id, &attestation)?;

    Ok(Response::new()
        .add_attribute("action", "activate_attestation")
        .add_attribute("attestation_id", attestation_id.to_string()))
}

// ---------------------------------------------------------------------------
// ChallengeAttestation
// ---------------------------------------------------------------------------

fn execute_challenge_attestation(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    attestation_id: u64,
    evidence_iri: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut attestation = ATTESTATIONS
        .load(deps.storage, attestation_id)
        .map_err(|_| ContractError::AttestationNotFound {})?;

    // Must be in Bonded or Active state
    match attestation.status {
        AttestationStatus::Bonded | AttestationStatus::Active => {}
        _ => return Err(ContractError::AttestationNotChallengeable {}),
    }

    // Check challenge window
    if env.block.time > attestation.challenge_window_closes_at {
        return Err(ContractError::ChallengeWindowClosed {});
    }

    // Only one active challenge at a time
    if ATTESTATION_CHALLENGE.has(deps.storage, attestation_id) {
        return Err(ContractError::AlreadyChallenged {});
    }

    // Validate deposit: must be >= bond * challenge_deposit_ratio / BASIS_POINTS_DIVISOR
    let min_deposit = attestation
        .bond
        .amount
        .multiply_ratio(config.min_challenge_deposit_ratio, Uint128::from(BASIS_POINTS_DIVISOR));

    let deposit = info
        .funds
        .iter()
        .find(|c| c.denom == config.bond_denom)
        .ok_or(ContractError::NoDepositProvided {})?;

    if deposit.amount < min_deposit {
        return Err(ContractError::InsufficientChallengeDeposit {
            required: min_deposit,
            provided: deposit.amount,
        });
    }

    // Create challenge
    let challenge_id = NEXT_CHALLENGE_ID.load(deps.storage)?;
    let challenge = Challenge {
        id: challenge_id,
        attestation_id,
        challenger: info.sender.clone(),
        evidence_iri,
        deposit: deposit.clone(),
        status: ChallengeStatus::Pending,
        created_at: env.block.time,
        resolved_at: None,
        resolution: None,
    };

    CHALLENGES.save(deps.storage, challenge_id, &challenge)?;
    ATTESTATION_CHALLENGE.save(deps.storage, attestation_id, &challenge_id)?;
    NEXT_CHALLENGE_ID.save(deps.storage, &(challenge_id + 1))?;

    // Update attestation status
    attestation.status = AttestationStatus::Challenged;
    ATTESTATIONS.save(deps.storage, attestation_id, &attestation)?;

    Ok(Response::new()
        .add_attribute("action", "challenge_attestation")
        .add_attribute("challenge_id", challenge_id.to_string())
        .add_attribute("attestation_id", attestation_id.to_string())
        .add_attribute("challenger", info.sender)
        .add_attribute("deposit_amount", deposit.amount))
}

// ---------------------------------------------------------------------------
// ResolveChallenge — only arbiter DAO / admin
// ---------------------------------------------------------------------------

fn execute_resolve_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    challenge_id: u64,
    resolution: ChallengeResolutionInput,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Only arbiter DAO or admin can resolve
    if info.sender != config.arbiter_dao && info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    let mut challenge = CHALLENGES
        .load(deps.storage, challenge_id)
        .map_err(|_| ContractError::ChallengeNotFound {})?;

    // Cannot re-resolve
    if challenge.status == ChallengeStatus::Resolved {
        return Err(ContractError::ChallengeAlreadyResolved {});
    }

    let mut attestation = ATTESTATIONS.load(deps.storage, challenge.attestation_id)?;

    // Arbiter neutrality: resolver cannot be attester or challenger
    if info.sender == attestation.attester || info.sender == challenge.challenger {
        return Err(ContractError::ArbiterConflict {});
    }

    // Calculate arbiter fee
    let arbiter_fee = attestation
        .bond
        .amount
        .multiply_ratio(config.arbiter_fee_ratio, Uint128::from(BASIS_POINTS_DIVISOR));

    let mut messages: Vec<CosmosMsg> = vec![];

    match resolution {
        ChallengeResolutionInput::AttesterWins => {
            // Attester gets: bond + challenge_deposit - arbiter_fee
            let attester_amount = attestation
                .bond
                .amount
                .checked_add(challenge.deposit.amount)
                .map_err(|_| ContractError::Overflow {})?
                .checked_sub(arbiter_fee)
                .map_err(|_| ContractError::Overflow {})?;

            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: attestation.attester.to_string(),
                amount: vec![Coin {
                    denom: config.bond_denom.clone(),
                    amount: attester_amount,
                }],
            }));

            attestation.status = AttestationStatus::ResolvedValid;
            challenge.resolution = Some(ChallengeResolution::AttesterWins);
        }
        ChallengeResolutionInput::ChallengerWins => {
            // Challenger gets: 50% bond + deposit - arbiter_fee
            let half_bond = attestation
                .bond
                .amount
                .multiply_ratio(50u128, 100u128);

            let challenger_amount = half_bond
                .checked_add(challenge.deposit.amount)
                .map_err(|_| ContractError::Overflow {})?
                .checked_sub(arbiter_fee)
                .map_err(|_| ContractError::Overflow {})?;

            // Community pool gets remaining 50% of bond
            let community_amount = attestation
                .bond
                .amount
                .checked_sub(half_bond)
                .map_err(|_| ContractError::Overflow {})?;

            messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: challenge.challenger.to_string(),
                amount: vec![Coin {
                    denom: config.bond_denom.clone(),
                    amount: challenger_amount,
                }],
            }));

            // Community pool — in production this would use x/distribution FundCommunityPool.
            // For v1, we send to the arbiter_dao which acts as community treasury.
            if !community_amount.is_zero() {
                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    to_address: config.arbiter_dao.to_string(),
                    amount: vec![Coin {
                        denom: config.bond_denom.clone(),
                        amount: community_amount,
                    }],
                }));
            }

            attestation.status = AttestationStatus::Slashed;
            challenge.resolution = Some(ChallengeResolution::ChallengerWins);
        }
    }

    // Send arbiter fee
    if !arbiter_fee.is_zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.arbiter_dao.to_string(),
            amount: vec![Coin {
                denom: config.bond_denom.clone(),
                amount: arbiter_fee,
            }],
        }));
    }

    challenge.status = ChallengeStatus::Resolved;
    challenge.resolved_at = Some(env.block.time);

    CHALLENGES.save(deps.storage, challenge_id, &challenge)?;
    ATTESTATIONS.save(deps.storage, challenge.attestation_id, &attestation)?;

    // Remove the active-challenge mapping so re-challenge is possible after resolution
    ATTESTATION_CHALLENGE.remove(deps.storage, challenge.attestation_id);

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "resolve_challenge")
        .add_attribute("challenge_id", challenge_id.to_string())
        .add_attribute("attestation_id", challenge.attestation_id.to_string())
        .add_attribute(
            "resolution",
            match &challenge.resolution {
                Some(ChallengeResolution::AttesterWins) => "attester_wins",
                Some(ChallengeResolution::ChallengerWins) => "challenger_wins",
                None => "none",
            },
        ))
}

// ---------------------------------------------------------------------------
// ReleaseBond
// ---------------------------------------------------------------------------

fn execute_release_bond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    attestation_id: u64,
) -> Result<Response, ContractError> {
    let mut attestation = ATTESTATIONS
        .load(deps.storage, attestation_id)
        .map_err(|_| ContractError::AttestationNotFound {})?;

    // Only the attester can release
    if info.sender != attestation.attester {
        return Err(ContractError::NotAttester {});
    }

    // Must be Active (not Bonded, Challenged, or terminal)
    if attestation.status != AttestationStatus::Active {
        return Err(ContractError::AttestationNotReleasable {});
    }

    // Lock period must have expired
    if env.block.time < attestation.lock_expires_at {
        return Err(ContractError::LockPeriodNotExpired {});
    }

    // No active challenge
    if ATTESTATION_CHALLENGE.has(deps.storage, attestation_id) {
        return Err(ContractError::AttestationNotReleasable {});
    }

    // Release bond to attester
    attestation.status = AttestationStatus::Released;
    ATTESTATIONS.save(deps.storage, attestation_id, &attestation)?;

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: attestation.attester.to_string(),
        amount: vec![attestation.bond.clone()],
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "release_bond")
        .add_attribute("attestation_id", attestation_id.to_string())
        .add_attribute("amount", attestation.bond.amount))
}

// ---------------------------------------------------------------------------
// UpdateConfig — admin only
// ---------------------------------------------------------------------------

fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    arbiter_dao: Option<String>,
    min_challenge_deposit_ratio: Option<Uint128>,
    arbiter_fee_ratio: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(dao) = arbiter_dao {
        config.arbiter_dao = deps.api.addr_validate(&dao)?;
    }
    if let Some(ratio) = min_challenge_deposit_ratio {
        config.min_challenge_deposit_ratio = ratio;
    }
    if let Some(ratio) = arbiter_fee_ratio {
        config.arbiter_fee_ratio = ratio;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

// ---------------------------------------------------------------------------
// AddAttestationType — admin only
// ---------------------------------------------------------------------------

fn execute_add_attestation_type(
    deps: DepsMut,
    info: MessageInfo,
    attestation_type: AttestationTypeInput,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    // Reject duplicates
    if ATTESTATION_TYPES.has(deps.storage, &attestation_type.name) {
        return Err(ContractError::AttestationTypeAlreadyExists {
            name: attestation_type.name,
        });
    }

    let att_type = AttestationType {
        name: attestation_type.name.clone(),
        min_bond: attestation_type.min_bond,
        lock_period_days: attestation_type.lock_period_days,
        challenge_window_days: attestation_type.challenge_window_days,
    };

    ATTESTATION_TYPES.save(deps.storage, &attestation_type.name, &att_type)?;

    Ok(Response::new()
        .add_attribute("action", "add_attestation_type")
        .add_attribute("name", attestation_type.name))
}

// ============================================================================
// Query
// ============================================================================

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Attestation { id } => to_json_binary(&query_attestation(deps, id)?),
        QueryMsg::AttestationsByAttester {
            attester,
            start_after,
            limit,
        } => to_json_binary(&query_attestations_by_attester(deps, attester, start_after, limit)?),
        QueryMsg::Challenge { id } => to_json_binary(&query_challenge(deps, id)?),
        QueryMsg::ChallengesByAttestation { attestation_id } => {
            to_json_binary(&query_challenges_by_attestation(deps, attestation_id)?)
        }
        QueryMsg::AttestationType { name } => {
            to_json_binary(&query_attestation_type(deps, &name)?)
        }
        QueryMsg::AllAttestationTypes {} => to_json_binary(&query_all_attestation_types(deps)?),
        QueryMsg::BondInfo { attestation_id } => {
            to_json_binary(&query_bond_info(deps, env, attestation_id)?)
        }
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

fn query_attestation(deps: Deps, id: u64) -> StdResult<AttestationResponse> {
    let attestation = ATTESTATIONS.load(deps.storage, id)?;
    Ok(AttestationResponse { attestation })
}

fn query_attestations_by_attester(
    deps: Deps,
    attester: String,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<AttestationsResponse> {
    let attester_addr = deps.api.addr_validate(&attester)?;
    let limit = limit.unwrap_or(30).min(100) as usize;
    let start = start_after.unwrap_or(0);

    let attestations: Vec<Attestation> = ATTESTATIONS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .filter(|(id, a)| a.attester == attester_addr && *id > start)
        .take(limit)
        .map(|(_, a)| a)
        .collect();

    Ok(AttestationsResponse { attestations })
}

fn query_challenge(deps: Deps, id: u64) -> StdResult<ChallengeResponse> {
    let challenge = CHALLENGES.load(deps.storage, id)?;
    Ok(ChallengeResponse { challenge })
}

fn query_challenges_by_attestation(
    deps: Deps,
    attestation_id: u64,
) -> StdResult<ChallengesResponse> {
    let challenges: Vec<Challenge> = CHALLENGES
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .filter(|(_, c)| c.attestation_id == attestation_id)
        .map(|(_, c)| c)
        .collect();

    Ok(ChallengesResponse { challenges })
}

fn query_attestation_type(deps: Deps, name: &str) -> StdResult<AttestationTypeResponse> {
    let attestation_type = ATTESTATION_TYPES.load(deps.storage, name)?;
    Ok(AttestationTypeResponse { attestation_type })
}

fn query_all_attestation_types(deps: Deps) -> StdResult<AllAttestationTypesResponse> {
    let attestation_types: Vec<AttestationType> = ATTESTATION_TYPES
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|item| item.ok())
        .map(|(_, at)| at)
        .collect();

    Ok(AllAttestationTypesResponse { attestation_types })
}

fn query_bond_info(deps: Deps, env: Env, attestation_id: u64) -> StdResult<BondInfoResponse> {
    let attestation = ATTESTATIONS.load(deps.storage, attestation_id)?;

    let is_locked = env.block.time < attestation.lock_expires_at;
    let is_challengeable = matches!(
        attestation.status,
        AttestationStatus::Bonded | AttestationStatus::Active
    ) && env.block.time <= attestation.challenge_window_closes_at;

    Ok(BondInfoResponse {
        attestation_id,
        bond_amount: attestation.bond.amount,
        bond_denom: attestation.bond.denom,
        status: attestation.status,
        lock_expires_at: attestation.lock_expires_at.seconds(),
        challenge_window_closes_at: attestation.challenge_window_closes_at.seconds(),
        is_locked,
        is_challengeable,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env, MockApi};
    use cosmwasm_std::Timestamp;

    // -----------------------------------------------------------------------
    // Helpers — all addresses use MockApi::addr_make for valid bech32
    // -----------------------------------------------------------------------

    const DENOM: &str = "uregen";

    fn addr(label: &str) -> cosmwasm_std::Addr {
        MockApi::default().addr_make(label)
    }

    fn default_instantiate_msg() -> InstantiateMsg {
        InstantiateMsg {
            arbiter_dao: addr("arbiter_dao").to_string(),
            min_challenge_deposit_ratio: Uint128::from(1000u128), // 10%
            arbiter_fee_ratio: Uint128::from(500u128),            // 5%
            bond_denom: DENOM.to_string(),
            attestation_types: vec![
                AttestationTypeInput {
                    name: "ProjectBoundary".to_string(),
                    min_bond: Uint128::from(500u128),
                    lock_period_days: 90,
                    challenge_window_days: 60,
                },
                AttestationTypeInput {
                    name: "BaselineMeasurement".to_string(),
                    min_bond: Uint128::from(1000u128),
                    lock_period_days: 180,
                    challenge_window_days: 120,
                },
                AttestationTypeInput {
                    name: "CreditIssuanceClaim".to_string(),
                    min_bond: Uint128::from(2000u128),
                    lock_period_days: 365,
                    challenge_window_days: 300,
                },
                AttestationTypeInput {
                    name: "MethodologyValidation".to_string(),
                    min_bond: Uint128::from(5000u128),
                    lock_period_days: 730,
                    challenge_window_days: 600,
                },
            ],
        }
    }

    fn setup_contract() -> (
        cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        Env,
    ) {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = message_info(&addr("admin"), &[]);

        instantiate(deps.as_mut(), env.clone(), info, default_instantiate_msg()).unwrap();
        (deps, env)
    }

    fn env_at(secs: u64) -> Env {
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(secs);
        env
    }

    fn coin(amount: u128) -> Vec<Coin> {
        vec![Coin {
            denom: DENOM.to_string(),
            amount: Uint128::from(amount),
        }]
    }

    // -----------------------------------------------------------------------
    // Instantiate tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_instantiate_stores_config() {
        let (deps, _env) = setup_contract();
        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_eq!(config.admin, addr("admin"));
        assert_eq!(config.arbiter_dao, addr("arbiter_dao"));
        assert_eq!(config.min_challenge_deposit_ratio, Uint128::from(1000u128));
        assert_eq!(config.arbiter_fee_ratio, Uint128::from(500u128));
        assert_eq!(config.bond_denom, DENOM);
    }

    #[test]
    fn test_instantiate_registers_attestation_types() {
        let (deps, _env) = setup_contract();
        let pb = ATTESTATION_TYPES
            .load(deps.as_ref().storage, "ProjectBoundary")
            .unwrap();
        assert_eq!(pb.min_bond, Uint128::from(500u128));
        assert_eq!(pb.lock_period_days, 90);
        assert_eq!(pb.challenge_window_days, 60);

        let mv = ATTESTATION_TYPES
            .load(deps.as_ref().storage, "MethodologyValidation")
            .unwrap();
        assert_eq!(mv.min_bond, Uint128::from(5000u128));
        assert_eq!(mv.lock_period_days, 730);
    }

    // -----------------------------------------------------------------------
    // CreateAttestation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_attestation_happy_path() {
        let (mut deps, env) = setup_contract();
        let attester = addr("attester1");
        let beneficiary = addr("beneficiary1");
        let info = message_info(&attester, &coin(1000));

        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "BaselineMeasurement".to_string(),
                attestation_iri: "koi://att/baseline-42".to_string(),
                beneficiary: Some(beneficiary.to_string()),
            },
        )
        .unwrap();

        assert_eq!(res.attributes.len(), 5);
        assert_eq!(res.attributes[0].value, "create_attestation");
        assert_eq!(res.attributes[1].value, "1"); // id

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.attester, attester);
        assert_eq!(att.attestation_type, "BaselineMeasurement");
        assert_eq!(att.status, AttestationStatus::Bonded);
        assert_eq!(att.bond.amount, Uint128::from(1000u128));
        assert_eq!(att.beneficiary, Some(beneficiary));
        assert!(att.activated_at.is_none());

        // Next ID incremented
        let next_id = NEXT_ATTESTATION_ID.load(deps.as_ref().storage).unwrap();
        assert_eq!(next_id, 2);
    }

    #[test]
    fn test_create_attestation_insufficient_bond() {
        let (mut deps, env) = setup_contract();
        let info = message_info(&addr("attester1"), &coin(200));

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "ProjectBoundary".to_string(),
                attestation_iri: "koi://att/boundary-1".to_string(),
                beneficiary: None,
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::InsufficientBond {
                required: Uint128::from(500u128),
                provided: Uint128::from(200u128),
            }
        );
    }

    #[test]
    fn test_create_attestation_unknown_type() {
        let (mut deps, env) = setup_contract();
        let info = message_info(&addr("attester1"), &coin(1000));

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "InvalidType".to_string(),
                attestation_iri: "koi://att/invalid".to_string(),
                beneficiary: None,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::UnknownAttestationType {});
    }

    #[test]
    fn test_create_attestation_no_bond_provided() {
        let (mut deps, env) = setup_contract();
        let info = message_info(&addr("attester1"), &[]); // no funds

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "ProjectBoundary".to_string(),
                attestation_iri: "koi://att/boundary-1".to_string(),
                beneficiary: None,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::NoBondProvided {});
    }

    #[test]
    fn test_create_attestation_over_minimum_bond() {
        let (mut deps, env) = setup_contract();
        // Bond 2x minimum
        let info = message_info(&addr("attester1"), &coin(1000));

        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "ProjectBoundary".to_string(),
                attestation_iri: "koi://att/boundary-2x".to_string(),
                beneficiary: None,
            },
        );
        assert!(res.is_ok());

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.bond.amount, Uint128::from(1000u128));
    }

    // -----------------------------------------------------------------------
    // ActivateAttestation tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_activate_attestation_happy_path() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();

        // Create attestation
        let info = message_info(&addr("attester1"), &coin(500));
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "ProjectBoundary".to_string(),
                attestation_iri: "koi://att/pb-1".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        // Attempt activation before 48h — should fail
        let env_early = env_at(base_time + 47 * 3600);
        let err = execute(
            deps.as_mut(),
            env_early,
            message_info(&addr("anyone"), &[]),
            ExecuteMsg::ActivateAttestation { attestation_id: 1 },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::LockPeriodNotExpired {});

        // Activation after 48h — should succeed
        let env_after = env_at(base_time + 49 * 3600);
        execute(
            deps.as_mut(),
            env_after.clone(),
            message_info(&addr("anyone"), &[]),
            ExecuteMsg::ActivateAttestation { attestation_id: 1 },
        )
        .unwrap();

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Active);
        assert_eq!(att.activated_at, Some(env_after.block.time));
    }

    // -----------------------------------------------------------------------
    // ChallengeAttestation tests
    // -----------------------------------------------------------------------

    fn create_and_activate(
        deps: &mut cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        base_time: u64,
    ) {
        // Create
        let env = env_at(base_time);
        let info = message_info(&addr("attester1"), &coin(1000));
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "BaselineMeasurement".to_string(),
                attestation_iri: "koi://att/baseline-42".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        // Activate after 48h
        let env = env_at(base_time + 49 * 3600);
        execute(
            deps.as_mut(),
            env,
            message_info(&addr("crank"), &[]),
            ExecuteMsg::ActivateAttestation { attestation_id: 1 },
        )
        .unwrap();
    }

    #[test]
    fn test_challenge_active_attestation() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        let challenger = addr("challenger1");

        // Challenge with 10% deposit (100 uregen for 1000 bond)
        let challenge_time = base_time + 50 * 3600;
        let env = env_at(challenge_time);
        let info = message_info(&challenger, &coin(100));

        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/dispute-42".to_string(),
            },
        )
        .unwrap();

        assert_eq!(res.attributes[0].value, "challenge_attestation");

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Challenged);

        let challenge = CHALLENGES.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(challenge.challenger, challenger);
        assert_eq!(challenge.deposit.amount, Uint128::from(100u128));
    }

    #[test]
    fn test_challenge_during_activation_delay() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();

        // Create attestation (stays in Bonded state)
        let info = message_info(&addr("attester1"), &coin(1000));
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "BaselineMeasurement".to_string(),
                attestation_iri: "koi://att/baseline-early".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        // Challenge during 48h delay — should succeed (Bonded is challengeable)
        let env = env_at(base_time + 24 * 3600);
        let info = message_info(&addr("challenger1"), &coin(100));
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/early".to_string(),
            },
        );
        assert!(res.is_ok());

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Challenged);
    }

    #[test]
    fn test_challenge_insufficient_deposit() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        // Deposit only 50 uregen (need 100 = 10% of 1000 bond)
        let env = env_at(base_time + 50 * 3600);
        let info = message_info(&addr("challenger1"), &coin(50));

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/weak".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::InsufficientChallengeDeposit {
                required: Uint128::from(100u128),
                provided: Uint128::from(50u128),
            }
        );
    }

    #[test]
    fn test_challenge_window_expired() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        // BaselineMeasurement challenge window = 120 days
        let past_window = base_time + 121 * 24 * 3600;
        let env = env_at(past_window);
        let info = message_info(&addr("challenger1"), &coin(100));

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/late".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::ChallengeWindowClosed {});
    }

    #[test]
    fn test_challenge_released_attestation() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        // Release bond after lock period (180 days for BaselineMeasurement)
        let env = env_at(base_time + 181 * 24 * 3600);
        let info = message_info(&addr("attester1"), &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::ReleaseBond { attestation_id: 1 },
        )
        .unwrap();

        // Try to challenge Released attestation
        let info = message_info(&addr("challenger1"), &coin(100));
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/too-late".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::AttestationNotChallengeable {});
    }

    #[test]
    fn test_single_active_challenge_only() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        // First challenge succeeds
        let env = env_at(base_time + 50 * 3600);
        let info = message_info(&addr("challenger1"), &coin(100));
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/first".to_string(),
            },
        )
        .unwrap();

        // Second challenge fails — attestation is already in Challenged state,
        // so it is no longer challengeable (not Bonded or Active).
        let info = message_info(&addr("challenger2"), &coin(100));
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/second".to_string(),
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::AttestationNotChallengeable {});
    }

    // -----------------------------------------------------------------------
    // ResolveChallenge tests
    // -----------------------------------------------------------------------

    fn create_activate_and_challenge(
        deps: &mut cosmwasm_std::OwnedDeps<
            cosmwasm_std::MemoryStorage,
            cosmwasm_std::testing::MockApi,
            cosmwasm_std::testing::MockQuerier,
        >,
        base_time: u64,
    ) {
        create_and_activate(deps, base_time);

        let env = env_at(base_time + 50 * 3600);
        let info = message_info(&addr("challenger1"), &coin(100));
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/dispute".to_string(),
            },
        )
        .unwrap();
    }

    #[test]
    fn test_resolve_attester_wins() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        let attester = addr("attester1");

        let env = env_at(base_time + 60 * 3600);
        let info = message_info(&addr("arbiter_dao"), &[]);
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::AttesterWins,
            },
        )
        .unwrap();

        // Attester gets: 1000 (bond) + 100 (deposit) - 50 (5% fee) = 1050
        assert!(res.messages.len() >= 1);
        let bank_msg = &res.messages[0].msg;
        match bank_msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, &attester.to_string());
                assert_eq!(amount[0].amount, Uint128::from(1050u128));
            }
            _ => panic!("expected bank send"),
        }

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::ResolvedValid);

        let challenge = CHALLENGES.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(challenge.status, ChallengeStatus::Resolved);
        assert_eq!(
            challenge.resolution,
            Some(ChallengeResolution::AttesterWins)
        );
    }

    #[test]
    fn test_resolve_challenger_wins() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        let challenger = addr("challenger1");
        let arbiter = addr("arbiter_dao");

        let env = env_at(base_time + 60 * 3600);
        let info = message_info(&arbiter, &[]);
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::ChallengerWins,
            },
        )
        .unwrap();

        // Challenger gets: 500 (50% bond) + 100 (deposit) - 50 (5% fee) = 550
        // Community (arbiter_dao in v1) gets: 500 (50% bond)
        // Arbiter fee: 50
        let msgs = &res.messages;
        assert!(msgs.len() >= 2);

        // First message: challenger payment
        match &msgs[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, &challenger.to_string());
                assert_eq!(amount[0].amount, Uint128::from(550u128));
            }
            _ => panic!("expected bank send to challenger"),
        }

        // Second message: community pool (sent to arbiter_dao in v1)
        match &msgs[1].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, &arbiter.to_string());
                assert_eq!(amount[0].amount, Uint128::from(500u128));
            }
            _ => panic!("expected bank send to community"),
        }

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Slashed);
    }

    #[test]
    fn test_resolve_unauthorized() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        let env = env_at(base_time + 60 * 3600);
        let info = message_info(&addr("random_user"), &[]);

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::AttesterWins,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_resolve_arbiter_is_attester() {
        let (mut deps, _env) = setup_contract();

        // Create attestation where attester IS the admin (who can also resolve)
        let admin = addr("admin");
        let base_time = 1_000_000u64;
        let env = env_at(base_time);
        let info = message_info(&admin, &coin(1000));
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "BaselineMeasurement".to_string(),
                attestation_iri: "koi://att/self-attest".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        // Activate
        let env = env_at(base_time + 49 * 3600);
        execute(
            deps.as_mut(),
            env,
            message_info(&addr("crank"), &[]),
            ExecuteMsg::ActivateAttestation { attestation_id: 1 },
        )
        .unwrap();

        // Challenge
        let env = env_at(base_time + 50 * 3600);
        let info = message_info(&addr("challenger1"), &coin(100));
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/conflict".to_string(),
            },
        )
        .unwrap();

        // Admin tries to resolve own attestation — arbiter conflict
        let env = env_at(base_time + 60 * 3600);
        let info = message_info(&admin, &[]);
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::AttesterWins,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::ArbiterConflict {});
    }

    #[test]
    fn test_resolve_already_resolved() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        let arbiter = addr("arbiter_dao");

        // Resolve first time
        let env = env_at(base_time + 60 * 3600);
        let info = message_info(&arbiter, &[]);
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::AttesterWins,
            },
        )
        .unwrap();

        // Attempt to re-resolve
        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::ChallengerWins,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::ChallengeAlreadyResolved {});
    }

    // -----------------------------------------------------------------------
    // ReleaseBond tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_release_bond_happy_path() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        let attester = addr("attester1");

        // BaselineMeasurement lock = 180 days
        let env = env_at(base_time + 181 * 24 * 3600);
        let info = message_info(&attester, &[]);

        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseBond { attestation_id: 1 },
        )
        .unwrap();

        assert_eq!(res.attributes[0].value, "release_bond");

        // Bond returned to attester
        assert_eq!(res.messages.len(), 1);
        match &res.messages[0].msg {
            CosmosMsg::Bank(BankMsg::Send { to_address, amount }) => {
                assert_eq!(to_address, &attester.to_string());
                assert_eq!(amount[0].amount, Uint128::from(1000u128));
            }
            _ => panic!("expected bank send"),
        }

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Released);
    }

    #[test]
    fn test_release_bond_before_lock_expires() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        // Only 90 days passed (need 180 for BaselineMeasurement)
        let env = env_at(base_time + 90 * 24 * 3600);
        let info = message_info(&addr("attester1"), &[]);

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseBond { attestation_id: 1 },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::LockPeriodNotExpired {});
    }

    #[test]
    fn test_release_bond_not_attester() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        let env = env_at(base_time + 181 * 24 * 3600);
        let info = message_info(&addr("random_user"), &[]);

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseBond { attestation_id: 1 },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::NotAttester {});
    }

    #[test]
    fn test_release_bond_during_challenge() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        // Even after lock period, should fail because challenged
        let env = env_at(base_time + 181 * 24 * 3600);
        let info = message_info(&addr("attester1"), &[]);

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ReleaseBond { attestation_id: 1 },
        )
        .unwrap_err();

        // Attestation is in Challenged state, not Active
        assert_eq!(err, ContractError::AttestationNotReleasable {});
    }

    // -----------------------------------------------------------------------
    // Full lifecycle test (SPEC acceptance test #1)
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_lifecycle_happy_path() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        let attester = addr("attester1");

        // 1. Submit BaselineMeasurement with 1000 uregen bond
        let info = message_info(&attester, &coin(1000));
        execute(
            deps.as_mut(),
            env_at(base_time),
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "BaselineMeasurement".to_string(),
                attestation_iri: "koi://att/baseline-lifecycle".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        // Status: Bonded
        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Bonded);

        // 2. 48h activation delay passes — activate
        execute(
            deps.as_mut(),
            env_at(base_time + 49 * 3600),
            message_info(&addr("crank"), &[]),
            ExecuteMsg::ActivateAttestation { attestation_id: 1 },
        )
        .unwrap();

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Active);

        // 3. No challenge filed within 120-day window (time passes)

        // 4. Lock period (180 days) expires — release bond
        execute(
            deps.as_mut(),
            env_at(base_time + 181 * 24 * 3600),
            message_info(&attester, &[]),
            ExecuteMsg::ReleaseBond { attestation_id: 1 },
        )
        .unwrap();

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Released);
    }

    // -----------------------------------------------------------------------
    // Admin tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_update_config() {
        let (mut deps, env) = setup_contract();
        let admin = addr("admin");
        let new_arbiter = addr("new_arbiter");
        let info = message_info(&admin, &[]);

        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::UpdateConfig {
                arbiter_dao: Some(new_arbiter.to_string()),
                min_challenge_deposit_ratio: Some(Uint128::from(2000u128)), // 20%
                arbiter_fee_ratio: None,
            },
        )
        .unwrap();

        let config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_eq!(config.arbiter_dao, new_arbiter);
        assert_eq!(config.min_challenge_deposit_ratio, Uint128::from(2000u128));
        assert_eq!(config.arbiter_fee_ratio, Uint128::from(500u128)); // unchanged
    }

    #[test]
    fn test_update_config_unauthorized() {
        let (mut deps, env) = setup_contract();
        let info = message_info(&addr("random_user"), &[]);

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::UpdateConfig {
                arbiter_dao: None,
                min_challenge_deposit_ratio: None,
                arbiter_fee_ratio: None,
            },
        )
        .unwrap_err();

        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[test]
    fn test_add_attestation_type() {
        let (mut deps, env) = setup_contract();
        let info = message_info(&addr("admin"), &[]);

        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::AddAttestationType {
                attestation_type: AttestationTypeInput {
                    name: "CustomType".to_string(),
                    min_bond: Uint128::from(3000u128),
                    lock_period_days: 200,
                    challenge_window_days: 150,
                },
            },
        )
        .unwrap();

        let at = ATTESTATION_TYPES
            .load(deps.as_ref().storage, "CustomType")
            .unwrap();
        assert_eq!(at.min_bond, Uint128::from(3000u128));
    }

    #[test]
    fn test_add_attestation_type_duplicate() {
        let (mut deps, env) = setup_contract();
        let info = message_info(&addr("admin"), &[]);

        let err = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::AddAttestationType {
                attestation_type: AttestationTypeInput {
                    name: "ProjectBoundary".to_string(), // already exists
                    min_bond: Uint128::from(999u128),
                    lock_period_days: 10,
                    challenge_window_days: 5,
                },
            },
        )
        .unwrap_err();

        assert_eq!(
            err,
            ContractError::AttestationTypeAlreadyExists {
                name: "ProjectBoundary".to_string()
            }
        );
    }

    // -----------------------------------------------------------------------
    // Query tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_config() {
        let (deps, env) = setup_contract();
        let bin = query(deps.as_ref(), env, QueryMsg::Config {}).unwrap();
        let res: ConfigResponse = cosmwasm_std::from_json(bin).unwrap();
        assert_eq!(res.config.admin, addr("admin"));
    }

    #[test]
    fn test_query_attestation() {
        let (mut deps, env) = setup_contract();
        let attester = addr("attester1");
        let info = message_info(&attester, &coin(500));
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "ProjectBoundary".to_string(),
                attestation_iri: "koi://att/query-test".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        let bin = query(deps.as_ref(), env, QueryMsg::Attestation { id: 1 }).unwrap();
        let res: AttestationResponse = cosmwasm_std::from_json(bin).unwrap();
        assert_eq!(res.attestation.attestation_iri, "koi://att/query-test");
        assert_eq!(res.attestation.attester, attester);
    }

    #[test]
    fn test_query_attestations_by_attester() {
        let (mut deps, env) = setup_contract();
        let attester1 = addr("attester1");
        let attester2 = addr("attester2");

        // Create 2 attestations from attester1
        for i in 0..2 {
            let info = message_info(&attester1, &coin(500));
            execute(
                deps.as_mut(),
                env.clone(),
                info,
                ExecuteMsg::CreateAttestation {
                    attestation_type: "ProjectBoundary".to_string(),
                    attestation_iri: format!("koi://att/by-attester-{}", i),
                    beneficiary: None,
                },
            )
            .unwrap();
        }

        // Create 1 from attester2
        let info = message_info(&attester2, &coin(500));
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "ProjectBoundary".to_string(),
                attestation_iri: "koi://att/by-attester2".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        let bin = query(
            deps.as_ref(),
            env,
            QueryMsg::AttestationsByAttester {
                attester: attester1.to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        let res: AttestationsResponse = cosmwasm_std::from_json(bin).unwrap();
        assert_eq!(res.attestations.len(), 2);
    }

    #[test]
    fn test_query_challenge() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        let bin = query(deps.as_ref(), env, QueryMsg::Challenge { id: 1 }).unwrap();
        let res: ChallengeResponse = cosmwasm_std::from_json(bin).unwrap();
        assert_eq!(res.challenge.challenger, addr("challenger1"));
        assert_eq!(res.challenge.attestation_id, 1);
    }

    #[test]
    fn test_query_attestation_type() {
        let (deps, env) = setup_contract();
        let bin = query(
            deps.as_ref(),
            env,
            QueryMsg::AttestationType {
                name: "CreditIssuanceClaim".to_string(),
            },
        )
        .unwrap();
        let res: AttestationTypeResponse = cosmwasm_std::from_json(bin).unwrap();
        assert_eq!(res.attestation_type.min_bond, Uint128::from(2000u128));
        assert_eq!(res.attestation_type.lock_period_days, 365);
    }

    #[test]
    fn test_query_all_attestation_types() {
        let (deps, env) = setup_contract();
        let bin = query(deps.as_ref(), env, QueryMsg::AllAttestationTypes {}).unwrap();
        let res: AllAttestationTypesResponse = cosmwasm_std::from_json(bin).unwrap();
        assert_eq!(res.attestation_types.len(), 4);
    }

    #[test]
    fn test_query_bond_info() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();

        let info = message_info(&addr("attester1"), &coin(1000));
        execute(
            deps.as_mut(),
            env_at(base_time),
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "BaselineMeasurement".to_string(),
                attestation_iri: "koi://att/bond-info-test".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        // Query while bond is active
        let bin = query(
            deps.as_ref(),
            env_at(base_time + 100),
            QueryMsg::BondInfo { attestation_id: 1 },
        )
        .unwrap();
        let res: BondInfoResponse = cosmwasm_std::from_json(bin).unwrap();
        assert_eq!(res.bond_amount, Uint128::from(1000u128));
        assert_eq!(res.bond_denom, DENOM);
        assert!(res.is_locked);
        assert!(res.is_challengeable);
    }

    #[test]
    fn test_query_bond_info_after_lock_expires() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_and_activate(&mut deps, base_time);

        // After lock period
        let bin = query(
            deps.as_ref(),
            env_at(base_time + 181 * 24 * 3600),
            QueryMsg::BondInfo { attestation_id: 1 },
        )
        .unwrap();
        let res: BondInfoResponse = cosmwasm_std::from_json(bin).unwrap();
        assert!(!res.is_locked);
        assert!(!res.is_challengeable); // past challenge window too
    }

    // -----------------------------------------------------------------------
    // Bond conservation invariant test
    // -----------------------------------------------------------------------

    #[test]
    fn test_bond_conservation_on_slash() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        // Bond: 1000, Deposit: 100, Fee: 50 (5% of 1000)
        // Slash:
        //   challenger = 500 (50% bond) + 100 (deposit) - 50 (fee) = 550
        //   community  = 500 (50% bond)
        //   arbiter    = 50 (fee)
        //   total out  = 550 + 500 + 50 = 1100
        //   total in   = 1000 (bond) + 100 (deposit) = 1100

        let env = env_at(base_time + 60 * 3600);
        let info = message_info(&addr("arbiter_dao"), &[]);
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::ChallengerWins,
            },
        )
        .unwrap();

        // Verify all outflows sum to total inflows
        let mut total_out = Uint128::zero();
        for sub in &res.messages {
            match &sub.msg {
                CosmosMsg::Bank(BankMsg::Send { amount, .. }) => {
                    for c in amount {
                        total_out = total_out.checked_add(c.amount).unwrap();
                    }
                }
                _ => {}
            }
        }

        let total_in = Uint128::from(1000u128) + Uint128::from(100u128); // bond + deposit
        assert_eq!(total_out, total_in);
    }

    #[test]
    fn test_bond_conservation_on_attester_wins() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        // Bond: 1000, Deposit: 100, Fee: 50
        // Attester wins:
        //   attester = 1000 + 100 - 50 = 1050
        //   arbiter  = 50
        //   total    = 1100 = bond + deposit

        let env = env_at(base_time + 60 * 3600);
        let info = message_info(&addr("arbiter_dao"), &[]);
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::AttesterWins,
            },
        )
        .unwrap();

        let mut total_out = Uint128::zero();
        for sub in &res.messages {
            match &sub.msg {
                CosmosMsg::Bank(BankMsg::Send { amount, .. }) => {
                    for c in amount {
                        total_out = total_out.checked_add(c.amount).unwrap();
                    }
                }
                _ => {}
            }
        }

        let total_in = Uint128::from(1100u128);
        assert_eq!(total_out, total_in);
    }

    // -----------------------------------------------------------------------
    // Multiple attestation IDs
    // -----------------------------------------------------------------------

    #[test]
    fn test_attestation_id_auto_increments() {
        let (mut deps, env) = setup_contract();

        for i in 0..3 {
            let info = message_info(&addr("attester1"), &coin(500));
            execute(
                deps.as_mut(),
                env.clone(),
                info,
                ExecuteMsg::CreateAttestation {
                    attestation_type: "ProjectBoundary".to_string(),
                    attestation_iri: format!("koi://att/auto-{}", i),
                    beneficiary: None,
                },
            )
            .unwrap();
        }

        assert!(ATTESTATIONS.load(deps.as_ref().storage, 1).is_ok());
        assert!(ATTESTATIONS.load(deps.as_ref().storage, 2).is_ok());
        assert!(ATTESTATIONS.load(deps.as_ref().storage, 3).is_ok());
        assert!(ATTESTATIONS.load(deps.as_ref().storage, 4).is_err());
    }

    // -----------------------------------------------------------------------
    // MethodologyValidation with high bond
    // -----------------------------------------------------------------------

    #[test]
    fn test_methodology_validation_full_cycle() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();

        // Bond 10000 uregen for MethodologyValidation (min 5000)
        let info = message_info(&addr("validator1"), &coin(10_000));
        execute(
            deps.as_mut(),
            env_at(base_time),
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "MethodologyValidation".to_string(),
                attestation_iri: "koi://att/methodology-v4".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        // Activate after 48h
        execute(
            deps.as_mut(),
            env_at(base_time + 49 * 3600),
            message_info(&addr("crank"), &[]),
            ExecuteMsg::ActivateAttestation { attestation_id: 1 },
        )
        .unwrap();

        // Challenge with 10% deposit = 1000
        let env = env_at(base_time + 100 * 24 * 3600);
        let info = message_info(&addr("challenger1"), &coin(1000));
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/methodology-flaw".to_string(),
            },
        )
        .unwrap();

        // Resolve: challenger wins
        let env = env_at(base_time + 200 * 24 * 3600);
        let info = message_info(&addr("arbiter_dao"), &[]);
        let res = execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::ChallengerWins,
            },
        )
        .unwrap();

        // Verify distributions:
        // arbiter_fee = 10000 * 500 / 10000 = 500
        // challenger = 5000 (50%) + 1000 (deposit) - 500 (fee) = 5500
        // community = 5000 (50%)
        // arbiter = 500
        let mut total_out = Uint128::zero();
        for sub in &res.messages {
            if let CosmosMsg::Bank(BankMsg::Send { amount, .. }) = &sub.msg {
                for c in amount {
                    total_out = total_out.checked_add(c.amount).unwrap();
                }
            }
        }
        assert_eq!(total_out, Uint128::from(11000u128)); // 10000 bond + 1000 deposit

        let att = ATTESTATIONS.load(deps.as_ref().storage, 1).unwrap();
        assert_eq!(att.status, AttestationStatus::Slashed);
    }

    // -----------------------------------------------------------------------
    // Query challenges by attestation
    // -----------------------------------------------------------------------

    #[test]
    fn test_query_challenges_by_attestation() {
        let (mut deps, env) = setup_contract();
        let base_time = env.block.time.seconds();
        create_activate_and_challenge(&mut deps, base_time);

        let bin = query(
            deps.as_ref(),
            env,
            QueryMsg::ChallengesByAttestation { attestation_id: 1 },
        )
        .unwrap();
        let res: ChallengesResponse = cosmwasm_std::from_json(bin).unwrap();
        assert_eq!(res.challenges.len(), 1);
        assert_eq!(res.challenges[0].attestation_id, 1);
    }

    // -----------------------------------------------------------------------
    // Admin resolve (v0 compatibility)
    // -----------------------------------------------------------------------

    #[test]
    fn test_admin_can_resolve_challenge() {
        let (mut deps, _env) = setup_contract();
        let base_time = 1_000_000u64;
        let attester = addr("attester1");

        // Create attestation from non-admin
        let env = env_at(base_time);
        let info = message_info(&attester, &coin(1000));
        execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::CreateAttestation {
                attestation_type: "BaselineMeasurement".to_string(),
                attestation_iri: "koi://att/admin-resolve".to_string(),
                beneficiary: None,
            },
        )
        .unwrap();

        // Activate
        execute(
            deps.as_mut(),
            env_at(base_time + 49 * 3600),
            message_info(&addr("crank"), &[]),
            ExecuteMsg::ActivateAttestation { attestation_id: 1 },
        )
        .unwrap();

        // Challenge
        let info = message_info(&addr("challenger1"), &coin(100));
        execute(
            deps.as_mut(),
            env_at(base_time + 50 * 3600),
            info,
            ExecuteMsg::ChallengeAttestation {
                attestation_id: 1,
                evidence_iri: "koi://evidence/admin-test".to_string(),
            },
        )
        .unwrap();

        // Admin resolves (v0 admin acts as arbiter)
        let info = message_info(&addr("admin"), &[]);
        let res = execute(
            deps.as_mut(),
            env_at(base_time + 60 * 3600),
            info,
            ExecuteMsg::ResolveChallenge {
                challenge_id: 1,
                resolution: ChallengeResolutionInput::AttesterWins,
            },
        );
        assert!(res.is_ok());
    }
}
