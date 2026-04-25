# RFC Governance Review Schema

Machine-readable schema for reviewing RFCs against the agentic-tokenomics graduation rules. Designed so an agent (or human reviewer) can mechanically check whether an RFC meets process and content criteria for any given lifecycle gate, then emit a structured judgment.

## Files

| File | Purpose |
|---|---|
| [`rfc-review-schema.yaml`](./rfc-review-schema.yaml) | The rules an agent runs over an RFC. Grouped into `always` rules + per-gate rules. Each rule has id, category, severity, pass/fail criteria, and citations back to graduation rules. |
| [`rfc-review-judgment.yaml`](./rfc-review-judgment.yaml) | The output shape — what the reviewer emits. Includes verdict, per-rule findings, recommendations, next_action, and audit trail. |

## How agents use this

The flow for an agent doing a review:

1. **Locate the RFC** by path or ID; read the file at the requested commit SHA.
2. **Determine the gate** — the agent is asked to evaluate either: (a) a specific transition (e.g., "is this RFC ready for `forum-posted`?"), or (b) general validity (`always` rules only).
3. **Run rules** in order: `always` rules first, then gate rules. Short-circuit blockers if desired, but prefer to run all rules and emit a complete finding list — humans want to see the full picture, not just the first failure.
4. **Emit judgment** in the schema above. Persist to KOI (`koi:judgment:<rfc-id>:<gate>:<commit-sha>`) so reviews are queryable historically.
5. **Notify the next actor** per `next_action`. RegenOS handles the routing.

## Division of labor

Not all rules are equally automatable. The expected division:

- **Mechanical checks** (`R-HEADER-*`, `R-LIFECYCLE-*`, `R-EVIDENCE-002`, `R-G-FP-001..002`, `R-G-ENA-001..003`) — agents run these reliably; humans should rarely need to verify.
- **Judgment-heavy checks** (`R-CLAIM-001`, `R-EVIDENCE-001`, `R-G-RAT-002..004`) — agents flag candidates; humans make the final call. The schema is a checklist for sponsors and ratifiers, not a substitute for them.

## What this schema deliberately does NOT do

- **It does not decide ratification.** The schema can verify that ratification *criteria are met* but the ratifier (a human steward) still signs the §6 log row. This preserves the two-eyes principle.
- **It does not resolve substantive disputes.** If two reviewers disagree on whether a counter-position has been "addressed", the schema will flag both findings; humans arbitrate. The schema is a checklist, not a judge of philosophical adequacy.
- **It does not handle on-chain submission.** Schema rules verify post-hoc that the proposer was human, but never authorize submission. That remains a human action.

## Recommended self-review before requesting human review

Before opening a PR to add or transition an RFC, run the schema against your draft. If your draft has any `blocker`-severity failures, fix them before requesting human review. This concentrates reviewer attention on substantive judgment (does this claim hold?) rather than mechanical hygiene (is the header complete?).

## Schema evolution

This is `schema_version: 0.1`. Expect breaking changes through 1.0 as the first concrete RFCs (starting with the proposed RFC-0001 — M013 fee split, see `GRADUATION_RULES.md` §Smallest possible first cut) expose gaps. Schema versions should themselves be governed by an RFC once the system is mature enough to bootstrap on itself.
