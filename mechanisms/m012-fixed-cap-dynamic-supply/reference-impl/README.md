# m012 reference implementation (v0)

This folder provides a **canonical computation** for m012 outputs so that different agents/runners
produce consistent numbers.

## Inputs

### Supply period computation (`computeSupplyPeriod`)
- `supply_state.current_supply` (string, uregen) -- current circulating supply
- `supply_state.hard_cap` (string, uregen, default "221000000000000") -- hard cap
- `supply_state.staked_amount` (string, uregen) -- staked tokens
- `supply_state.stability_committed` (string, uregen, default "0") -- M015 stability commitments
- `supply_state.m014_state` (string) -- INACTIVE | TRANSITION | ACTIVE | EQUILIBRIUM
- `supply_state.ecological_multiplier_enabled` (boolean) -- false in v0
- `burn_amount` (string, uregen) -- tokens burned this period (from M013)
- `config.r_base` (number, default 0.02) -- base regrowth rate

### KPI computation (`computeM012KPI`)
- `as_of` (ISO-8601 Z string) -- point-in-time
- `periods[]` -- array of period records with minted, burned, supply_after, rates

## Outputs

### Supply period
- `next_supply` -- supply after period (uregen string)
- `minted` -- tokens minted as regrowth (uregen string)
- `burned` -- tokens burned (uregen string)
- `regrowth_rate` -- effective rate r = r_base * multiplier * eco_mult
- `effective_multiplier` -- phase-gated multiplier
- `staking_multiplier`, `stability_multiplier`, `ecological_multiplier`
- `headroom_before`, `headroom_after` -- cap headroom (uregen strings)

### KPI block
- `mechanism_id` = "m012"
- `current_supply`, `hard_cap`, `cap_headroom` -- uregen strings
- `total_minted`, `total_burned`, `net_supply_change` -- uregen strings
- `latest_regrowth_rate`, `latest_effective_multiplier`
- `periods_evaluated`, `equilibrium_status`, `equilibrium_gap_pct`

## Self-test

```bash
node m012_supply.js
```

Reads `test_vectors/vector_v0_sample.input.json` and validates against
`test_vectors/vector_v0_sample.expected.json`.

## Design notes
- All token amounts use **BigInt** to avoid floating-point precision issues on large uregen values.
- Rates and multipliers remain as floating-point numbers (they are small dimensionless multipliers).
- The `Math.floor()` truncation when converting rate * headroom to BigInt is intentional and matches the expected test vectors.
