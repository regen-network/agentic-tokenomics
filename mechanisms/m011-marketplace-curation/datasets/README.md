# m011 replay datasets

Fixture files for replay testing of m011 (Marketplace Curation & Quality Signals) computations.

## Files
- `schema.json` — JSON Schema for the replay dataset format.
- `fixtures/v0_sample.json` — Five scored credit batches across carbon (C), biodiversity (BT), and Kasigau (KSH) types, with three curated collections (ACTIVE and CLOSED). Quality scores match reference-impl output.
- `fixtures/v0_collection_sample.json` — Three collections covering challenge lifecycle scenarios: UNDER_REVIEW (pending), SUSPENDED (challenger wins), ACTIVE (curator wins, challenge resolved).

## Usage
Feed fixture files into `m011_kpi.js` to verify KPI computation. Quality scores in fixtures correspond to `m011_score.js` output for the matching factor inputs.
