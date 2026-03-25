# m010 — Reputation Signal (v0 advisory)

m010 defines a reputation / legitimacy signal for Regen ecosystem subjects (e.g., credit classes, projects, verifiers, methodologies, addresses) based on **stake-weighted endorsements** with **time decay**.

## What it outputs
- A normalized **reputation score** (`reputation_score_0_1`, range `0–1`) per `(subject_type, subject_id, category)` in v0 advisory mode.
- A queryable history of submitted signals (endorsements), including state transitions (submitted, active, challenged, resolved_valid, resolved_invalid, withdrawn, invalidated).
- Challenge workflow events with evidence, resolution, and rationale.

## What it does not do (v0)
- No enforcement, gating, fee changes, or automatic voting-weight changes.
- No transactions are initiated by agents; the signal is informational/advisory.
- Challenge resolution is admin-driven (no economic stakes). v1 adds Arbiter DAO resolution and challenge deposits.

## How to reference
- Canonical spec: `mechanisms/m010-reputation-signal/SPEC.md`
- Challenge workflow: SPEC.md section 6 (state machine, participants, parameters, resolution process)
- Other mechanisms may treat m010 as an input signal (e.g., class creator reputation, attester reputation, service-provider reputation, marketplace curation signals).

## Replay datasets
See `datasets/` for deterministic fixtures used to generate non-zero KPI outputs without MCP.
- `v0_sample.json` — basic signal scoring (all signals active)
- `v0_challenge_sample.json` — challenge workflow with varied signal statuses and challenge resolutions

## Schemas
Canonical JSON schemas for m010 outputs live in `schemas/`.
- `m010_signal.schema.json` — signal items with `status` lifecycle field
- `m010_challenge.schema.json` — challenge events with evidence and resolution
- `m010_kpi.schema.json` — KPI output including optional `challenge_kpis`

## Reference implementation checks
- Deterministic vectors are validated by `scripts/verify-m010-reference-impl.mjs`
- `challenge_rate` is computed as `challenges_filed / signals_emitted` for replay-period KPI reporting

## Consumer compatibility notes
- v0 score contract: consumers should read `score.reputation_score_0_1` (`0..1`).
- Status contribution contract: only `active` and `resolved_valid` contribute to score.
- Challenge KPI contract: when `challenges[]` are present, `challenge_kpis` includes:
  - `challenges_filed`
  - `challenge_rate`
  - `avg_resolution_time_hours`
  - `challenge_success_rate`
  - `admin_resolution_timeout_rate`

Breaking changes for downstream consumers include renaming/removing these keys, changing denominator semantics, or changing lifecycle contribution rules.
