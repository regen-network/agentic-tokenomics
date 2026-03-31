use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized: only admin can perform this action")]
    Unauthorized,

    #[error("validator already exists: {address}")]
    ValidatorAlreadyExists { address: String },

    #[error("validator not found: {address}")]
    ValidatorNotFound { address: String },

    #[error("invalid status transition from {from} to {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("validator set full: {current}/{max} validators")]
    ValidatorSetFull { current: u32, max: u32 },

    #[error("minimum validators reached: cannot remove below {min}")]
    MinimumValidatorsReached { min: u32 },

    #[error("composition violation: category {category} would drop below minimum {min}")]
    CompositionViolation { category: String, min: u32 },

    #[error("proposal not found: {id}")]
    ProposalNotFound { id: u64 },

    #[error("proposal not active: {id}")]
    ProposalNotActive { id: u64 },

    #[error("voting period expired for proposal: {id}")]
    VotingPeriodExpired { id: u64 },

    #[error("voting period not expired for proposal: {id}")]
    VotingPeriodNotExpired { id: u64 },

    #[error("already voted on proposal: {id}")]
    AlreadyVoted { id: u64 },

    #[error("not an active validator: {address}")]
    NotActiveValidator { address: String },

    #[error("invalid score: must be between 0 and 10000 (basis points)")]
    InvalidScore,

    #[error("invalid category: must be infrastructure_builders, trusted_refi_partners, or ecological_data_stewards")]
    InvalidCategory,

    #[error("invalid voting period: must be > 0")]
    InvalidVotingPeriod,

    #[error("term expired for validator: {address}")]
    TermExpired { address: String },

    #[error("validator on probation: {address}")]
    ValidatorOnProbation { address: String },
}
