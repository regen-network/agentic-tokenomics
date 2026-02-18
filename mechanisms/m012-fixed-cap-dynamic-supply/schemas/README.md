# m012 output schemas

These JSON Schemas define **canonical output shapes** for m012 (Fixed Cap Dynamic Supply) artifacts.

## Files
- `m012_supply_state.schema.json` -- schema for the supply state input (current supply, cap, staked amounts, multiplier config).
- `m012_period_record.schema.json` -- schema for per-period mint/burn records (supply before/after, minted, burned, rates, multipliers).
- `m012_kpi.schema.json` -- schema for the KPI JSON block emitted by agents/digests, including equilibrium status.

## Notes
- These schemas are intended for **validation** and consistency across repos (Heartbeat, agent skills, etc.).
- All token amounts are in **uregen** (1 REGEN = 1,000,000 uregen) and represented as strings to preserve BigInt precision.
- The `m014_state` field gates which multiplier is used (staking vs stability vs max of both).
- v0 disables the ecological multiplier (set to 1.0).
