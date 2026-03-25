# SDK v0.53 → v0.54 Migration Checklist

## Purpose

This document connects the economic mechanism specs (M012–M015) to the concrete engineering work required to upgrade Regen Ledger from Cosmos SDK v0.53 to v0.54, integrating the native x/poa module.

---

## Current Stack (Post Regen Ledger v7.0, February 2026)

| Component | Current Version | Target Version | Change Type |
|-----------|----------------|----------------|-------------|
| Cosmos SDK | v0.53.4 | v0.54.x | Minor version upgrade |
| CometBFT | v0.38.19 | v0.39.x | Minor version upgrade |
| IBC-Go | v10.4.0 | v11.x | Major version upgrade |
| CosmWasm wasmd | v0.60.1 | TBD | Compatibility check needed |
| WasmVM | v2.2.4 | TBD | Compatibility check needed |

## Expected Release Timeline

| Component | Expected Release |
|-----------|------------------|
| CometBFT v0.39 | End Q1 / Early Q2 2026 |
| Cosmos SDK v0.54 | Early Q2 2026 |
| IBC-Go v11 | Q2 2026 |

---

## Pre-Migration Checklist

### 1. SDK v0.54 Assessment
- [ ] Review SDK v0.54 changelog for breaking changes
- [ ] Verify x/poa module API and integration requirements
- [ ] Check migration tooling provided by Cosmos Labs (code migration tools mentioned in roadmap)
- [ ] Assess impact on existing custom modules (x/ecocredit, x/data)
- [ ] Verify x/protocolpool compatibility (added in v7.0)
- [ ] Verify x/circuit compatibility (added in v7.0)

### 2. CometBFT v0.39 Assessment
- [ ] Review BLS signing changes
- [ ] Check experimental libp2p/QUIC networking implications
- [ ] Verify backward compatibility with existing node configurations
- [ ] Test state sync and snapshot compatibility

### 3. IBC-Go v11 Assessment
- [ ] Review GMP (General Message Passing) support
- [ ] Review IFT (Interchain Fungible Token) support
- [ ] Verify existing IBC channel compatibility
- [ ] Test IBC Wasm Light Client (added in v7.0) with new IBC version
- [ ] Check relayer compatibility (Hermes, Go relayer)

### 4. CosmWasm Compatibility
- [ ] Verify wasmd compatibility with SDK v0.54
- [ ] Check existing uploaded contracts for compatibility
- [ ] Verify governance-gated upload restrictions still function

---

## x/poa Integration Steps

### Step 1: Module Registration
- [ ] Add x/poa to app.go module manager
- [ ] Configure x/poa to replace x/staking for consensus
- [ ] Set initial admin authority (x/gov module address)
- [ ] Define genesis configuration for initial validator set

### Step 2: x/staking Deprecation
- [ ] Plan staking → PoA transition (not instant cutover)
- [ ] Handle existing delegations (unbonding period: 21 days)
- [ ] Disable new delegations
- [ ] Migrate validator records

### Step 3: Governance Integration
- [ ] Verify x/gov works with x/poa authority-weighted voting
- [ ] Test proposal submission by PoA validators
- [ ] Test voting with validator power as weight
- [ ] Verify weighted vote splits function

### Step 4: Fee Distribution
- [ ] Configure fee_collector for x/poa checkpoint distribution
- [ ] Test fee accumulation and withdrawal
- [ ] Verify compatibility with M013 fee routing architecture

---

## Migration Sequence

```
Phase A: SDK Upgrade (no consensus change)
  1. Upgrade SDK v0.53 → v0.54
  2. Upgrade CometBFT v0.38 → v0.39
  3. Upgrade IBC-Go v10 → v11
  4. Add x/poa module (inactive)
  5. Test on regen-upgrade testnet
  6. Governance proposal for mainnet upgrade

Phase B: Consensus Migration
  1. Activate x/poa with seed validator set
  2. Begin delegation wind-down period
  3. Disable new x/staking delegations
  4. Existing delegations unbond (21-day period)
  5. Deactivate x/staking rewards
  6. Monitor IBC channel health throughout

Phase C: Economic Integration
  1. Deploy M013 fee routing (CosmWasm or native)
  2. Configure fee → burn/validator/community splits
  3. Activate M012 supply algorithm
  4. Deploy M015 contribution tracking (later phase)
```

---

## Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| SDK upgrade breaks custom modules | High | Medium | Extensive testnet testing; SDK migration tooling |
| IBC channels break during validator set change | High | Low | Power change rate limiting; IBC health monitoring |
| Existing stakers panic-sell on PoA announcement | Medium | Medium | Pre-socialization (ongoing since 2024); M015 stability tier as alternative |
| x/poa module immaturity at launch | Medium | Medium | Strangelove module as fallback; extensive testnet testing |
| CosmWasm incompatibility | Medium | Low | Test all existing contracts on testnet |
| Low transaction volume masks bugs | Low | Medium | Synthetic load testing on testnet |

### Safety Mechanisms Available

- **Circuit Breaker** (x/circuit, added in v7.0): Can halt specific message types in emergency
- **Governance**: Can propose parameter changes or rollback via governance
- **IBC Connection Monitoring**: Relayer health checks
- **AGENT-004**: Automated validator monitoring and alerting

---

## Engineering Coordination

### Cosmos Labs
- Contact: institutions@cosmoslabs.io
- Request: x/poa integration support, early access to SDK v0.54 release candidates
- Timeline alignment: Regen upgrade should follow SDK v0.54 stable release

### Regen Engineering Team
- Sam (DevOps/Validators): Testnet infrastructure, validator coordination
- Anil (Core Dev): SDK upgrade implementation, module integration
- Community validators: Testnet participation, upgrade readiness

### Dependencies on Other Workstreams

| Dependency | Workstream | Status |
|-----------|------------|--------|
| M013 fee routing spec finalized | Tokenomics WG | PR #23 open |
| M014 validator set defined | Tokenomics WG | PR #24 open |
| Seed validator candidates identified | Foundation + Community | Not started |
| CosmWasm contracts for fee routing | Engineering | Not started |
| Governance proposal drafted | Community | Not started |

---

## Related

- Cosmos Labs x/poa module: `docs/cosmos-poa-module.md`
- M014 spec: `phase-2/2.6-economic-reboot-mechanisms.md`
- M013 spec: `phase-2/2.6-economic-reboot-mechanisms.md`
- Regen Ledger v7.0 upgrade: https://forum.regen.network/t/software-upgrade-proposal-regen-ledger-v7-0/574
- Cosmos Stack 2026 Roadmap: https://www.cosmoslabs.io/blog/the-cosmos-stack-roadmap-2026
