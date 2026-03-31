use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

/// Contract configuration — governance-controlled parameters
#[cw_serde]
pub struct Config {
    /// Admin address (typically x/gov module account)
    pub admin: Addr,
    /// Maximum validators in the authority set (default: 21)
    pub max_validators: u32,
    /// Minimum validators required (default: 15)
    pub min_validators: u32,
    /// Minimum validators per category (default: 5)
    pub min_per_category: u32,
    /// Term length in seconds (default: 12 months = 31_536_000)
    pub term_length_seconds: u64,
    /// Probation period in seconds (default: 30 days = 2_592_000)
    pub probation_period_seconds: u64,
    /// Minimum uptime threshold (basis points, e.g. 9950 = 99.50%)
    pub min_uptime_bps: u16,
    /// Performance threshold below which a validator is flagged (bps, e.g. 7000 = 70%)
    pub performance_threshold_bps: u16,
    /// Performance bonus share of validator fund (bps, e.g. 1000 = 10%)
    pub performance_bonus_bps: u16,
    /// Voting period for governance proposals in seconds
    pub voting_period_seconds: u64,
}

/// Validator lifecycle status — see SPEC.md section 6.1
#[cw_serde]
pub enum ValidatorStatus {
    Candidate,
    Approved,
    Active,
    Probation,
    Removed,
    TermExpired,
}

impl std::fmt::Display for ValidatorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidatorStatus::Candidate => write!(f, "candidate"),
            ValidatorStatus::Approved => write!(f, "approved"),
            ValidatorStatus::Active => write!(f, "active"),
            ValidatorStatus::Probation => write!(f, "probation"),
            ValidatorStatus::Removed => write!(f, "removed"),
            ValidatorStatus::TermExpired => write!(f, "term_expired"),
        }
    }
}

/// Validator category — composition requirements per SPEC.md section 6.2
#[cw_serde]
pub enum ValidatorCategory {
    InfrastructureBuilders,
    TrustedRefiPartners,
    EcologicalDataStewards,
}

impl std::fmt::Display for ValidatorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ValidatorCategory::InfrastructureBuilders => write!(f, "infrastructure_builders"),
            ValidatorCategory::TrustedRefiPartners => write!(f, "trusted_refi_partners"),
            ValidatorCategory::EcologicalDataStewards => write!(f, "ecological_data_stewards"),
        }
    }
}

/// Performance scores stored as basis points (0-10000) for integer arithmetic
#[cw_serde]
pub struct PerformanceScores {
    /// blocks_signed / blocks_expected (bps, None if unavailable)
    pub uptime_bps: Option<u16>,
    /// votes_cast / proposals_available (bps, None if unavailable)
    pub governance_participation_bps: Option<u16>,
    /// AGENT-004 assessed score (bps, None if unavailable)
    pub ecosystem_contribution_bps: Option<u16>,
}

/// Authority validator record
#[cw_serde]
pub struct Validator {
    pub address: Addr,
    pub moniker: String,
    pub category: ValidatorCategory,
    pub status: ValidatorStatus,
    /// Timestamp (seconds) when current term started
    pub term_start: u64,
    /// Timestamp (seconds) when current term ends
    pub term_end: u64,
    /// Latest performance scores
    pub performance: PerformanceScores,
    /// Timestamp when probation began (0 if not on probation)
    pub probation_start: u64,
    /// Reason for removal (empty if not removed)
    pub removal_reason: String,
}

/// Governance proposal
#[cw_serde]
pub struct Proposal {
    pub id: u64,
    pub proposer: Addr,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    /// Timestamp when voting started
    pub start_time: u64,
    /// Timestamp when voting ends
    pub end_time: u64,
    /// Total weighted yes votes (sum of voter scores)
    pub yes_votes: u64,
    /// Total weighted no votes
    pub no_votes: u64,
    /// Total weighted abstain votes
    pub abstain_votes: u64,
}

/// Types of governance proposals
#[cw_serde]
pub enum ProposalType {
    /// Add a new validator to the authority set
    AddValidator {
        address: String,
        moniker: String,
        category: ValidatorCategory,
    },
    /// Remove a validator from the authority set
    RemoveValidator {
        address: String,
        reason: String,
    },
    /// Update contract configuration parameters
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

/// Proposal lifecycle status
#[cw_serde]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

/// Vote option
#[cw_serde]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
}

/// Individual vote record
#[cw_serde]
pub struct Vote {
    pub voter: Addr,
    pub option: VoteOption,
    /// Weight derived from voter's composite performance score
    pub weight: u64,
}

// ── Storage items ────────────────────────────────────────────────────

/// Contract configuration
pub const CONFIG: Item<Config> = Item::new("config");

/// Validators indexed by address
pub const VALIDATORS: Map<&Addr, Validator> = Map::new("validators");

/// Proposals indexed by ID
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");

/// Next proposal ID counter
pub const NEXT_PROPOSAL_ID: Item<u64> = Item::new("next_proposal_id");

/// Votes indexed by (proposal_id, voter_address)
pub const VOTES: Map<(u64, &Addr), Vote> = Map::new("votes");
