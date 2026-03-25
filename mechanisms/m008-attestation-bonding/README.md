# m008 — Data Attestation Bonding (v0 advisory)

m008 creates economic incentives for high-quality ecological data attestations by requiring attesters to **bond REGEN tokens** that can be slashed for false or misleading claims. Attestation types include project boundaries, baseline measurements, credit issuance claims, and methodology validations, each with risk-proportional bond requirements.

## What it outputs
- An **attestation quality score** (0–1000) per attestation, based on bond adequacy, attester reputation, evidence completeness, and attestation type risk.
- An **attestation lifecycle** tracking state transitions from BONDED through ACTIVE, CHALLENGED, to RELEASED/RESOLVED_VALID/SLASHED.
- **KPI metrics**: attestations submitted, challenge rate, resolution outcomes, average bond amount, slashing rate.

## What it does not do (v0)
- No on-chain bond escrow. Bond requirements are published as guidelines only.
- No on-chain challenge deposits or arbiter resolution. Challenge workflow is admin-driven.
- No automated slashing. Quality scores are advisory/informational only.
- No Arbiter DAO integration (v1 uses DAO DAO subDAO).

## How to reference
- Canonical spec: `mechanisms/m008-attestation-bonding/SPEC.md`
- State machine: SPEC.md section 6
- Bond schedule: SPEC.md section 7 (ProjectBoundary 500, Baseline 1000, CreditIssuance 2000, Methodology 5000 REGEN)
- Scoring function: SPEC.md section 5 (4-factor weighted composite)
- Other mechanisms may treat m008 attestation status as an input (e.g., M009 service escrow, M001-ENH class approval).

## Replay datasets
See `datasets/` for deterministic fixtures used to generate non-zero KPI outputs without MCP.
- `v0_sample.json` — attestations covering full lifecycle (active, released, challenged, resolved)
- `v0_challenge_sample.json` — challenge scenarios with varied resolution outcomes

## Schemas
Canonical JSON schemas for m008 outputs live in `schemas/`.
- `m008_attestation.schema.json` — attestation lifecycle objects with bond and status
- `m008_quality_score.schema.json` — attestation quality score output
- `m008_kpi.schema.json` — KPI metrics (attestations, challenges, bond economics)
