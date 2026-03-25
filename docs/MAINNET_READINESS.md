# Mainnet Readiness Checklist

## 1. Overview

This document is the **single source of truth** for the Regen Network mainnet launch readiness assessment. It consolidates all pre-launch requirements from Phases 1 through 5 into a unified decision framework that Gregory's team will use to make the **go/no-go decision** for activating the economic reboot mechanisms (M012, M013, M014, M015), agent infrastructure, and associated governance upgrades on Regen Ledger mainnet.

**Scope**: All mechanisms, modules, agents, MCP services, governance proposals, validator onboarding, rollback procedures, and monitoring operations required for a safe mainnet launch.

**Audience**: Core team, validator operators, Tokenomics Working Group, auditors, and governance participants.

**Status**: DRAFT — All gates must show PASS before launch authorization.

---

## 2. Smart Contract Audit Requirements

### 2.1 CosmWasm Contracts

The following CosmWasm contracts require independent external audit before mainnet deployment:

| Contract | Mechanism | Scope | Audit Firm Requirements |
|----------|-----------|-------|------------------------|
| `attestation-bond` | M008 | Bond lifecycle, challenge/resolution, slashing, fund conservation | CosmWasm audit experience; prior Cosmos ecosystem audits preferred |
| `service-escrow` | M009 | Escrow lifecycle, milestone progression, dispute resolution, fund conservation | Same as above |
| `reputation-registry` | M010 | Signal lifecycle, weight bounds, stake requirements, sybil resistance | Same as above |
| `marketplace-curation` | M011 | Collection management, quality scoring, curator incentives | Same as above |

**Audit firm requirements**:
- Minimum 2 prior CosmWasm or Cosmos SDK audit engagements
- Published audit reports for at least one top-50 Cosmos chain
- Team includes at least one auditor with Rust and CosmWasm expertise
- Firm listed on Immunefi, Code4rena, or equivalent reputable platform

**Scope per contract**:
- All security invariants verified (INV-001 through INV-011 per Phase 3.4)
- All attack vectors tested (AV-001 through AV-006 per Phase 3.4)
- Static analysis: Rust clippy, cargo-audit, Semgrep custom rules
- Manual review: access control, economic logic, state management, external calls
- CosmWasm-specific checks: no unbounded iteration, correct `deps.querier` usage, reply/submessage ordering
- Fuzzing: all entry points fuzzed with randomized inputs

**Timeline**:
- Code freeze: T-12 weeks before target launch
- Audit engagement: T-10 weeks (2 weeks for scoping and kickoff)
- Audit execution: T-10 to T-4 weeks (6-week audit window)
- Remediation: T-4 to T-2 weeks (2 weeks for fix + re-review)
- Final report: T-2 weeks

### 2.2 Native Modules

The following Cosmos SDK native modules require internal code review plus external audit:

| Module | Mechanism | Scope |
|--------|-----------|-------|
| `x/feerouter` | M013 | Fee calculation, value-based routing, pool distribution, share sum unity |
| `x/supply` | M012 | Mint/burn algorithm, hard cap enforcement, regrowth rate bounds, period cycling |
| `x/authority` | M014 | Validator lifecycle, composition guarantees, compensation distribution, term management |
| `x/rewards` | M015 | Activity scoring, stability tier, distribution calculation, revenue constraint |

**Internal review process**:
- Full code review by at least 2 core Regen Ledger engineers not involved in implementation
- All security invariants specified in Phase 2.6 verified with test coverage
- Go static analysis (golangci-lint, gosec)
- Integration testing against testnet with realistic transaction volumes

**External audit**:
- Same audit firm as CosmWasm contracts (preferred for cross-module coverage) or a second firm with Cosmos SDK module audit experience
- Scope includes module keeper logic, message handlers, begin/end block hooks, parameter validation, and migration paths

### 2.3 Audit Report Acceptance Criteria

An audit report is accepted when ALL of the following hold:

- [ ] Zero open **Critical** severity findings
- [ ] Zero open **High** severity findings
- [ ] All **Medium** findings either resolved or have documented mitigation with team sign-off
- [ ] All **Low** / **Informational** findings reviewed and triaged
- [ ] Audit firm provides written confirmation that remediation is satisfactory
- [ ] Audit report is published publicly (after coordinated disclosure window)

### 2.4 Bug Bounty Program Activation

A bug bounty program must be live **before** mainnet launch to incentivize responsible disclosure:

- **Platform**: Immunefi (https://immunefi.com/bounty/regen)
- **Activation**: At least 4 weeks before mainnet launch (T-4 weeks)
- **Scope**: All 4 CosmWasm contracts, all 4 native modules, agent runtime critical functions, MCP authentication
- **Rewards**:
  | Severity | Smart Contract | Infrastructure |
  |----------|---------------|----------------|
  | Critical | $50,000 -- $100,000 | $20,000 -- $50,000 |
  | High | $10,000 -- $50,000 | $5,000 -- $20,000 |
  | Medium | $2,500 -- $10,000 | $1,000 -- $5,000 |
  | Low | $500 -- $2,500 | $250 -- $1,000 |
- **Response SLA**: Initial response within 24h, triage within 72h
- **Disclosure policy**: Coordinated disclosure, 90-day timeline after fix deployed

---

## 3. Governance Approval Sequence

Each proposal must pass in dependency order. No proposal may be submitted until its predecessor has been finalized.

### Proposal Checklist

- [ ] **M013 software upgrade proposal passed**
  - Description: Deploy `x/feerouter` module via Regen Ledger software upgrade
  - Dependency: None (first in sequence)
  - Estimated timeline: T-24 weeks
  - Risk level: MEDIUM -- new module, backward compatible (flat gas retained for non-credit txs)

- [ ] **M013 fee configuration proposal passed**
  - Description: Set initial fee rates (issuance 1-3%, transfer 0.1%, retirement 0.5%, marketplace 1%) and distribution shares
  - Dependency: M013 software upgrade active
  - Estimated timeline: T-22 weeks
  - Risk level: MEDIUM -- economic parameters, affects all credit transactions

- [ ] **M013 90-day transition period completed**
  - Description: Dual-fee mode (flat gas + value-based fees) operational for 90 days, fee revenue > 0 for 30 consecutive days
  - Dependency: M013 fee configuration active
  - Estimated timeline: T-9 weeks (after 90-day transition from T-22)
  - Risk level: LOW -- monitoring only, rollback available

- [ ] **M014 + M012 combined upgrade proposal passed**
  - Description: Deploy `x/authority` and `x/supply` modules via single software upgrade; activate PoA validator governance and fixed-cap dynamic supply simultaneously
  - Dependency: M013 transition period completed (fee revenue flowing to validator fund and burn pool)
  - Estimated timeline: T-7 weeks
  - Risk level: HIGH -- fundamental consensus and monetary policy change

- [ ] **Authority validator seed set approved (15+ validators, 5/5/5 composition)**
  - Description: Governance vote approving initial authority validator set with minimum 5 infrastructure builders, 5 trusted ReFi partners, 5 ecological data stewards
  - Dependency: M014 module deployed
  - Estimated timeline: T-6 weeks
  - Risk level: HIGH -- determines network security and decentralization

- [ ] **M012 supply activation confirmed**
  - Description: Hard cap set (~221M REGEN), inflation module disabled, mint/burn algorithm enabled
  - Dependency: M014 PoA active, M013 burn pool receiving revenue
  - Estimated timeline: T-5 weeks
  - Risk level: HIGH -- irreversible monetary policy change (hard cap is Layer 4 constitutional)

- [ ] **M015 activation proposal passed**
  - Description: Deploy `x/rewards` module, begin activity score tracking (TRACKING state)
  - Dependency: M013 Community Pool receiving revenue, M014 active
  - Estimated timeline: T-4 weeks
  - Risk level: MEDIUM -- tracking only, no distributions yet

- [ ] **M015 90-day tracking period completed**
  - Description: Activity scoring data validated, no anomalies detected, calibration complete
  - Dependency: M015 TRACKING state active for 90 days
  - Estimated timeline: T+9 weeks (post initial activation)
  - Risk level: LOW -- monitoring only

- [ ] **PoS inflation disable proposal passed**
  - Description: Final governance vote to disable PoS inflation module and transition to full M014 Phase 3 (PoA-only)
  - Dependency: M014 fully active, M012 active, all validators compensated via fee revenue for 30+ days
  - Estimated timeline: T+12 weeks
  - Risk level: HIGH -- removes legacy staking rewards, mandatory unbonding period for delegators

---

## 4. Validator Onboarding Minimum Thresholds

All thresholds must be met before the M014 PoA activation proposal is submitted:

- [ ] **Minimum 15 authority validators active** before PoA activation
  - At least 15 validators from the approved seed set have operational nodes, have bonded minimum stake, and passed infrastructure verification

- [ ] **5/5/5 composition requirement met**
  - At least 5 infrastructure builders
  - At least 5 trusted ReFi partners
  - At least 5 ecological data stewards
  - Each category verified against published criteria (Phase 2.6, M014 validator composition)

- [ ] **All validators passing 99.5% uptime for 30+ days on testnet**
  - Measured continuously over a rolling 30-day window on the staging/testnet environment
  - Uptime = blocks signed / blocks expected
  - Validators failing this threshold must remediate before mainnet inclusion

- [ ] **Validator governance participation >80% on testnet proposals**
  - Each validator must have voted on at least 80% of testnet governance proposals during the evaluation period
  - Measured over a minimum of 10 testnet proposals

- [ ] **Infrastructure verification completed for each validator**
  - Hardware security module (HSM) or equivalent key management verified
  - Sentry node architecture confirmed
  - Monitoring and alerting configured
  - Backup and disaster recovery procedures documented and tested
  - Contact information and escalation procedures current
  - Geographic diversity requirement checked against full set

---

## 5. Agent Deployment Verification

All 4 agents must demonstrate production readiness on the staging environment before mainnet activation:

### 5.1 Operational Stability

- [ ] **All 4 agents operational on staging for 30+ days**
  - AGENT-001 Registry Reviewer: continuously processing credit class and project registration events
  - AGENT-002 Governance Analyst: continuously processing governance proposals and voting events
  - AGENT-003 Market Monitor: continuously processing marketplace trades, retirement events, and price data
  - AGENT-004 Validator Monitor: continuously processing validator performance and delegation flow events

### 5.2 Decision Quality

- [ ] **Agent accuracy >90%** (decisions not overridden by human reviewers)
  - Measured as: (total agent decisions - human overrides) / total agent decisions >= 0.90
  - Evaluated across all 4 agents independently; each must meet threshold
  - Override data tracked in KOI audit trail

- [ ] **Escalation rate <15%**
  - Measured as: escalated decisions / total decisions < 0.15
  - Escalation is expected and healthy; >15% suggests confidence thresholds need recalibration

### 5.3 Workflow Coverage

- [ ] **All 14 workflows tested end-to-end**
  - Registry Reviewer: WF-RR-01, WF-RR-02, WF-RR-03, WF-RR-04
  - Governance Analyst: WF-GA-01, WF-GA-02, WF-GA-03
  - Market Monitor: WF-MM-01, WF-MM-02, WF-MM-03, WF-MM-04
  - Validator Monitor: WF-VM-01, WF-VM-02, WF-VM-03
  - Each workflow tested with realistic inputs, edge cases, and escalation triggers

### 5.4 Audit Trail and Tooling

- [ ] **KOI audit trail verified** (all decisions logged)
  - Every agent decision has a corresponding KOI object with full context (observation data, orientation analysis, confidence score, action taken)
  - Audit log completeness verified: zero gaps in decision logging over 30-day staging period
  - Logs are tamper-evident and queryable

- [ ] **MCP services healthy** (Ledger MCP, KOI MCP, TX Builder)
  - Ledger MCP: responding to all query types (credit classes, proposals, validator data, account data)
  - KOI MCP: search, entity resolution, and SPARQL queries returning valid results
  - TX Builder: constructing and signing transactions correctly
  - All MCP services passing health checks continuously for 30+ days
  - Rate limiting and circuit breakers configured and tested

### 5.5 Economic Reboot Workflows

- [ ] **Economic reboot workflows validated**
  - WF-MM-05: Fee revenue monitoring and burn pool tracking (M013 integration)
  - WF-MM-06: Supply dynamics monitoring and equilibrium tracking (M012 integration)
  - WF-VM-04: Authority validator performance and compensation tracking (M014 integration)
  - WF-VM-05: Contribution score monitoring and distribution verification (M015 integration)
  - Each tested against staging instances of `x/feerouter`, `x/supply`, `x/authority`, and `x/rewards`

---

## 6. Rollback Triggers

Each mechanism has specific, measurable conditions that trigger rollback. For each trigger: automated detection is configured, manual trigger is documented, and the notification chain is established.

### 6.1 M013: Fee Routing Rollback

| Trigger | Condition | Response |
|---------|-----------|----------|
| Fee revenue drops to zero | Zero fee revenue collected for 48 consecutive hours | Rollback to FLAT_GAS state |
| **Automated detection** | `x/feerouter` emits `EventFeeCollected` — monitor for absence over 48h window | Alert fires at 24h (WARNING) and 48h (CRITICAL) |
| **Manual trigger** | Governance emergency proposal to revert fee configuration OR core team parameter change (if authority granted) | |
| **Notification chain** | Automated alert -> on-call engineer (5 min) -> security lead (15 min) -> core team Slack channel -> validator operator channel -> public status page |

### 6.2 M012: Supply Cap Breach

| Trigger | Condition | Response |
|---------|-----------|----------|
| Supply exceeds hard cap | `S[t] > hard_cap` (should be impossible by INV-001) | **Emergency chain halt** |
| **Automated detection** | `x/supply` module enforces cap invariant at every state transition; secondary monitoring via indexer comparing total supply to configured cap | Immediate P0 alert |
| **Manual trigger** | Any validator can call emergency halt via governance; core team can coordinate validator halt via out-of-band communication | |
| **Notification chain** | Automated P0 alert -> all validators simultaneously -> core team -> security lead -> CTO -> public disclosure within 1h |

### 6.3 M014: Validator Set Degradation

| Trigger | Condition | Response |
|---------|-----------|----------|
| Active validators = 15 | Count at `min_validators` — WARNING, no buffer remaining | **P0 alert; accelerate validator onboarding; pause non-critical governance** |
| Active validators < 13 | Count below Byzantine fault tolerance floor for a 21-validator design (3f+1 where f=4 requires ≥13) | **Emergency PoS restore** |
| **Automated detection** | AGENT-004 (WF-VM-01) tracks active count every block; `x/authority` emits `EventValidatorRemoved` | Alert at 17 (ADVISORY), 15 (WARNING/P0), <13 (CRITICAL/emergency halt) |
| **Manual trigger** | Governance emergency proposal to re-enable PoS module parameters and restore delegated staking | |
| **Notification chain** | AGENT-004 alert -> core team (immediate) -> all remaining validators -> emergency governance channel -> public communication within 2h |

### 6.4 M015: Distribution Overrun

| Trigger | Condition | Response |
|---------|-----------|----------|
| Distributions exceed Community Pool inflow | `total_distributions[period] > community_pool_inflow[period]` (violates Security Invariant 1) | **Pause distributions** |
| **Automated detection** | `x/rewards` module enforces revenue constraint at distribution time; secondary monitoring compares distribution totals to inflow per period | Alert on any period where distributions approach 95% of inflow (WARNING) |
| **Manual trigger** | Governance proposal to pause M015 distributions; core team parameter change to set distribution to zero | |
| **Notification chain** | Automated alert -> core team (15 min) -> Tokenomics WG -> governance forum post -> affected participants notified |

### 6.5 Agent Error Rate Spike

| Trigger | Condition | Response |
|---------|-----------|----------|
| Agent error rate >50% for 1 hour | Any single agent produces errors (failed workflows, unhandled exceptions, invalid outputs) on >50% of invocations over a 1h rolling window | **Pause affected agent** |
| **Automated detection** | Agent runtime health metrics exported to monitoring stack; error rate computed per agent over 1h sliding window | Alert at 30% (WARNING), pause at 50% (CRITICAL) |
| **Manual trigger** | Core team can pause individual agents via runtime configuration without chain governance | |
| **Notification chain** | Automated alert -> on-call engineer (5 min) -> agent team lead (15 min) -> dependent workflow consumers notified -> status page updated |

### 6.6 MCP Service Unavailability

| Trigger | Condition | Response |
|---------|-----------|----------|
| MCP service unavailable >15 minutes | Any MCP service (Ledger MCP, KOI MCP, TX Builder) fails health checks for >15 consecutive minutes | **Pause dependent workflows** |
| **Automated detection** | Health check probes every 30 seconds; failure threshold: 30 consecutive failures (15 min) | Alert at 5 min (WARNING), pause at 15 min (CRITICAL) |
| **Manual trigger** | Core team can pause MCP-dependent agent workflows without pausing the agents themselves | |
| **Notification chain** | Automated alert -> infrastructure on-call (5 min) -> agent team (10 min) -> dependent agents gracefully degrade to cached data or queue mode |

---

## 7. Go/No-Go Decision Framework

### 7.1 Decision Authority

The go/no-go decision requires agreement from:
- **Core team**: Engineering lead, security lead, product lead (all three must approve)
- **Validator governance**: Majority of approved authority validators signal readiness
- **Tokenomics WG**: Working group chair confirms economic parameters are finalized

No single individual can authorize launch. All three parties must independently confirm readiness.

### 7.2 Gate Criteria (ALL Must Pass)

| Gate | Criteria | Status |
|------|----------|--------|
| **Security** | All contract and module audits complete; zero open critical/high findings; bug bounty program active for 4+ weeks; no critical bugs reported during bounty period | [ ] PASS / FAIL |
| **Testing** | All P0 test cases passing; testnet stable for 30+ consecutive days; load testing confirms SLO targets met (p95 response <2s, error rate <10%); all 14 workflows tested E2E | [ ] PASS / FAIL |
| **Governance** | All required governance proposals passed (see Section 3); no active disputes or legal challenges to proposals | [ ] PASS / FAIL |
| **Infrastructure** | All 15+ authority validators operational and meeting thresholds (Section 4); all 4 agents operational on staging for 30+ days (Section 5); all MCP services healthy | [ ] PASS / FAIL |
| **Community** | Validator onboarding guide published; token holder migration FAQ published (PoS -> PoA transition); governance proposal summaries published for all 9 proposals; community AMA completed | [ ] PASS / FAIL |
| **Rollback** | All 6 rollback procedures tested on testnet (Section 6); automated detection confirmed working; notification chains verified end-to-end; rollback execution time <30 min for each scenario | [ ] PASS / FAIL |

### 7.3 Conditional Gates

Items that can proceed with documented mitigations (require team sign-off on risk acceptance):

| Item | Condition for Proceeding | Required Mitigation |
|------|--------------------------|---------------------|
| M012 ecological multiplier oracle not available | v0 deployment uses `ecological_multiplier = 1.0` (disabled) | Document plan for oracle integration in post-launch roadmap; set governance parameter to enable later |
| Open questions (OQ-*) not fully resolved | WG has documented consensus on all critical-path OQs; non-critical OQs have interim defaults | Interim defaults documented; governance can adjust post-launch |
| Agent accuracy between 85-90% | Escalation rate is healthy (<15%) and overrides are minor/non-critical | Increase monitoring frequency; review weekly; target 90%+ within 30 days post-launch |
| Bug bounty window shorter than 4 weeks | No critical/high findings in first 2 weeks; external audit clean | Extend bounty program post-launch; increase reward multiplier for first 90 days |

### 7.4 Hard Blockers

Items that **absolutely must be resolved** before launch -- no exceptions, no mitigations:

- **Open critical or high audit findings** in any contract or module
- **Active security incident** or unresolved vulnerability report
- **Fewer than 15 authority validators** meeting all onboarding thresholds
- **Any mechanism module failing its security invariants** on testnet
- **Rollback procedures not tested** -- every rollback in Section 6 must have been executed on testnet at least once
- **Governance proposals not passed** -- all proposals in Section 3 must be finalized
- **Supply cap invariant violation** observed at any point during testing
- **Fee conservation violation** observed at any point during testing

---

## 8. Launch Day Runbook

### T-24h: Final Verification

- [ ] Final testnet verification run: all P0 tests passing
- [ ] Team on-call assignments confirmed and published
  - Engineering lead: primary on-call
  - Security lead: secondary on-call
  - Agent team lead: agent systems on-call
  - Infrastructure lead: validator/MCP systems on-call
- [ ] War room communication channels created (Slack, video call link)
- [ ] Validator operators notified of upgrade height and timeline
- [ ] Status page prepared with launch day update template
- [ ] Rollback scripts staged and verified on testnet (final dry run)

### T-1h: Pre-Flight Checks

- [ ] Chain health: block production stable, no missed blocks in last 100 blocks
- [ ] Agent health: all 4 agents responding to health checks, last workflow execution successful
- [ ] MCP health: Ledger MCP, KOI MCP, TX Builder all passing health checks
- [ ] Validator coordination: 15+ validators confirm readiness via signed message
- [ ] Monitoring dashboards open and visible to all on-call team members
- [ ] External communication drafted: announcement for Discord, Twitter, forum

### T-0: Governance Proposal Execution

- [ ] Upgrade height reached; Regen Ledger binary upgrade executes
- [ ] New modules initialized: `x/feerouter`, `x/supply`, `x/authority`, `x/rewards` (per activation sequence)
- [ ] Chain produces first post-upgrade block successfully
- [ ] All validators signing blocks (confirm >2/3 signing power)
- [ ] Genesis state migration verified (no state corruption)

### T+1h: Post-Upgrade Verification

- [ ] Module state verification:
  - `x/feerouter`: fee configuration matches approved parameters; first fee collection confirmed
  - `x/supply`: hard cap set correctly; current supply within bounds; mint/burn algorithm responding
  - `x/authority`: validator set matches approved seed set; compensation distribution correct
  - `x/rewards`: activity tracking active (TRACKING state); no premature distributions
- [ ] Fee collection: submit test credit transaction; verify fee collected and distributed to correct pools
- [ ] Agent connectivity: all 4 agents successfully querying new module state via Ledger MCP
- [ ] MCP services: all endpoints returning data from upgraded chain
- [ ] No error spikes in any monitoring dashboard

### T+24h: First Full-Day Review

- [ ] Fee revenue summary: total fees collected, distribution per pool verified
- [ ] Validator performance: all authority validators signed >99% of blocks
- [ ] Agent performance: all agents processed events correctly; error rate <5%
- [ ] MCP uptime: all services >99.9% availability
- [ ] Community feedback review: Discord, forum, Twitter -- any reported issues triaged
- [ ] Incident log review: any P0/P1 incidents during first 24h documented and resolved

### T+7d: First Weekly Review

- [ ] Fee revenue trend: week-over-week trajectory; compare to testnet projections
- [ ] Supply dynamics: mint/burn quantities for first week; trajectory toward equilibrium
- [ ] Validator performance report: composite scores for all authority validators
- [ ] Agent accuracy assessment: override rates, escalation rates, decision quality scores
- [ ] Community sentiment: governance forum activity, staking migration questions
- [ ] Bug bounty status: any submissions during first week reviewed and triaged
- [ ] Parameter tuning assessment: any fee rates or distribution shares needing adjustment?

### T+30d: First Monthly Review

- [ ] Comprehensive fee revenue analysis: actual vs. projected; revenue sustainability assessment
- [ ] Supply model validation: M[t] and B[t] tracking; distance from equilibrium
- [ ] Validator set health: any probation actions needed; composition still meeting 5/5/5
- [ ] Agent maturity assessment: 30-day accuracy and reliability metrics; ready for expanded autonomy?
- [ ] M015 tracking data review: activity scoring calibration; preparation for DISTRIBUTING transition
- [ ] Parameter tuning recommendations: any governance proposals needed for adjustments
- [ ] Security review: bug bounty summary; any new findings since launch
- [ ] Post-launch retrospective: lessons learned document drafted

---

## 9. Post-Launch Monitoring Plan

### First 24 Hours: Continuous Monitoring

- **Coverage**: All on-call team members active; 24/7 monitoring rotation
- **Response SLA**: P0 incidents -- immediate response (within 5 minutes); P1 -- within 15 minutes
- **Check frequency**: Every block for validator/consensus health; every 5 minutes for agent/MCP health
- **Escalation**: Any anomaly triggers immediate war room activation
- **Communication**: Hourly status updates to validator channel; real-time updates to core team

### First Week: Daily Check-Ins

- **Coverage**: Daily 30-minute stand-up with full team; on-call rotation continues (reduced to 12h shifts)
- **Response SLA**: P0 -- within 5 minutes; P1 -- within 15 minutes; P2 -- within 4 hours
- **Check frequency**: Every 15 minutes for critical systems; hourly for secondary metrics
- **Deliverable**: Daily status report (fee revenue, validator health, agent accuracy, MCP uptime)
- **Communication**: Daily summary post to governance forum and Discord

### First Month: Weekly Reviews

- **Coverage**: Weekly 1-hour review meeting with core team, WG chair, validator representative
- **Response SLA**: P0 -- within 15 minutes; P1 -- within 1 hour; P2 -- within 4 hours
- **Check frequency**: Every 30 minutes for critical systems; every 4 hours for secondary metrics
- **Deliverable**: Weekly review report covering all metrics in T+7d checklist (Section 8)
- **Parameter tuning**: Any recommended parameter adjustments submitted as governance proposals
- **Communication**: Weekly governance forum update; monthly community AMA

### Ongoing: Per Phase 5.1 Monitoring Operations

- **Coverage**: Standard on-call rotation (single engineer); agent-assisted monitoring (AGENT-004 primary)
- **Response SLA**: Per incident response plan (Phase 3.4) -- P0 immediate, P1 <1h, P2 <4h, P3 <24h
- **Check frequency**: Agent-driven continuous monitoring; human review quarterly
- **Deliverable**: Monthly health report; quarterly comprehensive review
- **Governance**: Parameter tuning proposals as needed; annual validator term renewals (M014)
- **Evolution**: Progress toward Phase B agentic governance integration (per governance roadmap)

---

## 10. Summary: Consolidated Gate Status

| # | Gate | Owner | Status | Notes |
|---|------|-------|--------|-------|
| 1 | CosmWasm contract audit (M008, M009, M010, M011) | Security Lead | [ ] NOT STARTED | Requires code freeze T-12w |
| 2 | Native module audit (x/feerouter, x/supply, x/authority, x/rewards) | Security Lead | [ ] NOT STARTED | Requires code freeze T-12w |
| 3 | Bug bounty program active 4+ weeks | Security Lead | [ ] NOT STARTED | Activate after audit reports accepted |
| 4 | M013 software upgrade proposal | Engineering Lead | [ ] NOT STARTED | First governance action |
| 5 | M013 fee configuration proposal | Tokenomics WG | [ ] NOT STARTED | Requires OQ-M013-1 through OQ-M013-5 resolved |
| 6 | M013 90-day transition completed | Engineering Lead | [ ] NOT STARTED | Monitoring period |
| 7 | M014 + M012 combined upgrade proposal | Engineering Lead | [ ] NOT STARTED | Depends on gate 6 |
| 8 | Authority validator seed set approved (15+, 5/5/5) | Validator Governance | [ ] NOT STARTED | Depends on gate 7 |
| 9 | M012 supply activation confirmed | Tokenomics WG | [ ] NOT STARTED | Depends on gate 8 |
| 10 | M015 activation proposal | Engineering Lead | [ ] NOT STARTED | Depends on gate 7 |
| 11 | 15+ validators meeting all thresholds | Validator Governance | [ ] NOT STARTED | 99.5% uptime, 80% gov participation |
| 12 | All 4 agents operational 30+ days on staging | Agent Team Lead | [ ] NOT STARTED | Accuracy >90%, escalation <15% |
| 13 | All 14 workflows tested E2E | Agent Team Lead | [ ] NOT STARTED | Includes economic reboot workflows |
| 14 | KOI audit trail verified | Agent Team Lead | [ ] NOT STARTED | Zero gaps in decision logging |
| 15 | MCP services healthy 30+ days | Infrastructure Lead | [ ] NOT STARTED | Ledger MCP, KOI MCP, TX Builder |
| 16 | All rollback procedures tested on testnet | Engineering Lead | [ ] NOT STARTED | All 6 scenarios in Section 6 |
| 17 | Community materials published | Product Lead | [ ] NOT STARTED | Migration FAQ, onboarding guide, AMA |
| 18 | Go/no-go decision: all 3 parties approve | Core Team + Validators + WG | [ ] NOT STARTED | Final gate |

**Launch authorization requires**: All 18 gates showing PASS, with no open hard blockers (Section 7.4).

---

*This document is part of the Regen Network Agentic Tokenomics framework. It will be updated as gates are completed. Last updated: 2026-03-24.*
