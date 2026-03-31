use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

// ── Config ──────────────────────────────────────────────────────────

#[cw_serde]
pub struct Config {
    /// Contract admin (can submit scores, resolve challenges)
    pub admin: Addr,
    /// Required bond denomination (e.g. "uregen")
    pub bond_denom: String,
    /// Minimum curation bond (default 1_000_000_000 uregen = 1000 REGEN)
    pub min_curation_bond: Uint128,
    /// Listing fee per batch added (default 10_000_000 uregen = 10 REGEN)
    pub listing_fee: Uint128,
    /// Curation fee rate in basis points (default 50 = 0.5%)
    pub curation_fee_bps: u64,
    /// Challenge deposit amount
    pub challenge_deposit: Uint128,
    /// Slash percentage in basis points (default 2000 = 20%)
    pub slash_pct_bps: u64,
    /// Challenger reward share of slashed amount in bps (default 5000 = 50%)
    pub challenge_reward_bps: u64,
    /// Activation delay in seconds (default 172800 = 48h)
    pub activation_delay_s: u64,
    /// Unbonding period in seconds (default 1_209_600 = 14 days)
    pub unbonding_period_s: u64,
    /// Bond top-up window in seconds (default 604_800 = 7 days)
    pub top_up_window_s: u64,
    /// Minimum quality score for batch inclusion (default 300)
    pub min_quality_score: u64,
    /// Maximum collections per curator (default 5)
    pub max_collections_per_curator: u64,
}

// ── Collection ──────────────────────────────────────────────────────

#[cw_serde]
pub enum CollectionStatus {
    Proposed,
    Active,
    UnderReview,
    Suspended,
    Closed,
}

#[cw_serde]
pub struct CurationCriteria {
    /// Minimum project reputation (optional filter)
    pub min_project_reputation: Option<u64>,
    /// Minimum class reputation (optional filter)
    pub min_class_reputation: Option<u64>,
    /// Allowed credit types (empty = all)
    pub allowed_credit_types: Vec<String>,
    /// Minimum vintage year (optional)
    pub min_vintage_year: Option<u64>,
    /// Maximum vintage year (optional)
    pub max_vintage_year: Option<u64>,
}

#[cw_serde]
pub struct Collection {
    pub id: u64,
    pub curator: Addr,
    pub name: String,
    pub description: String,
    pub criteria: CurationCriteria,
    pub bond_amount: Uint128,
    pub bond_remaining: Uint128,
    pub status: CollectionStatus,
    pub members: Vec<String>,
    pub trade_volume: Uint128,
    pub total_rewards: Uint128,
    pub created_at_s: u64,
    pub activated_at_s: Option<u64>,
    /// Timestamp when suspension started (for top-up window tracking)
    pub suspended_at_s: Option<u64>,
    /// Timestamp when close was initiated (for unbonding period)
    pub close_initiated_at_s: Option<u64>,
}

// ── Challenge ───────────────────────────────────────────────────────

#[cw_serde]
pub enum ChallengeOutcome {
    CuratorWins,
    ChallengerWins,
}

#[cw_serde]
pub struct Challenge {
    pub id: u64,
    pub collection_id: u64,
    pub challenger: Addr,
    pub batch_denom: String,
    pub reason: String,
    pub deposit: Uint128,
    pub outcome: Option<ChallengeOutcome>,
    pub challenged_at_s: u64,
    pub resolved_at_s: Option<u64>,
}

// ── Quality Score ───────────────────────────────────────────────────

#[cw_serde]
pub struct QualityFactors {
    pub project_reputation: u64,
    pub class_reputation: u64,
    pub vintage_freshness: u64,
    pub verification_recency: u64,
    pub seller_reputation: u64,
    pub price_fairness: u64,
    pub additionality_confidence: u64,
}

#[cw_serde]
pub struct QualityScore {
    pub batch_denom: String,
    pub score: u64,
    pub confidence: u64,
    pub factors: QualityFactors,
    pub scored_at_s: u64,
}

// ── Storage keys ────────────────────────────────────────────────────

pub const CONFIG: Item<Config> = Item::new("config");
pub const COLLECTION_SEQ: Item<u64> = Item::new("collection_seq");
pub const CHALLENGE_SEQ: Item<u64> = Item::new("challenge_seq");

/// collection_id -> Collection
pub const COLLECTIONS: Map<u64, Collection> = Map::new("collections");

/// (curator_addr) -> count of collections owned
pub const CURATOR_COLLECTION_COUNT: Map<&Addr, u64> = Map::new("curator_col_count");

/// challenge_id -> Challenge
pub const CHALLENGES: Map<u64, Challenge> = Map::new("challenges");

/// collection_id -> active challenge_id (only one pending at a time)
pub const ACTIVE_CHALLENGE: Map<u64, u64> = Map::new("active_challenge");

/// batch_denom -> latest QualityScore
pub const QUALITY_SCORES: Map<&str, QualityScore> = Map::new("quality_scores");

/// batch_denom -> Vec<QualityScore> (append-only history)
pub const QUALITY_HISTORY: Map<&str, Vec<QualityScore>> = Map::new("quality_history");
