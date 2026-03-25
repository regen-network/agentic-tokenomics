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
