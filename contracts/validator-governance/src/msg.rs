use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::{
    Config, Proposal, ProposalType, Validator, ValidatorCategory, ValidatorStatus, Vote,
    VoteOption,
};

/// Instantiate message — sets initial configuration
#[cw_serde]
pub struct InstantiateMsg {
    /// Admin address (typically x/gov module account)
    pub admin: String,
    /// Maximum validators in authority set (default: 21)
    pub max_validators: Option<u32>,
    /// Minimum validators required (default: 15)
    pub min_validators: Option<u32>,
    /// Minimum validators per category (default: 5)
    pub min_per_category: Option<u32>,
    /// Term length in seconds (default: 31_536_000 = 12 months)
    pub term_length_seconds: Option<u64>,
    /// Probation period in seconds (default: 2_592_000 = 30 days)
    pub probation_period_seconds: Option<u64>,
    /// Minimum uptime threshold in bps (default: 9950 = 99.50%)
    pub min_uptime_bps: Option<u16>,
    /// Performance threshold in bps (default: 7000 = 70%)
    pub performance_threshold_bps: Option<u16>,
    /// Performance bonus share in bps (default: 1000 = 10%)
    pub performance_bonus_bps: Option<u16>,
    /// Voting period in seconds (default: 604_800 = 7 days)
    pub voting_period_seconds: Option<u64>,
}

/// Execute messages
#[cw_serde]
pub enum ExecuteMsg {
    /// Apply to join the validator set (creates a Candidate)
    ApplyValidator {
        moniker: String,
        category: ValidatorCategory,
    },

    /// Admin: approve a candidate validator
    ApproveValidator { address: String },

    /// Admin: activate an approved validator (begins term)
    ActivateValidator { address: String },

    /// Admin: place a validator on probation
    PutOnProbation {
        address: String,
        reason: String,
    },

    /// Admin: restore a validator from probation to active
    RestoreValidator { address: String },

    /// Admin: remove a validator from the set
    RemoveValidator {
        address: String,
        reason: String,
    },

    /// Admin or self: mark a term-expired validator for re-application
    Reapply { address: String },

    /// Admin: update performance scores for a validator (typically called by AGENT-004)
    UpdateScores {
        address: String,
        uptime_bps: Option<u16>,
        governance_participation_bps: Option<u16>,
        ecosystem_contribution_bps: Option<u16>,
    },

    /// Active validator: create a governance proposal
    CreateProposal {
        title: String,
        description: String,
        proposal_type: ProposalType,
    },

    /// Active validator: vote on an active proposal (weighted by performance score)
    CastVote {
        proposal_id: u64,
        vote: VoteOption,
    },

    /// Anyone: execute a passed proposal after voting period ends
    ExecuteProposal { proposal_id: u64 },

    /// Admin: update contract configuration directly
    UpdateConfig {
        max_validators: Option<u32>,
        min_validators: Option<u32>,
        min_per_category: Option<u32>,
        term_length_seconds: Option<u64>,
        probation_period_seconds: Option<u64>,
        min_uptime_bps: Option<u16>,
        performance_threshold_bps: Option<u16>,
        performance_bonus_bps: Option<u16>,
        voting_period_seconds: Option<u64>,
    },
}

/// Query messages
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get contract configuration
    #[returns(ConfigResponse)]
    Config {},

    /// Get full validator set
    #[returns(ValidatorSetResponse)]
    ValidatorSet {
        /// Optional filter by status
        status: Option<ValidatorStatus>,
        /// Optional filter by category
        category: Option<ValidatorCategory>,
    },

    /// Get a single validator's info
    #[returns(ValidatorResponse)]
    Validator { address: String },

    /// Get the composite performance score for a validator (SPEC section 5)
    #[returns(PerformanceScoreResponse)]
    PerformanceScore { address: String },

    /// List all proposals
    #[returns(ProposalListResponse)]
    Proposals {
        /// Optional filter by status
        status: Option<ProposalStatusFilter>,
    },

    /// Get a single proposal with vote tally
    #[returns(ProposalResponse)]
    Proposal { id: u64 },

    /// Get votes for a proposal
    #[returns(VotesResponse)]
    Votes { proposal_id: u64 },

    /// Get validator set composition counts (per category)
    #[returns(CompositionResponse)]
    Composition {},
}

// ── Response types ───────────────────────────────────────────────────

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct ValidatorSetResponse {
    pub validators: Vec<Validator>,
    pub total: u32,
}

#[cw_serde]
pub struct ValidatorResponse {
    pub validator: Validator,
    pub composite_score: u16,
    pub confidence: u16,
    pub flags: Vec<String>,
}

#[cw_serde]
pub struct PerformanceScoreResponse {
    pub address: String,
    pub composite_score: u16,
    pub confidence: u16,
    pub factors: FactorsResponse,
    pub flags: Vec<String>,
}

#[cw_serde]
pub struct FactorsResponse {
    pub uptime_bps: Option<u16>,
    pub governance_participation_bps: Option<u16>,
    pub ecosystem_contribution_bps: Option<u16>,
}

#[cw_serde]
pub struct ProposalListResponse {
    pub proposals: Vec<Proposal>,
}

#[cw_serde]
pub struct ProposalResponse {
    pub proposal: Proposal,
    pub votes: Vec<Vote>,
}

#[cw_serde]
pub struct VotesResponse {
    pub votes: Vec<Vote>,
}

#[cw_serde]
pub struct CompositionResponse {
    pub infrastructure_builders: u32,
    pub trusted_refi_partners: u32,
    pub ecological_data_stewards: u32,
    pub total_active: u32,
}

#[cw_serde]
pub enum ProposalStatusFilter {
    Active,
    Passed,
    Rejected,
    Executed,
}
