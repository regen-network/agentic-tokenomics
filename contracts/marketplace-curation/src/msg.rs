use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::{ChallengeOutcome, CurationCriteria, QualityFactors};

// ── Instantiate ─────────────────────────────────────────────────────

#[cw_serde]
pub struct InstantiateMsg {
    /// Bond denomination (e.g. "uregen")
    pub bond_denom: String,
    /// Minimum curation bond in bond_denom units
    pub min_curation_bond: Uint128,
    /// Listing fee per batch added
    pub listing_fee: Uint128,
    /// Curation fee rate in basis points (e.g. 50 = 0.5%)
    pub curation_fee_bps: u64,
    /// Challenge deposit amount
    pub challenge_deposit: Uint128,
    /// Slash percentage in basis points (e.g. 2000 = 20%)
    pub slash_pct_bps: u64,
    /// Challenger reward share in bps (e.g. 5000 = 50%)
    pub challenge_reward_bps: u64,
    /// Activation delay in seconds
    pub activation_delay_s: u64,
    /// Unbonding period in seconds
    pub unbonding_period_s: u64,
    /// Bond top-up window in seconds
    pub top_up_window_s: u64,
    /// Minimum quality score for batch inclusion
    pub min_quality_score: u64,
    /// Maximum collections per curator
    pub max_collections_per_curator: u64,
}

// ── Execute ─────────────────────────────────────────────────────────

#[cw_serde]
pub enum ExecuteMsg {
    /// Create a new curated collection (must send bond funds)
    CreateCollection {
        name: String,
        description: String,
        criteria: CurationCriteria,
    },

    /// Activate a collection after the activation delay has elapsed
    ActivateCollection { collection_id: u64 },

    /// Add a batch to an active collection (must send listing fee)
    AddBatch {
        collection_id: u64,
        batch_denom: String,
    },

    /// Remove a batch from an active collection
    RemoveBatch {
        collection_id: u64,
        batch_denom: String,
    },

    /// Challenge inclusion of a batch in a collection (must send challenge deposit)
    ChallengeInclusion {
        collection_id: u64,
        batch_denom: String,
        reason: String,
    },

    /// Resolve a pending challenge (admin only)
    ResolveChallenge {
        challenge_id: u64,
        outcome: ChallengeOutcome,
    },

    /// Top up bond on a suspended collection (must send funds)
    TopUpBond { collection_id: u64 },

    /// Close a collection (curator only, no pending challenges, starts unbonding)
    CloseCollection { collection_id: u64 },

    /// Claim refund after unbonding period
    ClaimRefund { collection_id: u64 },

    /// Record trade volume and distribute curation rewards (admin only)
    RecordTrade {
        collection_id: u64,
        trade_amount: Uint128,
    },

    /// Submit a quality score for a batch (admin / agent only)
    SubmitQualityScore {
        batch_denom: String,
        score: u64,
        confidence: u64,
        factors: QualityFactors,
    },

    /// Force-close a suspended collection after top-up window expires (anyone)
    ForceCloseSuspended { collection_id: u64 },

    /// Update config parameters (admin only)
    UpdateConfig {
        min_curation_bond: Option<Uint128>,
        listing_fee: Option<Uint128>,
        curation_fee_bps: Option<u64>,
        challenge_deposit: Option<Uint128>,
        slash_pct_bps: Option<u64>,
        challenge_reward_bps: Option<u64>,
        activation_delay_s: Option<u64>,
        unbonding_period_s: Option<u64>,
        top_up_window_s: Option<u64>,
        min_quality_score: Option<u64>,
        max_collections_per_curator: Option<u64>,
    },
}

// ── Query ───────────────────────────────────────────────────────────

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Return contract configuration
    #[returns(ConfigResponse)]
    Config {},

    /// Return a single collection by ID
    #[returns(CollectionResponse)]
    Collection { collection_id: u64 },

    /// List collections, optionally filtered by curator or status
    #[returns(CollectionsResponse)]
    Collections {
        curator: Option<String>,
        status: Option<String>,
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    /// Return the latest quality score for a batch
    #[returns(QualityScoreResponse)]
    QualityScore { batch_denom: String },

    /// Return full quality score history for a batch
    #[returns(QualityHistoryResponse)]
    QualityHistory { batch_denom: String },

    /// Return a challenge by ID
    #[returns(ChallengeResponse)]
    Challenge { challenge_id: u64 },

    /// Return the active challenge for a collection, if any
    #[returns(Option<ChallengeResponse>)]
    ActiveChallenge { collection_id: u64 },

    /// Return curator stats (number of collections, total bond, total rewards)
    #[returns(CuratorStatsResponse)]
    CuratorStats { curator: String },

    /// Return the listing score for a batch across all collections it appears in
    #[returns(ListingScoreResponse)]
    ListingScore { batch_denom: String },
}

// ── Response types ──────────────────────────────────────────────────

#[cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub bond_denom: String,
    pub min_curation_bond: Uint128,
    pub listing_fee: Uint128,
    pub curation_fee_bps: u64,
    pub challenge_deposit: Uint128,
    pub slash_pct_bps: u64,
    pub challenge_reward_bps: u64,
    pub activation_delay_s: u64,
    pub unbonding_period_s: u64,
    pub top_up_window_s: u64,
    pub min_quality_score: u64,
    pub max_collections_per_curator: u64,
}

#[cw_serde]
pub struct CollectionResponse {
    pub id: u64,
    pub curator: String,
    pub name: String,
    pub description: String,
    pub criteria: CurationCriteria,
    pub bond_amount: Uint128,
    pub bond_remaining: Uint128,
    pub status: String,
    pub members: Vec<String>,
    pub trade_volume: Uint128,
    pub total_rewards: Uint128,
    pub created_at_s: u64,
    pub activated_at_s: Option<u64>,
}

#[cw_serde]
pub struct CollectionsResponse {
    pub collections: Vec<CollectionResponse>,
}

#[cw_serde]
pub struct QualityScoreResponse {
    pub batch_denom: String,
    pub score: u64,
    pub confidence: u64,
    pub factors: QualityFactors,
    pub scored_at_s: u64,
}

#[cw_serde]
pub struct QualityHistoryResponse {
    pub batch_denom: String,
    pub scores: Vec<QualityScoreResponse>,
}

#[cw_serde]
pub struct ChallengeResponse {
    pub id: u64,
    pub collection_id: u64,
    pub challenger: String,
    pub batch_denom: String,
    pub reason: String,
    pub deposit: Uint128,
    pub outcome: Option<String>,
    pub challenged_at_s: u64,
    pub resolved_at_s: Option<u64>,
}

#[cw_serde]
pub struct CuratorStatsResponse {
    pub curator: String,
    pub collection_count: u64,
    pub total_bond: Uint128,
    pub total_rewards: Uint128,
}

#[cw_serde]
pub struct ListingScoreResponse {
    pub batch_denom: String,
    pub quality_score: Option<u64>,
    pub confidence: Option<u64>,
    pub collection_count: u64,
    pub featured: bool,
}
