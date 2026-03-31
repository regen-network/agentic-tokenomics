use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Collection not found: {id}")]
    CollectionNotFound { id: u64 },

    #[error("Challenge not found: {id}")]
    ChallengeNotFound { id: u64 },

    #[error("Quality score not found for batch: {batch_denom}")]
    ScoreNotFound { batch_denom: String },

    #[error("Bond amount {sent} is below minimum {min}")]
    InsufficientBond { sent: u128, min: u128 },

    #[error("Challenge deposit {sent} is below required {required}")]
    InsufficientChallengeDeposit { sent: u128, required: u128 },

    #[error("Wrong denomination: sent {sent}, expected {expected}")]
    WrongDenom { sent: String, expected: String },

    #[error("Collection is not in ACTIVE status")]
    CollectionNotActive {},

    #[error("Collection is not in PROPOSED status")]
    CollectionNotProposed {},

    #[error("Collection is not in SUSPENDED status")]
    CollectionNotSuspended {},

    #[error("Collection is not in UNDER_REVIEW status")]
    CollectionNotUnderReview {},

    #[error("Activation delay has not elapsed")]
    ActivationDelayNotElapsed {},

    #[error("Collection has a pending challenge")]
    PendingChallenge {},

    #[error("Batch {batch_denom} is already in collection {collection_id}")]
    BatchAlreadyInCollection {
        batch_denom: String,
        collection_id: u64,
    },

    #[error("Batch {batch_denom} is not in collection {collection_id}")]
    BatchNotInCollection {
        batch_denom: String,
        collection_id: u64,
    },

    #[error("Quality score {score} is below minimum {min}")]
    QualityScoreTooLow { score: u64, min: u64 },

    #[error("Curator has reached the maximum number of collections ({max})")]
    MaxCollectionsReached { max: u64 },

    #[error("Curator cannot challenge own collection")]
    SelfChallenge {},

    #[error("Unbonding period has not elapsed")]
    UnbondingNotElapsed {},

    #[error("Top-up window has expired")]
    TopUpWindowExpired {},

    #[error("Top-up window has not expired")]
    TopUpWindowNotExpired {},

    #[error("Bond remaining is below minimum after top-up")]
    BondBelowMinAfterTopUp {},

    #[error("Suspension period has not expired")]
    SuspensionNotExpired {},

    #[error("Only the admin can submit quality scores")]
    OnlyAdminCanScore {},

    #[error("Only the admin can resolve challenges")]
    OnlyAdminCanResolve {},

    #[error("No funds sent")]
    NoFundsSent {},
}
