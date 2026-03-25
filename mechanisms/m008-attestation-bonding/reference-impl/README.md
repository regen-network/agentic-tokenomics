# m008 reference implementation (v0 advisory)

This folder provides a **canonical computation** for m008 outputs so that different agents/runners
produce consistent numbers.

## Inputs

### Scoring (`m008_score.js`)
An input object per attestation:

- `attestation` — attestation metadata:
  - `attestation_type` (string, one of ProjectBoundary/BaselineMeasurement/CreditIssuanceClaim/MethodologyValidation)
  - `attestation_iri` (string)
  - `bond` (object with `amount` and `denom`)
- `factors` — pre-computed factor scores (each 0–1000):
  - `bond_adequacy` — bond amount relative to minimum
  - `attester_reputation` — M010 reputation score (default 300 if unavailable)
  - `evidence_completeness` — document completeness checklist
  - `type_risk` — attestation type risk factor
  - `reputation_available` (boolean), `iri_resolvable` (boolean), `has_prior_attestations` (boolean), `type_recognized` (boolean)

### KPI (`m008_kpi.js`)
- `as_of` (ISO-8601 string, Z-suffixed)
- `attestations[]` — each with `status`, `attestation_type`, `bond`, `quality_score` (optional)

## Outputs

### Score
- `score` (0–1000) — weighted composite
- `confidence` (0–1000) — data availability
- `factors` — individual factor scores

Formula:
```
score = 0.30 × bond_adequacy + 0.30 × attester_reputation + 0.25 × evidence_completeness + 0.15 × type_risk
```

### KPI block
- Attestation counts by status (submitted, active, released, challenged, slashed)
- `challenge_rate`, `slash_rate`
- Bond economics (total bonded/released/slashed, average)
- Average quality score, type breakdown
