use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{
    Attestation, AttestationType, Challenge, ChallengeResolution, Config,
};

// ---------------------------------------------------------------------------
// Instantiate
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Arbiter DAO address (bech32).
    pub arbiter_dao: String,
    /// Minimum challenge deposit ratio in basis points (e.g. 1000 = 10%).
    pub min_challenge_deposit_ratio: Uint128,
    /// Arbiter fee ratio in basis points (e.g. 500 = 5%).
    pub arbiter_fee_ratio: Uint128,
    /// Bond denom (e.g. "uregen").
    pub bond_denom: String,
    /// Initial attestation type configurations.
    pub attestation_types: Vec<AttestationTypeInput>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AttestationTypeInput {
    pub name: String,
    pub min_bond: Uint128,
    pub lock_period_days: u64,
    pub challenge_window_days: u64,
}

// ---------------------------------------------------------------------------
// Execute
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Submit a new bonded attestation. Must attach funds >= min_bond for the type.
    CreateAttestation {
        attestation_type: String,
        attestation_iri: String,
        beneficiary: Option<String>,
    },

    /// Activate a bonded attestation after the 48h activation delay.
    /// Can be called by anyone (permissionless crank).
    ActivateAttestation {
        attestation_id: u64,
    },

    /// Challenge an active or bonded attestation. Must attach deposit >= bond * challenge_deposit_ratio.
    ChallengeAttestation {
        attestation_id: u64,
        evidence_iri: String,
    },

    /// Resolve a challenge. Only callable by the arbiter DAO address.
    ResolveChallenge {
        challenge_id: u64,
        resolution: ChallengeResolutionInput,
    },

    /// Release bond after lock period expires. Only callable by the attester.
    ReleaseBond {
        attestation_id: u64,
    },

    /// Update contract configuration. Only callable by admin.
    UpdateConfig {
        arbiter_dao: Option<String>,
        min_challenge_deposit_ratio: Option<Uint128>,
        arbiter_fee_ratio: Option<Uint128>,
    },

    /// Add a new attestation type. Only callable by admin.
    AddAttestationType {
        attestation_type: AttestationTypeInput,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeResolutionInput {
    AttesterWins,
    ChallengerWins,
}

// ---------------------------------------------------------------------------
// Query
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the contract configuration.
    Config {},
    /// Returns a single attestation by ID.
    Attestation { id: u64 },
    /// Returns attestations submitted by a given attester address (paginated).
    AttestationsByAttester {
        attester: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    /// Returns a single challenge by ID.
    Challenge { id: u64 },
    /// Returns challenges filed against a given attestation.
    ChallengesByAttestation { attestation_id: u64 },
    /// Returns a single attestation type definition.
    AttestationType { name: String },
    /// Returns all registered attestation types.
    AllAttestationTypes {},
    /// Returns bond info for an attestation (bond amount, status, lock expiry).
    BondInfo { attestation_id: u64 },
}

// ---------------------------------------------------------------------------
// Query responses
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AttestationResponse {
    pub attestation: Attestation,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AttestationsResponse {
    pub attestations: Vec<Attestation>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ChallengeResponse {
    pub challenge: Challenge,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ChallengesResponse {
    pub challenges: Vec<Challenge>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AttestationTypeResponse {
    pub attestation_type: AttestationType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllAttestationTypesResponse {
    pub attestation_types: Vec<AttestationType>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BondInfoResponse {
    pub attestation_id: u64,
    pub bond_amount: Uint128,
    pub bond_denom: String,
    pub status: crate::state::AttestationStatus,
    pub lock_expires_at: u64,
    pub challenge_window_closes_at: u64,
    pub is_locked: bool,
    pub is_challengeable: bool,
}

impl From<ChallengeResolutionInput> for ChallengeResolution {
    fn from(input: ChallengeResolutionInput) -> Self {
        match input {
            ChallengeResolutionInput::AttesterWins => ChallengeResolution::AttesterWins,
            ChallengeResolutionInput::ChallengerWins => ChallengeResolution::ChallengerWins,
        }
    }
}
