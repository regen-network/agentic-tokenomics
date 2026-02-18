# m011 reference implementation (v0 advisory)

This folder provides a **canonical computation** for m011 outputs so that different agents/runners
produce consistent numbers.

## Inputs

### Scoring (`m011_score.js`)
An input object per credit batch:

- `batch` — batch metadata:
  - `batch_denom` (string)
  - `credit_type` (string, e.g., "C", "BT")
  - `class_id` (string)
  - `project_id` (string)
  - `seller` (string, optional)
- `factors` — pre-computed factor scores (each 0–1000):
  - `project_reputation` — M010 score for the project
  - `class_reputation` — M010 score for the credit class
  - `vintage_freshness` — linear decay: 1000 at issuance, 0 at 10 years
  - `verification_recency` — linear decay: 1000 at last verification, 0 at 3 years
  - `seller_reputation` — M010 score for the seller address
  - `price_fairness` — 1000 at median, 0 at ±50% deviation
  - `additionality_confidence` — methodology additionality score
  - Boolean availability signals: `project_reputation_available`, `class_reputation_available`, `seller_reputation_available`, `vintage_known`, `verification_date_known`, `price_data_available`, `methodology_available`

### KPI (`m011_kpi.js`)
- `as_of` (ISO-8601 string, Z-suffixed)
- `collections[]` — each with `status`, `bond`, `members`, `trade_volume`, `total_rewards`, `challenge` (optional)
- `scored_batches[]` — each with `quality_score`

## Outputs

### Score
- `score` (0–1000) — weighted composite
- `confidence` (0–1000) — data availability (7 signals)
- `factors` — individual factor scores

Formula:
```
score = 0.25 × project_reputation + 0.20 × class_reputation + 0.15 × vintage_freshness
      + 0.15 × verification_recency + 0.10 × seller_reputation + 0.10 × price_fairness
      + 0.05 × additionality_confidence
```

### KPI block
- Collection counts by status (active, closed, suspended, under_review)
- Batch scoring stats (scored, avg score, featured count)
- `challenge_rate`, `challenge_outcome_breakdown`
- Curation economics (total bonded/slashed/rewards/volume, avg collection size)
- Quality score distribution
