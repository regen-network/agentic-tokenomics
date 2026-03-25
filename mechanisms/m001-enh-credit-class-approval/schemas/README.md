# m001-enh output schemas

These JSON Schemas define **canonical output shapes** for m001-enh (Credit Class Approval Voting Enhancement) artifacts.

## Files
- `m001_agent_score.schema.json` — schema for agent pre-screening score output (score, confidence, recommendation, factor breakdown).
- `m001_proposal.schema.json` — schema for ClassCreatorProposal lifecycle objects (status, deposit, agent score, outcome).
- `m001_kpi.schema.json` — schema for KPI metrics (proposals submitted, agent accuracy, approval rate, time-to-decision, deposit economics).

## Notes
- These schemas are intended for **validation** and consistency across repos (Heartbeat, agent skills, etc.).
- v0 is advisory-only: schemas describe outputs, not enforcement.
- The `status` field on proposals tracks lifecycle state (DRAFT → AGENT_REVIEW → VOTING → APPROVED/REJECTED/EXPIRED). See SPEC.md section 6.
- Credit types are constrained to current on-chain values: C, KSH, BT, MBS, USS.
