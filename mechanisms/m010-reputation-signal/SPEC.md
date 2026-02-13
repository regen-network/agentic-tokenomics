# m010 — Reputation Signal (SPEC)

## 0. Header
- **ID:** m010
- **Name:** Reputation / Legitimacy Signal
- **Status:** draft (advisory-only v0)
- **Owner:** (unset)
- **Last updated:** 2026-02-04
- **Scope:** **v0 advisory** (signal computation + publication only; no enforced on-chain policy changes)

## 1. Problem
Regen ecosystem decisions (registry quality, governance, marketplace curation, and agentic pre-screening) benefit from a shared, queryable signal of **legitimacy/reputation** across key subject types. The m010 design introduces a stake-weighted endorsement signal with time decay, along with challenge/invalidations, so downstream workflows can reference an interpretable score.

## 2. Target actor and action
- **Actors:** signalers (staked REGEN holders), subjects (entities being endorsed), an admin (for invalidation), challengers.
- **Action being evaluated (one action):** a **stake-weighted endorsement** (level 1–5) submitted by a signaler about a subject in a category.
- **Event source:** “submit/withdraw/challenge/invalidate signal” events in the m010 reputation registry workflow (advisory in v0; may be implemented via CosmWasm in v1).

## 3. Signal definition
- **Signal name:** Reputation Score
- **Unit:** score (0–1000)
- **Directionality:** higher = better
- **Granularity:** per `(subject_type, subject_id, category)`
- **Subject types (as specified):** `CreditClass`, `Project`, `Verifier`, `Methodology`, `Address`.
- **Categories:** multiple configurable reputation categories.

## 4. Evidence inputs

| Input | Source | Fields | Validity rules | Anti-spoof assumptions | Refresh cadence |
|---|---|---|---|---|---|
| Endorsement signal | m010 signal store | `subject_type`, `subject_id`, `category`, `endorsement_level (1–5)`, `signaler`, `timestamp` | `endorsement_level` must be within 1–5; subject type must be supported; category must be configured | signer controls their key; uniqueness constraints handled by contract/indexing | on each new signal / state change |
| Stake weight | Regen staking/bank data | `staked_amount` for signaler (and/or category min stake) | must meet `min_stake_by_category` (if configured) | stake is read from chain state (not self-reported) | on signal submission and/or periodic re-evaluation |
| Signal status | m010 workflow state | active / withdrawn / challenged / invalidated | withdrawn signals don’t count; challenged signals pause contribution; invalidated signals are excluded | admin key is trusted for invalidation | on status updates |
| Time decay params | m010 config | `decay_half_life_days` | must be set at instantiate/config update time | config changes are governance/admin-controlled | when config changes |
| Optional rationale links | KOI knowledge layer | rationale IRIs / document links | used for context only; not a scoring input in v0 | KOI sources can be cited but not trusted for stake | on digest generation |

## 5. Scoring function
### 5.1 Time decay
A decay factor is applied based on age of a signal using a configurable half-life:

```
decay_factor = (1/2)^(age / half_life)
```

Where:
- `age` is elapsed time since signal timestamp (same unit as `half_life`)
- `half_life` corresponds to `decay_half_life_days`.

### 5.2 Contribution weight

#### v0 (advisory — current implementation)

v0 omits stake weighting per WG_PACKET scope. The score is a decay-weighted average of endorsement levels:

```
score = sum(decay * endorsement_level / 5) / sum(decay)
```

- Output range: **0–1** (normalized)
- See `reference-impl/m010_score.js` for canonical implementation
- Test vector: v0_sample yields `reputation_score_0_1: 0.5488`

#### Target (with on-chain stake — future v1)

When on-chain stake data is available, contribution becomes stake-weighted:

```
score = sum(stake * decay * endorsement_level / 5) / total_weight * 1000
```

- "total_weight" refers to the normalization denominator (total stake weight over included signals)
- Output range: **0–1000**

Notes (both versions):
- Signals with status **withdrawn**, **challenged**, or **invalidated** do not contribute.

### 5.3 Normalization
- **v0:** Final score is normalized to **0–1**.
- **Target (v1):** Final score is normalized to **0–1000**.
- Endorsement levels are 1–5.

### 5.4 Controls
- **Category-specific minimum stake** may be required to submit a signal (`min_stake_by_category`).
- Non-transferable reputation: reputation is derived from signals; it is not a transferable asset.

## 6. Challenge & Dispute Workflow

This section specifies how reputation signals are contested, resolved, and — where appropriate — invalidated. The challenge workflow is critical for maintaining signal integrity: without it, low-quality or malicious endorsements cannot be corrected.

### 6.1 Signal Lifecycle State Machine

```
States: {SUBMITTED, ACTIVE, CHALLENGED, ESCALATED, RESOLVED_VALID, RESOLVED_INVALID, WITHDRAWN, INVALIDATED}

SUBMITTED → ACTIVE
  trigger: activation_delay_passed(24h) AND no_challenge_filed
  guard: signal passes format validation (section 4)
  action: signal begins contributing to reputation score
  note: 24h activation delay allows early challenges before score impact

ACTIVE → CHALLENGED
  trigger: challenger.submit_challenge(signal_id, evidence, rationale)
  guard:
    - challenger meets min_stake for signal's category
    - challenger is not the original signaler (no self-challenge)
    - signal is within challenge_window (see 6.3)
    - v1 only: challenger posts challenge_deposit
  action:
    - signal contribution to reputation score is PAUSED
    - challenge record created with timestamp, evidence, rationale
    - notification emitted to signaler and digest subscribers

CHALLENGED → RESOLVED_VALID
  trigger:
    v0: admin.resolve(signal_id, VALID, rationale)
    v1: arbiter_dao.vote(signal_id, VALID) (reuse M008 Arbiter DAO)
  guard: resolution_authority_verified
  action:
    - signal contribution RESTORED to reputation score
    - challenger marked as unsuccessful for this challenge
    - v1: challenger forfeits challenge_deposit (anti-spam)
    - resolution rationale recorded in audit log

CHALLENGED → RESOLVED_INVALID
  trigger:
    v0: admin.resolve(signal_id, INVALID, rationale)
    v1: arbiter_dao.vote(signal_id, INVALID)
  guard: resolution_authority_verified
  action:
    - signal contribution PERMANENTLY REMOVED from reputation score
    - signal status set to RESOLVED_INVALID (terminal)
    - v1: signaler's challenge_deposit (if bonded in v1) slashed;
           challenger receives reward from slash
    - signaler's accuracy_record updated (affects future signal weight in v1)
    - resolution rationale recorded in audit log

CHALLENGED → ESCALATED
  trigger: resolution_deadline_expired AND challenge.status == pending
  guard: no admin/arbiter resolution submitted within deadline
  action:
    - escalate to governance Layer 3 vote
    - emit EventChallengeEscalated
    - notify governance module
  note: ESCALATED is a distinct state from CHALLENGED because:
    1. The resolution authority changes (admin/arbiter → governance)
    2. The timeline changes (governance voting period, not admin deadline)
    3. Visibility changes (escalated challenges appear in governance queue)

ESCALATED → RESOLVED_VALID
  trigger: governance.vote(VALID)
  action: signal contribution RESTORED, challenge.close()

ESCALATED → RESOLVED_INVALID
  trigger: governance.vote(INVALID)
  action: signal contribution PERMANENTLY REMOVED, challenge.close()

ACTIVE → WITHDRAWN
  trigger: signaler.withdraw(signal_id)
  guard: signaler owns the signal, signal is not currently CHALLENGED
  action: signal contribution removed from reputation score (non-punitive)

SUBMITTED → CHALLENGED
  trigger: challenger.submit_challenge(signal_id, evidence, rationale)
  guard: same as ACTIVE → CHALLENGED
  action: signal never contributes to score; proceeds to resolution
  note: early challenge during activation delay prevents score impact entirely

ACTIVE → INVALIDATED
  trigger: admin.invalidate(signal_id, rationale)
  guard: admin authorization (v0: designated admin key; v1: governance vote)
  action:
    - signal contribution PERMANENTLY REMOVED
    - invalidation rationale MUST be provided and published
    - all invalidation events published in digest for transparency
  note: admin invalidation is a powerful override; v1 adds governance checks

Terminal states: RESOLVED_INVALID, WITHDRAWN, INVALIDATED (no further transitions)
Note: ESCALATED is non-terminal — it resolves to RESOLVED_VALID or RESOLVED_INVALID via governance
```

### 6.2 Challenge Participants

| Role | Description | Requirements | v0 | v1 |
|------|-------------|--------------|----|----|
| Challenger | Entity disputing a signal's validity | Meets `min_stake` for category | Yes | Yes |
| Signaler (respondent) | Original signal author | Owns the challenged signal | Yes (passive) | Yes (may respond) |
| Admin / Resolver | Authority resolving the challenge | Designated admin key | Yes (sole resolver) | Replaced by Arbiter DAO |
| Arbiter DAO | Decentralized resolution body | Staked in arbiter pool (reuse M008) | No | Yes |
| Agent (AGENT-002) | Governance Analyst agent | Monitors challenges, summarizes evidence | Yes (informational) | Yes (informational) |

### 6.3 Challenge Parameters

| Parameter | v0 Value | v1 Value | Rationale |
|-----------|----------|----------|-----------|
| `activation_delay` | 24 hours | 24 hours | Window for early challenge before score impact |
| `challenge_window` | 180 days from signal submission | 365 days | Longer window in v1 due to higher stakes |
| `challenge_min_stake` | Same as signal category `min_stake` | Same as signal category `min_stake` | Challenger must have equivalent skin-in-game |
| `challenge_deposit` | None (v0 is advisory) | 10% of challenger's staked amount (min 100 REGEN) | Anti-spam; forfeited if challenge fails |
| `resolution_deadline` | 14 days | 21 days | Maximum time for admin/arbiter to resolve |
| `response_window` | 7 days | 10 days | Time for signaler to respond to challenge |
| `max_active_challenges_per_signal` | 1 | 1 | One challenge at a time; re-challenge allowed after resolution |

### 6.4 Challenge Submission Requirements

A challenge MUST include:

```yaml
challenge_submission:
  required:
    signal_id: string        # ID of the signal being challenged
    challenger_id: string    # Address of the challenger
    category: string         # Must match the signal's category
    rationale: string        # Human-readable reason for challenge (min 50 chars)
    evidence:
      koi_links: string[]    # At least one KOI or ledger reference required
      ledger_refs: string[]
      web_links: string[]    # Optional supporting web references
  optional:
    severity: enum           # LOW, MEDIUM, HIGH, CRITICAL
    requested_outcome: enum  # INVALIDATE, DOWNGRADE, FLAG_FOR_REVIEW
```

A challenge MUST NOT be accepted if:
- The signal is already in CHALLENGED, WITHDRAWN, INVALIDATED, or RESOLVED_INVALID state
- The challenger is the same entity as the signaler
- The challenger does not meet the minimum stake requirement
- No evidence is provided (at least one `koi_links` or `ledger_refs` entry required)
- The signal is outside the `challenge_window`

### 6.5 Resolution Process

#### v0 (Admin Resolution)

```
1. Challenge submitted → notification to admin + signaler
2. Signaler response window (7 days):
   - Signaler MAY submit counter-evidence and rationale
   - If signaler does not respond, challenge proceeds to admin
3. Admin review (within 14 days of challenge):
   - Admin reviews challenge evidence, signaler response, signal context
   - Admin resolves as VALID (signal restored) or INVALID (signal removed)
   - Resolution rationale MUST be recorded
4. All resolution events published in digest
```

**Admin safeguards (v0):**
- All invalidation/resolution events are logged with rationale and published
- Monthly digest includes a "Challenge Summary" section
- If admin does not resolve within `resolution_deadline`, challenge auto-escalates to governance (Layer 3 vote)
- Admin cannot challenge and resolve the same signal

#### v1 (Arbiter DAO Resolution)

```
1. Challenge submitted → arbiter_dao.assign(challenge_id)
   - Reuse M008 Arbiter DAO infrastructure
   - Arbiter selection: random subset from arbiter pool
   - Conflict check: no arbiter is signaler, challenger, or subject
2. Evidence phase (10 days):
   - Both parties submit evidence
   - Arbiter DAO members review
3. Voting phase (11 days remaining):
   - Arbiter DAO votes: VALID or INVALID
   - Quorum: 51% of assigned arbiters
   - Simple majority determines outcome
4. Economic settlement:
   - VALID: challenger loses deposit; signaler restored
   - INVALID: signaler's signal removed; challenger receives
     reward from challenge deposit pool
5. Appeal window (7 days after resolution):
   - Either party may appeal to governance (Layer 3)
   - Appeal requires deposit of 2x original challenge_deposit
   - Governance vote is final
```

### 6.6 Impact on Reputation Score

During and after challenge:

| Signal State | Score Contribution | Rationale |
|--------------|-------------------|-----------|
| SUBMITTED | None (not yet active) | Activation delay prevents premature impact |
| ACTIVE | Full contribution per section 5 | Normal operation |
| CHALLENGED | **Paused** (0 contribution) | Presumption of caution during dispute |
| ESCALATED | **Paused** (0 contribution) | Governance resolution pending; same caution as CHALLENGED |
| RESOLVED_VALID | Full contribution restored | Challenge dismissed; signal vindicated |
| RESOLVED_INVALID | Permanently removed | Signal found to be invalid |
| WITHDRAWN | Removed (non-punitive) | Voluntary withdrawal |
| INVALIDATED | Permanently removed | Administrative override |

**Signaler accuracy tracking (v1):**
```
signaler_accuracy[s] = valid_signals[s] / (valid_signals[s] + invalidated_signals[s])

# Signals that have never been challenged count as valid.
# Signals currently challenged are excluded from the ratio until resolved.
# In v1, signaler_accuracy is a multiplier on future signal weight:
#   effective_weight = stake_weight × signaler_accuracy × decay
```

### 6.7 Challenge KPIs

The following metrics should be tracked and published in periodic digests:

| KPI | Formula | Target |
|-----|---------|--------|
| `challenges_filed` | count of challenges per period | Informational |
| `challenge_rate` | challenges / total active signals | < 5% (healthy ecosystem) |
| `avg_resolution_time_hours` | mean(resolution_timestamp - challenge_timestamp) | < 168h (7 days) |
| `challenge_success_rate` | resolved_invalid / (resolved_valid + resolved_invalid) | Informational |
| `admin_resolution_timeout_rate` | auto_escalated / total_challenges | < 5% |
| `signaler_accuracy_median` | median(signaler_accuracy) across active signalers | > 0.9 |

---

## 7. Economic linkage
**v0 (advisory):** No direct economic enforcement is specified in the provided inputs. The score is intended to be consumed by other processes (governance analysis, agent pre-screening, arbiter selection, and other mechanisms). Challenge resolution is admin-driven with no economic stakes.

**v1 (on-chain):** Challenge deposits create economic skin-in-game. Failed challenges forfeit deposits (anti-spam). Successful challenges reward the challenger from the forfeited signal bond. Signaler accuracy history affects future signal weight, creating long-term incentive for honest signaling.

**Intended downstream use:** reputation scores may influence voting weight, pre-screening, and selection/curation decisions in related mechanisms (M001-ENH, M008, M009, M011).

## 8. On-chain vs off-chain boundary
- **v0:** compute and publish the signal in off-chain systems (e.g., daily/weekly digests) using verified chain stake data plus a maintained signal store. Challenge resolution is admin-driven, off-chain, with all events logged.
- **Optional v1:** a CosmWasm smart contract implementation exists in the provided materials as an implementation summary ("production-ready contract"), including execute/query messages and tests. v1 would formalize the signal store, scoring, **and challenge resolution** on-chain via Arbiter DAO (reusing M008 infrastructure), but that is out of scope for v0.

## 9. Attack model
- **Sybil:** mitigated by stake-weighting and minimum stake thresholds per category.
- **Collusion / cartels:** challengers and admin invalidation exist; residual risk remains if large stakeholders coordinate. v1 Arbiter DAO with random selection and conflict checks reduces collusion risk.
- **Bribery:** endorsements can be bought; mitigations are social/process (challenge workflows) and downstream reliance only where appropriate. v1 signaler accuracy tracking creates long-term reputational cost for dishonest signaling.
- **Data poisoning:** KOI rationale links are context-only; stake and signal store are the authoritative scoring inputs.
- **Reputation capture / censorship:** admin-only invalidation is powerful; governance controls and auditability are required in v1. In v0, publish all invalidation events and rationale in the digest. Resolution deadline with auto-escalation prevents admin inaction.
- **Challenge spam:** v1 challenge deposits prevent frivolous challenges. v0 relies on min_stake requirement and the social cost of being publicly identified as an unsuccessful challenger.
- **Challenge suppression:** the `challenge_window` ensures signals remain contestable for an extended period. Auto-escalation on resolution timeout prevents suppression through inaction.

## 10. Integration points
- **KOI MCP (knowledge):** resolve rationale IRIs, cross-reference methodology documents, and provide context around subjects and categories. Challenge evidence may reference KOI documents.
- **Ledger MCP (chain data):** verify stake amounts, validate subject identifiers (credit classes, projects), and check validator status. v1 challenge deposits and settlements are on-chain.
- **Governance:** reputation may be consumed by governance analysis and pre-screening workflows. Challenge resolution timeout auto-escalates to governance (Layer 3).
- **M008 (Data Attestation Bonding):** v1 challenge resolution reuses M008's Arbiter DAO infrastructure for decentralized dispute resolution.
- **Other mechanisms:** explicitly called out as consumers: M001-ENH (class creators), M008 (attesters), M009 (service providers), M011 (marketplace curation).

## 11. Acceptance tests
The following tests are derived from the specified behaviors and the listed integration test scenarios.

**Signal lifecycle:**
1) **Full workflow:** multiple signalers submit signals on the same subject/category; reputation query returns a normalized score; withdrawing a signal updates score accordingly.
2) **Insufficient stake:** signal submission below the category minimum stake is rejected.
3) **Invalid endorsement level:** values outside 1–5 are rejected; 1–5 are accepted.
4) **Ownership:** only the original signaler can withdraw their signal.
5) **Activation delay:** signals in SUBMITTED state do not contribute to reputation score until 24h activation delay passes.

**Challenge workflow:**
6) **Challenge submission:** a valid challenge transitions signal from ACTIVE to CHALLENGED; challenged signals do not contribute to reputation until resolved.
7) **Challenge rejection — self-challenge:** challenger cannot challenge their own signal.
8) **Challenge rejection — insufficient stake:** challenger below min_stake is rejected.
9) **Challenge rejection — no evidence:** challenge without at least one koi_link or ledger_ref is rejected.
10) **Challenge rejection — wrong state:** cannot challenge a WITHDRAWN, INVALIDATED, or already CHALLENGED signal.
11) **Challenge rejection — expired window:** challenge outside challenge_window is rejected.
12) **Resolution — VALID:** admin resolves as VALID; signal restored to ACTIVE, score contribution resumes.
13) **Resolution — INVALID:** admin resolves as INVALID; signal permanently removed from score.
14) **Resolution timeout:** if admin does not resolve within resolution_deadline, challenge auto-escalates.
15) **Withdrawal during challenge:** signaler cannot withdraw a signal that is currently CHALLENGED.

**Admin invalidation:**
16) **Admin invalidation:** only admin can invalidate; invalidated signals do not contribute.
17) **Invalidation rationale:** invalidation without rationale is rejected.
18) **Invalidation audit:** all invalidation events appear in the digest with rationale.

**Adversarial:**
19) **Sybil attempt:** many low-stake identities submit endorsements; due to stake-weighting/min stake, aggregate influence remains bounded relative to a single larger stake holder.
20) **Challenge spam (v1):** repeated frivolous challenges by the same challenger result in cumulative deposit loss.

## 12. Rollout plan
### v0 checklist (advisory-only)
- Define categories and minimum stakes per category (policy).
- Implement an off-chain signal store that supports submit/withdraw/challenge/invalidate states.
- **Implement challenge submission, signaler response, and admin resolution workflows.**
- **Implement resolution_deadline auto-escalation.**
- Compute score using the specified formulas and publish in a periodic digest.
- **Include challenge summary metrics in periodic digest.**
- Verify stake weights via Ledger MCP.
- Record all state transitions and (when present) rationale links.

### Optional v1 outline (non-binding)
- Deploy a CosmWasm contract implementing message handlers for submit/withdraw/challenge/invalidate and query interfaces for signals and aggregated reputation.
- **Integrate Arbiter DAO (from M008) for decentralized challenge resolution.**
- **Implement challenge deposit and settlement economics.**
- **Implement signaler accuracy tracking and weight adjustment.**
- Add governance-controlled admin/parameters, plus audit tooling.
- **Add appeal workflow for contested resolutions (Layer 3 governance vote).**

---

## Appendix A — Source anchors
- `files/M010_IMPLEMENTATION_SUMMARY.md`
  - “Core Features” (Reputation signals, aggregated calculation, security features)
  - “Key Algorithms” (time decay; reputation score)
  - “Integration Points” (KOI MCP, Ledger MCP, governance, other mechanisms)
  - “Integration Tests” (workflow, insufficient stake, invalid endorsement, ownership, challenge system, admin powers, edge cases)
- `files/ECOSYSTEM_MAP.md`
  - “Priority build targets” / “The gaps” (context on repo roles and where m010 fits)
