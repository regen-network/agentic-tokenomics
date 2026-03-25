# M011 — Marketplace Curation & Quality Signals

## 0. Header

| Field | Value |
|-------|-------|
| **Mechanism ID** | m011 |
| **Name** | Marketplace Curation & Quality Signals |
| **Status** | Draft — v0 advisory |
| **Owner** | Regen Network core / community |
| **Scope** | Credit marketplace quality scoring, curated collections, and price discovery |

---

## 1. Problem

The existing credit marketplace lacks quality differentiation. Buyers cannot easily distinguish high-integrity credits from low-quality ones. Without quality signals, curated collections, or price discovery mechanisms, the marketplace suffers from information asymmetry that depresses prices for high-quality credits and erodes buyer confidence. Curators who invest effort in vetting credits have no economic incentive mechanism.

---

## 2. Target actor and action

| Actor | Action |
|-------|--------|
| **Curator** | Creates collections of vetted credit batches, bonds REGEN, earns trade fee share |
| **AGENT-003** | Computes quality scores for credit batches; monitors pricing; flags quality violations |
| **Buyer** | Purchases credits through curated collections (higher confidence) |
| **Seller** | Lists credits on marketplace (listing fee) |
| **Challenger** | Disputes curator inclusion of low-quality batches |

---

## 3. Signal definition

### Quality Score

The agent produces a quality assessment for each credit batch:

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `score` | integer | 0–1000 | 7-factor weighted quality composite |
| `confidence` | integer | 0–1000 | Data availability indicator |
| `factors` | object | — | Individual factor scores |

---

## 4. Evidence inputs

| Input | Source | Required |
|-------|--------|----------|
| Project reputation score | M010 reputation signal | Preferred |
| Credit class reputation score | M010 reputation signal | Preferred |
| Seller reputation score | M010 reputation signal | Optional |
| Batch start date (vintage) | x/ecocredit batch metadata | Yes |
| Last verification timestamp | Most recent MsgCreateBatch or attestation | Preferred |
| Listing price | x/ecocredit/marketplace sell order | If listed |
| Class price median | AGENT-003 price tracking | Optional |
| Methodology metadata IRI | Credit class metadata | Optional |

---

## 5. Scoring function

### Quality score

```
score = 0.25 × project_reputation
      + 0.20 × class_reputation
      + 0.15 × vintage_freshness
      + 0.15 × verification_recency
      + 0.10 × seller_reputation
      + 0.10 × price_fairness
      + 0.05 × additionality_confidence
```

| Factor | Weight | Description | Range |
|--------|--------|-------------|-------|
| `project_reputation` | 0.25 | M010 reputation score for the project | 0–1000 |
| `class_reputation` | 0.20 | M010 reputation score for the credit class | 0–1000 |
| `vintage_freshness` | 0.15 | Linear decay: 1000 at issuance, 0 at 10 years | 0–1000 |
| `verification_recency` | 0.15 | Linear decay: 1000 at last verification, 0 at 3 years | 0–1000 |
| `seller_reputation` | 0.10 | M010 reputation score for the seller address | 0–1000 |
| `price_fairness` | 0.10 | 1000 at class median, decaying by deviation (0 at ±50%) | 0–1000 |
| `additionality_confidence` | 0.05 | Methodology additionality assessment from credit class metadata | 0–1000 |

All factors are clamped to [0, 1000]. Final score is rounded to the nearest integer.

### Confidence

Confidence reflects data availability across seven boolean signals:

| Signal | Check |
|--------|-------|
| `project_reputation_available` | Project has M010 reputation score |
| `class_reputation_available` | Credit class has M010 reputation score |
| `seller_reputation_available` | Seller has M010 reputation score |
| `vintage_known` | Batch start date is available |
| `verification_date_known` | Last verification timestamp available |
| `price_data_available` | Listing price and class median available |
| `methodology_available` | Credit class methodology metadata accessible |

`confidence = round(count(true signals) / 7 × 1000)`

### Price fairness calculation

```
median = class_batch_price_median(credit_type, vintage_year)
deviation = |listing_price - median| / median
price_fairness = max(0, round((1.0 - deviation * 2) × 1000))
```

At median → 1000; 25% deviation → 500; ≥50% deviation → 0.

---

## 6. State machine

### Collection lifecycle

```
PROPOSED → ACTIVE → UNDER_REVIEW → ACTIVE (curator wins)
                                  → SUSPENDED (challenger wins) → ACTIVE (top-up)
                                                                → CLOSED
         → CLOSED (curator exit, no pending challenges)
```

| Transition | Trigger | Guard |
|------------|---------|-------|
| → PROPOSED | `curator.create_collection(name, criteria, bond)` | bond ≥ min_curation_bond |
| PROPOSED → ACTIVE | `activation_delay(48h) AND no_challenge` | — |
| ACTIVE → ACTIVE (add) | `curator.add_to_collection(batch_denom)` | meets criteria, quality ≥ min_quality_score |
| ACTIVE → ACTIVE (remove) | `curator.remove_from_collection(batch_denom)` | batch in collection |
| ACTIVE → UNDER_REVIEW | `challenger.challenge_inclusion(evidence) OR agent.flag_violation` | deposit ≥ challenge_deposit |
| UNDER_REVIEW → ACTIVE | `challenge.resolved(CURATOR_WINS)` | challenger deposit slashed |
| UNDER_REVIEW → SUSPENDED | `challenge.resolved(CHALLENGER_WINS)` | batch removed, bond slashed |
| SUSPENDED → ACTIVE | `curator.top_up_bond() AND suspension_expired` | bond ≥ min_curation_bond |
| SUSPENDED → CLOSED | `timeout(top_up_window) OR curator.close()` | remaining bond refunded |
| ACTIVE → CLOSED | `curator.close_collection()` | no pending challenges, unbonding period |

---

## 7. Token flows

### Collection creation

```
Curator ──(curation_bond: 1000 REGEN)──→ Bond Pool
```

### Trade through collection

```
Buyer ──(purchase_price)──→ Marketplace
Marketplace ──(price − trade_fee − curation_fee)──→ Seller
Marketplace ──(curation_fee: 0.5% of trade)──→ Curator
Marketplace ──(trade_fee)──→ Community Pool (via M013)
```

### Challenge resolution — Challenger wins

```
Bond Pool ──(bond × slash_percentage)──→ slash
  slash ──(50%)──→ Challenger (reward)
  slash ──(50%)──→ Community Pool
```

### Challenge resolution — Curator wins

```
Challenge Deposit ──(100%)──→ Community Pool (challenger loses deposit)
```

### Curator exit

```
Bond Pool ──(remaining bond)──→ Curator (after unbonding_period, 14 days)
```

### Governance parameters

| Parameter | Default | Range |
|-----------|---------|-------|
| `min_curation_bond` | 1,000 REGEN | 100–10,000 |
| `listing_fee` | 10 REGEN | 0–100 |
| `curation_fee_rate` | 0.5% (50 bps) | 0–2% |
| `challenge_deposit_amount` | 100 REGEN | 10–1,000 |
| `slash_percentage` | 20% of bond | 5–50% |
| `challenge_reward_share` | 50% of slash | 25–75% |
| `activation_delay` | 48 hours | 24h–7d |
| `unbonding_period` | 14 days | 7–30 days |
| `bond_top_up_window` | 7 days | 3–14 days |
| `min_quality_score` | 300 | 100–500 |
| `max_collections_per_curator` | 5 | 1–20 |
| `quality_refresh_interval` | 24 hours | 6h–7d |

---

## 8. Featured batches

AGENT-003 can recommend batches for featured placement:

| Criteria | Threshold |
|----------|-----------|
| Quality score | ≥ 800 (top 20%) |
| Pending challenges | None |
| Active sell orders | > 0 |

Featured status lasts 7 days (renewable). Agent recommends (Layer 1), curator confirms (Layer 2).

---

## 9. Security invariants

1. **Bond Conservation**: `bond_pool.balance = sum(active_collection_bonds) + sum(pending_refunds)` at all times.
2. **Criteria Enforcement**: Every batch in a collection must satisfy the collection's `CurationCriteria` at addition time; AGENT-003 re-verifies daily for drift.
3. **Curator Authority**: Only the collection curator can add/remove batches; agents cannot modify collection membership directly.
4. **Slash Cap**: Cumulative slashing cannot exceed the curator's total bond. If `bond_remaining < min_curation_bond` after a slash, the collection transitions to SUSPENDED. If the bond is not topped up within `bond_top_up_window`, the collection then transitions to CLOSED.
5. **No Self-Challenge**: Curator cannot challenge own collection; challenger cannot be seller of challenged batch.
6. **Fee Integrity**: Curation fees only on trades through a collection; direct trades incur no curation fee.
7. **Quality Score Immutability**: Scores are append-only (superseded, never edited); full history preserved.
8. **Collection Uniqueness**: Each `(collection_id, batch_denom)` pair is unique; a batch can belong to multiple collections.

---

## 10. Attack model

| Attack | Mitigation |
|--------|-----------|
| **Low-quality curation** | Challenge mechanism slashes curator bond; AGENT-003 flags quality violations |
| **Sybil curators** | min_curation_bond (1000 REGEN) makes Sybil attacks expensive |
| **Challenge spam** | Challenge deposit (100 REGEN) lost if challenge fails |
| **Price manipulation** | AGENT-003 detects outlier pricing (z-score > 2.5) |
| **Wash trading for fees** | Fee rate is small (0.5%); on-chain detection via AGENT-003 |
| **Quality score gaming** | 7-factor scoring with diverse inputs; immutable score history |
| **Curator front-running** | Activation delay (48h) prevents immediate inclusion of batches |
| **Bond grinding** | max_collections_per_curator limits; unbonding_period prevents rapid cycling |

---

## 11. Integration points

| System | Integration |
|--------|-------------|
| **M010 Reputation** | Project, class, and seller reputation scores as quality inputs |
| **M013 Fee Routing** | Trade fees follow M013 distribution model |
| **AGENT-003** | Autonomous quality scoring, price monitoring, collection monitoring |
| **x/ecocredit** | Batch metadata (vintage, class, project), marketplace sell orders |
| **x/ecocredit/basket** | Basket token quality scoring (constituent analysis) |
| **KOI MCP** | Methodology metadata analysis, additionality confidence |
| **Ledger MCP** | Bond balance queries, trade volume tracking |
| **Heartbeat** | KPI metrics published in weekly digest |

---

## 12. Acceptance tests

1. **Collection lifecycle**: Curator bonds → creates collection → adds batches → earns trade fees → closes → bond refunded.
2. **Quality scoring**: AGENT-003 scores batch → score meets criteria → batch eligible for collection inclusion.
3. **Challenge — challenger wins**: Challenger deposits → challenge resolved in challenger's favor → batch removed → curator bond slashed → challenger rewarded.
4. **Challenge — curator wins**: Challenge resolved in curator's favor → challenger deposit slashed to community pool.
5. **Suspension and recovery**: Curator slashed below min bond → collection SUSPENDED → curator tops up bond → collection ACTIVE.
6. **Suspension and closure**: Curator slashed → top_up_window expires → collection CLOSED → remaining bond refunded.
7. **Featured batch**: Quality score ≥ 800, no challenges, active orders → featured for 7 days.
8. **Price fairness**: Listing at median → 1000; listing 50% above median → 0.
9. **Max collections**: Curator at max_collections_per_curator cannot create new collection.

---

## 13. Rollout plan

| Phase | Scope | Dependencies |
|-------|-------|-------------|
| **v0 (advisory)** | AGENT-003 computes quality scores off-chain; KPI metrics in digest. No on-chain curation bonds or collections. | M010 (reputation), KOI MCP |
| **v1 (on-chain curation)** | CosmWasm `marketplace-curation` contract with collection creation, bond lock, batch inclusion, challenge mechanism. Quality scores submitted on-chain. | M010, AGENT-003, DAO DAO |
| **v2 (full marketplace)** | Curation fee routing via M013, featured batches, collection performance reports, basket token support. | M013, x/ecocredit/basket |

---

## Appendix: Source anchors

| Document | Section |
|----------|---------|
| `phase-2/2.1-token-utility-mechanisms.md` | M011 Marketplace Curation & Quality Signals (lines 650–908) |
| `phase-3/3.1-smart-contract-specs.md` | CosmWasm Contract: Marketplace Curation (lines 1080–1155) |
