use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Valid credit types on Regen Network.
pub const VALID_CREDIT_TYPES: &[&str] = &["C", "KSH", "BT", "MBS", "USS"];

// ── Storage keys ───────────────────────────────────────────────────────

/// Contract configuration (set at instantiation, updatable by admin).
pub const CONFIG: Item<Config> = Item::new("config");

/// Auto-incrementing proposal counter.
pub const PROPOSAL_COUNT: Item<u64> = Item::new("proposal_count");

/// Proposals keyed by proposal ID.
pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");

/// Votes keyed by (proposal_id, voter_address).
pub const VOTES: Map<(u64, &Addr), Vote> = Map::new("votes");

/// Approved credit classes keyed by class_id string.
pub const APPROVED_CLASSES: Map<&str, ApprovedClass> = Map::new("approved_classes");

// ── Structs ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Contract admin — can submit agent scores and override rejections.
    pub admin: Addr,
    /// Quorum: minimum total vote weight required (0-1000, representing 0.0%-100.0%).
    /// e.g. 100 = 10.0% of possible weight must participate.
    pub quorum_threshold: u64,
    /// Pass threshold: minimum yes-weight ratio to approve (0-1000).
    /// e.g. 500 = 50.0% of participating weight must vote yes.
    pub pass_threshold: u64,
    /// Veto threshold: if veto weight exceeds this ratio, proposal is rejected (0-1000).
    /// e.g. 334 = 33.4%.
    pub veto_threshold: u64,
    /// Voting period in seconds.
    pub voting_period_seconds: u64,
    /// Human override window in seconds (for agent auto-reject).
    pub override_window_seconds: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Addr,
    pub credit_type: String,
    pub methodology_iri: String,
    pub admin_address: String,
    pub status: ProposalStatus,
    /// Block time (seconds) when proposal was submitted.
    pub submit_time: u64,
    /// Agent pre-screening score, if submitted.
    pub agent_score: Option<AgentScore>,
    /// Block time when voting started (if advanced to VOTING).
    pub voting_start_time: Option<u64>,
    /// Whether this proposal was advanced via human override.
    pub human_override: bool,
    /// Accumulated vote tallies.
    pub tally: Tally,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ProposalStatus {
    /// Initial submission, awaiting agent review.
    AgentReview,
    /// Active voting period.
    Voting,
    /// Approved by governance.
    Approved,
    /// Rejected by governance or agent.
    Rejected,
    /// Voting period ended without quorum.
    Expired,
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::AgentReview => write!(f, "AGENT_REVIEW"),
            ProposalStatus::Voting => write!(f, "VOTING"),
            ProposalStatus::Approved => write!(f, "APPROVED"),
            ProposalStatus::Rejected => write!(f, "REJECTED"),
            ProposalStatus::Expired => write!(f, "EXPIRED"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AgentScore {
    /// Composite score 0-1000.
    pub score: u64,
    /// Confidence 0-1000.
    pub confidence: u64,
    /// Agent recommendation.
    pub recommendation: Recommendation,
    /// Individual factor scores.
    pub factors: ScoringFactors,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Recommendation {
    Approve,
    Conditional,
    Reject,
}

impl std::fmt::Display for Recommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Recommendation::Approve => write!(f, "APPROVE"),
            Recommendation::Conditional => write!(f, "CONDITIONAL"),
            Recommendation::Reject => write!(f, "REJECT"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ScoringFactors {
    /// Methodology quality (0-1000), weight 0.4.
    pub methodology_quality: u64,
    /// Admin/proposer reputation from M010 (0-1000), weight 0.3.
    pub admin_reputation: u64,
    /// Novelty vs existing classes (0-1000), weight 0.2.
    pub novelty: u64,
    /// Application completeness (0-1000), weight 0.1.
    pub completeness: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct Tally {
    /// Sum of yes-vote weights.
    pub yes_weight: u64,
    /// Sum of no-vote weights.
    pub no_weight: u64,
    /// Sum of veto-vote weights.
    pub veto_weight: u64,
    /// Sum of abstain-vote weights.
    pub abstain_weight: u64,
}

impl Tally {
    /// Total participating weight (excluding abstain for ratio calculations,
    /// but including abstain for quorum).
    pub fn total_participating(&self) -> u64 {
        self.yes_weight + self.no_weight + self.veto_weight + self.abstain_weight
    }

    /// Total non-abstain weight used for yes/no/veto ratio calculations.
    pub fn total_non_abstain(&self) -> u64 {
        self.yes_weight + self.no_weight + self.veto_weight
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Vote {
    pub voter: Addr,
    pub proposal_id: u64,
    pub option: VoteOption,
    pub weight: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum VoteOption {
    Yes,
    No,
    Veto,
    Abstain,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApprovedClass {
    /// Unique class identifier (e.g. "C10", "KSH02").
    pub class_id: String,
    pub credit_type: String,
    pub methodology_iri: String,
    pub admin_address: String,
    /// Proposal ID that led to approval.
    pub proposal_id: u64,
    /// Block time of approval.
    pub approved_at: u64,
}
