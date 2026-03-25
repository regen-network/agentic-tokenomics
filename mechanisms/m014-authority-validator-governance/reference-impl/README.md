# m014 reference implementation (v0)

This folder provides a **canonical computation** for m014 outputs so that different agents/runners
produce consistent numbers.

## Inputs
### Score computation (`m014_score.js`)
- `validator` — `{ address }` identifying the validator
- `factors` — `{ uptime, governance_participation, ecosystem_contribution }` each 0.0–1.0 or null

### KPI computation (`m014_kpi.js`)
- `as_of` (ISO-8601 string, Z-suffixed)
- `validators[]` where each validator includes:
  - `address`, `moniker`, `category`, `status`
  - `factors` — `{ uptime, governance_participation, ecosystem_contribution }`
- `validator_fund_balance` (optional) — total fund from M013

## Outputs
### Score block
- `performance_score` — composite weighted score (0.0–1.0):
  - `uptime * 0.4 + governance_participation * 0.3 + ecosystem_contribution * 0.3`
- `confidence` — data availability confidence (1.0, 0.67, 0.33, or 0.0)
- `factors` — individual factor values
- `flags` — performance warnings (`below_performance_threshold`, `below_uptime_minimum`, `probation_recommended`)

### KPI block
- `total_validators` — count across all statuses
- `validators_by_status` — breakdown by lifecycle state
- `validators_by_category` — active validators per composition category
- `avg_performance_score`, `min_performance_score`, `max_performance_score`
- `validators_below_threshold` — count with score < 0.70
- `composition_valid` — boolean: each category has >= 5 active validators
- `byzantine_tolerance` — active count, max f, tolerance met
- `compensation` — base per validator and bonus pool (when fund balance provided)

## Self-test
```bash
node mechanisms/m014-authority-validator-governance/reference-impl/m014_score.js
```
