# Cosmos Labs x/poa Module — Technical Integration Reference

## Overview

Cosmos Labs is shipping a native Proof-of-Authority module (`x/poa`) in **Cosmos SDK v0.54 (Q2 2026)**. This module is a direct replacement for `x/staking` and is the recommended implementation path for M014 (Authority Validator Governance).

This document bridges the M014 mechanism specification with the actual Cosmos SDK tooling that will implement it.

## Sources

- Cosmos Labs blog: https://www.cosmoslabs.io/blog/inside-the-poa-module
- Cosmos Stack 2026 Roadmap: https://www.cosmoslabs.io/blog/the-cosmos-stack-roadmap-2026
- Strangelove Ventures PoA (reference impl): https://github.com/strangelove-ventures/poa
- Cosmos Hub Forum discussion: https://forum.cosmos.network/t/proof-of-authority-module/5818
- Gregory_Regen reference (Jan 2026): https://forum.regen.network/t/regen-economic-reboot-roadmap-v0-1/567/4

---

## Cosmos Labs Native x/poa Module

### Architecture

- **Direct replacement** for `x/staking` — not a wrapper
- Token-free alternative for PoA operation
- Decoupled governance from staking mechanism
- Built-in migration pathway between PoA and PoS

### Validator Lifecycle

```
REGISTERED (power=0) → ACTIVE (power>0) → REMOVED (power=0, historical)
```

1. **Registration**: Any account can register as a validator candidate. Candidates start at power 0 and do not participate in consensus.
2. **Activation**: An authorized administrator uses `MsgUpdateValidators` to add, remove, replace, reweight, or rotate validator keys — all in a single atomic transaction.
3. **Consensus Commit**: "At EndBlock, the module commits the updated validator set to CometBFT" — atomic and predictable at block boundaries.
4. **Removal**: Setting power to 0 removes from consensus but maintains historical records for potential reactivation.

### Governance Model

- Authority-weighted governance replaces token-weighted
- Only active validators (power > 0) can submit proposals, deposit, and vote
- Voting weight directly linked to validator consensus power
- Supports weighted vote splits
- Compatible with `x/gov` module

### Fee Distribution

- Checkpoint-based accounting system
- Fees accumulate in `fee_collector` module account
- Checkpoints trigger when validator power changes or validators withdraw rewards
- Avoids per-block overhead while preserving fair distribution across validator set transitions

### Key Messages

| Message | Description | Authority |
|---------|-------------|----------|
| `MsgUpdateValidators` | Add/remove/reweight validators atomically | Admin (x/gov) |
| `MsgRegisterValidator` | Register as candidate (power=0) | Any account |
| `MsgWithdrawRewards` | Claim accumulated fee rewards | Validator |

---

## Strangelove Ventures PoA Module (Reference)

An alternative implementation audited by Hashlock (November 2024). The author recommends the Cosmos Labs module for new implementations on SDK v53+.

### Key Differences from Cosmos Labs

| Aspect | Cosmos Labs | Strangelove |
|--------|-------------|-------------|
| Approach | Replaces x/staking | Wraps x/staking |
| Backward compat | New API | Preserves existing UIs/bots |
| Delegation | Disabled by design | Disabled via ante handler |
| Power change limit | TBD | Max 30% per block |
| Audit | Cosmos Labs maintained | Hashlock Nov 2024 |
| Maturity | New (H1 2026) | Production-ready |

### Strangelove CLI Commands (useful for testing)

```bash
poad tx poa create-validator [config-file]
poad tx poa set-power [validator] [amount]
poad tx poa remove [validator]
poad q poa pending-validators
poad q poa power [validator]
```

---

## Release Timeline

| Component | Version | Expected | Relevance |
|-----------|---------|----------|----------|
| Cosmos SDK | v0.54 | Early Q2 2026 | Native x/poa included |
| CometBFT | v0.39 | End Q1/Early Q2 2026 | BLS signing, libp2p |
| ibc-go | v11 | Q2 2026 | GMP, IFT support |
| Performance | — | Q4 2026 | 5,000 TPS target, 500ms blocks |

Regen Ledger is currently on SDK v0.53.4, CometBFT v0.38.19, IBC-go v10.4.0 (post v7.0 upgrade, Feb 2026). The v0.54 upgrade is the natural next step.

---

## Integration with M014

### Mapping M014 Concepts to x/poa

| M014 Concept | x/poa Implementation |
|-------------|---------------------|
| Validator categories (3 types) | Application-level metadata; x/poa manages power |
| 15-21 validator set | `MsgUpdateValidators` with governance-controlled list |
| Term-based rotation | Governance proposals to update validator set periodically |
| Fixed compensation | `fee_collector` + checkpoint distribution |
| Performance bonus | Application-level calculation, distributed via governance |
| Probation | Reduce power to minimum (not 0) or flag via metadata |
| Removal | Set power to 0 via `MsgUpdateValidators` |
| Seed set | Genesis configuration or initial `MsgUpdateValidators` |

### What x/poa Does NOT Provide (M014 Must Implement)

1. **Category enforcement** — x/poa doesn't know about infrastructure/ReFi/steward categories. Need CosmWasm or governance-level logic.
2. **Term tracking** — No built-in term expiry. Need governance calendar or CosmWasm contract.
3. **Performance scoring** — No built-in uptime/governance/ecosystem contribution tracking. AGENT-004 (Validator Monitor) fills this role.
4. **Compensation splitting** — x/poa distributes fees equally by power. Performance bonuses need additional logic.
5. **Candidate application process** — x/poa allows anyone to register; approval/curation is application-level.

### Recommended Architecture

```
x/poa (Cosmos SDK native)
  ├── Validator set management
  ├── Consensus power allocation
  └── Fee distribution (checkpoint-based)

CosmWasm contracts (application layer)
  ├── Validator registry (categories, terms, metadata)
  ├── Performance tracking (uptime, governance, ecosystem)
  ├── Compensation calculator (base + bonus)
  └── Candidate application/approval workflow

AGENT-004 (off-chain)
  ├── Validator monitoring (WF-VM-01)
  ├── Performance scoring
  └── Probation/removal recommendations
```

---

## IBC Safety During Migration

Both Cosmos Labs and Strangelove modules address IBC connection safety:

- **Strangelove**: Enforces max 30% power change per block. Tracks previous block power to prevent breaking IBC connections (07-tendermint protocol).
- **Cosmos Labs**: Atomic validator set updates at EndBlock. Details on power change limits TBD.

Regen has active IBC channels. The migration plan MUST include:
1. Power change rate limiting during transition
2. IBC connection health monitoring
3. Rollback plan if IBC channels break
4. Coordination with counterparty chain relayers

---

## Contact

Cosmos Labs institutional support: institutions@cosmoslabs.io

## Related

- M014 spec: `phase-2/2.6-economic-reboot-mechanisms.md` (Authority Validator Governance section)
- M013 spec: `phase-2/2.6-economic-reboot-mechanisms.md` (Value-Based Fee Routing — compensation source)
- AGENT-004: `phase-2/2.4-agent-orchestration.md` (Validator Monitor)
- Regen Ledger v7.0: https://forum.regen.network/t/software-upgrade-proposal-regen-ledger-v7-0/574
