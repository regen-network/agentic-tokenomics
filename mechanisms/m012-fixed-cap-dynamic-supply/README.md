# m012 — Fixed Cap Dynamic Supply

m012 replaces the current inflationary proof-of-stake supply model with a **hard-capped, algorithmically managed supply** that ties token minting and burning to real ecological activity on the network. Inspired by Blockscience's carrying capacity model and Ethereum's EIP-1559 burn mechanics.

## What it does
- Enforces a **hard cap** of 221,000,000 REGEN (221,000,000,000,000 uregen).
- Computes per-period **regrowth** (minting): `M[t] = r * (C - S[t])`, where `r` is dynamically adjusted based on staking participation and ecological metrics.
- Consumes per-period **burns** from M013 fee routing: `B[t] = sum(burn_share * fee)`.
- Updates supply: `S[t+1] = S[t] + M[t] - B[t]`.
- Approaches **dynamic equilibrium** where minting equals burning.

## What it does not do (v0)
- No ecological multiplier oracle integration (set to 1.0 / disabled).
- No on-chain enforcement (reference computation only).
- No per-block granularity (operates per-epoch).

## How to reference
- Canonical spec: `mechanisms/m012-fixed-cap-dynamic-supply/SPEC.md`
- Reference implementation: `reference-impl/m012_supply.js`
- KPI computation: `reference-impl/m012_kpi.js`

## Replay datasets
See `datasets/` for deterministic fixtures used to generate non-zero KPI outputs without MCP.
- `v0_sample.json` — 5 periods of supply state transitions
- `v0_equilibrium_sample.json` — periods approaching equilibrium (mint ~ burn)

## Schemas
Canonical JSON schemas for m012 outputs live in `schemas/`.
- `m012_supply_state.schema.json` — supply state (supply, cap, minted, burned, rates, multipliers)
- `m012_period_record.schema.json` — per-period mint/burn record
- `m012_kpi.schema.json` — KPI output with mechanism_id const "m012"
