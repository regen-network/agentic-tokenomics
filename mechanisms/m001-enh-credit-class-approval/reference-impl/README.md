# m001-enh reference implementation (v0 advisory)

This folder provides a **canonical computation** for m001-enh outputs so that different agents/runners
produce consistent numbers.

## Inputs

### Scoring (`m001_score.js`)
An input object per proposal:

- `proposal` — proposal metadata:
  - `credit_type` (string, one of C/KSH/BT/MBS/USS)
  - `methodology_iri` (string)
  - `admin_address` (bech32 string)
- `factors` — pre-computed factor scores (each 0–1000):
  - `methodology_quality` — rigor across additionality, baseline, MRV, permanence
  - `admin_reputation` — M010 reputation score (default 500 if unavailable)
  - `novelty` — `(1 - max_similarity) × 1000`
  - `completeness` — application completeness checklist (250 per item)
  - `reputation_available` (boolean) — M010 score exists
  - `methodology_resolvable` (boolean) — IRI resolves
  - `sufficient_classes` (boolean) — >= 3 existing classes
  - `history_available` (boolean) — historical proposal data exists

### KPI (`m001_kpi.js`)
- `as_of` (ISO-8601 string, Z-suffixed)
- `proposals[]` where each includes:
  - `status` — DRAFT, AGENT_REVIEW, VOTING, APPROVED, REJECTED, EXPIRED
  - `agent_score` (optional) — `{ score, confidence, recommendation }`
  - `submit_time`, `decision_time` (ISO-8601 Z)
  - `outcome` (optional) — `{ result, deposit_refunded, deposit_slashed }`
  - `override` (optional) — `{ overrider, override_time, rationale }`

## Outputs

### Score
- `score` (0–1000) — weighted composite
- `confidence` (0–1000) — data availability
- `recommendation` — APPROVE, CONDITIONAL, or REJECT
- `factors` — individual factor scores

Formula:
```
score = 0.4 × methodology_quality + 0.3 × admin_reputation + 0.2 × novelty + 0.1 × completeness
```

### KPI block
- `proposals_submitted`, `proposals_approved`, `proposals_rejected`, `proposals_expired`
- `approval_rate` = approved / decided
- `agent_scoring` — scored count, accuracy, avg score, auto-reject count, override count
- `avg_time_to_decision_hours`

This is advisory and intended for digest/reporting only (no enforcement).
