use crate::state::{
    ApprovedClass, Config, Proposal, ProposalStatus, Tally, Vote, VoteOption,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ── Instantiate ────────────────────────────────────────────────────────

#[cw_serde]
pub struct InstantiateMsg {
    /// Contract admin (can submit agent scores, override rejections, update config).
    /// Defaults to the message sender if not provided.
    pub admin: Option<String>,
    /// Quorum threshold (0-1000). Default: 100 (10%).
    pub quorum_threshold: Option<u64>,
    /// Pass threshold (0-1000). Default: 500 (50%).
    pub pass_threshold: Option<u64>,
    /// Veto threshold (0-1000). Default: 334 (33.4%).
    pub veto_threshold: Option<u64>,
    /// Voting period in seconds. Default: 604800 (7 days).
    pub voting_period_seconds: Option<u64>,
    /// Human override window in seconds. Default: 21600 (6 hours).
    pub override_window_seconds: Option<u64>,
}

// ── Execute ────────────────────────────────────────────────────────────

#[cw_serde]
pub enum ExecuteMsg {
    /// Submit a credit class proposal for approval.
    ProposeClass {
        credit_type: String,
        methodology_iri: String,
        admin_address: String,
    },

    /// Submit an agent pre-screening score (admin only).
    SubmitAgentScore {
        proposal_id: u64,
        score: u64,
        confidence: u64,
        factors: ScoringFactorsMsg,
    },

    /// Cast a weighted vote on a proposal in VOTING state.
    CastVote {
        proposal_id: u64,
        vote: VoteOption,
        weight: u64,
    },

    /// Human override: advance an agent-rejected proposal to VOTING (admin only).
    /// Must be called within the override window.
    OverrideAgentReject {
        proposal_id: u64,
    },

    /// Execute a proposal after the voting period ends.
    /// Determines APPROVED, REJECTED, or EXPIRED based on tally.
    ExecuteProposal {
        proposal_id: u64,
        /// Class ID to assign if approved (e.g. "C10").
        class_id: Option<String>,
    },

    /// Finalize an agent-rejected proposal after the override window expires
    /// without a human override. Transitions to REJECTED.
    FinalizeAgentReject {
        proposal_id: u64,
    },

    /// Update contract configuration (admin only).
    UpdateConfig {
        admin: Option<String>,
        quorum_threshold: Option<u64>,
        pass_threshold: Option<u64>,
        veto_threshold: Option<u64>,
        voting_period_seconds: Option<u64>,
        override_window_seconds: Option<u64>,
    },
}

#[cw_serde]
pub struct ScoringFactorsMsg {
    pub methodology_quality: u64,
    pub admin_reputation: u64,
    pub novelty: u64,
    pub completeness: u64,
}

// ── Query ──────────────────────────────────────────────────────────────

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Get contract configuration.
    #[returns(ConfigResponse)]
    Config {},

    /// Get a single proposal by ID.
    #[returns(ProposalResponse)]
    Proposal { id: u64 },

    /// List proposals with optional status filter.
    #[returns(ProposalsResponse)]
    Proposals {
        status: Option<ProposalStatusFilter>,
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    /// Get the vote tally for a proposal.
    #[returns(TallyResponse)]
    Tally { proposal_id: u64 },

    /// Get a voter's vote on a proposal.
    #[returns(VoteResponse)]
    Vote { proposal_id: u64, voter: String },

    /// List approved credit classes.
    #[returns(ApprovedClassesResponse)]
    ApprovedClasses {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Get a single approved class by class_id.
    #[returns(ApprovedClassResponse)]
    ApprovedClass { class_id: String },
}

// ── Query filter enum ──────────────────────────────────────────────────

#[cw_serde]
pub enum ProposalStatusFilter {
    AgentReview,
    Voting,
    Approved,
    Rejected,
    Expired,
}

impl ProposalStatusFilter {
    pub fn matches(&self, status: &ProposalStatus) -> bool {
        matches!(
            (self, status),
            (ProposalStatusFilter::AgentReview, ProposalStatus::AgentReview)
                | (ProposalStatusFilter::Voting, ProposalStatus::Voting)
                | (ProposalStatusFilter::Approved, ProposalStatus::Approved)
                | (ProposalStatusFilter::Rejected, ProposalStatus::Rejected)
                | (ProposalStatusFilter::Expired, ProposalStatus::Expired)
        )
    }
}

// ── Response types ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalResponse {
    pub proposal: Proposal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalsResponse {
    pub proposals: Vec<Proposal>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TallyResponse {
    pub proposal_id: u64,
    pub tally: Tally,
    pub total_participating: u64,
    pub yes_ratio: Option<u64>,
    pub veto_ratio: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoteResponse {
    pub vote: Option<Vote>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApprovedClassesResponse {
    pub classes: Vec<ApprovedClass>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApprovedClassResponse {
    pub class: Option<ApprovedClass>,
}
