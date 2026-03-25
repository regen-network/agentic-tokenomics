# m014 datasets (replay fixtures)

These fixtures are **deterministic inputs** for generating non-zero m014 KPI outputs **without MCP**.

## Files
- `schema.json` — JSON schema for replay datasets
- `fixtures/v0_sample.json` — 5 active validators across 3 categories with varied performance profiles
- `fixtures/v0_transition_sample.json` — PoS-to-PoA transition snapshot with validators in all lifecycle states (active, probation, candidate, removed, term_expired)

## How they are used
A replay runner (e.g., in `regen-heartbeat`) can read a fixture file and compute:
- `total_validators` = number of validators in the fixture
- `validators_by_status` = count per lifecycle state
- `validators_by_category` = count of active validators per composition category
- `avg_performance_score` = mean composite score across active validators
- `composition_valid` = whether each category has >= 5 active validators
- `byzantine_tolerance` = whether active set satisfies 3f + 1

The transition fixture (`v0_transition_sample.json`) additionally exercises:
- Mixed lifecycle states representing a mid-migration snapshot
- Probation with poor performance factors
- Removed validator with documented reason
- Term-expired validator awaiting re-application
- The `expected_outputs` field documents expected aggregate state

These datasets are **specification-only** and do not imply enforcement or on-chain actions.
