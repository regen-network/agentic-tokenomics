# m001-enh datasets (replay fixtures)

These fixtures are **deterministic inputs** for generating non-zero m001-enh KPI outputs **without MCP**.

## Files
- `schema.json` — JSON schema for replay datasets
- `fixtures/v0_sample.json` — 5 proposals covering the full lifecycle (approved, rejected, expired) with varied agent scores
- `fixtures/v0_rejection_sample.json` — rejection and edge-case scenarios (auto-reject, human override, low confidence, agent timeout)

## How they are used
A replay runner can read a fixture file and compute:
- `proposals_submitted` = count(proposals)
- `proposals_approved` = count(status == "APPROVED")
- `proposals_rejected` = count(status == "REJECTED" or outcome.result == "AUTO_REJECTED")
- `approval_rate` = approved / decided
- `agent_scoring.agent_accuracy` = fraction of agent recommendations matching governance outcome
- `avg_time_to_decision_hours` = mean(decision_time - submit_time) in hours

The rejection fixture (`v0_rejection_sample.json`) additionally exercises:
- Auto-reject: score < 300 and confidence > 900, override window expires
- Human override: validator overrides agent rejection within 6h window
- Low confidence: score < 300 but confidence <= 900, advances as CONDITIONAL
- Agent timeout: no score within 24h, proposal advances without agent input

These datasets are **advisory-only** and do not imply enforcement or on-chain actions.

## Grounding
Proposals reference realistic Regen Network credit types (C, KSH, BT, MBS, USS) and methodology IRIs based on existing credit class patterns (C01–C09, KSH01, BT01, MBS01, USS01).
