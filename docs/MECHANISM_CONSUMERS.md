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

## m012 — Fixed Cap Dynamic Supply
**Canonical spec**
- `mechanisms/m012-fixed-cap-dynamic-supply/SPEC.md`

**Outputs**
- KPI JSON block schema: `mechanisms/m012-fixed-cap-dynamic-supply/schemas/m012_kpi.schema.json`
- Supply state schema: `mechanisms/m012-fixed-cap-dynamic-supply/schemas/m012_supply_state.schema.json`
- Period record schema: `mechanisms/m012-fixed-cap-dynamic-supply/schemas/m012_period_record.schema.json`

**Reference implementation**
- Supply period computation: `mechanisms/m012-fixed-cap-dynamic-supply/reference-impl/m012_supply.js` (`computeSupplyPeriod`)
- KPI computation: `mechanisms/m012-fixed-cap-dynamic-supply/reference-impl/m012_kpi.js` (`computeM012KPI`)

**Datasets (deterministic)**
- Replay fixtures: `mechanisms/m012-fixed-cap-dynamic-supply/datasets/fixtures/v0_sample.json`
- Equilibrium fixtures: `mechanisms/m012-fixed-cap-dynamic-supply/datasets/fixtures/v0_equilibrium_sample.json`

**Known consumers**
- Reference self-test: `node mechanisms/m012-fixed-cap-dynamic-supply/reference-impl/m012_supply.js`
