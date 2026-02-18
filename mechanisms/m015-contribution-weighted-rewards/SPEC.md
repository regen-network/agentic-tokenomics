# m015 — Contribution-Weighted Rewards (SPEC)

## 0. Header
- **ID:** m015
- **Name:** Contribution-Weighted Rewards
- **Status:** draft
- **Owner:** (unset)
- **Last updated:** 2026-02-18
- **Scope:** Replace passive staking rewards with activity-based distribution from Community Pool
- **Dependencies:** M013 (Value-Based Fee Routing — Community Pool inflow), M014 (Authority Validator Governance — staking rewards disabled)

## 1. Problem
Traditional proof-of-stake networks reward token holders simply for locking tokens, creating passive income with no connection to ecological value creation. In a network dedicated to ecological outcomes, rewards should flow to participants who actively contribute to the ecosystem: purchasing credits, retiring credits, facilitating transactions, participating in governance, and submitting proposals. M015 replaces passive staking yield with a contribution-weighted distribution system funded by Community Pool inflow.

## 2. Target actor and action
- **Actors:** credit purchasers, credit retirers, platform operators (brokers/marketplaces), governance participants (voters, proposers), long-term holders (stability tier).
- **Action being evaluated:** ecological and governance contributions per distribution period (epoch).
- **Event source:** on-chain transactions for credit purchases, retirements, platform facilitation, governance votes, and proposal submissions — all sourced from chain state within the epoch.

## 3. Signal definition
- **Signal name:** Activity Score
- **Unit:** weighted sum (dimensionless; higher = more contribution)
- **Directionality:** higher = more reward
- **Granularity:** per participant address per epoch
- **Epoch:** weekly distribution period (52 periods per year)

## 4. Evidence inputs

| Input | Source | Fields | Validity rules | Anti-spoof assumptions | Refresh cadence |
|---|---|---|---|---|---|
| Credit purchase value | Ledger (ecocredit module) | `buyer_address`, `credit_value_uregen`, `tx_hash` | On-chain transaction only; min transaction size may apply | M013 fees make wash trading unprofitable | Per epoch |
| Credit retirement value | Ledger (ecocredit module) | `retirer_address`, `credit_value_uregen`, `tx_hash` | On-chain retirement event | Terminal action; cannot be reversed | Per epoch |
| Platform facilitation value | Ledger (marketplace/broker metadata) | `facilitator_address`, `facilitated_value_uregen`, `tx_hash` | Registered dApp address or originating API key (OQ-M015-2) | Platform registration required | Per epoch |
| Governance votes cast | Ledger (x/gov module) | `voter_address`, `proposal_id`, `vote_option` | Vote recorded on-chain | One vote per proposal per address | Per epoch |
| Proposals submitted | Ledger (x/gov module) | `proposer_address`, `proposal_id`, `status`, `reached_quorum` | Deposit paid via x/gov; status determines weight | Deposit mechanism provides natural friction | Per epoch |
| Stability commitments | Ledger (x/rewards or x/distribution) | `holder_address`, `committed_amount_uregen`, `lock_period_months`, `committed_at` | Min 100 REGEN, min 6 month lock, max 24 month lock | Tokens locked on-chain | On commitment/maturity |
| Community Pool inflow | M013 output | `community_share`, `total_fees_collected` | Computed by M013 per epoch | M013 is a prerequisite; inflow verified on-chain | Per epoch |

## 5. Scoring function

### 5.1 Activity weights

| Activity | Key | Weight | Rationale |
|----------|-----|--------|-----------|
| Credit Purchase | `credit_purchase` | 0.30 | Primary demand signal; directs resources toward ecological outcomes |
| Credit Retirement | `credit_retirement` | 0.30 | Terminal impact event; strongest ecological commitment |
| Platform Facilitation | `platform_facilitation` | 0.20 | Enables ecosystem growth; rewards infrastructure builders |
| Governance Voting | `governance_voting` | 0.10 | Maintains governance participation incentive |
| Proposal Submission | `proposal_submission` | 0.10 | Rewards initiative; conditional on quorum (see 5.2) |

### 5.2 Proposal anti-gaming

To prevent spam proposals from inflating activity scores, proposal submission credit is conditional:

| Proposal outcome | Effective weight | Rationale |
|---|---|---|
| Passed quorum and approved | Full weight (0.10) | Genuine contribution to governance |
| Reached quorum but failed | 50% weight (0.05 effective) | Meaningful attempt; engaged voters |
| Failed to reach quorum | 0 weight | No demonstrated community engagement; potential spam |

This leverages the existing x/gov deposit mechanism as natural friction.

### 5.3 Reward calculation

```
For each distribution period (epoch):

  community_pool_inflow = community_share * total_fees_collected  (from M013)

  # 1. Stability Tier Allocation (first priority)
  stability_allocation = min(
    sum(commitment.amount * 0.06 / periods_per_year for each commitment),
    community_pool_inflow * max_stability_share
  )
  max_stability_share = 0.30   # At most 30% of Community Pool goes to stability tier
  periods_per_year = 52        # Weekly epochs

  # 2. Remaining Pool for Activity-Based Distribution
  activity_pool = community_pool_inflow - stability_allocation

  # 3. Activity Scoring
  For each participant p in period:
    proposal_credit = sum(
      1.0 if proposal.passed and proposal.reached_quorum,
      0.5 if not proposal.passed and proposal.reached_quorum,
      0.0 if not proposal.reached_quorum
      for each proposal submitted by p in period
    )

    activity_score[p] = (
      credit_purchase_value[p] * 0.30 +
      credit_retirement_value[p] * 0.30 +
      platform_facilitation_value[p] * 0.20 +
      governance_votes_cast[p] * 0.10 +
      proposal_credit * 0.10
    )

  # 4. Distribution
  total_score = sum(activity_score[p] for all p)
  reward[p] = activity_pool * (activity_score[p] / total_score)
```

### 5.4 Design note: scoring units

The activity score formula intentionally combines monetary values (uregen) with
counts (votes, proposal credits) without normalization. This means monetary
activities (credit purchases/retirements/facilitation at 0.80 combined weight)
will dominate over governance activities (votes/proposals at 0.20 combined weight)
in absolute score terms. This is deliberate: M015's primary objective is
incentivizing ecological transactions (the network's core value proposition),
with governance participation as a secondary but still-rewarded signal.

v1 may introduce normalization (e.g., log-scaling monetary values, or normalizing
all inputs to a common 0–1 range per epoch) if WG analysis shows governance
participation is insufficiently incentivized. See OQ-M015-4 for related
anti-gaming measures that may also address score balance.

### 5.5 Controls
- **Revenue constraint:** total distributions per period <= Community Pool inflow for that period.
- **Stability cap:** stability tier allocation <= 30% of Community Pool inflow.
- **Minimum transaction size:** governance may set a minimum transaction value for reward eligibility (OQ-M015-4).
- **Governance override:** all distribution parameters changeable via Layer 3 governance at any time.

## 6. Stability Tier

The stability tier is an optional commitment mechanism for long-term holders who want predictable returns without active participation. Holders lock REGEN for a minimum period and receive a fixed annual return from the Community Pool.

### 6.1 Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| `annual_return` | 6% | OQ-M015-1: sustainability depends on Community Pool inflow |
| `min_lock_period` | 6 months | |
| `max_lock_period` | 24 months | |
| `min_commitment` | 100 REGEN | 100,000,000 uregen |
| `early_exit_penalty` | 50% of accrued rewards forfeited | |
| `max_stability_share` | 30% of Community Pool inflow per period | |
| `overflow_handling` | New commitments queued; filled in order when capacity available | |

### 6.2 Stability tier lifecycle

```
States: {UNCOMMITTED, COMMITTED, MATURED, EARLY_EXIT}

UNCOMMITTED -> COMMITTED
  trigger: holder.commit_stability(amount, lock_period)
  guard: amount >= 100 REGEN, lock_period >= 6 months, lock_period <= 24 months
  action: lock tokens, begin accruing 6% annual return

COMMITTED -> MATURED
  trigger: lock_period_expired
  action: unlock tokens + accrued rewards (full amount)

COMMITTED -> EARLY_EXIT
  trigger: holder.exit_early()
  action: unlock tokens, forfeit 50% of accrued rewards
```

## 7. State transitions (mechanism lifecycle)

```
States: {INACTIVE, TRACKING, DISTRIBUTING}

INACTIVE -> TRACKING
  trigger: governance.approve(m015_activation_proposal)
  guard: M013 active (Community Pool receiving fee revenue)
  action: begin recording activity scores per address per period

TRACKING -> DISTRIBUTING
  trigger: tracking_period_complete(3 months) AND sufficient_data
  guard: activity_score data validated, no anomalies detected
  action: begin automatic distribution at end of each period
  note: 3-month tracking-only period allows calibration before payouts
```

## 8. Economic linkage
- **Funding source:** Community Pool inflow from M013 fee routing. No new token minting.
- **Stability tier:** capped at 30% of inflow; provides predictable return for committed holders.
- **Activity pool:** remaining 70%+ of inflow; distributed proportional to activity scores.
- **Fee friction:** M013 fees on every transaction make wash trading unprofitable (fees paid > rewards earned for circular transactions).
- **Governance-directed spending:** separate from automatic M015 distribution. The Community Pool may still fund governance proposals (GOV-004) from funds not allocated to M015 (OQ-M015-3).

## 9. On-chain vs off-chain boundary
- **On-chain:** all activity inputs (purchases, retirements, facilitation, votes, proposals), stability commitments, token locks, distribution payouts.
- **Off-chain (optional):** activity score aggregation at period boundaries may use off-chain indexer with on-chain anchoring for gas efficiency. Score computation is deterministic and verifiable.
- **Module:** new `x/rewards` module or extend `x/distribution`.
- **Storage:** `ActivityScore` per address per period, `StabilityCommitment`, `DistributionRecord`, `RewardConfig`.
- **Events:** `EventActivityRecorded`, `EventRewardDistributed`, `EventStabilityCommitted`, `EventStabilityMatured`.

## 10. Attack model
1. **Sybil / wash trading:** M013 fee extraction on every transaction ensures circular trading costs more in fees than it earns in rewards. Activity weights further dilute wash-trading benefit across all participants.
2. **Proposal spam:** x/gov deposit requirement provides friction. Proposals failing to reach quorum earn 0 weight. Only passed proposals earn full 0.10 weight.
3. **Stability tier gaming:** minimum 6-month lock and 50% early exit penalty prevent short-term arbitrage. 30% cap prevents stability commitments from consuming all rewards.
4. **Score manipulation via small transactions:** governance may set minimum transaction sizes for reward eligibility (OQ-M015-4). Per-transaction fees from M013 provide natural lower bound.
5. **Double-counting:** each transaction counted once. Marketplace trades: buyer gets credit (demand signal). Seller activity captured via issuance/facilitation. Credit retirements: retirer gets credit.
6. **Collusion:** large coordinated groups cannot extract more than their proportional share; the system is purely pro-rata on activity scores.

## 11. Integration points
- **M013 (Value-Based Fee Routing):** provides Community Pool inflow that funds M015 distributions.
- **M014 (Authority Validator Governance):** disables passive staking rewards, making M015 the primary reward mechanism.
- **x/gov module:** source of governance voting and proposal data.
- **x/ecocredit module:** source of credit purchase and retirement data.
- **Marketplace module:** source of platform facilitation data.
- **AGENT-003 (Market Monitor):** can track activity patterns and flag anomalies.
- **KOI knowledge layer:** may reference distribution records and activity summaries.

## 12. Acceptance tests

**Activity scoring:**
1) Participant with only credit purchases receives score = purchase_value * 0.30.
2) Participant with all five activity types receives correct weighted sum.
3) Proposal that passed quorum earns full 0.10 weight per proposal credit.
4) Proposal that reached quorum but failed earns 0.05 effective weight.
5) Proposal that failed to reach quorum earns 0 weight.
6) Participant with zero activity receives 0 score and 0 reward.

**Distribution:**
7) Sum of all participant rewards equals activity_pool (no leakage, no excess).
8) Each participant's reward is proportional to their share of total activity score.
9) Stability allocation is capped at 30% of community_pool_inflow.
10) When stability demand < cap, exact 6%/periods_per_year accrual is paid.
11) Activity pool = community_pool_inflow - stability_allocation.

**Stability tier:**
12) Commitment with amount < 100 REGEN is rejected.
13) Commitment with lock_period < 6 months is rejected.
14) Commitment with lock_period > 24 months is rejected.
15) Matured commitment returns full accrued rewards.
16) Early exit forfeits 50% of accrued rewards.

**Security invariants:**
17) Total distributions per period <= Community Pool inflow for that period (revenue constraint).
18) No self-reported activity contributes to score (on-chain only).
19) Each transaction counted exactly once for reward scoring (no double-counting).
20) Distribution parameters changeable via governance at any time (governance override).

**State machine:**
21) Mechanism starts INACTIVE; transitions to TRACKING only via governance approval with M013 active.
22) TRACKING transitions to DISTRIBUTING after 3-month calibration period with validated data.
23) Stability commitment transitions UNCOMMITTED -> COMMITTED -> MATURED on lock expiry.
24) Stability commitment transitions COMMITTED -> EARLY_EXIT on early withdrawal with penalty.

## 13. Rollout plan

### v0 checklist (off-chain scoring)
- Implement activity score computation using on-chain data (purchases, retirements, facilitation, votes, proposals).
- Implement stability tier allocation calculation with 30% cap.
- Implement distribution calculation (pro-rata on activity scores).
- Validate against test vectors (see `reference-impl/test_vectors/`).
- Publish activity scores and distribution projections in periodic digest (no actual payouts in v0).
- 3-month tracking period for calibration and anomaly detection.

### v1 outline (on-chain distribution)
- Deploy `x/rewards` module (or extend `x/distribution`) with automated epoch-based distribution.
- Implement stability tier commitment/lock/maturity/early-exit on-chain.
- Integrate with M013 for automated Community Pool inflow routing.
- Add governance-controlled parameter updates for weights, caps, and lock periods.
- Agent integration for anomaly detection and activity pattern monitoring.

---

## Appendix A -- Source anchors
- `phase-2/2.6-economic-reboot-mechanisms.md` lines 535--701
  - "PROTOCOL SPECIFICATION: M015" (purpose, design philosophy, participants, reward calculation)
  - "Activity Weights" table and proposal anti-gaming rules
  - "Stability Tier" parameters and lifecycle
  - "Token Flows" diagram
  - "State Transitions" (mechanism lifecycle and stability lifecycle)
  - "Security Invariants" (6 invariants)
  - "Open Questions" (OQ-M015-1 through OQ-M015-4)
  - "Implementation Notes" (module, storage, events, dependencies)

## Appendix B -- Open questions
> **OQ-M015-1**: Is 6% the right stability tier return? Must be sustainable from Community Pool inflows.

> **OQ-M015-2**: Should platform facilitation credit use metadata fields or originating API key / registered dApp address?

> **OQ-M015-3**: What share of Community Pool goes to automatic distribution vs. remaining available for governance proposals?

> **OQ-M015-4**: Are additional anti-gaming measures needed beyond M013 fee friction (e.g., minimum transaction size for reward eligibility)?
