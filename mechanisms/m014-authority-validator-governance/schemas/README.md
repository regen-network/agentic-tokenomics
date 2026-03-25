# m014 output schemas

These JSON Schemas define **canonical output shapes** for m014 (Authority Validator Governance) artifacts.

## Files
- `m014_validator.schema.json` — schema for individual authority validators (address, category, lifecycle status, term, performance).
- `m014_performance.schema.json` — schema for computed performance score output (composite score, confidence, factor breakdown, flags).
- `m014_kpi.schema.json` — schema for the KPI JSON block emitted by agents/digests (validator counts by status/category, average performance, compensation stats, composition validity).

## Notes
- These schemas are intended for **validation** and consistency across repos (Heartbeat, agent skills, etc.).
- v0 is specification-only: schemas describe outputs, not enforcement.
- The `status` field on validators tracks lifecycle state (candidate, approved, active, probation, removed, term_expired). See SPEC.md section 6.1.
- Performance scoring uses a 3-factor weighted model: uptime (0.4) + governance participation (0.3) + ecosystem contribution (0.3). See SPEC.md section 5.
