# m011 output schemas

These JSON Schemas define **canonical output shapes** for m011 (Marketplace Curation & Quality Signals) artifacts.

## Files
- `m011_quality_score.schema.json` — schema for credit batch quality score output (score, confidence, 7-factor breakdown).
- `m011_collection.schema.json` — schema for curated collection lifecycle objects (bond, members, criteria, challenge details).
- `m011_kpi.schema.json` — schema for KPI metrics (collection counts, quality distribution, curation economics, challenge rate).

## Notes
- These schemas are intended for **validation** and consistency across repos (Heartbeat, agent skills, etc.).
- v0 is advisory-only: schemas describe outputs, not enforcement.
- The `status` field on collections tracks lifecycle state (PROPOSED → ACTIVE → UNDER_REVIEW/SUSPENDED → CLOSED). See SPEC.md section 6.
- Quality scores use 7 weighted factors from diverse data sources (M010 reputation, x/ecocredit metadata, AGENT-003 pricing).
