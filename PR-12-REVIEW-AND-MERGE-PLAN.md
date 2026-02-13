# PR #12 Review, Upgrade & Merge Plan

**PR**: [feat: Specification gaps — Economic Reboot, M010 Challenges, OPAL Scoring, M009/M011 Expansion, PoA Annotations](https://github.com/regen-network/agentic-tokenomics/pull/12)
**Author**: CShear
**Branch**: `feat/economic-reboot-mechanisms` → `main`
**Scope**: ~2,500 additions across 19 files (3 new, 16 modified)
**Date**: 2026-02-13

---

## Part 1: Our Review Comments

These are comments we should post on the PR before merging, organized by priority.

### 1.1 High Priority — Must Address Before Merge

#### H1: Staking Multiplier ↔ PoA Conflict (Agree with Gemini)

**File**: `phase-2/2.6-economic-reboot-mechanisms.md` (M012 Supply Algorithm)
**Line**: `staking_multiplier = 1 + (S_staked / S_total)`

The Gemini review correctly identified that M012's `staking_multiplier` relies on PoS staking data (`S_staked`), but M014 transitions the network to PoA and disables staking inflation. Post-M014, `S_staked` approaches zero, collapsing `staking_multiplier` to 1.0 and effectively disabling half the regrowth rate formula.

**Our comment**: The `staking_multiplier` should be reworked for PoA context. Two options:
1. Replace with a **stability tier multiplier** from M015 (using `S_stability_committed / S_total`), which preserves the intent of rewarding long-term commitment.
2. Replace with a **validator participation multiplier** (using `active_validators / max_validators` from M014), since PoA validator health is the new security signal.

Either approach maintains the formula's intent (reward committed participation) while being consistent with the PoA model. The current formula should include an explicit `staking_multiplier_poa_variant` field or a conditional: "Pre-PoA: uses S_staked; Post-PoA: uses S_stability_committed."

**glandua's related comment** — glandua asks whether PoA could simply be a validator allow list + stake threshold. This is a simpler model than M014's full composition framework. Worth noting as OQ-M014-6: "Should PoA retain a minimum stake threshold for allowed validators (simpler model) or use the full composition-based selection criteria (richer model)?" The two aren't mutually exclusive — the allow list is the M014 validator governance approval, and a stake threshold could be one of the admission criteria.

#### H2: `avg_automation_score` Calculation Error (Agree with Gemini)

**File**: `phase-1/1.2-tokenomic-mechanisms.md` (JSON registry summary)
**Line**: `"avg_automation_score": 0.71`

The JSON registry now contains 14 mechanisms with the following automation scores:
- Active (7): M001=0.7, M002=0.5, M003=0.3, M004=0.7, M005=0.3, M006=0.8, M007=0.8
- Proposed ecosystem (3): M008=0.6, M009=0.5, M011=0.85
- Proposed economic reboot (4): M012=0.9, M013=0.95, M014=0.5, M015=0.8

Sum = 0.7 + 0.5 + 0.3 + 0.7 + 0.3 + 0.8 + 0.8 + 0.6 + 0.5 + 0.85 + 0.9 + 0.95 + 0.5 + 0.8 = **9.7**
Average = 9.7 / 14 = **0.693** ≈ 0.69

**Our comment**: Update `avg_automation_score` from 0.71 to 0.69. Straightforward fix.

#### H3: Ecological Multiplier Can Go Negative (Agree with Gemini)

**File**: `phase-2/2.6-economic-reboot-mechanisms.md` (M012 Supply Algorithm)
**Line**: `ecological_multiplier = 1 - (ΔCO₂ / reference_value)`

If ΔCO₂ > reference_value (ecological metrics worsening significantly), this produces a negative ecological_multiplier, which makes the regrowth rate negative — meaning the mint formula (`M[t] = r × (C - S[t])`) would subtract from supply even in the mint step.

**Our comment**: Add explicit bounds. Either:
- `ecological_multiplier = max(0, 1 - (ΔCO₂ / reference_value))` — floor at zero (no minting when ecology degrades, but no extra burning)
- Or document that negative regrowth is intentional (ecology-penalized contraction). If intentional, add it to the Security Invariants and explain the interaction with B[t] burning (double contraction).

The v0 default of 1.0 (disabled) sidesteps this for now, but the formula as written needs clarification for v1.

### 1.2 Medium Priority — Should Address Before or Shortly After Merge

#### M1: Fee Distribution Range Critique — Author's Rebuttal Is Correct

**File**: `phase-2/2.6-economic-reboot-mechanisms.md` (M013 Model B)

Gemini flagged Model B's ranges (25-35% burn, 15-25% validator, 50-60% community) as summing to 90-120%. CShear correctly responded that these are coordinated ranges where valid combinations sum to 100% (e.g., 25% + 15% + 60% = 100%).

**Our comment**: Agree with CShear's rebuttal. However, the ranges as written could confuse future readers. Suggest adding an explicit note:

> "These are coordinated ranges, not independent selections. Any valid configuration must satisfy the Share Sum Unity invariant (sum = 1.0). Example valid configurations: {25%, 15%, 60%}, {30%, 20%, 50%}, {35%, 15%, 50%}."

This makes the constraint explicit without changing the spec content.

#### M2: Proposal Submission Gaming (Agree with Gemini)

**File**: `phase-2/2.6-economic-reboot-mechanisms.md` (M015 Activity Weights)
**Line**: `Proposal Submission | 0.10`

Rewarding proposal submission at 10% weight without quality filters creates a spam incentive. Anyone can submit low-quality proposals to inflate their activity score.

**Our comment**: Add a qualifying condition: "Proposals must reach quorum OR pass to earn the full submission weight. Proposals that fail to reach quorum earn 0. Proposals that reach quorum but fail earn 50% weight. This leverages the existing x/gov deposit mechanism (proposals that don't reach quorum lose their deposit, creating natural friction)." Alternatively, defer to OQ-M015-4 (anti-gaming measures) as the resolution venue.

#### M3: Ambiguous Trade Scoring — Buyer vs. Seller (Agree with Gemini)

**File**: `phase-2/2.6-economic-reboot-mechanisms.md` (M015 Security Invariants)
**Line**: "marketplace trades count for buyer OR seller, not both"

The invariant states no double-counting but doesn't specify which party gets credit.

**Our comment**: Specify a default rule: "Credit purchase activity is counted for the buyer (as demand signal); credit retirement is counted for the retirer. Marketplace trade fee is counted for the buyer (who initiated the purchase). Seller activity is captured via issuance scores." Alternatively, make this an explicit open question (OQ-M015-5) if the WG hasn't decided.

#### M4: M010 Challenge Schema — Status Enum Mismatch

**File**: `mechanisms/m010-reputation-signal/schemas/m010_challenge.schema.json`
**Line**: `"enum": ["pending", "resolved_valid", "resolved_invalid", "escalated"]`

The challenge schema status enum includes `"escalated"` but the SPEC.md state machine (section 6.1) doesn't define an ESCALATED state for challenges. The SPEC mentions auto-escalation to governance when `resolution_deadline` expires, but it's described as an action, not a challenge status.

**Our comment**: Either:
- Add ESCALATED as an explicit challenge state in SPEC.md section 6.1 (recommended — it represents a real workflow step).
- Or remove `"escalated"` from the schema and handle escalation as a separate governance event.

#### M5: Sequence Diagram Formatting

**File**: `phase-2/2.6-economic-reboot-mechanisms.md` (Implementation Sequence)
**Line**: Around the `Phase 2 → Phase 3` arrow

The implementation sequence ASCII diagram has a stray arrow formatting issue. Minor but visible.

**Our comment**: Fix the alignment in the implementation sequence diagram.

### 1.3 Low Priority — Non-blocking Observations

#### L1: M010 v0_challenge_sample.json Signal ID References

The `expected_outputs` in the challenge fixture reference signals by sequential IDs (signal-1 through signal-8), but the events array doesn't include explicit `signal_id` fields — they're identified by array position. Adding a `signal_id` field to each event would make the fixture self-documenting and less fragile.

#### L2: OPAL Weight Sums for Category Overrides

The category-specific OPAL weights are documented to override defaults:
- governance: 0.30 + 0.25 + 0.15 + 0.15 + 0.15 = 1.0 ✓
- technical: 0.35 + 0.25 + 0.20 + 0.10 + 0.10 = 1.0 ✓
- registry: 0.30 + 0.25 + 0.20 + 0.15 + 0.10 = 1.0 ✓
- treasury: 0.30 + 0.20 + 0.20 + 0.15 + 0.15 = 1.0 ✓

All correct. No action needed — just confirming we validated.

#### L3: M009 Event Name Typo

**File**: `phase-2/2.1-token-utility-mechanisms.md` (M009 Implementation Notes)
**Line**: `EventMilestonSubmitted` (missing 'e' — should be `EventMilestoneSubmitted`)

#### L4: Work Order Schema `opal_scores` Field

The diff shows `schemas/work-order.schema.json` gained an `opal_scores` field. This is properly cross-referenced from the OPAL scoring algorithm in `pacto-opal-alignment.md`. The integration is clean.

---

## Part 2: Upgrade Plan

Based on the review findings, here are the specific changes needed before merge.

### 2.1 Required Fixes (Pre-Merge)

| # | File | Change | Severity |
|---|------|--------|----------|
| U1 | `phase-2/2.6-economic-reboot-mechanisms.md` | Add PoA-aware variant for `staking_multiplier` in M012 | High |
| U2 | `phase-1/1.2-tokenomic-mechanisms.md` | Fix `avg_automation_score` from 0.71 to 0.69 | High |
| U3 | `phase-2/2.6-economic-reboot-mechanisms.md` | Add explicit bounds to `ecological_multiplier` formula or document intentional negative behavior | High |
| U4 | `phase-2/2.6-economic-reboot-mechanisms.md` | Add coordination note to Model B fee distribution ranges | Medium |
| U5 | `phase-2/2.6-economic-reboot-mechanisms.md` | Add qualifying condition for proposal submission rewards (M015) or expand OQ-M015-4 | Medium |
| U6 | `phase-2/2.6-economic-reboot-mechanisms.md` | Clarify buyer vs. seller credit for trade scoring (M015) | Medium |
| U7 | `mechanisms/m010-reputation-signal/schemas/m010_challenge.schema.json` | Reconcile `"escalated"` status with SPEC.md states | Medium |
| U8 | `phase-2/2.6-economic-reboot-mechanisms.md` | Fix implementation sequence diagram formatting | Low |
| U9 | `phase-2/2.1-token-utility-mechanisms.md` | Fix `EventMilestonSubmitted` typo → `EventMilestoneSubmitted` | Low |
| U10 | `mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_sample.json` | Add explicit `signal_id` fields to events | Low |

### 2.2 Recommended Additions (Post-Merge or Follow-up PR)

| # | Description | Rationale |
|---|-------------|-----------|
| A1 | Add OQ-M014-6 re: glandua's simpler PoA model (allow list + stake threshold) | Captures community feedback |
| A2 | Add OQ-M015-5 re: buyer vs. seller activity credit assignment | Explicitly surfaces unresolved design decision |
| A3 | Add explicit `staking_multiplier` deprecation note for post-PoA | Future-proofing |
| A4 | Contributor guide (deferred from this PR per author) | PR description mentions this as gap #6 |

### 2.3 Upgrade Implementation Details

#### U1: Staking Multiplier PoA Variant

In `phase-2/2.6-economic-reboot-mechanisms.md`, in the M012 Supply Algorithm section, after the existing `staking_multiplier` definition, add:

```
  Post-PoA variant (activated with M014):
    staking_multiplier is replaced by stability_multiplier:
    stability_multiplier = 1 + (S_stability_committed / S_total)
      range: [1.0, 2.0]
      effect: higher stability tier commitment → faster regrowth
      source: M015 stability tier commitments

    NOTE: During PoA transition (M014 Phase 2-3), whichever multiplier
    yields the higher value is used, ensuring no regrowth disruption.
```

Also add to Open Questions:

```
> **OQ-M012-5**: Should the staking_multiplier be replaced by a stability_multiplier
> (from M015 commitments) or a validator_participation_multiplier (from M014 active
> set health)? The stability multiplier preserves the "reward commitment" intent;
> the validator multiplier links supply health to network security.
```

#### U2: Fix avg_automation_score

In `phase-1/1.2-tokenomic-mechanisms.md`, change:
```json
"avg_automation_score": 0.71
```
to:
```json
"avg_automation_score": 0.69
```

#### U3: Ecological Multiplier Bounds

In `phase-2/2.6-economic-reboot-mechanisms.md`, change:
```
  ecological_multiplier = 1 - (ΔCO₂ / reference_value)
```
to:
```
  ecological_multiplier = max(0, 1 - (ΔCO₂ / reference_value))
    range: [0.0, 1.0+]
    effect: improving ecological metrics → faster regrowth;
            worsening metrics → regrowth slows toward zero but does not invert
    NOTE: ecological_multiplier is floored at 0 to prevent negative minting.
          Supply contraction occurs exclusively through burning (B[t]),
          not through negative regrowth. This separation maintains
          Mint-Burn Independence (Security Invariant 4).
```

#### U4: Model B Coordination Note

After Model B's table, add:
```
> **Note**: These are coordinated ranges — any valid configuration must satisfy
> Share Sum Unity (sum = 1.0). Not all combinations within ranges are valid.
> Example valid configurations: {25%, 15%, 60%}, {30%, 20%, 50%}, {35%, 15%, 50%}.
```

#### U5: Proposal Submission Qualification

In M015 Activity Weights, change the Proposal Submission row to:
```
| Proposal Submission | 0.10 | Rewards initiative; requires reaching quorum to earn credit (see note) |
```

Add note below the table:
```
**Proposal submission anti-gaming**: To prevent spam proposals from inflating
activity scores, proposal submission credit is conditional:
- Proposals reaching quorum and passing: full 0.10 weight
- Proposals reaching quorum but failing: 50% weight (0.05 effective)
- Proposals failing to reach quorum: 0 weight
This leverages the existing x/gov deposit mechanism as natural friction.
```

#### U6: Trade Scoring Clarity

In M015 Security Invariants, expand invariant 5:
```
5. **No Double-Counting**: Each transaction counted once for reward scoring.
   Marketplace trades: credit is assigned to the **buyer** (as the demand signal).
   Seller activity is captured via issuance and facilitation scores, not trade scores.
   Credit retirements: credit is assigned to the **retirer**.
```

#### U7: Challenge Schema Escalation Status

In `mechanisms/m010-reputation-signal/schemas/m010_challenge.schema.json`, the `"escalated"` status should be kept (it's a valid workflow state when auto-escalation occurs). Add to SPEC.md section 6.5 under "Admin safeguards (v0)":

```
Note: When a challenge auto-escalates due to resolution timeout, its status
transitions to ESCALATED. An escalated challenge follows the governance
(Layer 3) resolution process rather than admin resolution.

CHALLENGED → ESCALATED (added transition):
  trigger: resolution_deadline_expired AND challenge.status == pending
  action: escalate to governance Layer 3 vote,
          emit EventChallengeEscalated
```

#### U8: Sequence Diagram Fix

Fix the arrow alignment in the implementation sequence in M012-M015 summary.

#### U9: Event Name Typo

Change `EventMilestonSubmitted` to `EventMilestoneSubmitted`.

#### U10: Challenge Fixture Signal IDs

Add `"signal_id": "signal-N"` to each event in the `v0_challenge_sample.json` events array.

---

## Part 3: Merge Plan

### 3.1 Pre-Merge Checklist

```
[ ] 1. Apply required fixes U1-U3 (high priority)
[ ] 2. Apply medium-priority fixes U4-U7
[ ] 3. Apply low-priority fixes U8-U10
[ ] 4. Post review comments on PR for traceability
[ ] 5. Verify all cross-references are intact:
       - M012-M015 references from phase-1/1.2 ✓
       - OPAL scoring referenced from work-order.schema.json ✓
       - M010 challenge workflow referenced from M010 README ✓
       - PoA annotations referenced from governance processes ✓
       - New agent workflows (WF-RR-04, WF-MM-04) referenced from workflow summary ✓
[ ] 6. Validate JSON schemas parse correctly (m010_challenge, m010_kpi, m010_signal, work-order)
[ ] 7. Confirm no broken markdown links
```

### 3.2 Merge Strategy

**Recommended**: **Squash merge** with a comprehensive commit message.

Rationale:
- The 7 commits in the PR are logically related and represent a single feature set.
- Squash merge keeps `main` history clean while preserving the PR link for full commit-level detail.
- The PR touches 19 files with 2,500+ lines — a single merge commit is easier to revert if needed.

**Merge commit message**:
```
feat: Add Economic Reboot specs (M012-M015), expand M009/M010/M011, add OPAL scoring, PoA annotations (#12)

- M012: Fixed Cap Dynamic Supply with algorithmic mint/burn
- M013: Value-Based Fee Routing with 4-pool distribution
- M014: Authority Validator Governance (PoS→PoA transition)
- M015: Contribution-Weighted Rewards replacing passive staking
- M010: Challenge/dispute workflow with 7-state lifecycle
- OPAL: Coherence scoring algorithm with 5-dimension rubric
- M009: Full escrow spec with milestone lifecycle and agent integration
- M011: Full curation spec with collection lifecycle and quality scoring
- PoA transition annotations across M001, M002, governance processes
- 36 open questions documented for WG resolution

Co-authored-by: CShear
```

### 3.3 Merge Sequence

```
Step 1: Create a fixup branch from PR head
        git checkout pr-12
        git checkout -b claude/review-pr-merge-plan-TOewb

Step 2: Apply all upgrades (U1-U10) as a single commit
        "fix: Address review findings — formula bounds, score correction, schema alignment"

Step 3: Push fixup branch, open as suggestion or push directly to PR branch
        (Depending on permissions — if we can push to the fork, push directly;
         otherwise, suggest changes via review comments)

Step 4: After fixes are applied, approve the PR

Step 5: Squash merge to main

Step 6: Verify main builds clean (no broken links, valid JSON schemas)

Step 7: Tag or note the 36 open questions as WG action items
```

### 3.4 Post-Merge Actions

1. **Open follow-up issues** for the 36 open questions, grouped by mechanism:
   - Issue: "Resolve M012 open questions (OQ-M012-1 through OQ-M012-5)"
   - Issue: "Resolve M013 open questions (OQ-M013-1 through OQ-M013-5)"
   - Issue: "Resolve M014 open questions (OQ-M014-1 through OQ-M014-6)"
   - Issue: "Resolve M015 open questions (OQ-M015-1 through OQ-M015-5)"
   - Issue: "Resolve OPAL open questions (OQ-OPAL-1 through OQ-OPAL-4)"
   - Issue: "Resolve governance transition open questions (OQ-GOV-POA-1 through OQ-GOV-POA-3)"
   - Issue: "Resolve M001-ENH open questions (OQ-M001-1, OQ-M001-2)"
   - Issue: "Resolve M009 open questions (OQ-M009-1 through OQ-M009-4)"
   - Issue: "Resolve M011 open questions (OQ-M011-1 through OQ-M011-5)"

2. **Contributor guide** (gap #6): Open a separate issue for community-driven contributor guide, deferred per author's note.

3. **Respond to glandua**: Post a reply acknowledging glandua's PoA simplification suggestion, noting it's been captured as OQ-M014-6 and that the allow-list + stake-threshold model is compatible with M014's composition framework (the allow list IS the governance approval; stake threshold could be one admission criterion).

---

## Summary

| Category | Count | Blocking? |
|----------|-------|-----------|
| High priority review comments | 3 | Yes — must fix before merge |
| Medium priority review comments | 5 | Recommended before merge |
| Low priority observations | 4 | Non-blocking |
| Required code fixes | 10 | U1-U3 must be fixed; U4-U10 recommended |
| Post-merge follow-up issues | 9 | Track open questions |

The PR is **high quality overall** — it fills significant specification gaps with well-structured, internally consistent documentation. The issues found are formula edge cases and minor inconsistencies, not fundamental design problems. After applying the fixes, this PR is ready for merge.
