# m008 replay datasets

Fixture files for replay testing of m008 (Data Attestation Bonding) computations.

## Files
- `schema.json` — JSON Schema for the replay dataset format.
- `fixtures/v0_sample.json` — Five attestations across all types, with quality scores matching reference-impl output. Includes ACTIVE, RELEASED, and SUBMITTED statuses.
- `fixtures/v0_challenge_sample.json` — Six attestations covering challenge and resolution scenarios: CHALLENGED (pending), SLASHED, RESOLVED_VALID, RESOLVED_INVALID, ACTIVE, and RELEASED.

## Usage
Feed fixture files into `m008_kpi.js` to verify KPI computation. Quality scores in fixtures correspond to `m008_score.js` output for the matching factor inputs.
