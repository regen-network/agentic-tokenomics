# Mainnet Readiness Checklist and Go/No-Go Framework

> **Status:** DRAFT — All gates PENDING
> **Last Updated:** 2026-03-24
> **Decision Authority:** Core Team + Governance Council
> **Target Launch Window:** TBD (pending gate completion)

---

## Table of Contents

1. [Overview](#overview)
2. [Smart Contract Audit Requirements](#smart-contract-audit-requirements)
3. [Governance Approval Sequence](#governance-approval-sequence)
4. [Validator Onboarding Thresholds](#validator-onboarding-thresholds)
5. [Agent Deployment Verification](#agent-deployment-verification)
6. [Rollback Triggers](#rollback-triggers)
7. [Go/No-Go Framework](#gono-go-framework)
8. [Launch Day Runbook](#launch-day-runbook)
9. [Post-Launch Monitoring](#post-launch-monitoring)
10. [Summary](#summary)

---

## Overview

This document is the **single source of truth** for the mainnet go/no-go decision.
It consolidates every pre-launch requirement across security, governance, infrastructure,
agent readiness, and community preparedness into a structured framework with measurable
gate criteria, clear decision authority, and explicit rollback procedures.

### Purpose

- Provide a unified checklist that all stakeholders reference when evaluating launch readiness
- Define measurable, binary gate criteria — each gate is PASS or FAIL with no ambiguity
- Establish rollback triggers with automated detection so the network can degrade safely
- Document the launch day runbook from T-24h through T+30d to ensure coordinated execution
- Create accountability by assigning owners to every gate and every runbook step

### Scope

This checklist covers mechanisms M008 through M015 and all supporting infrastructure:

| Mechanism | Name | Type |
|-----------|------|------|
| M008 | Fee Router | CosmWasm Contract |
| M009 | Dynamic Fee Adjustment | CosmWasm Contract |
| M010 | Treasury Management | CosmWasm Contract |
| M011 | Staking Derivatives | CosmWasm Contract |
| M012 | Supply Dampening | Native Module (x/supply) |
| M013 | Fee Distribution | Native Module (x/feerouter) |
| M014 | Authority & Governance | Native Module (x/authority) |
| M015 | Reward Distribution | Native Module (x/rewards) |

Additionally, this checklist covers:

- AI Agent deployment (KOI framework, MCP integration)
- Validator infrastructure and onboarding
- Governance proposal sequencing
- Community readiness and documentation
- Rollback and emergency procedures

### How to Use This Document

1. Each section contains checklists with `[ ]` items
2. As gates are satisfied, update to `[x]` with the date and evidence link
3. The [Go/No-Go Framework](#gono-go-framework) section aggregates all gates into a final decision matrix
4. No launch proceeds until all HARD BLOCKER gates are `[x]`
5. CONDITIONAL gates may be waived by decision authority with documented rationale

### Document Conventions

- **HARD BLOCKER** — Launch cannot proceed without this gate passing
- **CONDITIONAL** — Launch can proceed with documented risk acceptance
- **INFORMATIONAL** — Tracked for awareness, does not block launch
- **Owner** — Individual or team accountable for the gate
- **Evidence** — Link to artifact proving gate satisfaction (PR, report, dashboard)

---

## Smart Contract Audit Requirements

All CosmWasm contracts (M008-M011) and native modules (x/feerouter, x/supply, x/authority,
x/rewards) must undergo security review before mainnet deployment. This section defines
audit firm criteria, scope, timeline, acceptance criteria, and bug bounty activation.

### Audit Firm Selection Criteria

The selected audit firm(s) must meet ALL of the following criteria:

- [ ] **Cosmos/CosmWasm Experience** — Minimum 5 prior CosmWasm audits published, with at least 2 on production mainnet contracts managing >$10M TVL
- [ ] **Team Composition** — Audit team includes at least 2 senior auditors with Rust expertise and at least 1 with Cosmos SDK native module experience
- [ ] **Reputation** — No unresolved disputes or missed critical vulnerabilities in past 24 months (verified via public disclosure records)
- [ ] **Availability** — Firm can commence audit within 2 weeks of engagement and complete within agreed timeline
- [ ] **Insurance** — Professional liability insurance covering at least $5M
- [ ] **Methodology** — Published audit methodology that includes: manual code review, automated static analysis, formal verification (where applicable), economic attack modeling

#### Preferred Audit Firms (Pre-Qualified)

| Firm | CosmWasm Audits | Cosmos Native Module Experience | Status |
|------|----------------|--------------------------------|--------|
| Oak Security | 15+ | Yes | Pre-qualified |
| Halborn | 10+ | Yes | Pre-qualified |
| SCV Security | 8+ | Yes | Pre-qualified |
| Informal Systems | 5+ | Yes (Cosmos SDK core) | Pre-qualified |

#### Engagement Structure

- **Primary Audit** — Full-scope audit by one firm
- **Secondary Review** — Focused review of critical paths by a second firm
- **Recommendation** — Engage primary + secondary for maximum coverage

### CosmWasm Contract Audit Scope (M008-M011)

#### M008: Fee Router Contract

- [ ] **Entry Points** — All execute, query, and migrate entry points reviewed
- [ ] **Fund Flow** — Fee collection, splitting logic, and distribution paths verified
- [ ] **Access Control** — Admin/governance-only functions properly gated
- [ ] **Overflow/Underflow** — All arithmetic operations checked for overflow/underflow
- [ ] **Reentrancy** — No reentrancy vulnerabilities in fund-handling paths
- [ ] **Storage** — State management reviewed for consistency and migration safety
- [ ] **Gas Optimization** — No unbounded loops or excessive gas consumption patterns
- [ ] **Integration** — Cross-contract calls to M009, M010 verified for correctness

#### M009: Dynamic Fee Adjustment Contract

- [ ] **Oracle Integration** — Fee parameter update logic reviewed for manipulation resistance
- [ ] **Bounds Checking** — Fee parameters constrained within governance-approved ranges
- [ ] **Update Frequency** — Rate limiting on fee adjustments verified
- [ ] **Fallback Behavior** — Graceful degradation when oracle data is unavailable
- [ ] **Economic Attack Vectors** — Fee manipulation, front-running, sandwich attacks analyzed
- [ ] **Parameter Validation** — All governance-submitted parameters validated on-chain
- [ ] **Historical State** — Fee adjustment history stored correctly for auditability

#### M010: Treasury Management Contract

- [ ] **Multi-sig/Governance Control** — Treasury operations require proper authorization
- [ ] **Spending Limits** — Per-transaction and per-epoch spending caps enforced
- [ ] **Asset Accounting** — Balance tracking matches actual contract holdings
- [ ] **Withdrawal Logic** — Withdrawal paths verified for correctness and authorization
- [ ] **Emergency Drain** — Emergency withdrawal mechanism exists and is properly gated
- [ ] **Investment Strategy** — Any yield-generating integrations reviewed for risk
- [ ] **Reporting** — On-chain treasury reports generate accurate data

#### M011: Staking Derivatives Contract

- [ ] **Mint/Burn Logic** — Derivative token minting and burning mathematically verified
- [ ] **Exchange Rate** — Staking derivative exchange rate calculation is manipulation-resistant
- [ ] **Slashing Handling** — Derivative value correctly adjusts on validator slashing events
- [ ] **Unbonding Queue** — Unbonding period management is correct and gas-efficient
- [ ] **Validator Selection** — Delegation strategy cannot be exploited to concentrate stake
- [ ] **Reward Distribution** — Staking rewards correctly attributed to derivative holders
- [ ] **Liquidity** — Redemption mechanism functions under high-demand scenarios
- [ ] **Migration** — Contract migration preserves all user positions and balances

### Native Module Audit Scope

#### x/feerouter (M013: Fee Distribution)

- [ ] **Message Handlers** — All Msg types reviewed for authorization and correctness
- [ ] **Keeper Logic** — State transitions are atomic and consistent
- [ ] **Fee Splitting** — Distribution percentages sum to 100% across all recipients
- [ ] **Begin/EndBlock** — Block lifecycle hooks execute correctly under all conditions
- [ ] **Genesis Import/Export** — Genesis state round-trips correctly
- [ ] **Parameter Changes** — Governance parameter updates apply atomically
- [ ] **Event Emission** — All state changes emit queryable events
- [ ] **Integration with x/auth, x/bank** — Cross-module interactions are correct
- [ ] **Upgrade Compatibility** — Module state migration is backward-compatible

#### x/supply (M012: Supply Dampening)

- [ ] **Burn Mechanism** — Token burn operations are irreversible and correctly reflected in total supply
- [ ] **Mint Controls** — Minting is strictly gated by governance and supply schedule
- [ ] **Cap Enforcement** — Supply cap is enforced at the protocol level, not just application level
- [ ] **Dampening Algorithm** — Mathematical correctness of supply adjustment curve verified
- [ ] **Edge Cases** — Behavior at supply boundaries (zero, cap, overflow) is correct
- [ ] **Interaction with x/staking** — Staked supply correctly excluded/included per specification
- [ ] **Query Accuracy** — Supply queries return accurate, real-time data

#### x/authority (M014: Authority & Governance)

- [ ] **Permission System** — Role-based access control is correctly implemented
- [ ] **Proposal Lifecycle** — Proposal creation, voting, execution, and expiry all correct
- [ ] **Vote Tallying** — Voting power calculation matches specification
- [ ] **Execution Safety** — Approved proposals execute atomically or roll back completely
- [ ] **Emergency Powers** — Emergency governance actions are properly scoped and logged
- [ ] **Delegation** — Authority delegation and revocation work correctly
- [ ] **Timelock** — Governance timelocks are enforced and cannot be bypassed

#### x/rewards (M015: Reward Distribution)

- [ ] **Distribution Algorithm** — Reward calculation matches specification exactly
- [ ] **Epoch Handling** — Epoch boundaries are processed correctly, no rewards lost or duplicated
- [ ] **Claim Mechanism** — Users can claim rewards correctly, no double-claiming possible
- [ ] **Vesting Integration** — Vesting schedules applied correctly to reward distributions
- [ ] **Source Verification** — Reward funding sources verified (fees, inflation, treasury)
- [ ] **Accounting Integrity** — Sum of all distributed rewards equals total reward pool
- [ ] **Gas Efficiency** — Distribution does not exceed block gas limits even with maximum participants

### Internal Review Requirements

Before external audit engagement, ALL modules must pass internal review:

- [ ] **Code Review** — Every file reviewed by at least 2 core team members (not the author)
- [ ] **Test Coverage** — Minimum 90% line coverage, 80% branch coverage for all modules
- [ ] **Integration Tests** — End-to-end tests covering all cross-module interactions
- [ ] **Fuzzing** — Minimum 72 hours of fuzzing with no crashes on all public entry points
- [ ] **Static Analysis** — Clippy (Rust), gosec (Go) pass with zero warnings
- [ ] **Formal Specification** — Critical invariants documented in TLA+ or equivalent
- [ ] **Documentation** — All public APIs documented with examples

### Audit Timeline

| Phase | Duration | Description |
|-------|----------|-------------|
| Pre-Audit Prep | 2 weeks | Internal review completion, documentation, test harness |
| Audit Firm Onboarding | 1 week | Codebase walkthrough, architecture review, scope agreement |
| Primary Audit | 4-6 weeks | Full-scope audit of all contracts and modules |
| Remediation | 2 weeks | Address all findings, re-test |
| Re-Audit | 1-2 weeks | Verify fixes, no new issues introduced |
| Secondary Review | 2-3 weeks | Focused review of critical paths by second firm |
| Final Report | 1 week | Consolidated findings, risk assessment, sign-off |
| **Total** | **12-17 weeks** | **From prep start to final sign-off** |

### Acceptance Criteria

The audit is considered PASSED when ALL of the following are true:

- [ ] **Zero Critical Findings** — No unresolved critical-severity findings
- [ ] **Zero High Findings** — No unresolved high-severity findings
- [ ] **Medium Findings Addressed** — All medium-severity findings either resolved or accepted with documented rationale
- [ ] **Low Findings Tracked** — All low-severity findings tracked in issue tracker with resolution plan
- [ ] **Informational Reviewed** — All informational findings reviewed and documented
- [ ] **Audit Report Published** — Full audit report published publicly (redacting only active exploit details during fix window)
- [ ] **Re-Audit Clean** — Re-audit confirms all critical/high fixes are correct and introduce no new issues
- [ ] **Team Sign-Off** — Both audit firm and core team sign off on readiness

### Bug Bounty Activation

A bug bounty program must be active before mainnet launch:

- [ ] **Platform Selected** — Bug bounty hosted on Immunefi, HackerOne, or equivalent
- [ ] **Scope Defined** — All mainnet contracts and modules in scope
- [ ] **Reward Tiers Published** — Clear reward structure:
  - Critical (funds at risk): $50,000 - $250,000
  - High (protocol disruption): $10,000 - $50,000
  - Medium (limited impact): $2,000 - $10,000
  - Low (informational): $500 - $2,000
- [ ] **Funding Secured** — Bug bounty reward pool funded for minimum 12 months
- [ ] **Response SLA Published** — Triage within 24h, initial response within 48h, resolution within 7 days (critical) / 30 days (other)
- [ ] **Safe Harbor Policy** — Responsible disclosure safe harbor terms published
- [ ] **Pre-Launch Window** — Bug bounty active minimum 2 weeks before mainnet launch for early detection
- [ ] **Escalation Path** — Clear escalation from bug bounty to emergency governance if needed

---

## Governance Approval Sequence

The following governance proposals must be submitted and approved **in dependency order**.
Each proposal has prerequisites that must be satisfied before submission. Submitting
out of order risks inconsistent chain state or failed upgrades.

### Proposal Dependency Graph

```
M013 Upgrade ─────────────────┐
                               ▼
M013 Config ──────────────────┐│
                               ▼▼
M014 + M012 Upgrade ─────────┐
                               ▼
Seed Validator Set ───────────┐
                               ▼
M012 Activation ──────────────┐
                               ▼
M015 Activation ──────────────┐
                               ▼
M015 Tracking Completion ─────┐
                               ▼
PoS Inflation Disable ────────┘
```

### Proposal 1: M013 Chain Upgrade

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Upgrade Executed**

| Field | Value |
|-------|-------|
| **Type** | Software Upgrade Proposal |
| **Dependency** | Audit complete for x/feerouter |
| **Timeline Estimate** | 7-day voting + 3-day upgrade window |
| **Risk Level** | HIGH — Chain halt if upgrade fails |
| **Description** | Binary upgrade introducing the x/feerouter module to the chain. This is the foundation for all fee-based mechanisms. |
| **Rollback** | Binary downgrade to previous version; x/feerouter state is empty pre-activation |
| **Verification** | Module registered in app, genesis export includes x/feerouter section |

### Proposal 2: M013 Fee Distribution Configuration

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Configuration Applied**

| Field | Value |
|-------|-------|
| **Type** | Parameter Change Proposal |
| **Dependency** | Proposal 1 executed successfully |
| **Timeline Estimate** | 7-day voting + immediate application |
| **Risk Level** | MEDIUM — Incorrect parameters cause fee misrouting |
| **Description** | Sets initial fee distribution parameters: split ratios between treasury, stakers, validators, and community pool. |
| **Rollback** | Parameter change proposal to revert to defaults |
| **Verification** | Query x/feerouter params, confirm values match proposal |

**Configuration Parameters:**

| Parameter | Initial Value | Range |
|-----------|--------------|-------|
| treasury_split | 30% | 10-50% |
| staker_split | 40% | 20-60% |
| validator_split | 20% | 10-40% |
| community_pool_split | 10% | 5-20% |
| min_fee | 0.001 REGEN | >0 |
| fee_adjustment_epoch | 100 blocks | 50-500 |

### Proposal 3: M014 + M012 Chain Upgrade

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Upgrade Executed**

| Field | Value |
|-------|-------|
| **Type** | Software Upgrade Proposal |
| **Dependency** | Proposal 2 applied; Audit complete for x/authority and x/supply |
| **Timeline Estimate** | 7-day voting + 3-day upgrade window |
| **Risk Level** | HIGH — Dual-module upgrade increases failure surface |
| **Description** | Binary upgrade introducing x/authority (governance) and x/supply (dampening) modules simultaneously. Bundled because x/supply requires x/authority for its governance controls. |
| **Rollback** | Binary downgrade; both modules have empty state pre-activation |
| **Verification** | Both modules registered, genesis export includes both sections, x/authority can create proposals |

### Proposal 4: Seed Validator Set Approval

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Validator Set Activated**

| Field | Value |
|-------|-------|
| **Type** | Text / Signaling Proposal |
| **Dependency** | Proposal 3 executed; Validator onboarding thresholds met (see [Validator Onboarding Thresholds](#validator-onboarding-thresholds)) |
| **Timeline Estimate** | 7-day voting |
| **Risk Level** | LOW — Signaling only; actual validator set determined by stake |
| **Description** | Community approval of the initial validator set composition meeting the 5/5/5 diversity requirement. Documents the verified validators and their infrastructure. |
| **Rollback** | N/A — Signaling proposal |
| **Verification** | Proposal passed; validator set on testnet matches approved list |

### Proposal 5: M012 Supply Dampening Activation

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Dampening Active**

| Field | Value |
|-------|-------|
| **Type** | Parameter Change Proposal |
| **Dependency** | Proposal 4 passed; Testnet dampening running 30+ days without issues |
| **Timeline Estimate** | 7-day voting + 1 epoch activation delay |
| **Risk Level** | HIGH — Incorrect dampening parameters can destabilize supply |
| **Description** | Activates the supply dampening mechanism with initial parameters. Enables automatic supply adjustment based on economic indicators. |
| **Rollback** | Parameter change to set dampening_enabled = false |
| **Verification** | x/supply params show dampening active; first dampening event observed within expected timeframe |

**Activation Parameters:**

| Parameter | Initial Value | Range |
|-----------|--------------|-------|
| dampening_enabled | true | bool |
| dampening_rate | 0.5% per epoch | 0.1-2.0% |
| supply_floor | 80% of genesis | 50-95% |
| supply_ceiling | 120% of genesis | 105-200% |
| adjustment_period | 1 epoch (1 day) | 1-7 epochs |
| burn_threshold | 110% of target | 105-150% |

### Proposal 6: M015 Reward Distribution Activation

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Rewards Flowing**

| Field | Value |
|-------|-------|
| **Type** | Parameter Change Proposal |
| **Dependency** | Proposal 5 passed; M013 fee revenue confirmed flowing for 7+ days |
| **Timeline Estimate** | 7-day voting + 1 epoch activation delay |
| **Risk Level** | HIGH — Reward distribution errors directly affect token holder economics |
| **Description** | Activates the reward distribution system. Rewards begin flowing from fee revenue and other sources to eligible participants. |
| **Rollback** | Parameter change to set rewards_enabled = false; unclaimed rewards remain claimable |
| **Verification** | First reward epoch completes; reward amounts match expected calculations; users can claim |

### Proposal 7: M015 Tracking Configuration Completion

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Tracking Confirmed**

| Field | Value |
|-------|-------|
| **Type** | Parameter Change Proposal |
| **Dependency** | Proposal 6 passed; Reward distribution running 14+ days; No accounting discrepancies |
| **Timeline Estimate** | 7-day voting + immediate application |
| **Risk Level** | MEDIUM — Tracking misconfiguration causes reporting errors |
| **Description** | Finalizes reward tracking configuration including: epoch reporting, participant categorization, vesting schedule integration, and cross-module accounting reconciliation. |
| **Rollback** | Parameter change to revert tracking configuration |
| **Verification** | Tracking reports generated correctly for 3 consecutive epochs; cross-module balances reconcile to zero delta |

### Proposal 8: PoS Inflation Schedule Modification

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Inflation Modified**

| Field | Value |
|-------|-------|
| **Type** | Parameter Change Proposal |
| **Dependency** | Proposal 7 passed; Fee revenue sufficient to sustain validator economics without full inflation (demonstrated over 30+ days) |
| **Timeline Estimate** | 14-day voting (extended due to economic significance) |
| **Risk Level** | CRITICAL — Premature inflation reduction can cause validator exodus |
| **Description** | Begins the transition from PoS inflation to fee-based validator compensation. Reduces inflation rate according to the phased schedule while fee revenue fills the gap. |
| **Rollback** | Parameter change to restore previous inflation rate |
| **Verification** | Inflation rate matches new target; validator reward total (inflation + fees) remains within 10% of pre-change levels |

**Inflation Reduction Schedule:**

| Phase | Inflation Rate | Fee Revenue Target | Duration |
|-------|---------------|-------------------|----------|
| Current | 10% | N/A | — |
| Phase 1 | 7% | 30% of validator rewards from fees | 90 days |
| Phase 2 | 4% | 60% of validator rewards from fees | 90 days |
| Phase 3 | 2% | 80% of validator rewards from fees | 90 days |
| Phase 4 | 0% | 100% of validator rewards from fees | Permanent |

### Proposal 9: Emergency Governance Framework Ratification

- [ ] **Proposal Submitted**
- [ ] **Proposal Passed**
- [ ] **Framework Active**

| Field | Value |
|-------|-------|
| **Type** | Text / Signaling Proposal |
| **Dependency** | Proposal 3 executed (x/authority available) |
| **Timeline Estimate** | 7-day voting |
| **Risk Level** | LOW — Framework proposal, no on-chain execution |
| **Description** | Ratifies the emergency governance procedures including: rapid-response proposal type (24h voting), emergency validator communication channels, and rollback trigger escalation paths. |
| **Rollback** | N/A — Signaling proposal |
| **Verification** | Proposal passed; emergency procedures documented and communicated to all validators |

### Governance Timeline Summary

| Week | Proposals | Cumulative Status |
|------|-----------|-------------------|
| 1-2 | P1 (M013 Upgrade) voting | M013 binary deployed |
| 3 | P1 executed, P2 (M013 Config) submitted | Fee router active |
| 4-5 | P2 voting + applied, P3 (M014+M012) submitted | Fee distribution configured |
| 6-7 | P3 voting, P9 (Emergency Framework) submitted | Authority + Supply modules staged |
| 8 | P3 executed, P4 (Validator Set) submitted, P9 voting | Dual modules active |
| 9-10 | P4 voting, P9 passed | Validator set approved |
| 10-11 | P5 (M012 Activation) submitted + voting | Supply dampening staged |
| 12 | P5 passed + activated, fee revenue monitored | Dampening active |
| 13-14 | P6 (M015 Activation) submitted + voting | Rewards staged |
| 15 | P6 passed + activated | Rewards flowing |
| 17-18 | P7 (Tracking) submitted + voting | Tracking finalized |
| 19+ | P8 (Inflation) submitted after 30-day fee revenue verification | Inflation transition begins |

**Total Governance Timeline: ~19-22 weeks from first proposal submission**

---

## Validator Onboarding Thresholds

The network requires a minimum viable validator set before mechanisms are activated
on mainnet. These thresholds ensure decentralization, reliability, and governance
participation from day one.

### Minimum Validator Count

- [ ] **15+ Active Validators** — Minimum 15 validators in the active set with non-trivial stake
- [ ] **20+ Total Validators** — At least 20 validators registered (including those outside the active set)
- [ ] **No Single Validator >10% Voting Power** — Nakamoto coefficient of at least 4

### 5/5/5 Composition Requirement

The active validator set must include diversity across three dimensions:

#### Geographic Distribution (5+ Regions)

- [ ] **Region 1** — North America (at least 3 validators)
- [ ] **Region 2** — Europe (at least 3 validators)
- [ ] **Region 3** — Asia-Pacific (at least 2 validators)
- [ ] **Region 4** — South America (at least 1 validator)
- [ ] **Region 5** — Africa/Middle East (at least 1 validator)
- [ ] **No single region >40% of validators**

#### Infrastructure Provider Distribution (5+ Providers)

- [ ] **Provider 1** — AWS (maximum 4 validators)
- [ ] **Provider 2** — GCP (maximum 4 validators)
- [ ] **Provider 3** — Azure (maximum 3 validators)
- [ ] **Provider 4** — Hetzner/OVH/other EU providers (minimum 2 validators)
- [ ] **Provider 5** — Bare metal / self-hosted (minimum 2 validators)
- [ ] **No single provider >33% of validators**

#### Organization Distribution (5+ Organizations)

- [ ] **5+ Independent Organizations** — At least 5 distinct legal entities operating validators
- [ ] **No Single Organization >3 Validators** — Prevents organizational centralization
- [ ] **At Least 2 Professional Validator Companies** — Operators with 3+ other network deployments
- [ ] **At Least 2 Community/Individual Validators** — Non-corporate operators
- [ ] **At Least 1 Institutional Validator** — Foundation, university, or research organization

### Uptime Requirements

- [ ] **99.5% Uptime Over 30+ Days on Testnet** — Each validator must demonstrate 99.5% uptime over a minimum 30-day continuous period on the staging testnet
- [ ] **Uptime Measurement** — Measured by missed blocks; maximum 216 missed blocks per 30 days (43,200 blocks/day * 30 days * 0.5% = 6,480 max missed; at 30-block window = 216 windows)
- [ ] **No Extended Downtime** — No single downtime event exceeding 2 hours
- [ ] **Recovery Demonstration** — Each validator has demonstrated at least 1 clean recovery from simulated failure
- [ ] **Monitoring Active** — Each validator has monitoring dashboards accessible to the core team

### Governance Participation

- [ ] **>80% Governance Participation** — Each validator has voted on at least 80% of testnet governance proposals
- [ ] **Minimum 5 Testnet Proposals** — At least 5 governance proposals have been submitted and voted on during the testnet period
- [ ] **Vote Diversity** — At least 1 proposal has seen a "No" or "Abstain" from at least 3 validators (demonstrates independent decision-making)
- [ ] **Proposal Submission** — At least 3 different validators have submitted governance proposals

### Infrastructure Verification

Each validator must pass an infrastructure checklist:

- [ ] **Hardware Meets Minimum Specs**
  - CPU: 8+ cores (recommended 16)
  - RAM: 32 GB+ (recommended 64 GB)
  - Storage: 1 TB NVMe SSD (recommended 2 TB)
  - Network: 1 Gbps symmetric (recommended 10 Gbps)
- [ ] **Sentry Node Architecture** — At least 1 sentry node in front of the validator
- [ ] **Key Management** — Hardware security module (HSM) or equivalent for validator key
- [ ] **Backup Procedures** — Documented and tested backup/restore procedure
- [ ] **Monitoring Stack** — Prometheus + Grafana (or equivalent) with alerting configured
- [ ] **Upgrade Procedures** — Documented upgrade procedure, tested on testnet
- [ ] **Communication Channel** — Validator operator reachable via designated channel within 15 minutes during business hours, 1 hour outside
- [ ] **Security Practices** — Firewall rules, SSH key-only access, regular OS updates documented

### Validator Readiness Summary Table

| Validator | Org | Region | Provider | Uptime (30d) | Gov Part. | Infra Verified | Status |
|-----------|-----|--------|----------|-------------|-----------|---------------|--------|
| val-01 | — | — | — | —% | —% | [ ] | PENDING |
| val-02 | — | — | — | —% | —% | [ ] | PENDING |
| val-03 | — | — | — | —% | —% | [ ] | PENDING |
| val-04 | — | — | — | —% | —% | [ ] | PENDING |
| val-05 | — | — | — | —% | —% | [ ] | PENDING |
| val-06 | — | — | — | —% | —% | [ ] | PENDING |
| val-07 | — | — | — | —% | —% | [ ] | PENDING |
| val-08 | — | — | — | —% | —% | [ ] | PENDING |
| val-09 | — | — | — | —% | —% | [ ] | PENDING |
| val-10 | — | — | — | —% | —% | [ ] | PENDING |
| val-11 | — | — | — | —% | —% | [ ] | PENDING |
| val-12 | — | — | — | —% | —% | [ ] | PENDING |
| val-13 | — | — | — | —% | —% | [ ] | PENDING |
| val-14 | — | — | — | —% | —% | [ ] | PENDING |
| val-15 | — | — | — | —% | —% | [ ] | PENDING |

---

## Agent Deployment Verification

The KOI (Knowledge, Orchestration, Intelligence) agent framework and its associated
agents must be thoroughly validated on staging before mainnet deployment. This section
defines staging duration, accuracy thresholds, escalation rates, workflow coverage,
audit trail requirements, MCP health, and economic reboot validation.

### Staging Duration Requirement

- [ ] **4 Agents Deployed on Staging for 30+ Days** — All 4 core agents must be continuously operational on the staging environment for a minimum of 30 calendar days before mainnet consideration
- [ ] **No Redeployments in Final 7 Days** — The exact binary/configuration deployed on staging for the last 7 days is what gets promoted to mainnet
- [ ] **Staging Environment Mirrors Mainnet** — Network topology, validator count, transaction volume patterns, and governance activity on staging must approximate expected mainnet conditions

### Core Agent Roster

| Agent | Function | Staging Deploy Date | Days Active | Status |
|-------|----------|-------------------|-------------|--------|
| Fee Optimization Agent | Monitors fee markets, suggests M009 parameter adjustments | — | — | PENDING |
| Supply Monitor Agent | Tracks supply metrics, triggers M012 dampening alerts | — | — | PENDING |
| Governance Assistant Agent | Facilitates proposal creation, voter information, quorum tracking | — | — | PENDING |
| Reward Distribution Agent | Calculates distributions, verifies M015 accounting, flags discrepancies | — | — | PENDING |

### Accuracy Requirements

- [ ] **>90% Decision Accuracy** — Each agent's autonomous decisions must be correct >90% of the time as measured by post-hoc review
- [ ] **Accuracy Measurement Methodology** — Weekly sampling of 100 random decisions per agent, reviewed by 2 independent reviewers
- [ ] **No Critical Errors** — Zero instances of an agent making a decision that would have caused fund loss, supply manipulation, or governance subversion
- [ ] **Accuracy Trend** — Accuracy must be stable or improving over the 30-day period (no declining trend)

#### Per-Agent Accuracy Targets

| Agent | Min Accuracy | Measurement Method |
|-------|-------------|-------------------|
| Fee Optimization | >92% | Fee adjustment suggestions compared to optimal (backtested) |
| Supply Monitor | >95% | Alert correctness (true positive rate) |
| Governance Assistant | >90% | Information accuracy, proposal formatting correctness |
| Reward Distribution | >98% | Calculation accuracy vs. reference implementation |

### Escalation Rate Requirements

- [ ] **<15% Escalation Rate** — Each agent must resolve at least 85% of its assigned tasks without human intervention
- [ ] **Escalation Categories Tracked** — Each escalation categorized as: confidence threshold, novel scenario, system error, or policy boundary
- [ ] **Escalation Trend** — Escalation rate must be stable or declining over the 30-day period
- [ ] **Escalation Response Time** — Human response to escalations within 1 hour during business hours, 4 hours outside

### Workflow Coverage

All 14 defined agent workflows must be tested and verified on staging:

#### Fee Management Workflows (4)

- [ ] **W01: Routine Fee Adjustment** — Agent proposes fee parameter changes based on network utilization
- [ ] **W02: Fee Spike Response** — Agent detects and responds to sudden fee increases
- [ ] **W03: Fee Revenue Reporting** — Agent generates accurate fee revenue reports per epoch
- [ ] **W04: Fee Parameter Governance** — Agent formats fee parameter change proposals for governance

#### Supply Management Workflows (3)

- [ ] **W05: Supply Monitoring** — Agent continuously tracks total supply, circulating supply, and staked supply
- [ ] **W06: Dampening Event Processing** — Agent processes supply dampening events and verifies correctness
- [ ] **W07: Supply Anomaly Detection** — Agent detects unexpected supply changes and alerts operators

#### Governance Workflows (4)

- [ ] **W08: Proposal Drafting Assistance** — Agent helps format governance proposals from natural language input
- [ ] **W09: Voter Information Summary** — Agent generates unbiased proposal summaries for voters
- [ ] **W10: Quorum Tracking** — Agent tracks voting progress and alerts when quorum is near
- [ ] **W11: Proposal Execution Verification** — Agent verifies that passed proposals were correctly executed

#### Reward Workflows (3)

- [ ] **W12: Reward Calculation** — Agent calculates expected rewards per participant per epoch
- [ ] **W13: Distribution Verification** — Agent cross-checks distributed rewards against calculations
- [ ] **W14: Reward Anomaly Detection** — Agent detects distribution anomalies (underpayment, overpayment, missing claims)

### KOI Audit Trail Requirements

- [ ] **Complete Decision Logging** — Every agent decision is logged with: timestamp, input data, reasoning chain, output action, confidence score
- [ ] **Immutable Audit Trail** — Audit logs are append-only and tamper-evident (hash-chained or stored on-chain)
- [ ] **Queryable History** — Audit trail is queryable by: agent, time range, decision type, confidence level, outcome
- [ ] **Retention Policy** — Minimum 1 year retention for all audit data
- [ ] **Access Control** — Audit trail readable by governance participants, writable only by agent framework
- [ ] **Regular Audit Review** — Weekly review of audit trail samples during staging period

### MCP Health Requirements

- [ ] **MCP Server Operational** — Model Context Protocol server running with <1 second response time for 95th percentile
- [ ] **Connection Stability** — MCP connection uptime >99.9% over 30-day staging period
- [ ] **Tool Registration** — All agent tools registered and callable via MCP
- [ ] **Context Window Management** — Agent context windows managed efficiently, no context overflow errors
- [ ] **Rate Limiting** — MCP rate limits configured to prevent agent overload
- [ ] **Health Endpoint** — MCP exposes /health endpoint returning structured status
- [ ] **Graceful Degradation** — Agents degrade gracefully (pause decisions, alert operators) when MCP is unavailable

### Economic Reboot Workflow Validation

The economic reboot workflows handle the transition from pre-launch to live economics:

- [ ] **Genesis Distribution** — Agent correctly processes genesis token distribution
- [ ] **Initial Fee Activation** — Agent handles the first fee collection event correctly
- [ ] **First Supply Dampening** — Agent correctly processes the first dampening event
- [ ] **First Reward Distribution** — Agent correctly calculates and verifies the first reward epoch
- [ ] **Inflation Transition** — Agent correctly handles inflation parameter changes
- [ ] **Emergency Pause** — Agent correctly responds to emergency pause signals
- [ ] **Recovery from Pause** — Agent correctly resumes operations after an emergency pause
- [ ] **Cross-Agent Coordination** — All 4 agents correctly coordinate during multi-step economic events

---

## Rollback Triggers

Each mechanism has specific, measurable conditions that trigger an automatic or manual
rollback. Rollback triggers are designed to catch problems early and limit blast radius.
Every trigger has: a condition, detection method, response action, and notification chain.

### M013 Fee Distribution Rollback Triggers

#### Trigger: Zero Revenue for 48 Hours

- **Condition:** x/feerouter collects zero fees for 48 consecutive hours during a period when the chain is processing >100 transactions per hour
- **Detection:** Automated — Prometheus alert on `feerouter_total_fees_collected` metric
- **Alert Rule:** `feerouter_total_fees_collected increase over 48h == 0 AND chain_tx_count increase over 1h > 100`
- **Response:**
  1. Automated: Page on-call engineer via PagerDuty
  2. Automated: Post alert to #mainnet-ops Slack/Discord channel
  3. Manual (within 1h): Diagnose root cause
  4. Manual (within 4h): Submit emergency parameter change or binary rollback proposal
- **Rollback Action:** Set fee_enabled = false via emergency governance proposal (24h voting)
- **Notification Chain:** On-call engineer -> Core team lead -> Governance council

#### Trigger: Fee Distribution Imbalance

- **Condition:** Actual fee distribution deviates from configured split ratios by more than 5% for 3 consecutive epochs
- **Detection:** Automated — Epoch-end reconciliation check
- **Alert Rule:** `abs(actual_split - configured_split) > 0.05 for 3 consecutive epochs`
- **Response:**
  1. Automated: Alert core team
  2. Manual: Investigate whether the issue is in routing logic or parameter misconfiguration
  3. Manual: Submit corrective parameter change or emergency patch
- **Rollback Action:** Pause fee distribution; accumulated fees held in module account until resolved

### M012 Supply Dampening Rollback Triggers

#### Trigger: Supply Cap Breach

- **Condition:** Total supply exceeds configured supply_ceiling OR falls below supply_floor
- **Detection:** Automated — Block-level supply check in x/supply EndBlocker
- **Alert Rule:** `total_supply > supply_ceiling OR total_supply < supply_floor`
- **Response:**
  1. Automated: Dampening mechanism self-halts (circuit breaker built into module)
  2. Automated: Emergency event emitted, alerting all monitoring systems
  3. Manual (within 30min): Core team assesses whether breach is real or measurement error
  4. Manual (within 2h): Submit governance proposal if module fix is needed
- **Rollback Action:** Set dampening_enabled = false; no further supply adjustments until root cause resolved
- **Notification Chain:** Automated circuit breaker -> On-call engineer -> Core team -> All validators

#### Trigger: Excessive Burn Rate

- **Condition:** Supply decreases by more than 2% in a single epoch
- **Detection:** Automated — Epoch boundary supply delta check
- **Alert Rule:** `(supply_start_epoch - supply_end_epoch) / supply_start_epoch > 0.02`
- **Response:**
  1. Automated: Alert core team with full epoch supply data
  2. Manual: Verify burn was legitimate (not an exploit)
  3. Manual: Adjust dampening_rate parameter if needed
- **Rollback Action:** Reduce dampening_rate to minimum (0.1%) or disable entirely

### M014 Governance/Authority Rollback Triggers

#### Trigger: Active Validators Drop Below 15

- **Condition:** Active validator count falls below 15 for more than 1 hour
- **Detection:** Automated — Validator set monitoring
- **Alert Rule:** `active_validator_count < 15 for 60 consecutive minutes`
- **Response:**
  1. Automated: Alert all validators and core team
  2. Manual (within 30min): Contact offline validators
  3. Manual (within 2h): If validators cannot be restored, assess whether to pause governance operations
- **Rollback Action:** Pause non-emergency governance proposals until validator count recovers
- **Notification Chain:** On-call -> Validator operators -> Core team -> Community announcement

#### Trigger: Governance Deadlock

- **Condition:** 3 consecutive proposals fail to reach quorum
- **Detection:** Manual — Governance participation monitoring
- **Response:**
  1. Assess quorum requirements vs. current participation levels
  2. Community outreach to encourage participation
  3. Consider emergency proposal to adjust quorum requirements
- **Rollback Action:** No automated rollback; requires community-driven resolution

### M015 Reward Distribution Rollback Triggers

#### Trigger: Distribution Exceeds Inflow

- **Condition:** Total rewards distributed in an epoch exceed total reward pool inflow for the same epoch by more than 10%
- **Detection:** Automated — Epoch accounting reconciliation
- **Alert Rule:** `epoch_rewards_distributed > epoch_reward_inflow * 1.10`
- **Response:**
  1. Automated: Pause next epoch reward distribution
  2. Automated: Alert core team with full accounting data
  3. Manual (within 2h): Reconcile discrepancy — identify if issue is over-distribution or under-accounting of inflow
  4. Manual (within 24h): Fix root cause and re-enable distribution
- **Rollback Action:** Set rewards_enabled = false; unclaimed rewards preserved, no new distributions until resolved
- **Notification Chain:** Automated pause -> On-call engineer -> Core team -> Community announcement

#### Trigger: Reward Calculation Drift

- **Condition:** Agent-calculated expected rewards deviate from on-chain distributed rewards by more than 1% for any participant category
- **Detection:** Automated — Agent W13 (Distribution Verification)
- **Alert Rule:** `abs(agent_calculated - onchain_distributed) / agent_calculated > 0.01`
- **Response:**
  1. Automated: Flag discrepancy in audit trail
  2. Manual: Determine which calculation is correct (agent vs. on-chain)
  3. Manual: Fix the incorrect calculation source
- **Rollback Action:** If on-chain is incorrect, pause distributions; if agent is incorrect, update agent

### Agent Rollback Triggers

#### Trigger: Agent Error Rate Exceeds 50% for 1 Hour

- **Condition:** Any single agent's error rate exceeds 50% of actions attempted over a 1-hour sliding window
- **Detection:** Automated — Agent health monitoring dashboard
- **Alert Rule:** `agent_error_count / agent_action_count > 0.50 over 1h window`
- **Response:**
  1. Automated: Agent self-disables (enters safe mode — read-only, no autonomous actions)
  2. Automated: Alert on-call engineer
  3. Manual (within 30min): Diagnose error cause
  4. Manual: Fix and redeploy, or extend safe mode until fix is ready
- **Rollback Action:** Agent enters safe mode; all pending autonomous actions are queued for human review
- **Notification Chain:** Agent safe mode alert -> On-call engineer -> Agent team lead

#### Trigger: MCP Down for 15 Minutes

- **Condition:** MCP health endpoint returns non-200 status for 15 consecutive minutes
- **Detection:** Automated — MCP health check (every 30 seconds)
- **Alert Rule:** `mcp_health_status != 200 for 30 consecutive checks`
- **Response:**
  1. Automated: All agents enter safe mode
  2. Automated: Page on-call engineer via PagerDuty
  3. Manual (within 15min): Diagnose MCP failure
  4. Manual: Restart MCP or failover to backup instance
- **Rollback Action:** All agents in safe mode until MCP is confirmed healthy for 5 consecutive minutes
- **Recovery:** Agents automatically exit safe mode when MCP health is confirmed

### Rollback Trigger Summary Matrix

| Trigger | Mechanism | Detection | Auto Response | Manual SLA | Severity |
|---------|-----------|-----------|---------------|------------|----------|
| Zero revenue 48h | M013 | Automated | Page on-call | 1h diagnose, 4h action | HIGH |
| Fee imbalance >5% | M013 | Automated | Alert | 4h investigate | MEDIUM |
| Supply cap breach | M012 | Automated | Circuit breaker halt | 30min assess | CRITICAL |
| Excessive burn >2% | M012 | Automated | Alert | 2h assess | HIGH |
| Validators <15 | M014 | Automated | Alert | 30min contact, 2h assess | HIGH |
| Governance deadlock | M014 | Manual | None | Community-driven | MEDIUM |
| Distributions > inflow | M015 | Automated | Pause distributions | 2h reconcile | CRITICAL |
| Reward drift >1% | M015 | Automated | Flag in audit | 4h investigate | MEDIUM |
| Agent errors >50% 1h | Agents | Automated | Agent safe mode | 30min diagnose | HIGH |
| MCP down 15min | Agents | Automated | All agents safe mode | 15min diagnose | CRITICAL |

---

## Go/No-Go Framework

### Decision Authority

The go/no-go decision is made by the **Launch Decision Committee** consisting of:

| Role | Responsibility | Veto Power |
|------|---------------|------------|
| **Core Team Lead** | Overall launch coordination, final decision if no vetoes | Yes |
| **Security Lead** | Audit completion, vulnerability assessment | Yes (security gates) |
| **Infrastructure Lead** | Validator readiness, network stability | Yes (infrastructure gates) |
| **Agent Team Lead** | Agent deployment verification, KOI readiness | Yes (agent gates) |
| **Governance Council Rep** | Community readiness, governance preparation | No (advisory) |
| **External Auditor Rep** | Independent security assessment | Yes (audit gates) |

**Decision Process:**

1. All gate owners submit gate status reports 48 hours before scheduled go/no-go meeting
2. Launch Decision Committee reviews all gates in a synchronous meeting
3. Any member with veto power can block launch by identifying an unsatisfied HARD BLOCKER
4. CONDITIONAL gates can be waived by unanimous committee agreement with documented rationale
5. Decision is recorded and published to the community within 2 hours
6. If NO-GO: specific remediation requirements and re-evaluation date are documented

### Gate Criteria

#### Gate 1: Security — HARD BLOCKER

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 1.1 | Primary audit complete with zero critical/high findings | [ ] | — |
| 1.2 | Secondary review complete with no new critical findings | [ ] | — |
| 1.3 | All medium findings addressed or accepted with rationale | [ ] | — |
| 1.4 | Bug bounty program active for 14+ days | [ ] | — |
| 1.5 | No active bug bounty reports of medium or higher severity | [ ] | — |
| 1.6 | Audit report published publicly | [ ] | — |
| 1.7 | Internal security review complete for all modules | [ ] | — |

**Gate Owner:** Security Lead
**Veto Holder:** Security Lead, External Auditor Rep

#### Gate 2: Testing — HARD BLOCKER

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 2.1 | All P0 (critical path) tests passing on mainnet-candidate binary | [ ] | — |
| 2.2 | All P1 (important) tests passing | [ ] | — |
| 2.3 | Testnet running stable for 30+ consecutive days | [ ] | — |
| 2.4 | Full upgrade simulation successful on testnet (matching exact mainnet upgrade procedure) | [ ] | — |
| 2.5 | Load testing complete — network handles 2x expected peak TPS | [ ] | — |
| 2.6 | Chaos testing complete — network recovers from simulated node failures, network partitions | [ ] | — |
| 2.7 | Genesis file validated and reproducible from documented procedure | [ ] | — |
| 2.8 | All migration paths tested (state migration from current to new version) | [ ] | — |

**Gate Owner:** Infrastructure Lead
**Veto Holder:** Infrastructure Lead

#### Gate 3: Governance — HARD BLOCKER

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 3.1 | All prerequisite governance proposals passed (Proposals 1-4 minimum) | [ ] | — |
| 3.2 | Emergency governance framework ratified (Proposal 9) | [ ] | — |
| 3.3 | Governance participation >80% on testnet proposals | [ ] | — |
| 3.4 | At least 3 validators have submitted testnet proposals (demonstrates capability) | [ ] | — |
| 3.5 | Governance UI tested and functional for all proposal types | [ ] | — |

**Gate Owner:** Governance Council Rep
**Veto Holder:** Core Team Lead (on behalf of governance)

#### Gate 4: Infrastructure — HARD BLOCKER

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 4.1 | 15+ validators meeting all thresholds (see [Validator Onboarding Thresholds](#validator-onboarding-thresholds)) | [ ] | — |
| 4.2 | 5/5/5 composition requirement met | [ ] | — |
| 4.3 | All validators passed infrastructure verification | [ ] | — |
| 4.4 | Monitoring stack operational for all validators | [ ] | — |
| 4.5 | Communication channels tested (all validators reachable) | [ ] | — |
| 4.6 | Seed nodes and persistent peers configured and tested | [ ] | — |
| 4.7 | State sync / snapshot infrastructure operational | [ ] | — |
| 4.8 | Block explorer and public RPC endpoints operational | [ ] | — |

**Gate Owner:** Infrastructure Lead
**Veto Holder:** Infrastructure Lead

#### Gate 5: Agent Deployment — CONDITIONAL

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 5.1 | 4 agents deployed on staging 30+ days | [ ] | — |
| 5.2 | All agents >90% accuracy | [ ] | — |
| 5.3 | All agents <15% escalation rate | [ ] | — |
| 5.4 | All 14 workflows tested and verified | [ ] | — |
| 5.5 | KOI audit trail operational and queryable | [ ] | — |
| 5.6 | MCP healthy (>99.9% uptime over 30 days) | [ ] | — |
| 5.7 | Economic reboot workflows validated | [ ] | — |

**Gate Owner:** Agent Team Lead
**Veto Holder:** Agent Team Lead
**Conditional Waiver:** Network can launch without agents if all other gates pass; agents deploy post-launch with reduced initial scope

#### Gate 6: Community Readiness — CONDITIONAL

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 6.1 | Validator onboarding guide published | [ ] | — |
| 6.2 | User documentation published (staking, governance, fees) | [ ] | — |
| 6.3 | Agent interaction documentation published | [ ] | — |
| 6.4 | FAQ and troubleshooting guide published | [ ] | — |
| 6.5 | Community announcement made with 14+ days notice | [ ] | — |
| 6.6 | Support channels staffed (Discord, Telegram, Forum) | [ ] | — |

**Gate Owner:** Governance Council Rep
**Conditional Waiver:** Minimum viable documentation (6.1 + 6.2) required; remaining can follow within 1 week of launch

#### Gate 7: Rollback Readiness — HARD BLOCKER

| # | Criterion | Status | Evidence |
|---|-----------|--------|----------|
| 7.1 | All automated rollback triggers configured and tested | [ ] | — |
| 7.2 | Manual rollback procedures documented for each mechanism | [ ] | — |
| 7.3 | Rollback drill completed on testnet (simulated failure + recovery) | [ ] | — |
| 7.4 | Emergency governance proposal template prepared | [ ] | — |
| 7.5 | On-call rotation established for first 30 days | [ ] | — |
| 7.6 | PagerDuty/alerting integration tested end-to-end | [ ] | — |
| 7.7 | Communication templates prepared for incident response | [ ] | — |

**Gate Owner:** Core Team Lead
**Veto Holder:** Core Team Lead

### Hard Blockers vs. Conditional Gates

| Gate | Type | Waiver Process |
|------|------|---------------|
| Gate 1: Security | HARD BLOCKER | Cannot be waived under any circumstances |
| Gate 2: Testing | HARD BLOCKER | Cannot be waived under any circumstances |
| Gate 3: Governance | HARD BLOCKER | Cannot be waived under any circumstances |
| Gate 4: Infrastructure | HARD BLOCKER | Cannot be waived under any circumstances |
| Gate 5: Agent Deployment | CONDITIONAL | Unanimous committee agreement + documented risk + fallback plan |
| Gate 6: Community Readiness | CONDITIONAL | Minimum viable subset required; full set within 1 week |
| Gate 7: Rollback Readiness | HARD BLOCKER | Cannot be waived under any circumstances |

---

## Launch Day Runbook

This section provides a detailed timeline of activities from T-24h (24 hours before
the upgrade height) through T+30d (30 days post-launch). All times are relative to
the scheduled upgrade block height.

### T-24h: Pre-Flight

| Time | Action | Owner | Verification |
|------|--------|-------|-------------|
| T-24h | Final go/no-go meeting — committee confirms all gates | Core Team Lead | Meeting minutes published |
| T-24h | Announce confirmed launch time to community | Governance Council Rep | Blog post + social media + Discord/Telegram |
| T-23h | All validators confirm readiness via designated channel | Infrastructure Lead | 15+ validators confirmed |
| T-22h | Distribute signed mainnet binary hash to all validators | Security Lead | Hash published in multiple channels |
| T-20h | Validators begin downloading and verifying binary | Validators | Binary hash matches published hash |
| T-18h | Final testnet upgrade rehearsal (if not done in prior 48h) | Infrastructure Lead | Testnet upgrade successful |
| T-16h | Freeze all non-critical code changes | Core Team Lead | Code freeze announced |
| T-12h | On-call rotation begins for launch period | Core Team Lead | On-call engineer confirmed and reachable |
| T-8h | Pre-stage binary on all validator nodes (do not activate) | Validators | Binary present, cosmovisor configured |
| T-6h | Final monitoring dashboard review — all metrics baselined | Infrastructure Lead | Dashboard screenshots archived |
| T-4h | Communication channels go to "launch mode" — dedicated thread/channel for real-time coordination | Core Team Lead | Channel created, all validators joined |
| T-2h | Final validator roll call — confirm all 15+ ready | Infrastructure Lead | Roll call complete |
| T-1h | Silence non-critical alerts to reduce noise | Infrastructure Lead | Alert policies updated |
| T-30min | Pre-upgrade state export (backup) initiated on archive node | Infrastructure Lead | Export started |
| T-15min | Final coordination message: "Upgrade height approaching, all systems ready" | Core Team Lead | Message sent |

### T-0: Upgrade Execution

| Time | Action | Owner | Verification |
|------|--------|-------|-------------|
| T-0 | Chain halts at upgrade height | Automatic | All nodes report halt at correct height |
| T+0-5min | Cosmovisor automatically switches to new binary (or manual switch) | Validators | New binary running on all nodes |
| T+5-15min | Validators restart and begin producing blocks | Validators | Block production resumes |
| T+15min | Confirm 2/3+ voting power online and producing blocks | Infrastructure Lead | Block explorer shows new blocks |
| T+20min | Verify new modules registered in app state | Core Team Lead | Query shows x/feerouter (or applicable modules) registered |
| T+30min | Run post-upgrade verification script | Infrastructure Lead | Script passes all checks (see below) |
| T+45min | Announce successful upgrade to community | Governance Council Rep | Blog post + social media |
| T+1h | Resume non-critical alert policies | Infrastructure Lead | Alert policies restored |

#### Post-Upgrade Verification Script Checks

The automated verification script must confirm all of the following:

```
[ ] Chain is producing blocks at expected rate (within 10% of pre-upgrade)
[ ] All new modules are registered and queryable
[ ] Genesis state for new modules is correct
[ ] Existing modules (x/bank, x/staking, x/gov, etc.) functioning normally
[ ] Fee collection is operational (if applicable at this stage)
[ ] Governance module can accept new proposals
[ ] Validator set matches expected composition
[ ] No error logs in any validator nodes (beyond expected upgrade messages)
[ ] Block explorer is indexing new blocks
[ ] Public RPC endpoints are responsive
[ ] State sync is functional for new joiners
```

### T+1h to T+24h: First Day Review

| Time | Action | Owner | Verification |
|------|--------|-------|-------------|
| T+1h | Deploy agents to mainnet (if Gate 5 passed) | Agent Team Lead | Agents operational, first health check passes |
| T+2h | First fee collection event (if M013 active) | Automated | Fee collected and routed correctly |
| T+3h | Agent audit trail generating entries | Agent Team Lead | Audit entries visible in dashboard |
| T+4h | First comprehensive monitoring report | Infrastructure Lead | Report covers: blocks, validators, fees, agents |
| T+6h | Midday check-in — all teams report status | Core Team Lead | Status compiled, any issues tracked |
| T+8h | Review any community-reported issues | Governance Council Rep | Issue tracker updated |
| T+12h | Shift change for on-call rotation | Core Team Lead | New on-call confirmed |
| T+18h | Overnight monitoring report | On-call engineer | No critical alerts overnight |
| T+24h | **First day review meeting** — all teams | Core Team Lead | Day 1 report published |

#### First Day Review Meeting Agenda

1. Block production metrics (rate, missed blocks, finality)
2. Validator performance (uptime, missed blocks per validator)
3. Fee collection report (total fees, distribution accuracy)
4. Agent performance summary (accuracy, escalations, errors)
5. Community feedback summary
6. Issue tracker review
7. Decision: continue as planned or invoke any rollback triggers?

### T+1d to T+7d: First Week

| Day | Action | Owner | Verification |
|-----|--------|-------|-------------|
| Day 2 | Daily check-in (30 min, all teams) | Core Team Lead | Check-in notes published |
| Day 2 | Submit M013 config proposal (Proposal 2) if not done pre-launch | Governance Council Rep | Proposal on-chain |
| Day 3 | Daily check-in | Core Team Lead | Check-in notes published |
| Day 3 | First agent accuracy report (72-hour data) | Agent Team Lead | Report published |
| Day 4 | Daily check-in | Core Team Lead | Check-in notes published |
| Day 4 | Community Q&A session | Governance Council Rep | Session held, recording published |
| Day 5 | Daily check-in | Core Team Lead | Check-in notes published |
| Day 5 | First weekly monitoring report | Infrastructure Lead | Report covers full week of metrics |
| Day 6 | Daily check-in | Core Team Lead | Check-in notes published |
| Day 7 | **First week review meeting** — all teams + community | Core Team Lead | Week 1 report published |

#### First Week Review Meeting Agenda

1. Week 1 metrics summary (blocks, validators, fees, agents)
2. Governance proposal status (any proposals submitted or passed)
3. Agent performance 7-day summary
4. Rollback trigger status (any triggers activated? near-misses?)
5. Community sentiment assessment
6. Issue tracker review and prioritization
7. Decision: proceed to next governance proposals or pause?

### T+7d to T+30d: First Month

| Period | Action | Owner | Verification |
|--------|--------|-------|-------------|
| Week 2 | Weekly review meeting | Core Team Lead | Report published |
| Week 2 | Submit next governance proposal per sequence | Governance Council Rep | Proposal on-chain |
| Week 2 | Agent redeployment if needed (based on first week data) | Agent Team Lead | Agents updated, no downtime |
| Week 2 | Reduce on-call from 24/7 to business hours + 4h response | Core Team Lead | Rotation updated |
| Week 3 | Weekly review meeting | Core Team Lead | Report published |
| Week 3 | First monthly validator performance report | Infrastructure Lead | Report distributed to all validators |
| Week 3 | Community governance workshop | Governance Council Rep | Workshop held |
| Week 4 | Weekly review meeting | Core Team Lead | Report published |
| Week 4 | Assess readiness for M012 activation (Proposal 5) | Core Team Lead | Assessment documented |
| Day 30 | **First month review meeting** — comprehensive assessment | Core Team Lead | Month 1 report published |

#### First Month Review Meeting Agenda

1. Month 1 comprehensive metrics (blocks, validators, fees, supply, agents)
2. Governance activity summary (proposals submitted, passed, failed)
3. Validator performance rankings and any issues
4. Agent performance 30-day summary (accuracy, escalation, availability)
5. Fee revenue analysis (actual vs. projected)
6. Supply metrics (if M012 activated)
7. Community growth and engagement metrics
8. Rollback trigger history (any activations or near-misses)
9. Roadmap assessment: on track for next phases?
10. Decision: proceed with remaining governance proposals?

### T+30d: Transition to Normal Operations

At T+30d, the launch period ends and the network transitions to normal operations:

- [ ] On-call rotation transitions to standard (as defined in ongoing operations)
- [ ] Daily check-ins end; weekly check-ins continue for month 2
- [ ] Monitoring thresholds adjusted based on 30-day baseline data
- [ ] Agent confidence thresholds adjusted based on 30-day performance data
- [ ] Governance proposal cadence determined by community, not launch schedule
- [ ] Post-launch retrospective conducted and published

---

## Post-Launch Monitoring

### First 24 Hours — Continuous Monitoring

**Staffing:** All core team members on-call. Minimum 2 engineers actively monitoring at all times.

| Metric | Target | Alert Threshold | Check Frequency |
|--------|--------|----------------|-----------------|
| Block time | 5-7 seconds | >10 seconds for 5 consecutive blocks | Every block |
| Validator uptime | 100% | Any validator missing >10 blocks in 1 hour | Every block |
| Fee collection | >0 per hour (if M013 active) | Zero for 2 consecutive hours | Every hour |
| Agent health | All 4 healthy | Any agent unhealthy for >5 minutes | Every 30 seconds |
| MCP health | 200 OK | Non-200 for >2 minutes | Every 30 seconds |
| Memory usage | <80% on all nodes | >80% on any node | Every minute |
| Disk usage | <70% on all nodes | >70% on any node | Every 5 minutes |
| Network peers | >10 per node | <5 on any node | Every 5 minutes |
| Error rate | <0.1% of transactions | >1% of transactions failing | Every 5 minutes |
| Consensus rounds | 1 per block | >3 rounds for 5 consecutive blocks | Every block |

### First Week — Daily Check-ins

**Staffing:** On-call rotation (24/7). Daily 30-minute all-hands check-in.

| Metric | Target | Alert Threshold | Check Frequency |
|--------|--------|----------------|-----------------|
| Block time | 5-7 seconds | >8 seconds average over 1 hour | Every 5 minutes |
| Validator uptime | >99.5% each | <99% for any validator over 24h | Hourly |
| Fee collection | Consistent with transaction volume | >50% deviation from expected | Hourly |
| Fee distribution | Matches configured splits | >2% deviation for any split | Every epoch |
| Agent accuracy | >90% each | <85% for any agent over 24h | Daily |
| Agent escalation | <15% each | >20% for any agent over 24h | Daily |
| Governance activity | Responsive | Quorum at risk for active proposal | Daily |
| Community sentiment | Neutral/positive | Significant negative trend | Daily (manual) |

### First Month — Weekly Reviews

**Staffing:** On-call rotation (business hours + 4h off-hours response). Weekly 1-hour review meeting.

| Metric | Target | Alert Threshold | Review Frequency |
|--------|--------|----------------|------------------|
| Validator count | 15+ active | <15 for >1 hour | Daily automated |
| Network decentralization | Nakamoto coefficient >=4 | Coefficient drops to 3 | Weekly |
| Fee revenue trend | Growing or stable | >20% decline week-over-week | Weekly |
| Supply metrics (if M012 active) | Within dampening bounds | Any dampening trigger activated | Daily automated |
| Agent performance trend | Improving or stable | Any agent declining >5% week-over-week | Weekly |
| Governance participation | >80% | <70% on any proposal | Per proposal |
| Bug bounty submissions | Resolved promptly | Any unresolved high/critical >7 days | Weekly |
| Infrastructure costs | Within budget | >20% over budget | Weekly |

### Ongoing Monitoring (Per Phase 5.1)

After the first month, monitoring transitions to the Phase 5.1 operational framework:

| Category | Metrics | Review Cadence | Responsible Team |
|----------|---------|---------------|-----------------|
| Network Health | Block time, validator uptime, peer count, consensus efficiency | Continuous (automated) | Infrastructure |
| Economic Health | Fee revenue, supply metrics, reward distributions, treasury balance | Daily (automated) + weekly (manual) | Economics |
| Agent Health | Accuracy, escalation rate, latency, error rate, audit trail completeness | Daily (automated) + weekly (manual) | Agent Team |
| Governance Health | Participation rate, proposal throughput, quorum achievement | Per proposal + monthly summary | Governance Council |
| Security Health | Bug bounty status, vulnerability scan results, access audit | Weekly (automated) + monthly (manual) | Security |
| Community Health | Support ticket volume, sentiment analysis, documentation gaps | Weekly | Governance Council |

#### Alerting Tiers

| Tier | Response Time | Notification Method | Examples |
|------|-------------|-------------------|---------|
| P0 — Critical | 15 minutes | PagerDuty (phone call + SMS) | Chain halt, fund-at-risk vulnerability, supply cap breach |
| P1 — High | 1 hour | PagerDuty (push notification) | Validator <15, agent >50% errors, MCP down |
| P2 — Medium | 4 hours | Slack/Discord alert | Fee imbalance, reward drift, single validator down |
| P3 — Low | Next business day | Email + ticket | Documentation gap, minor UI issue, informational finding |

#### Dashboard Requirements

The following dashboards must be operational before launch and maintained continuously:

- [ ] **Network Overview** — Block production, validator set, consensus metrics
- [ ] **Fee Analytics** — Collection, distribution, routing accuracy
- [ ] **Supply Dashboard** — Total supply, circulating, staked, dampening events
- [ ] **Reward Dashboard** — Distribution per epoch, claim rates, accounting reconciliation
- [ ] **Agent Dashboard** — Per-agent health, accuracy, escalation, audit trail
- [ ] **Governance Dashboard** — Active proposals, voting progress, participation rates
- [ ] **Incident Dashboard** — Active alerts, rollback trigger status, on-call schedule

---

## Summary

### Consolidated Gate Status

| Gate | Type | Status | Owner | Blocking Issues |
|------|------|--------|-------|----------------|
| 1. Security | HARD BLOCKER | NOT STARTED | Security Lead | Audit not yet engaged |
| 2. Testing | HARD BLOCKER | NOT STARTED | Infrastructure Lead | Testnet not yet at 30 days |
| 3. Governance | HARD BLOCKER | NOT STARTED | Governance Council Rep | Proposals not yet submitted |
| 4. Infrastructure | HARD BLOCKER | NOT STARTED | Infrastructure Lead | Validator onboarding in progress |
| 5. Agent Deployment | CONDITIONAL | NOT STARTED | Agent Team Lead | Staging deployment not yet at 30 days |
| 6. Community Readiness | CONDITIONAL | NOT STARTED | Governance Council Rep | Documentation in progress |
| 7. Rollback Readiness | HARD BLOCKER | NOT STARTED | Core Team Lead | Rollback procedures not yet tested |

### Critical Path

The critical path to launch is:

1. **Audit engagement and completion** (12-17 weeks) — longest lead time item
2. **Testnet stability** (30 days minimum, concurrent with audit)
3. **Validator onboarding** (concurrent with audit and testnet)
4. **Agent staging** (30 days minimum, concurrent with above)
5. **Governance proposal sequence** (19-22 weeks from first submission)
6. **Bug bounty activation** (14 days before launch)
7. **Go/No-Go decision** (48 hours before launch)

**Estimated minimum time to launch:** 22-26 weeks from today (assuming parallel execution of independent workstreams).

### Launch Readiness Checklist (Executive Summary)

- [ ] Audit complete, zero critical/high findings — **HARD BLOCKER**
- [ ] Bug bounty active 14+ days — **HARD BLOCKER**
- [ ] All P0 tests passing, testnet stable 30+ days — **HARD BLOCKER**
- [ ] 15+ validators, 5/5/5 composition, 99.5% uptime — **HARD BLOCKER**
- [ ] Governance proposals 1-4 passed — **HARD BLOCKER**
- [ ] Emergency governance framework ratified — **HARD BLOCKER**
- [ ] Rollback triggers configured and tested — **HARD BLOCKER**
- [ ] On-call rotation established — **HARD BLOCKER**
- [ ] 4 agents on staging 30+ days, >90% accuracy — **CONDITIONAL**
- [ ] Community documentation published — **CONDITIONAL**
- [ ] Go/No-Go committee unanimous on launch — **REQUIRED**

### Document Maintenance

This document is maintained by the Core Team Lead and updated:

- Weekly during active launch preparation
- After every go/no-go meeting
- Immediately when any gate status changes
- After any rollback trigger activation (even on testnet)

All updates must be submitted via pull request and reviewed by at least one other
Launch Decision Committee member before merging.

---

*This document was created as part of the Agentic Tokenomics project. For questions
or contributions, see the [Contributor Guide](contributor-guide/) or open an issue
in the repository.*
