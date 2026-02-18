# m012 datasets (replay fixtures)

These fixtures are **deterministic inputs** for generating non-zero m012 KPI outputs **without MCP**.

## Files
- `schema.json` -- JSON schema for replay datasets
- `fixtures/v0_sample.json` -- 5 periods of supply state transitions (varying staking, standard burn)
- `fixtures/v0_equilibrium_sample.json` -- periods approaching equilibrium where minting approximately equals burning

## How they are used
A replay runner (e.g., in `regen-heartbeat`) can read a fixture file and compute:
- Per-period supply changes using `computeSupplyPeriod` from `reference-impl/m012_supply.js`
- Aggregated KPIs using `computeM012KPI` from `reference-impl/m012_kpi.js`

Key metrics produced:
- `current_supply` -- circulating supply at end of evaluated periods (uregen)
- `cap_headroom` -- remaining capacity before hard cap (uregen)
- `total_minted`, `total_burned` -- aggregate mint/burn over all periods
- `equilibrium_status` -- whether minting approximately equals burning

## Units
All token amounts are in **uregen** (1 REGEN = 1,000,000 uregen) and represented as strings to preserve BigInt precision.

These datasets are **reference-only** and do not imply enforcement or on-chain actions.
