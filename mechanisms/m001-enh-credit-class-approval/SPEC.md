# m001-enh — Credit Class Approval Voting Enhancement (SPEC)

## 0. Header
- **ID:** m001-enh
- **Name:** Credit Class Approval Voting Enhancement
- **Status:** draft (advisory-only v0)
- **Owner:** (unset)
- **Last updated:** 2026-02-18
- **Scope:** **v0 advisory** (agent pre-screening score computation + publication only; no on-chain deposit escrow or PoA tally in v0)

## 1. Problem
The Regen Network credit class creator allowlist (managed via `x/ecocredit` governance proposals) currently uses a binary approve/reject model with no quality pre-screening. With 13 active credit classes across 5 credit types (C, KSH, BT, MBS, USS) and ~224M REGEN supply, the governance process for allowlist additions has several gaps:

1. **No quality signal**: Governance voters receive no structured assessment of proposal quality before voting. All proposals look identical regardless of methodology rigor.
2. **Binary threshold**: Proposals either pass or fail a simple majority vote with no tiered confidence levels or expedited paths for high-quality submissions.
3. **No deposit incentive**: Proposers have no skin-in-the-game beyond the standard governance deposit, creating no deterrent for low-effort or speculative applications.
4. **No agent integration**: The ecosystem's agent infrastructure (AGENT-001 Registry Agent) cannot contribute structured assessments to the governance process.
5. **Governance burden**: Every proposal requires full community discussion and voting regardless of quality, consuming scarce governance attention.

## 2. Target actor and action
- **Actors:** proposer (credit class creator applicant), Registry Agent (AGENT-001, autonomous pre-screener), validators/delegators (governance voters), admin (override authority in v0).
- **Action being evaluated (one action):** a **credit class creator application** submitted by a proposer requesting addition to the allowlist, accompanied by a methodology IRI, credit type, and admin address.
- **Event source:** `MsgProposeClassCreator` submissions (v1 on-chain) or equivalent off-chain application intake (v0 advisory). Agent scores published as `MsgSubmitAgentScore` (v1) or digest entries (v0).

## 3. Signal definition
- **Signal name:** Agent Pre-Screening Score
- **Unit:** score (0–1000), confidence (0–1000)
- **Directionality:** higher score = stronger recommendation for approval
- **Granularity:** per proposal (`proposal_id`)
- **Recommendation enum:** `APPROVE`, `CONDITIONAL`, `REJECT`

### Thresholds (v0 advisory)
| Score Range | Confidence | Recommendation | Action |
|---|---|---|---|
| >= 700 | any | APPROVE | Advance to VOTING (auto-promote in v1) |
| 300–699 | any | CONDITIONAL | Advance to VOTING with agent notes |
| < 300 | > 900 | REJECT | Flag for rejection with 6h human override window (v1) |
| < 300 | <= 900 | CONDITIONAL | Advance to VOTING (low confidence prevents auto-reject) |

## 4. Evidence inputs

| Input | Source | Fields | Validity rules | Anti-spoof assumptions | Refresh cadence |
|---|---|---|---|---|---|
| Methodology document | KOI knowledge layer | `methodology_iri`, resolved document content | IRI must resolve to a valid document; document must address additionality, baseline, MRV, permanence | KOI IRI resolution is trusted; document integrity verified by hash | On proposal submission |
| Credit type | Regen Ledger `x/ecocredit` | `credit_type` string (e.g., "C", "KSH") | Must exist in `allowed_credit_types` on-chain (currently: C, KSH, BT, MBS, USS) | Chain state is authoritative | On proposal submission |
| Admin reputation | M010 reputation signal | `admin_address` → reputation score | Address must be valid bech32; reputation score queried from M010 | M010 signal store is trusted; address is not spoofable | On proposal submission + periodic re-evaluation |
| Duplicate detection | KOI + Ledger | Existing class metadata, methodology IRIs | Cosine similarity < 0.85 vs existing 13 classes | Similarity computation is deterministic and auditable | On proposal submission |
| Historical proposal data | Governance module | Past proposal outcomes for admin address | Query governance history for proposer's address | On-chain governance records are immutable | On proposal submission |

## 5. Scoring function

### 5.1 Weighted composite score

The agent pre-screening score is a weighted composite of four factors:

```
score = (w_methodology × f_methodology) + (w_reputation × f_reputation) + (w_novelty × f_novelty) + (w_completeness × f_completeness)
```

Where:
- `w_methodology = 0.4` — Methodology quality and rigor
- `w_reputation = 0.3` — Admin/proposer reputation from M010
- `w_novelty = 0.2` — Novelty relative to existing credit classes
- `w_completeness = 0.1` — Application completeness (all fields provided, supporting docs)

### 5.2 Factor definitions

#### Methodology quality (`f_methodology`, 0–1000)
Evaluates the methodology document against required components:
- **Additionality** (0–250): Does the methodology establish credible additionality criteria?
- **Baseline** (0–250): Is the baseline scenario well-defined and conservative?
- **MRV** (0–250): Are measurement, reporting, and verification procedures specified?
- **Permanence** (0–250): Are permanence risks addressed with buffer/insurance mechanisms?

#### Admin reputation (`f_reputation`, 0–1000)
Derived from M010 reputation signal for the admin address:
- If M010 score exists: `f_reputation = m010_score` (already 0–1000 in v1; 0–1 × 1000 in v0)
- If no M010 score: `f_reputation = 500` (neutral default)
- Historical proposal success rate is a secondary factor: `f_reputation = 0.7 × m010_score + 0.3 × (historical_pass_rate × 1000)`

#### Novelty (`f_novelty`, 0–1000)
Measures differentiation from existing credit classes:
- `max_similarity` = max cosine similarity against all existing class methodology IRIs
- `f_novelty = (1 - max_similarity) × 1000`
- If `max_similarity >= 0.85`: `f_novelty = 0` (duplicate detected, hard requirement failure)

#### Completeness (`f_completeness`, 0–1000)
Binary checklist scoring:
- Methodology IRI provided and resolvable: 250
- Credit type valid: 250
- Admin address valid with on-chain history: 250
- Supporting documents / endorsements provided: 250

### 5.3 Confidence calculation

Confidence reflects data availability for the scoring factors:

```
confidence = (data_available_factors / total_factors) × 1000
```

Where `data_available_factors` counts factors with non-default/non-estimated inputs:
- M010 reputation score exists (not defaulted to 500)
- Methodology document fully resolvable and parseable
- At least 3 existing classes for meaningful similarity comparison
- Historical proposal data available for admin address

### 5.4 Normalization
- **v0:** All factor scores are 0–1000. Composite score is 0–1000.
- **Output:** `{ score: 0–1000, confidence: 0–1000, recommendation: APPROVE|CONDITIONAL|REJECT }`

## 6. State machine

```
States: {DRAFT, AGENT_REVIEW, VOTING, APPROVED, REJECTED, EXPIRED}

DRAFT → AGENT_REVIEW
  trigger: proposer.submit(deposit=1000)
  guard: proposer.balance >= 1000 REGEN; credit_type valid
  action: escrow.lock(1000) (v1); record proposal (v0)

AGENT_REVIEW → VOTING
  trigger: agent.score_submitted AND (score >= 300 OR timeout(24h))
  guard: none
  action: governance.create_proposal() (v1); publish digest entry (v0)
  note: 24h timeout ensures proposals advance even without agent response

AGENT_REVIEW → REJECTED
  trigger: agent.score < 300 AND agent.confidence > 900
  guard: human_override_window_expired(6h)
  action: escrow.slash(50%), escrow.refund(50%) (v1); publish rejection (v0)
  note: 6h human override window — any validator/admin can force advancement to VOTING

AGENT_REVIEW → VOTING (human override)
  trigger: admin.override(proposal_id) during human_override_window
  guard: proposal in AGENT_REVIEW with REJECT recommendation
  action: governance.create_proposal() with override flag

VOTING → APPROVED
  trigger: tally.yes_ratio > 0.5 AND quorum_met
  guard: voting_period_complete (7 days)
  action: allowlist.add(proposer), escrow.refund(1000) (v1)
  poa_variant: see §8 for modified tally

VOTING → REJECTED
  trigger: tally.yes_ratio <= 0.5 OR veto > 0.334
  guard: voting_period_complete
  action: escrow.slash(200), escrow.refund(800) (v1)
  poa_variant: see §8 for modified tally

VOTING → EXPIRED
  trigger: voting_period_complete AND quorum NOT met
  guard: none
  action: escrow.refund(950), escrow.fee(50) to community pool (v1)
  note: small fee covers governance overhead; proposer recovers most deposit

Terminal states: APPROVED, REJECTED, EXPIRED (no further transitions)
```

## 7. Token flows

### Deposit escrow (v1 — not enforced in v0)

```
Deposit: 1,000 REGEN (minimum, governance-configurable)

┌─────────────┐     deposit      ┌──────────────┐
│  Proposer   │ ────(1000)────→  │ Escrow Pool  │
└─────────────┘                  └──────────────┘
                                        │
       ┌────────────────────────────────┼────────────────────────────────┐
       │ APPROVED                       │ REJECTED                       │ EXPIRED
       ▼                                ▼                                ▼
┌──────────────┐              ┌──────────────┐              ┌──────────────┐
│  Proposer    │              │  200 → Pool  │              │  50 → Pool   │
│  (1000 back) │              │  800 → Prop  │              │  950 → Prop  │
└──────────────┘              └──────────────┘              └──────────────┘

Agent auto-reject (score < 300, confidence > 900, no override):
  500 → Community Pool (slash)
  500 → Proposer (refund)
```

### Invariant
`sum(deposits) = sum(refunds) + sum(slashes) + escrow.balance` — must hold at all times.

### v0 (advisory)
No deposit escrow is enforced. The scoring and recommendation are informational only, published in the weekly digest. Existing governance deposit rules apply to the standard `x/gov` proposal flow.

## 8. PoA variant (v2 — requires M014 activation)

When M014 (Authority Validator Governance) is activated, M001-ENH voting mechanics change to a dual-track model:

### Dual-track tally

```yaml
tally_logic:
  # Track 1: Authority Validator Vote (weighted 60%)
  validator_track:
    eligible_voters: active_authority_validators  # 15-21 from M014
    weight_per_validator: 1 / active_validator_count  # equal weight
    threshold: simple_majority (> 50%)
    quorum: 2/3 of active validators must vote

  # Track 2: Token-Holder Vote (weighted 40%)
  holder_track:
    eligible_voters: all REGEN holders (excluding validator operational addresses)
    weight: contribution_score (from M015) OR 1-token-1-vote (pre-M015)
    threshold: simple_majority (> 50%)
    quorum: 10% of circulating supply participating

  # Combined approval
  approval_condition: >
    (validator_track.approved AND holder_track.approved)
    OR (validator_track.approved with >= 75% supermajority)
    OR (holder_track.approved with >= 75% supermajority AND validator_track.quorum_met)

  # Veto
  veto_condition: >
    validator_veto > 33.4% OR holder_veto > 33.4%
    (either track can veto; preserves minority protection)
```

### Rationale
The 60/40 weighting reflects that authority validators have demonstrated commitment through the M014 selection process, while ensuring token holders retain meaningful governance voice. Either track achieving supermajority can carry a proposal, preventing gridlock while maintaining checks.

### Transition phases
1. **Phase 1**: M014 active but M001-ENH uses legacy tally (backward compatible)
2. **Phase 2**: Dual-track tally activated via governance proposal
3. **Phase 3**: Contribution-weighted holder track (requires M015 active)

### Open questions
- **OQ-M001-1**: Is 60/40 the right validator-to-holder weight split? Should registry-specific proposals weight ecological data steward validators higher?
- **OQ-M001-2**: Should the agent pre-screening score influence which governance track is required? E.g., high-confidence agent approvals could use a lighter-touch validator-only fast track.

## 9. Security invariants

1. **Deposit conservation**: `sum(deposits) = sum(refunds) + sum(slashes) + escrow.balance` — must hold at all times.
2. **Agent authority bound**: Agent can only recommend (score + recommendation). Agent cannot directly approve or reject without a 6h human override window expiring. No agent action bypasses governance vote.
3. **Governance supremacy**: Human vote (validator + delegator) always overrides agent recommendation. A proposal rejected by the agent can be overridden to VOTING. A proposal approved by governance cannot be vetoed by the agent.
4. **Slash cap**: Maximum slash = 50% of deposit per proposal. No proposal can result in more than 50% deposit loss regardless of outcome path.
5. **PoA validator parity**: Under M014, no single authority validator's vote counts more than any other validator's vote (equal weight per validator, `1/active_validator_count`).

## 10. Attack model

### 10.1 Spam proposals
**Attack**: Flood governance with low-quality proposals to exhaust voter attention.
**Mitigation**: 1,000 REGEN deposit creates economic cost. Agent auto-reject (score < 300, confidence > 900) filters obvious spam before governance queue. 50% slash on auto-rejected proposals further deters repeat offenders.

### 10.2 Agent manipulation
**Attack**: Submit proposals specifically crafted to score high on the agent's weighted formula while being low quality (e.g., plagiarized methodology that scores high on completeness but low on novelty).
**Mitigation**: Duplicate detection (`cosine_similarity >= 0.85` triggers `f_novelty = 0`). Methodology quality factor (`0.4 weight`) requires substance, not just form. Human governance vote is always the final decision. Agent score is advisory — manipulation gains nothing if voters reject.

### 10.3 Governance apathy
**Attack**: Exploit low turnout to pass weak proposals that wouldn't survive scrutiny.
**Mitigation**: Quorum requirements (standard governance quorum, 10% circulating supply for PoA holder track, 2/3 validator quorum for PoA validator track). EXPIRED state returns 95% of deposit — proposers don't benefit from re-submitting to catch low-turnout windows. Veto threshold (33.4%) allows motivated minority to block.

### 10.4 Deposit grinding
**Attack**: Submit and withdraw proposals repeatedly to probe the agent scoring function.
**Mitigation**: v0 is advisory-only (no deposit). v1: deposits are locked on submission and cannot be retrieved without full lifecycle completion. Each submission incurs at minimum the 50 REGEN fee on EXPIRED. Agent scoring factors (methodology quality, reputation) are not trivially gameable through repeated submission.

### 10.5 Reputation bootstrap attack
**Attack**: New address with no M010 reputation gets `f_reputation = 500` (neutral default), gaming the scoring by avoiding negative reputation.
**Mitigation**: Neutral default (500) is below the `f_reputation` that established good actors achieve. `w_reputation = 0.3` limits the impact of this factor. The methodology quality factor (`w_methodology = 0.4`) remains the dominant scoring input.

## 11. Integration points

- **KOI MCP (knowledge):** Resolve methodology IRIs, extract document structure for methodology quality assessment, cross-reference existing credit class metadata for duplicate detection.
- **Ledger MCP (chain data):** Query `x/ecocredit` for existing credit types and classes (C01–C09, KSH01, BT01, MBS01, USS01), verify admin addresses, check governance proposal history.
- **M010 (Reputation Signal):** Query admin/proposer reputation score. M010 `f_reputation` factor consumes reputation scores per `(Address, admin_address, operator_trust)`.
- **M014 (Authority Validator Governance):** v2 PoA dual-track tally requires M014 active authority validator set and equal-weight voting. Not consumed in v0/v1.
- **M015 (Contribution-Weighted Holder Voting):** v2 PoA holder track uses M015 contribution scores for weighted voting. Not consumed in v0/v1.
- **GOV-001 (Credit Class Creator Allowlist Process):** M001-ENH implements the agent pre-screening and deposit escrow stages of the GOV-001 governance process defined in phase-2/2.3.
- **Governance module (`x/gov`):** v1 creates standard governance proposals for the VOTING stage. Agent scores are published as metadata on the proposal.

## 12. Acceptance tests

### Proposal lifecycle
1) **Full lifecycle (happy path):** Proposer submits application with valid methodology IRI, credit type "C", and admin address with M010 reputation. Agent scores 750/900 (APPROVE). Proposal advances to VOTING. Governance approves. Proposer added to allowlist, deposit refunded.
2) **Insufficient deposit:** Proposer submits with deposit < 1000 REGEN. Submission rejected with `ErrInsufficientDeposit`.
3) **Invalid credit type:** Proposer submits with credit type "XYZ" (not in C, KSH, BT, MBS, USS). Submission rejected with `ErrInvalidCreditType`.
4) **Duplicate methodology:** Proposer submits with methodology IRI that has cosine similarity >= 0.85 to existing class. Agent assigns `f_novelty = 0`, overall score drops. If score < 300 with high confidence, auto-reject fires.

### Agent scoring
5) **Agent auto-promote:** Agent scores proposal 800/950 (APPROVE). Proposal advances to VOTING without delay.
6) **Agent auto-reject with human override:** Agent scores proposal 150/950 (REJECT). 6h override window starts. Validator calls override within window. Proposal advances to VOTING with override flag.
7) **Agent auto-reject without override:** Agent scores proposal 150/950 (REJECT). Override window expires. Proposal rejected. 50% deposit slashed, 50% refunded.
8) **Agent timeout:** Agent does not submit score within 24h. Proposal advances to VOTING without agent score.
9) **Low confidence prevents auto-reject:** Agent scores proposal 200/400 (CONDITIONAL due to low confidence). Proposal advances to VOTING despite low score.

### Governance outcomes
10) **Governance approval:** Proposal in VOTING, yes_ratio = 0.65, quorum met. Proposal APPROVED, proposer added to allowlist, full deposit refunded.
11) **Governance rejection:** Proposal in VOTING, yes_ratio = 0.35. Proposal REJECTED. 200 REGEN slashed to community pool, 800 REGEN refunded.
12) **Governance veto:** Proposal in VOTING, veto > 33.4%. Proposal REJECTED via veto. Slash applies.
13) **Quorum not met:** Proposal in VOTING, quorum not met after voting period. Proposal EXPIRED. 950 REGEN refunded, 50 REGEN fee to community pool.

### PoA variant (v2)
14) **PoA dual-track approval:** Both validator track (60%) and holder track (40%) approve. Proposal APPROVED.
15) **PoA validator supermajority override:** Validator track approves with 80% (>= 75% supermajority). Holder track rejects. Proposal APPROVED via validator supermajority.
16) **PoA holder supermajority with quorum:** Holder track approves with 80% supermajority. Validator track quorum met but rejects. Proposal APPROVED via holder supermajority.
17) **PoA veto by either track:** Validator veto > 33.4%. Proposal REJECTED regardless of holder track outcome.

### Security
18) **Deposit conservation:** After any sequence of proposals (approved, rejected, expired, auto-rejected), verify `sum(deposits) = sum(refunds) + sum(slashes) + escrow.balance`.
19) **Agent cannot bypass governance:** Agent submits APPROVE recommendation. Governance rejects. Verify proposal is REJECTED (governance overrides agent).
20) **Slash cap enforced:** Verify no proposal outcome path results in > 50% deposit loss.

## 13. Rollout plan

### v0 checklist (advisory-only)
- Implement off-chain agent pre-screening scoring function (4-factor weighted composite).
- Publish agent scores in weekly digest alongside credit class governance proposals.
- Query M010 reputation scores for admin addresses via M010 off-chain signal store.
- Query KOI MCP for methodology document analysis.
- Query Ledger MCP for existing credit types, classes, and governance history.
- Implement deterministic test fixtures with realistic Regen Network data.
- Validate scoring function against test vectors.
- Record all agent scores with rationale in audit log.

### v1 outline (deposit escrow on-chain)
- Deploy `MsgProposeClassCreator` and `MsgSubmitAgentScore` in `x/ecocredit` extension.
- Implement deposit escrow with lock/refund/slash flows.
- Implement 6h human override window for agent auto-reject.
- Implement EXPIRED state with 50 REGEN community pool fee.
- Integrate agent scoring with on-chain proposal metadata.

### v2 outline (PoA dual-track)
- Requires M014 (Authority Validator Governance) active.
- Implement dual-track tally logic (60/40 validator/holder weighting).
- Implement supermajority override conditions.
- Integrate M015 contribution-weighted holder voting (when available).

---

## Appendix A — Source anchors
- `phase-2/2.1-token-utility-mechanisms.md` — M001-ENH protocol specification (participants, token flows, state transitions, security invariants, automation logic, PoA impact)
- `phase-2/2.3-governance-processes.md` — GOV-001 Credit Class Creator Allowlist process (stages, decision framework, PoA impact)
- `phase-3/3.1-smart-contract-specs.md` — x/ecocredit enhancement (protobuf message types, state objects, keeper implementation)
