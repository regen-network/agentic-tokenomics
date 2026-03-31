use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

/// Activity types that earn contribution rewards (from M015 SPEC section 5.1)
#[cw_serde]
pub enum ActivityType {
    /// Credit purchase — primary demand signal (weight 0.30)
    CreditPurchase,
    /// Credit retirement — terminal ecological impact (weight 0.30)
    CreditRetirement,
    /// Platform facilitation — ecosystem infrastructure (weight 0.20)
    PlatformFacilitation,
    /// Governance vote — governance participation (weight 0.10)
    GovernanceVote,
    /// Proposal submission — governance initiative (weight 0.10, conditional)
    ProposalSubmission,
}

/// Proposal outcome determines effective weight for ProposalSubmission activity
#[cw_serde]
pub enum ProposalOutcome {
    /// Passed quorum and approved — full weight (0.10)
    PassedAndApproved,
    /// Reached quorum but failed — half weight (0.05 effective)
    ReachedQuorumFailed,
    /// Failed to reach quorum — zero weight
    FailedQuorum,
}

/// Mechanism lifecycle state (from M015 SPEC section 7)
#[cw_serde]
pub enum MechanismState {
    /// Not yet activated by governance
    Inactive,
    /// Recording activity scores, no payouts (calibration period)
    Tracking,
    /// Fully active, distributing rewards each epoch
    Distributing,
}

/// Stability commitment state (from M015 SPEC section 6.2)
#[cw_serde]
pub enum CommitmentState {
    /// Active lock, accruing rewards
    Committed,
    /// Lock period complete, rewards claimable
    Matured,
    /// Exited early with penalty
    EarlyExit,
}

/// Activity weights configuration — basis points (sum must equal 10000)
#[cw_serde]
pub struct ActivityWeights {
    /// Credit purchase weight in bps (default: 3000 = 30%)
    pub credit_purchase_bps: u16,
    /// Credit retirement weight in bps (default: 3000 = 30%)
    pub credit_retirement_bps: u16,
    /// Platform facilitation weight in bps (default: 2000 = 20%)
    pub platform_facilitation_bps: u16,
    /// Governance voting weight in bps (default: 1000 = 10%)
    pub governance_voting_bps: u16,
    /// Proposal submission weight in bps (default: 1000 = 10%)
    pub proposal_submission_bps: u16,
}

impl ActivityWeights {
    pub fn sum(&self) -> u16 {
        self.credit_purchase_bps
            + self.credit_retirement_bps
            + self.platform_facilitation_bps
            + self.governance_voting_bps
            + self.proposal_submission_bps
    }

    pub fn default_weights() -> Self {
        ActivityWeights {
            credit_purchase_bps: 3000,
            credit_retirement_bps: 3000,
            platform_facilitation_bps: 2000,
            governance_voting_bps: 1000,
            proposal_submission_bps: 1000,
        }
    }

    /// Get weight for an activity type, returning bps value.
    /// For ProposalSubmission, caller must apply outcome scaling separately.
    pub fn weight_for(&self, activity: &ActivityType) -> u16 {
        match activity {
            ActivityType::CreditPurchase => self.credit_purchase_bps,
            ActivityType::CreditRetirement => self.credit_retirement_bps,
            ActivityType::PlatformFacilitation => self.platform_facilitation_bps,
            ActivityType::GovernanceVote => self.governance_voting_bps,
            ActivityType::ProposalSubmission => self.proposal_submission_bps,
        }
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    /// Admin address (typically x/gov module account)
    pub admin: String,
    /// Activity weights — if None, uses defaults from SPEC section 5.1
    pub activity_weights: Option<ActivityWeights>,
    /// Maximum share of community pool inflow for stability tier (bps, default 3000 = 30%)
    pub max_stability_share_bps: Option<u16>,
    /// Annual return for stability tier (bps, default 600 = 6%)
    pub stability_annual_return_bps: Option<u16>,
    /// Minimum stability commitment in uregen (default 100_000_000 = 100 REGEN)
    pub min_commitment_uregen: Option<Uint128>,
    /// Minimum lock period in months (default 6)
    pub min_lock_months: Option<u64>,
    /// Maximum lock period in months (default 24)
    pub max_lock_months: Option<u64>,
    /// Early exit penalty in bps (default 5000 = 50%)
    pub early_exit_penalty_bps: Option<u16>,
    /// Blocks per epoch (default ~60480 for ~1 week at 10s blocks)
    pub blocks_per_epoch: Option<u64>,
    /// Epochs required for calibration before TRACKING -> DISTRIBUTING (default 13 = ~3 months)
    pub calibration_epochs: Option<u64>,
    /// Number of epochs per year for stability calculations (default 52)
    pub epochs_per_year: Option<u64>,
    /// Token denom (default "uregen")
    pub denom: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // --- Activity tracking (called by hooks / authorized reporters) ---
    /// Record a contribution for a participant
    RecordContribution {
        participant: String,
        activity: ActivityType,
        /// Value in uregen for monetary activities, or count for governance activities
        value: Uint128,
        /// For ProposalSubmission only — determines effective weight
        proposal_outcome: Option<ProposalOutcome>,
        /// Transaction hash for dedup
        tx_hash: String,
    },

    // --- Epoch management ---
    /// Finalize the current epoch: snapshot scores, compute distributions
    FinalizeEpoch {
        /// Community Pool inflow for this epoch (from M013)
        community_pool_inflow: Uint128,
    },

    /// Claim pending rewards for the caller
    ClaimRewards {},

    // --- Stability tier ---
    /// Commit tokens to the stability tier for a fixed lock period
    CommitStability {
        /// Lock period in months (6-24)
        lock_months: u64,
    },

    /// Claim matured stability commitment (tokens + full rewards)
    ClaimMaturedCommitment {},

    /// Exit stability commitment early (50% reward penalty)
    ExitEarly {},

    // --- Governance admin ---
    /// Activate the mechanism (INACTIVE -> TRACKING)
    Activate {},

    /// Transition from TRACKING to DISTRIBUTING after calibration
    EnableDistribution {},

    /// Update activity weights (governance only)
    UpdateWeights {
        new_weights: ActivityWeights,
    },

    /// Update stability tier parameters (governance only)
    UpdateStabilityParams {
        max_stability_share_bps: Option<u16>,
        stability_annual_return_bps: Option<u16>,
        min_commitment_uregen: Option<Uint128>,
        min_lock_months: Option<u64>,
        max_lock_months: Option<u64>,
        early_exit_penalty_bps: Option<u16>,
    },

    /// Circuit breaker: pause the mechanism
    Pause {},

    /// Resume after pause
    Resume {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get contract configuration
    #[returns(ConfigResponse)]
    Config {},

    /// Get current mechanism state and epoch info
    #[returns(StateResponse)]
    State {},

    /// Get a participant's activity score for a specific epoch
    #[returns(ParticipantScoreResponse)]
    ParticipantScore {
        address: String,
        epoch: Option<u64>,
    },

    /// Get all participant scores for an epoch
    #[returns(EpochScoresResponse)]
    EpochScores {
        epoch: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Get epoch distribution history
    #[returns(DistributionHistoryResponse)]
    DistributionHistory {
        start_epoch: Option<u64>,
        limit: Option<u32>,
    },

    /// Get pending (unclaimed) rewards for an address
    #[returns(PendingRewardsResponse)]
    PendingRewards { address: String },

    /// Get stability commitment info for an address
    #[returns(StabilityCommitmentResponse)]
    StabilityCommitment { address: String },

    /// Get aggregate stability tier stats
    #[returns(StabilityStatsResponse)]
    StabilityStats {},

    /// Simulate score for a set of activities (does not store)
    #[returns(SimulateScoreResponse)]
    SimulateScore {
        activities: Vec<SimulateActivity>,
    },
}

// --- Query response types ---

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub activity_weights: ActivityWeights,
    pub max_stability_share_bps: u16,
    pub stability_annual_return_bps: u16,
    pub min_commitment_uregen: Uint128,
    pub min_lock_months: u64,
    pub max_lock_months: u64,
    pub early_exit_penalty_bps: u16,
    pub blocks_per_epoch: u64,
    pub calibration_epochs: u64,
    pub epochs_per_year: u64,
    pub denom: String,
}

#[cw_serde]
pub struct StateResponse {
    pub mechanism_state: MechanismState,
    pub current_epoch: u64,
    pub epoch_start_block: u64,
    pub activation_epoch: Option<u64>,
    pub paused: bool,
}

#[cw_serde]
pub struct ParticipantScoreResponse {
    pub address: String,
    pub epoch: u64,
    pub credit_purchase_value: Uint128,
    pub credit_retirement_value: Uint128,
    pub platform_facilitation_value: Uint128,
    pub governance_votes: Uint128,
    pub proposal_credits: Uint128,
    pub weighted_score: Uint128,
}

#[cw_serde]
pub struct EpochScoresResponse {
    pub epoch: u64,
    pub scores: Vec<ParticipantScoreResponse>,
}

#[cw_serde]
pub struct EpochDistribution {
    pub epoch: u64,
    pub community_pool_inflow: Uint128,
    pub stability_allocation: Uint128,
    pub activity_pool: Uint128,
    pub total_score: Uint128,
    pub participant_count: u32,
}

#[cw_serde]
pub struct DistributionHistoryResponse {
    pub distributions: Vec<EpochDistribution>,
}

#[cw_serde]
pub struct PendingRewardsResponse {
    pub address: String,
    pub pending_activity_rewards: Uint128,
    pub pending_stability_rewards: Uint128,
    pub total_pending: Uint128,
}

#[cw_serde]
pub struct StabilityCommitmentResponse {
    pub address: String,
    pub amount: Uint128,
    pub lock_months: u64,
    pub committed_at_block: u64,
    pub maturity_block: u64,
    pub state: CommitmentState,
    pub accrued_rewards: Uint128,
}

#[cw_serde]
pub struct StabilityStatsResponse {
    pub total_committed: Uint128,
    pub active_commitments: u32,
    pub total_stability_allocated: Uint128,
}

#[cw_serde]
pub struct SimulateActivity {
    pub activity: ActivityType,
    pub value: Uint128,
    pub proposal_outcome: Option<ProposalOutcome>,
}

#[cw_serde]
pub struct SimulateScoreResponse {
    pub weighted_score: Uint128,
    pub breakdown: Vec<(String, Uint128)>,
}
