# m010 datasets (replay fixtures)

These fixtures are **deterministic inputs** for generating non-zero m010 KPI outputs **without MCP**.

## Files
- `schema.json` — JSON schema for replay datasets
- `fixtures/v0_sample.json` — sample events used by Heartbeat replay runner (all signals active)
- `fixtures/v0_challenge_sample.json` — sample events exercising the challenge workflow (signals with varied statuses + challenge events with resolutions)

## How they are used
A replay runner (e.g., in `regen-heartbeat`) can read a fixture file and compute:
- `signals_emitted` = number of events in the fixture
- `subjects_touched` = unique `(subject_type, subject_id)`
- `evidence_coverage_rate` = fraction of events with **both** `koi_links[]` and `ledger_refs[]` non-empty
- `median_event_latency_hours` = median of `(as_of - timestamp)` in hours

The challenge fixture (`v0_challenge_sample.json`) additionally exercises:
- Status-aware scoring: only `active` and `resolved_valid` signals contribute to score
- Challenge KPIs: `challenges_filed`, `challenge_rate`, `challenge_success_rate`, `avg_resolution_time_hours`
- The `expected_outputs` field documents which signals should be included/excluded

These datasets are **advisory-only** and do not imply enforcement or on-chain actions.
