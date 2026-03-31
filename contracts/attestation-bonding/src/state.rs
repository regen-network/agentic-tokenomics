use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Basis points divisor for ratio calculations (10_000 = 100%).
pub const BASIS_POINTS_DIVISOR: u128 = 10_000;

/// Activation delay in seconds (48 hours).
pub const ACTIVATION_DELAY_SECS: u64 = 48 * 60 * 60;

// ---------------------------------------------------------------------------
// Storage items
// ---------------------------------------------------------------------------

pub const CONFIG: Item<Config> = Item::new("config");
pub const ATTESTATION_TYPES: Map<&str, AttestationType> = Map::new("attestation_types");
pub const ATTESTATIONS: Map<u64, Attestation> = Map::new("attestations");
pub const CHALLENGES: Map<u64, Challenge> = Map::new("challenges");
pub const ATTESTATION_CHALLENGE: Map<u64, u64> = Map::new("attestation_challenge");
pub const NEXT_ATTESTATION_ID: Item<u64> = Item::new("next_attestation_id");
pub const NEXT_CHALLENGE_ID: Item<u64> = Item::new("next_challenge_id");

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Admin address (config authority in v0/v1).
    pub admin: Addr,
    /// Arbiter DAO address — the only address allowed to resolve challenges.
    pub arbiter_dao: Addr,
    /// Minimum challenge deposit as basis points of the bond (e.g. 1000 = 10%).
    pub min_challenge_deposit_ratio: Uint128,
    /// Arbiter fee as basis points of the bond (e.g. 500 = 5%).
    pub arbiter_fee_ratio: Uint128,
    /// Required bond denom.
    pub bond_denom: String,
}

// ---------------------------------------------------------------------------
// Attestation type configuration
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AttestationType {
    pub name: String,
    /// Minimum bond in micro-denom units.
    pub min_bond: Uint128,
    /// Lock period in days — bond cannot be released before this expires.
    pub lock_period_days: u64,
    /// Challenge window in days — challenges accepted within this window.
    pub challenge_window_days: u64,
}

// ---------------------------------------------------------------------------
// Attestation
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum AttestationStatus {
    /// Submitted and bond received, in 48h activation delay.
    Bonded,
    /// Activation delay passed, no challenge — fully active.
    Active,
    /// Under active challenge.
    Challenged,
    /// Challenge resolved in attester's favour.
    ResolvedValid,
    /// Challenge resolved against attester — bond slashed.
    Slashed,
    /// Lock period expired, bond released to attester.
    Released,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Attestation {
    pub id: u64,
    pub attester: Addr,
    pub attestation_type: String,
    pub attestation_iri: String,
    pub beneficiary: Option<Addr>,
    pub bond: Coin,
    pub status: AttestationStatus,
    pub bonded_at: Timestamp,
    /// Set when activation delay passes and status moves to Active.
    pub activated_at: Option<Timestamp>,
    /// Block time after which the bond can be released.
    pub lock_expires_at: Timestamp,
    /// Block time after which no new challenges are accepted.
    pub challenge_window_closes_at: Timestamp,
    /// Earliest time the attestation can transition from Bonded to Active.
    pub activation_eligible_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Challenge
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ChallengeStatus {
    /// Challenge submitted, awaiting resolution.
    Pending,
    /// Resolved by arbiter DAO / admin.
    Resolved,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ChallengeResolution {
    AttesterWins,
    ChallengerWins,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Challenge {
    pub id: u64,
    pub attestation_id: u64,
    pub challenger: Addr,
    pub evidence_iri: String,
    pub deposit: Coin,
    pub status: ChallengeStatus,
    pub created_at: Timestamp,
    pub resolved_at: Option<Timestamp>,
    pub resolution: Option<ChallengeResolution>,
}
