# m008 output schemas

These JSON Schemas define **canonical output shapes** for m008 (Data Attestation Bonding) artifacts.

## Files
- `m008_attestation.schema.json` — schema for attestation lifecycle objects (bond, status, challenge details).
- `m008_quality_score.schema.json` — schema for attestation quality score output (score, confidence, factor breakdown).
- `m008_kpi.schema.json` — schema for KPI metrics (attestations submitted, challenge rate, bond economics, quality scores).

## Notes
- These schemas are intended for **validation** and consistency across repos (Heartbeat, agent skills, etc.).
- v0 is advisory-only: schemas describe outputs, not enforcement.
- The `status` field on attestations tracks lifecycle state (BONDED → ACTIVE → CHALLENGED → RESOLVED/RELEASED/SLASHED). See SPEC.md section 6.
- Attestation types are: ProjectBoundary, BaselineMeasurement, CreditIssuanceClaim, MethodologyValidation.
