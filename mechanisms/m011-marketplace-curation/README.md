# m011 — Marketplace Curation & Quality Signals

Quality signals, curated collections, and price discovery for the credit marketplace. Curators stake REGEN to create vetted credit batch collections and earn trade fee shares. AGENT-003 provides automated 7-factor quality scoring.

## What it outputs

- **Quality score** (0–1000): 7-factor weighted assessment (project reputation 0.25, class reputation 0.20, vintage freshness 0.15, verification recency 0.15, seller reputation 0.10, price fairness 0.10, additionality confidence 0.05)
- **Confidence** (0–1000): Data availability across 7 input signals
- **KPI block**: Collection counts by status, quality score distribution, trade volume through collections, curation economics, challenge rate

## What it does not do in v0

- No on-chain curation bonds or collection contracts
- No automated trade fee splitting to curators
- No on-chain challenge mechanism
- Agent quality scores are advisory only — published in digest

## Scoring formula

```
score = 0.25 × project_reputation + 0.20 × class_reputation + 0.15 × vintage_freshness
      + 0.15 × verification_recency + 0.10 × seller_reputation + 0.10 × price_fairness
      + 0.05 × additionality_confidence
```

## How to reference

- **Canonical spec**: `mechanisms/m011-marketplace-curation/SPEC.md`
- **Schemas**: `mechanisms/m011-marketplace-curation/schemas/`
- **Reference implementation**: `mechanisms/m011-marketplace-curation/reference-impl/`

## Replay datasets

- `datasets/fixtures/v0_sample.json` — Credit batches across multiple classes with quality scores
- `datasets/fixtures/v0_collection_sample.json` — Collection lifecycle scenarios including challenges
