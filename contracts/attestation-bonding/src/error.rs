use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error("unknown attestation type")]
    UnknownAttestationType {},

    #[error("no bond provided: must send uregen funds")]
    NoBondProvided {},

    #[error("insufficient bond: required {required}, provided {provided}")]
    InsufficientBond { required: Uint128, provided: Uint128 },

    #[error("attestation not challengeable: must be in Active or Bonded state")]
    AttestationNotChallengeable {},

    #[error("challenge window closed")]
    ChallengeWindowClosed {},

    #[error("no challenge deposit provided")]
    NoDepositProvided {},

    #[error("insufficient challenge deposit: required {required}, provided {provided}")]
    InsufficientChallengeDeposit { required: Uint128, provided: Uint128 },

    #[error("attestation already has an active challenge")]
    AlreadyChallenged {},

    #[error("challenge not found")]
    ChallengeNotFound {},

    #[error("challenge already resolved")]
    ChallengeAlreadyResolved {},

    #[error("attestation not found")]
    AttestationNotFound {},

    #[error("lock period not expired")]
    LockPeriodNotExpired {},

    #[error("attestation not releasable: must be in Active state with no active challenge")]
    AttestationNotReleasable {},

    #[error("only the attester can release the bond")]
    NotAttester {},

    #[error("attestation type already exists: {name}")]
    AttestationTypeAlreadyExists { name: String },

    #[error("arbiter cannot be the attester or challenger")]
    ArbiterConflict {},

    #[error("overflow error")]
    Overflow {},
}
