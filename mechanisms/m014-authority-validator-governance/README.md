# m014 — Authority Validator Governance (PoA Transition)

m014 specifies the transition from Proof of Stake (capital-weighted security) to Proof of Authority (contribution-weighted governance), replacing passive staking rewards with fee-based compensation and mission-aligned validator selection.

## What it outputs
- A composite **performance score** (0.0–1.0) per validator, based on uptime (0.4), governance participation (0.3), and ecosystem contribution (0.3).
- Validator lifecycle state tracking: CANDIDATE, APPROVED, ACTIVE, PROBATION, REMOVED, TERM_EXPIRED.
- KPI metrics: validator counts by status/category, average performance, compensation statistics.
- Compensation allocation: equal-share base from validator fund (M013) plus optional 10% performance bonus pool.

## What it does not do (v0)
- No on-chain authority module deployment (specification only).
- No automatic validator removal — governance process required.
- Ecosystem contribution scoring relies on AGENT-004 advisory assessment.

## How to reference
- Canonical spec: `mechanisms/m014-authority-validator-governance/SPEC.md`
- Validator lifecycle: SPEC.md section 6 (state machine, composition requirements)
- Performance scoring: SPEC.md section 5 (weighted factors, confidence, thresholds)
- Dependencies: M013 (validator fund), M012 (inflation disabled)

## Replay datasets
See `datasets/` for deterministic fixtures used to generate non-zero KPI outputs without MCP.
- `v0_sample.json` — 5 validators across 3 categories with varied performance profiles
- `v0_transition_sample.json` — PoS-to-PoA transition scenarios with lifecycle state changes

## Schemas
Canonical JSON schemas for m014 outputs live in `schemas/`.
- `m014_validator.schema.json` — validator lifecycle (address, category, status, term, performance)
- `m014_performance.schema.json` — performance score output (score, factors)
- `m014_kpi.schema.json` — KPI output with mechanism_id const "m014"
