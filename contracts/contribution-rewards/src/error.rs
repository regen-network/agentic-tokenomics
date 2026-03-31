use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized: only admin can execute this action")]
    Unauthorized {},

    #[error("mechanism is not active (current state: {state})")]
    NotActive { state: String },

    #[error("mechanism must be in TRACKING state to transition to DISTRIBUTING")]
    NotTracking {},

    #[error("calibration period not complete: {elapsed_epochs} of {required_epochs} epochs elapsed")]
    CalibrationIncomplete {
        elapsed_epochs: u64,
        required_epochs: u64,
    },

    #[error("epoch {epoch} has not ended yet (ends at block {end_block})")]
    EpochNotEnded { epoch: u64, end_block: u64 },

    #[error("epoch {epoch} already finalized")]
    EpochAlreadyFinalized { epoch: u64 },

    #[error("no contributions recorded in epoch {epoch}")]
    NoContributions { epoch: u64 },

    #[error("stability commitment amount {amount} below minimum {min}")]
    CommitmentTooSmall { amount: u128, min: u128 },

    #[error("lock period {months} months outside allowed range [{min}, {max}]")]
    InvalidLockPeriod { months: u64, min: u64, max: u64 },

    #[error("no active stability commitment for address {addr}")]
    NoCommitment { addr: String },

    #[error("commitment has not matured yet (matures at block {maturity_block})")]
    CommitmentNotMatured { maturity_block: u64 },

    #[error("commitment already matured or exited")]
    CommitmentNotActive {},

    #[error("pool distribution shares must sum to 10000 bps")]
    InvalidWeights {},

    #[error("stability allocation exceeds cap ({allocated} > {cap})")]
    StabilityCapExceeded { allocated: u128, cap: u128 },

    #[error("community pool inflow must be positive")]
    ZeroInflow {},

    #[error("duplicate contribution recording for tx_hash {tx_hash}")]
    DuplicateContribution { tx_hash: String },
}
