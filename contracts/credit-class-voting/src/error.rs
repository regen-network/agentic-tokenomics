use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized: only admin can perform this action")]
    Unauthorized,

    #[error("proposal {id} not found")]
    ProposalNotFound { id: u64 },

    #[error("invalid credit type: {credit_type} (allowed: C, KSH, BT, MBS, USS)")]
    InvalidCreditType { credit_type: String },

    #[error("invalid score: {value} (must be 0-1000)")]
    InvalidScore { value: u64 },

    #[error("invalid threshold: {value} (must be 1-1000, representing 0.1%-100%)")]
    InvalidThreshold { value: u64 },

    #[error("proposal {id} is not in {expected} state (currently {actual})")]
    InvalidState {
        id: u64,
        expected: String,
        actual: String,
    },

    #[error("voter {voter} has already voted on proposal {proposal_id}")]
    AlreadyVoted { voter: String, proposal_id: u64 },

    #[error("voting period for proposal {id} has not ended yet")]
    VotingPeriodNotEnded { id: u64 },

    #[error("voting period for proposal {id} has already ended")]
    VotingPeriodEnded { id: u64 },

    #[error("override window for proposal {id} has expired")]
    OverrideWindowExpired { id: u64 },

    #[error("override window for proposal {id} has not expired yet")]
    OverrideWindowNotExpired { id: u64 },

    #[error("proposal {id} was not agent-rejected (cannot override)")]
    NotAgentRejected { id: u64 },
}
