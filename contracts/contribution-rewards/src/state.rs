use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

use crate::msg::{ActivityWeights, CommitmentState, MechanismState};

/// Contract configuration — governance-controlled parameters
#[cw_serde]
pub struct Config {
    /// Admin address (x/gov module account)
    pub admin: Addr,
    /// Activity type weights (bps, must sum to 10000)
    pub activity_weights: ActivityWeights,
    /// Max share of community pool inflow for stability tier (bps)
    pub max_stability_share_bps: u16,
    /// Annual return for stability tier (bps, e.g. 600 = 6%)
    pub stability_annual_return_bps: u16,
    /// Minimum stability commitment (uregen)
    pub min_commitment_uregen: Uint128,
    /// Minimum lock period in months
    pub min_lock_months: u64,
    /// Maximum lock period in months
    pub max_lock_months: u64,
    /// Early exit penalty (bps of accrued rewards forfeited)
    pub early_exit_penalty_bps: u16,
    /// Blocks per epoch (~60480 for weekly at 10s blocks)
    pub blocks_per_epoch: u64,
    /// Epochs required in TRACKING before enabling distribution
    pub calibration_epochs: u64,
    /// Epochs per year for stability return calculation
    pub epochs_per_year: u64,
    /// Token denomination
    pub denom: String,
}

/// Contract operational state
#[cw_serde]
pub struct ContractState {
    /// Mechanism lifecycle state
    pub mechanism_state: MechanismState,
    /// Current epoch number (starts at 1 on activation)
    pub current_epoch: u64,
    /// Block height at which current epoch started
    pub epoch_start_block: u64,
    /// Epoch at which mechanism was activated (for calibration tracking)
    pub activation_epoch: Option<u64>,
    /// Circuit breaker
    pub paused: bool,
}

/// Per-participant, per-epoch activity record
#[cw_serde]
pub struct ActivityRecord {
    /// Cumulative credit purchase value (uregen)
    pub credit_purchase_value: Uint128,
    /// Cumulative credit retirement value (uregen)
    pub credit_retirement_value: Uint128,
    /// Cumulative platform facilitation value (uregen)
    pub platform_facilitation_value: Uint128,
    /// Count of governance votes cast
    pub governance_votes: Uint128,
    /// Weighted proposal credits (1.0 for passed+quorum, 0.5 for failed+quorum, 0 for no quorum)
    /// Stored as fixed-point with 2 decimal precision: 100 = 1.0, 50 = 0.5
    pub proposal_credits_x100: Uint128,
}

impl ActivityRecord {
    pub fn new() -> Self {
        ActivityRecord {
            credit_purchase_value: Uint128::zero(),
            credit_retirement_value: Uint128::zero(),
            platform_facilitation_value: Uint128::zero(),
            governance_votes: Uint128::zero(),
            proposal_credits_x100: Uint128::zero(),
        }
    }
}

impl Default for ActivityRecord {
    fn default() -> Self {
        Self::new()
    }
}

/// Epoch distribution record — stored after finalization
#[cw_serde]
pub struct DistributionRecord {
    pub community_pool_inflow: Uint128,
    pub stability_allocation: Uint128,
    pub activity_pool: Uint128,
    pub total_score: Uint128,
    pub participant_count: u32,
}

/// Stability commitment for a holder
#[cw_serde]
pub struct StabilityCommitment {
    /// Committed amount (uregen)
    pub amount: Uint128,
    /// Lock period in months
    pub lock_months: u64,
    /// Block at which commitment was made
    pub committed_at_block: u64,
    /// Block at which commitment matures
    pub maturity_block: u64,
    /// Current state
    pub state: CommitmentState,
    /// Accrued rewards (uregen)
    pub accrued_rewards: Uint128,
}

/// Aggregate stability tier statistics
#[cw_serde]
pub struct StabilityStats {
    /// Total uregen committed across all active commitments
    pub total_committed: Uint128,
    /// Number of active commitments
    pub active_commitments: u32,
    /// Total stability rewards allocated across all epochs
    pub total_stability_allocated: Uint128,
}

// ---- Storage keys ----

/// Contract configuration
pub const CONFIG: Item<Config> = Item::new("config");

/// Contract operational state
pub const STATE: Item<ContractState> = Item::new("state");

/// Activity records: (epoch, participant_address) -> ActivityRecord
pub const ACTIVITY_RECORDS: Map<(u64, &str), ActivityRecord> = Map::new("activity");

/// Set of participants who have activity in a given epoch: (epoch, participant_address) -> bool
pub const EPOCH_PARTICIPANTS: Map<(u64, &str), bool> = Map::new("epoch_parts");

/// Distribution records: epoch -> DistributionRecord
pub const DISTRIBUTIONS: Map<u64, DistributionRecord> = Map::new("distributions");

/// Per-participant pending rewards (activity-based): address -> Uint128
pub const PENDING_ACTIVITY_REWARDS: Map<&str, Uint128> = Map::new("pending_act");

/// Per-participant pending rewards (stability-based): address -> Uint128
pub const PENDING_STABILITY_REWARDS: Map<&str, Uint128> = Map::new("pending_stab");

/// Stability commitments: address -> StabilityCommitment
pub const STABILITY_COMMITMENTS: Map<&str, StabilityCommitment> = Map::new("stab_commit");

/// Aggregate stability stats
pub const STABILITY_STATS: Item<StabilityStats> = Item::new("stab_stats");

/// Dedup set for recorded contributions: tx_hash -> bool
pub const RECORDED_TX_HASHES: Map<&str, bool> = Map::new("tx_hashes");
