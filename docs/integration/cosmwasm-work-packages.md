# CosmWasm Integration Work Packages

## Overview

This document specifies work packages for implementing Claims Engine functionality via CosmWasm smart contracts on Regen Ledger.

---

## Work Package Summary

| WP ID | Name | Priority | Dependencies | Estimated Effort |
|-------|------|----------|--------------|------------------|
| WP-CW-001 | Attestation Bond Contract | High | None | 4-6 weeks |
| WP-CW-002 | Claims Registry Contract | High | WP-CW-001 | 6-8 weeks |
| WP-CW-003 | Evidence Aggregator Contract | Medium | WP-CW-002 | 4-6 weeks |
| WP-CW-004 | Reputation Registry Contract | Medium | WP-CW-001 | 4-6 weeks |
| WP-CW-005 | Arbiter DAO Integration | Medium | WP-CW-002, WP-CW-004 | 6-8 weeks |

---

## WP-CW-001: Attestation Bond Contract

### Purpose
Enable economic skin-in-the-game for claim attestors through bonded attestations.

### Key Functions
- `Attest`: Create a bonded attestation
- `IncreaseBond`: Increase bond on existing attestation
- `Challenge`: Challenge an attestation
- `ResolveChallenge`: Arbiter-only resolution
- `WithdrawBond`: Withdraw after unbonding period

### Attestation Types
- `DataAccuracy`: Attesting data is accurate
- `MethodologyCompliance`: Attesting methodology was followed
- `FieldVerification`: Attesting on-site verification
- `FullClaim`: Attesting entire claim validity

---

## WP-CW-002: Claims Registry Contract

### Purpose
Central registry for ecological state claims with lifecycle management.

### Key Functions
- `SubmitClaim`: Submit a new claim
- `UpdateClaimStatus`: Update claim status (authorized roles only)
- `LinkAttestation`: Link attestations to claim
- `RequestVerification`: Request claim verification
- `RequestCreditIssuance`: Convert verified claim to credit

### Claim Status Flow
```
Draft → Submitted → UnderReview → Validated → Attested → Verified
                                      ↓
                                  Rejected
```

---

## WP-CW-004: Reputation Registry Contract

### Purpose
Track and expose reputation scores for claim-related actors.

### Reputation Dimensions
- Claims submitted/verified/rejected
- Attestations made/challenged/slashed
- Verifications performed and accuracy
- Composite reliability and expertise scores

### Reputation Update Triggers
- Positive: claim_verified (+10), attestation_confirmed (+5), challenge_won (+15)
- Negative: claim_rejected (-15), attestation_slashed (-20), challenge_lost (-10)

---

## WP-CW-005: Arbiter DAO Integration

### Purpose
Connect claims dispute resolution to DAO DAO governance structures.

### Arbiter Qualifications
- Reputation score > 0.75
- Domain expertise (methodology-specific)
- Stake requirement met
- No conflict of interest

---

## Implementation Timeline

```
2026 Q2                    Q3                    Q4
├──────────────────────────┼──────────────────────┼──────────────────────┤
│ WP-CW-001                │                      │                      │
│ Attestation Bond         │                      │                      │
│ ████████████████         │                      │                      │
│                          │                      │                      │
│ WP-CW-002                │                      │                      │
│ Claims Registry          │                      │                      │
│      ████████████████████████████████           │                      │
│                          │                      │                      │
│                          │ WP-CW-003/004        │                      │
│                          │ Evidence/Reputation  │                      │
│                          │ ████████████████     │                      │
│                          │                      │                      │
│                          │                      │ WP-CW-005            │
│                          │                      │ Arbiter DAO          │
│                          │                      │ ████████████████████ │
└──────────────────────────┴──────────────────────┴──────────────────────┘
```

---

## References

- [CosmWasm Documentation](https://docs.cosmwasm.com/)
- [DAO DAO Contracts](https://github.com/DA0-DA0/dao-contracts)
- [Phase 3.1 Smart Contract Specs](../../phase-3/3.1-smart-contract-specs.md)

---

*This document is part of the Regen Network Agentic Tokenomics framework.*
