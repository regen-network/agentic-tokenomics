# m010 output schemas

These JSON Schemas define **canonical output shapes** for m010 (Reputation Signal) artifacts.

## Files
- `m010_kpi.schema.json` — schema for the KPI JSON block emitted by agents/digests. Includes optional `challenge_kpis` for challenge workflow metrics.
- `m010_signal.schema.json` — schema for individual signal items (evidence + subject + timestamp + status).
- `m010_challenge.schema.json` — schema for challenge events filed against signals (see SPEC.md section 6).

## Notes
- These schemas are intended for **validation** and consistency across repos (Heartbeat, agent skills, etc.).
- v0 is advisory-only: schemas describe outputs, not enforcement.
- The `status` field on signals tracks lifecycle state (submitted → active → challenged → resolved). See SPEC.md section 6.1.
- Challenge events are separate from signals; they reference a `signal_id` and track their own resolution lifecycle.
