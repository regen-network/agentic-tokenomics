# m001-enh — Credit Class Approval Voting Enhancement (v0 advisory)

m001-enh enhances the Regen Network credit class creator allowlist with **agent pre-screening**, **tiered approval thresholds**, and **deposit escrow** (v1). It builds on the existing `x/ecocredit` governance process with structured quality assessment before community vote.

## What it outputs
- An **agent pre-screening score** (0–1000) with confidence (0–1000) and recommendation (APPROVE / CONDITIONAL / REJECT) per proposal.
- A **proposal lifecycle** tracking state transitions from DRAFT through AGENT_REVIEW, VOTING, to APPROVED/REJECTED/EXPIRED.
- **KPI metrics**: proposals submitted, agent accuracy rate, approval rate, average time-to-decision.

## What it does not do (v0)
- No on-chain deposit escrow. Existing governance deposit rules apply.
- No on-chain agent scoring or `MsgSubmitAgentScore`. Agent scores are published in digest only.
- No PoA dual-track tally. Standard governance voting applies. v2 adds dual-track via M014.
- No automatic allowlist changes. Agent score is advisory/informational only.

## How to reference
- Canonical spec: `mechanisms/m001-enh-credit-class-approval/SPEC.md`
- State machine: SPEC.md section 6
- Scoring function: SPEC.md section 5 (4-factor weighted composite)
- PoA variant: SPEC.md section 8 (v2, requires M014)
- Other mechanisms may treat m001-enh agent scores as an input (e.g., GOV-001 process, governance digest).

## Replay datasets
See `datasets/` for deterministic fixtures used to generate non-zero KPI outputs without MCP.
- `v0_sample.json` — proposals with varied agent scores covering full lifecycle
- `v0_rejection_sample.json` — low-score proposals with human override scenarios

## Schemas
Canonical JSON schemas for m001-enh outputs live in `schemas/`.
- `m001_agent_score.schema.json` — agent pre-screening score output
- `m001_proposal.schema.json` — ClassCreatorProposal lifecycle object
- `m001_kpi.schema.json` — KPI metrics (proposals submitted, agent accuracy, approval rate)
