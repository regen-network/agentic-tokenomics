# m010 reference implementation (v0 advisory)

This folder provides a **canonical computation** for m010 outputs so that different agents/runners
produce consistent numbers.

## Inputs
An input object:

- `as_of` (ISO-8601 string, Z-suffixed)
- `events[]` where each event includes:
  - `timestamp` (ISO-8601 Z)
  - `subject_type`, `subject_id`
  - `endorsement_level` (1..5)
  - `evidence.koi_links[]`
  - `evidence.ledger_refs[]`

## Outputs
### KPI block
- `signals_emitted` = count(events)
- `subjects_touched` = unique `(subject_type, subject_id)`
- `evidence_coverage_rate` = fraction of events with **both** KOI and Ledger evidence
- `median_event_latency_hours` = median(as_of - timestamp) in hours

### Score (optional)
A normalized `reputation_score_0_1` computed as:
1) per-event weight = endorsement_level / 5
2) time decay = exp(-lambda * age_hours) where lambda corresponds to ~14 day half-life
3) score = weighted average of decayed weights, normalized to [0,1]

This is advisory and intended for digest/reporting only (no enforcement).

### Status-aware scoring behavior
- If an event has no `status`, it is treated as contributing (legacy v0 fixtures).
- If `status` is present, only `active` and `resolved_valid` contribute to score.
- `submitted`, `challenged`, `escalated`, `resolved_invalid`, `withdrawn`, and `invalidated` are excluded from score contribution.

### Challenge KPI behavior
When `challenges[]` are provided to `computeM010KPI`, output includes `challenge_kpis`:
- `challenges_filed`
- `challenge_rate` = `challenges_filed / signals_emitted`
- `avg_resolution_time_hours`
- `challenge_success_rate`
- `admin_resolution_timeout_rate`

### Deterministic vectors
Reference vectors live in `test_vectors/` and are validated by:
- `node scripts/verify-m010-reference-impl.mjs`
- Coverage includes:
  - baseline replay (`v0_sample`)
  - mixed challenge statuses (`v0_challenge_sample`)
  - escalated challenge path (`v0_challenge_escalated_sample`)
  - edge timing path (`v0_challenge_edge_timing_sample`)
