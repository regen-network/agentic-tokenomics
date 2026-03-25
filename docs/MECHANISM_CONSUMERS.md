# Mechanism consumers

This document maps **mechanism IDs** to known **consumers** (agents, digests, scripts).

## m010 — Reputation Signal
**Canonical spec**
- `mechanisms/m010-reputation-signal/SPEC.md`

**Outputs**
- KPI JSON block schema: `mechanisms/m010-reputation-signal/schemas/m010_kpi.schema.json`
- Signal item schema: `mechanisms/m010-reputation-signal/schemas/m010_signal.schema.json`

**Datasets (deterministic)**
- Replay fixtures: `mechanisms/m010-reputation-signal/datasets/fixtures/v0_sample.json`

**Known consumers**
- Heartbeat character: `signal-agent` (regen-heartbeat)
- Heartbeat replay runner: `scripts/replay-m010.mjs` (regen-heartbeat)
- Heartbeat stub runner: `scripts/stub-run-signal-agent.mjs` (regen-heartbeat)
- Heartbeat validator: `scripts/validate-signal-agent.mjs` (regen-heartbeat)

## m001-enh — Credit Class Approval Voting Enhancement
**Canonical spec**
- `mechanisms/m001-enh-credit-class-approval/SPEC.md`

**Outputs**
- KPI JSON block schema: `mechanisms/m001-enh-credit-class-approval/schemas/m001_kpi.schema.json`
- Agent score schema: `mechanisms/m001-enh-credit-class-approval/schemas/m001_agent_score.schema.json`
- Proposal lifecycle schema: `mechanisms/m001-enh-credit-class-approval/schemas/m001_proposal.schema.json`

**Datasets (deterministic)**
- Replay fixtures: `mechanisms/m001-enh-credit-class-approval/datasets/fixtures/v0_sample.json`
- Rejection scenarios: `mechanisms/m001-enh-credit-class-approval/datasets/fixtures/v0_rejection_sample.json`

**Known consumers**
- Heartbeat character: `signal-agent` (regen-heartbeat) — consumes agent scores for governance digest
- Governance workflows: GOV-001 Credit Class Creator Allowlist process (phase-2/2.3)

## m008 — Data Attestation Bonding
**Canonical spec**
- `mechanisms/m008-attestation-bonding/SPEC.md`

**Outputs**
- KPI JSON block schema: `mechanisms/m008-attestation-bonding/schemas/m008_kpi.schema.json`
- Quality score schema: `mechanisms/m008-attestation-bonding/schemas/m008_quality_score.schema.json`
- Attestation lifecycle schema: `mechanisms/m008-attestation-bonding/schemas/m008_attestation.schema.json`

**Datasets (deterministic)**
- Replay fixtures: `mechanisms/m008-attestation-bonding/datasets/fixtures/v0_sample.json`
- Challenge scenarios: `mechanisms/m008-attestation-bonding/datasets/fixtures/v0_challenge_sample.json`

**Known consumers**
- Heartbeat character: `attestation-agent` (regen-heartbeat, planned)
- KOI MCP: attestation quality lookups via `resolve_entity` / `get_entity_documents`
- Ledger MCP: bond balance queries via `get_balance` / `get_all_balances`
## m009 — Service Provision Escrow
**Canonical spec**
- `mechanisms/m009-service-escrow/SPEC.md`

**Outputs**
- KPI JSON block schema: `mechanisms/m009-service-escrow/schemas/m009_kpi.schema.json`
- Milestone review schema: `mechanisms/m009-service-escrow/schemas/m009_milestone_review.schema.json`
- Agreement lifecycle schema: `mechanisms/m009-service-escrow/schemas/m009_agreement.schema.json`

**Datasets (deterministic)**
- Replay fixtures: `mechanisms/m009-service-escrow/datasets/fixtures/v0_sample.json`
- Dispute scenarios: `mechanisms/m009-service-escrow/datasets/fixtures/v0_dispute_sample.json`

**Known consumers**
- Heartbeat character: `escrow-agent` (regen-heartbeat, planned)
- AGENT-001: Milestone deliverable review (advisory)
- AGENT-003: Pricing fairness monitor (advisory)
- KOI MCP: deliverable IRI resolution via `resolve_entity` / `get_entity_documents`
- Ledger MCP: escrow balance queries via `get_balance` / `get_all_balances`
## m011 — Marketplace Curation & Quality Signals
**Canonical spec**
- `mechanisms/m011-marketplace-curation/SPEC.md`

**Outputs**
- KPI JSON block schema: `mechanisms/m011-marketplace-curation/schemas/m011_kpi.schema.json`
- Quality score schema: `mechanisms/m011-marketplace-curation/schemas/m011_quality_score.schema.json`
- Collection lifecycle schema: `mechanisms/m011-marketplace-curation/schemas/m011_collection.schema.json`

**Datasets (deterministic)**
- Replay fixtures: `mechanisms/m011-marketplace-curation/datasets/fixtures/v0_sample.json`
- Collection challenges: `mechanisms/m011-marketplace-curation/datasets/fixtures/v0_collection_sample.json`

**Known consumers**
- AGENT-003: Autonomous quality scoring, price monitoring, collection monitoring
- KOI MCP: methodology metadata analysis via `resolve_entity`
- Ledger MCP: batch metadata and trade queries
- x/ecocredit: batch, class, project data; marketplace sell orders
- Heartbeat: KPI metrics in weekly digest
## m013 — Value-Based Fee Routing
**Canonical spec**
- `mechanisms/m013-value-based-fee-routing/SPEC.md`

**Outputs**
- KPI JSON block schema: `mechanisms/m013-value-based-fee-routing/schemas/m013_kpi.schema.json`
- Fee event schema: `mechanisms/m013-value-based-fee-routing/schemas/m013_fee_event.schema.json`
- Fee config schema: `mechanisms/m013-value-based-fee-routing/schemas/m013_fee_config.schema.json`

**Datasets (deterministic)**
- Replay fixtures: `mechanisms/m013-value-based-fee-routing/datasets/fixtures/v0_sample.json`

**Known consumers**
- Reference implementation self-test: `mechanisms/m013-value-based-fee-routing/reference-impl/m013_fee.js`
- KPI computation: `mechanisms/m013-value-based-fee-routing/reference-impl/m013_kpi.js`
## m014 — Authority Validator Governance
**Canonical spec**
- `mechanisms/m014-authority-validator-governance/SPEC.md`

**Outputs**
- KPI JSON block schema: `mechanisms/m014-authority-validator-governance/schemas/m014_kpi.schema.json`
- Validator item schema: `mechanisms/m014-authority-validator-governance/schemas/m014_validator.schema.json`
- Performance score schema: `mechanisms/m014-authority-validator-governance/schemas/m014_performance.schema.json`

**Datasets (deterministic)**
- Replay fixtures: `mechanisms/m014-authority-validator-governance/datasets/fixtures/v0_sample.json`
- Transition fixtures: `mechanisms/m014-authority-validator-governance/datasets/fixtures/v0_transition_sample.json`

**Known consumers**
- AGENT-004: Validator Monitor (performance tracking, probation recommendations)
- Heartbeat character: `validator-monitor-agent` (regen-heartbeat, planned)
- M013 integration: validator fund balance feeds compensation computation
