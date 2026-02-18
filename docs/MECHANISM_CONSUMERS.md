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
