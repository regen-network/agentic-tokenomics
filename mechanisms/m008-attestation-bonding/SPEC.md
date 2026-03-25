# m008 — Data Attestation Bonding (SPEC)

## 0. Header
- **ID:** m008
- **Name:** Data Attestation Bonding
- **Status:** draft (advisory-only v0)
- **Owner:** (unset)
- **Last updated:** 2026-02-18
- **Scope:** **v0 advisory** (bond schedule computation + quality signal publication only; no on-chain escrow or slashing in v0)

## 1. Problem
The Regen ecosystem relies on ecological data attestations (project boundaries, baseline measurements, credit issuance claims, methodology validations) to back real-world environmental credits. Currently there is no economic skin-in-the-game for attesters: anyone can make claims without risking capital if those claims prove false or misleading. This creates:

1. **No deterrent for low-quality data**: Attesters face no economic consequence for inaccurate claims.
2. **No incentive to challenge**: Without rewards, detecting and reporting bad attestations is a thankless task.
3. **No risk-proportional bonding**: High-value attestations (e.g., credit issuance claims) carry the same zero bond as trivial ones.
4. **No formalized dispute resolution**: Contested attestations have no structured resolution pathway.
5. **No attester track record**: No mechanism links attestation quality to future bonding requirements.

## 2. Target actor and action
- **Actors:** attester (entity making ecological claims), challenger (entity disputing an attestation), Arbiter DAO (decentralized resolution body via M008 subDAO), beneficiary (project receiving attestation), admin (config authority in v0).
- **Action being evaluated (one action):** a **bonded attestation** submitted by an attester, backed by locked REGEN tokens, where the bond is at risk of slashing if the attestation is proven invalid.
- **Event source:** `CreateAttestation` submissions (v1 on-chain via CosmWasm) or equivalent off-chain intake (v0 advisory). Challenges via `ChallengeAttestation`.

## 3. Signal definition
- **Signal name:** Attestation Bond Status
- **Unit:** bond amount (REGEN), attestation quality score (0–1000)
- **Directionality:** higher bond = higher confidence; higher quality score = better
- **Granularity:** per attestation (`attestation_id`)
- **Attestation types:** `ProjectBoundary`, `BaselineMeasurement`, `CreditIssuanceClaim`, `MethodologyValidation`

## 4. Evidence inputs

| Input | Source | Fields | Validity rules | Anti-spoof assumptions | Refresh cadence |
|---|---|---|---|---|---|
| Attestation document | KOI / x/data | `attestation_iri`, resolved content | IRI must resolve; content must match attestation type requirements | KOI IRI resolution is trusted; hash integrity verified | On submission |
| Bond amount | Regen bank/escrow | `bond.amount`, `bond.denom` | Must meet `min_bond[attestation_type]`; denom must be `uregen` | On-chain balances are authoritative | On submission |
| Attester identity | Regen Ledger | `attester` address | Valid bech32 address with on-chain history | Chain identity is not spoofable | On submission |
| Challenge evidence | KOI / external | `evidence_iri`, challenger rationale | IRI must resolve; evidence must be relevant to attestation type | Evidence integrity verified by hash; relevance assessed by Arbiter DAO | On challenge submission |
| Beneficiary identity | Regen Ledger | `beneficiary` address (optional) | Valid bech32 if provided | Chain identity trusted | On submission |
| Attester reputation | M010 signal store | M010 score for attester address | Score queried from M010 `(Address, attester, attestation_quality)` | M010 signal store trusted | On submission + periodic |

## 5. Scoring function

### 5.1 Attestation quality score

The quality score for an attestation is computed as a function of bond adequacy, attester reputation, and evidence completeness:

```
quality_score = (w_bond × f_bond) + (w_reputation × f_reputation) + (w_evidence × f_evidence) + (w_type × f_type)
```

Where:
- `w_bond = 0.3` — Bond adequacy relative to minimum
- `w_reputation = 0.3` — Attester M010 reputation
- `w_evidence = 0.25` — Evidence document completeness
- `w_type = 0.15` — Attestation type risk factor

### 5.2 Factor definitions

#### Bond adequacy (`f_bond`, 0–1000)
```
f_bond = min(1000, (bond_amount / min_bond[type]) × 500)
```
Bonds at minimum get 500; bonds at 2× minimum get 1000 (capped).

#### Attester reputation (`f_reputation`, 0–1000)
- If M010 score exists AND attester has track record (unchallenged_rate available):
  `f_reputation = 0.7 × m010_score + 0.3 × (unchallenged_rate × 1000)`
- If M010 score exists but no track record:
  `f_reputation = m010_score`
- If no M010 score: `f_reputation = 300` (cautious default, below neutral)

#### Evidence completeness (`f_evidence`, 0–1000)
Binary checklist by attestation type:
- IRI resolvable: 250
- Content matches declared type: 250
- Supporting documentation provided: 250
- Cross-references to existing data (KOI links): 250

#### Attestation type risk (`f_type`, 0–1000)
Higher-risk attestation types get higher scores (reflecting the higher bond requirement and scrutiny):
| Type | f_type | Rationale |
|---|---|---|
| MethodologyValidation | 1000 | Highest bond, longest lock |
| CreditIssuanceClaim | 800 | High value at risk |
| BaselineMeasurement | 600 | Moderate risk |
| ProjectBoundary | 400 | Lower risk |

### 5.3 Confidence

```
confidence = (data_available_factors / total_factors) × 1000
```

Factors: M010 reputation exists, IRI resolvable, attester has prior attestations, attestation type recognized.

## 6. State machine

```
States: {SUBMITTED, BONDED, ACTIVE, CHALLENGED, RESOLVED_VALID, SLASHED, RELEASED}

Note: RESOLVED_INVALID is not a distinct state; an invalid resolution transitions
      directly to SLASHED. RESOLVED_VALID is a terminal state; SLASHED is a terminal state.

Initial → SUBMITTED
  trigger: attester.submit_attestation(type, iri, bond)
  guard: attestation_type is valid
  action: attestation.create(status=SUBMITTED)

SUBMITTED → BONDED
  trigger: bond confirmed on-chain
  guard: bond >= min_bond[attestation_type]
  action: bond_pool.lock(bond), attestation.status = BONDED

BONDED → ACTIVE
  trigger: activation_delay_passed(48h) AND no_challenge_submitted
  guard: attestation still in BONDED state
  action: attestation.activate()
  note: 48h delay allows early challenges before activation

BONDED → CHALLENGED
  trigger: challenger.submit_challenge(attestation_id, evidence_iri, deposit)
  guard: deposit >= bond × challenge_deposit_ratio; within challenge_window
  action: challenge_pool.lock(deposit), attestation.status = CHALLENGED
  note: early challenge during activation delay

ACTIVE → CHALLENGED
  trigger: challenger.submit_challenge(attestation_id, evidence_iri, deposit)
  guard: deposit >= bond × challenge_deposit_ratio; within challenge_window
  action: challenge_pool.lock(deposit), attestation.status = CHALLENGED

CHALLENGED → RESOLVED_VALID
  trigger: arbiter_dao.vote(VALID) OR admin.resolve(VALID) (v0)
  guard: quorum_met (v1), resolution_authority_verified
  action: attester.receive(bond + challenge_deposit - arbiter_fee)
         attestation.status = RESOLVED_VALID

CHALLENGED → SLASHED
  trigger: arbiter_dao.vote(INVALID) OR admin.resolve(INVALID) (v0)
  guard: quorum_met (v1), resolution_authority_verified
  action: challenger.receive(bond × 0.5 + challenge_deposit - arbiter_fee)
         community_pool.receive(bond × 0.5)
         attestation.status = SLASHED

ACTIVE → RELEASED
  trigger: lock_period_expired AND no_active_challenge
  guard: env.block.time > lock_expires_at
  action: attester.receive(bond)
         attestation.status = RELEASED

Terminal states: RESOLVED_VALID, SLASHED, RELEASED
Note: RESOLVED_VALID attestations remain valid references; SLASHED attestations are permanently invalidated.
```

## 7. Token flows

### Bond schedule by attestation type

| Attestation Type | Min Bond (REGEN) | Lock Period | Challenge Window |
|---|---|---|---|
| ProjectBoundary | 500 | 90 days | 60 days |
| BaselineMeasurement | 1,000 | 180 days | 120 days |
| CreditIssuanceClaim | 2,000 | 365 days | 300 days |
| MethodologyValidation | 5,000 | 730 days | 600 days |

### Flow diagram

```
┌─────────────┐     bond         ┌──────────────┐
│  Attester   │ ────(B)────────→ │ Attestation  │
└─────────────┘                  │ Bond Pool    │
                                 └──────────────┘
                                        │
       ┌────────────────────────────────┼────────────────────────────────┐
       │ UNCHALLENGED (lock_period)     │ CHALLENGED                     │
       ▼                                ▼
┌──────────────┐              ┌──────────────┐
│  Attester    │              │ Arbiter DAO  │
│  (bond       │              │ Resolution   │
│   refund)    │              └──────┬───────┘
└──────────────┘                     │
                    ┌────────────────┼────────────────┐
                    │ ATTESTER WINS  │ CHALLENGER WINS│
                    ▼                ▼
              ┌──────────────┐ ┌──────────────┐
              │ Attester:    │ │ Challenger:  │
              │ bond+deposit │ │ 50% bond +   │
              │ - arb_fee    │ │ deposit -    │
              └──────────────┘ │ arb_fee      │
                               │ Community:   │
                               │ 50% bond     │
                               └──────────────┘
```

### Invariants
- `bond_pool.balance = sum(active_bonds) + sum(challenge_deposits) - sum(disbursements)`
- Challenge deposit ratio: 10% of bond amount (1000 basis points)
- Arbiter fee: 5% of bond amount (500 basis points)
- Slash distribution: 50% to challenger, 50% to community pool

### v0 (advisory)
No bond escrow is enforced. Quality scores and attestation lifecycle are tracked off-chain. Existing `x/data` IRI anchoring applies. Bond requirements are published as guidelines.

## 8. PoA variant (v2)
Under M014, Arbiter DAO composition may shift:
- Authority validators may serve as default arbiters for low-value attestations (< 1000 REGEN bond)
- High-value attestations (>= 2000 REGEN) require full Arbiter DAO vote
- Validator arbiters use equal-weight voting (consistent with M014 parity)

## 9. Security invariants

1. **Bond coverage**: `attestation.value_at_risk <= bond × coverage_ratio` — bond must be proportional to the ecological claim value.
2. **Challenge skin-in-game**: Challengers must deposit `bond × challenge_deposit_ratio` to prevent frivolous challenges. Deposit forfeited if challenge fails.
3. **Arbiter neutrality**: Arbiters cannot be the attester, challenger, or beneficiary. v1 adds 90-day transaction history conflict check.
4. **Slash distribution**: Slashed bonds split 50/50 between challenger reward and community pool. No other distribution ratio allowed.
5. **Lock period enforcement**: Bonds cannot be released before `lock_expires_at` regardless of attestation status (except via challenge resolution).
6. **Single challenge**: Only one active challenge per attestation at a time. Re-challenge allowed after resolution.
7. **Resolution finality**: Once resolved (VALID or INVALID), the resolution cannot be changed. Appeal to governance is a separate process.

## 10. Attack model

### 10.1 Frivolous attestations
**Attack**: Submit low-quality attestations with minimum bond, hoping they go unchallenged.
**Mitigation**: Challenge window is proportional to risk (60–600 days). Quality scoring flags low-evidence attestations in digest. Bond locked for full lock period regardless. Attester reputation (M010) degrades with challenged/invalidated attestations.

### 10.2 Challenge spam
**Attack**: File frivolous challenges to lock attester bonds and extract arbiter fees.
**Mitigation**: 10% deposit requirement creates economic cost. Failed challenges forfeit deposit to attester. Challenger reputation tracked via M010.

### 10.3 Arbiter collusion
**Attack**: Arbiter DAO members collude with challenger to slash bonds.
**Mitigation**: v0: admin resolution with audit trail. v1: Arbiter DAO with random arbiter selection, conflict checks (no relationship to parties within 90 days), and governance appeal. DAO DAO quorum requirements (51% majority, 15% quorum).

### 10.4 Bond grinding
**Attack**: Repeatedly submit and release attestations to game the quality scoring system.
**Mitigation**: Lock periods (90–730 days) make churning expensive in opportunity cost. Quality score tracks `unchallenged_rate` — successful releases improve reputation only gradually.

### 10.5 Evidence manipulation
**Attack**: Submit attestation with valid-looking IRI that points to fabricated data.
**Mitigation**: KOI cross-referencing against existing data. Challenge mechanism allows community to dispute. v1: IRI content hash verification on-chain via `x/data`.

## 11. Integration points

- **KOI MCP (knowledge):** Resolve attestation IRIs and challenge evidence IRIs. Cross-reference attestation claims against existing ecosystem data. Provide context for arbiter review.
- **Ledger MCP (chain data):** Verify attester balances for bond adequacy. Query `x/data` for existing attestation anchors. Verify beneficiary addresses.
- **M010 (Reputation Signal):** Query attester reputation for quality scoring. Attestation outcomes (valid release, challenged, slashed) feed back as M010 inputs.
- **M009 (Service Escrow):** Service providers (verifiers, MRV operators) may bond attestations related to their service deliverables.
- **M001-ENH (Credit Class Approval):** Methodology validation attestations may be required as part of credit class applications.
- **Arbiter DAO (DAO DAO):** v1 challenge resolution uses DAO DAO infrastructure. Arbiter selection, voting, and fee distribution.
- **x/data module:** On-chain IRI anchoring for attestation documents.

## 12. Acceptance tests

### Attestation lifecycle
1) **Full lifecycle (happy path):** Attester bonds 1000 REGEN for BaselineMeasurement. 48h activation delay passes. No challenge within 120-day window. Lock period (180 days) expires. Bond released to attester.
2) **Insufficient bond:** Attester submits ProjectBoundary with 200 REGEN (< 500 minimum). Rejected with `ErrInsufficientBond`.
3) **Unknown attestation type:** Attester submits with type "InvalidType". Rejected with `ErrUnknownAttestationType`.
4) **Activation delay:** Attestation in BONDED state does not activate until 48h passes.

### Challenge workflow
5) **Challenge during activation:** Challenger files challenge during 48h activation delay. Attestation transitions from BONDED to CHALLENGED (never reaches ACTIVE).
6) **Challenge active attestation:** Challenger deposits 100 REGEN (10% of 1000 bond) against active BaselineMeasurement. Attestation transitions to CHALLENGED.
7) **Insufficient challenge deposit:** Challenger deposits 50 REGEN (< 10% of 500 bond). Rejected with `ErrInsufficientChallengeDeposit`.
8) **Challenge window expired:** Challenger attempts to challenge after challenge_window_closes_at. Rejected with `ErrChallengeWindowClosed`.
9) **Challenge non-active attestation:** Challenger attempts to challenge RELEASED attestation. Rejected with `ErrAttestationNotChallengeable`.

### Resolution
10) **Arbiter resolves VALID:** Arbiter DAO votes VALID. Attester receives bond + challenge deposit - arbiter fee. Attestation status = RESOLVED_VALID.
11) **Arbiter resolves INVALID:** Arbiter DAO votes INVALID. Challenger receives 50% bond + deposit - arbiter fee. Community pool receives 50% bond. Attestation status = SLASHED.
12) **Only arbiter can resolve:** Non-arbiter address attempts resolution. Rejected with `ErrUnauthorized`.

### Bond release
13) **Release after lock period:** Lock period expires, no active challenge. Attester calls ReleaseBond. Bond returned.
14) **Release before lock period:** Attester attempts release before lock_expires_at. Rejected with `ErrLockPeriodNotExpired`.
15) **Release during active challenge:** Attester attempts release while attestation is CHALLENGED. Rejected.

### Security
16) **Bond conservation:** After any sequence of attestations (created, challenged, resolved, released), verify `bond_pool.balance = sum(locked_bonds) + sum(locked_deposits) - sum(disbursements)`.
17) **Arbiter neutrality:** Attester attempts to resolve their own challenge. Rejected.
18) **Slash distribution:** On INVALID resolution, verify challenger receives exactly 50% of bond + deposit - fee, and community pool receives exactly 50% of bond.
19) **Single active challenge:** Second challenge filed against already-CHALLENGED attestation. Rejected.
20) **Resolution finality:** Attempt to re-resolve an already-resolved challenge. Rejected.

## 13. Rollout plan

### v0 checklist (advisory-only)
- Define attestation types with bond schedules (ProjectBoundary, BaselineMeasurement, CreditIssuanceClaim, MethodologyValidation).
- Implement off-chain attestation quality scoring function (4-factor weighted composite).
- Publish quality scores in weekly digest alongside attestation activity.
- Query M010 reputation scores for attester addresses.
- Query KOI MCP for attestation document analysis.
- Implement deterministic test fixtures with realistic attestation data.
- Record all attestation lifecycle events in audit log.

### v1 outline (CosmWasm on-chain)
- Deploy `attestation-bond` CosmWasm contract with CreateAttestation, ChallengeAttestation, ResolveChallenge, ReleaseBond message handlers.
- Implement bond escrow with lock/release/slash flows.
- Integrate Arbiter DAO (DAO DAO subDAO) for decentralized challenge resolution.
- Implement challenge deposit and settlement economics.
- Implement 48h activation delay.
- Add governance-controlled parameters (min_bond, challenge_deposit_ratio, arbiter_fee_ratio).

---

## Appendix A — Source anchors
- `phase-2/2.1-token-utility-mechanisms.md` — M008 protocol specification (participants, token flows, state transitions, bond schedule, security invariants)
- `phase-3/3.1-smart-contract-specs.md` — CosmWasm contract: Attestation Bond (state, messages, core logic, Arbiter DAO integration)
