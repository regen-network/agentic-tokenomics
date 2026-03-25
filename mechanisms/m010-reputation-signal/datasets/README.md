# m010 datasets (replay fixtures)

These fixtures are **deterministic inputs** for generating non-zero m010 KPI outputs **without MCP**.

## Files
- `schema.json` — JSON schema for replay datasets
- `fixtures/v0_sample.json` — sample events used by Heartbeat replay runner (all signals active)
- `fixtures/v0_challenge_sample.json` — sample events exercising the challenge workflow (signals with varied statuses + challenge events with resolutions)
- `fixtures/v0_challenge_escalated_sample.json` — challenge replay including `escalated` status and timeout KPI behavior
- `fixtures/v0_challenge_edge_timing_sample.json` — challenge replay covering boundary timing (including zero-hour resolution)
- `fixtures/v0_challenge_invalid_resolution_sample.json` — intentionally invalid fixture for negative verification coverage
- `fixtures/v0_challenge_invalid_outcome_sample.json` — intentionally invalid fixture with status/outcome mismatch for negative verification coverage

## How they are used
A replay runner (e.g., in `regen-heartbeat`) can read a fixture file and compute:
- `signals_emitted` = number of events in the fixture
- `subjects_touched` = unique `(subject_type, subject_id)`
- `evidence_coverage_rate` = fraction of events with **both** `koi_links[]` and `ledger_refs[]` non-empty
- `median_event_latency_hours` = median of `(as_of - timestamp)` in hours

The challenge fixture (`v0_challenge_sample.json`) additionally exercises:
- Status-aware scoring: only `active` and `resolved_valid` signals contribute to score
- Challenge KPIs: `challenges_filed`, `challenge_rate` (`challenges_filed / signals_emitted`), `challenge_success_rate`, `avg_resolution_time_hours`
- The `expected_outputs` field documents which signals should be included/excluded

These datasets are **advisory-only** and do not imply enforcement or on-chain actions.

## Integrity checks
Dataset integrity is validated by `scripts/verify-m010-datasets.mjs`, including:
- challenge-to-signal linkage (`challenge.signal_id` must reference an existing signal)
- category consistency between challenge and targeted signal
- resolution timestamp ordering and resolution presence rules by challenge status
- consistency of `expected_outputs.contributing_signals` / `excluded_signals` with status-based contribution rules
- negative checks that intentionally invalid fixtures fail with the expected validation reason
