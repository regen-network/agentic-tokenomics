# PACTO & OPAL Alignment Framework

## PACTO: Participatory Co-Constitution Process

**Full Name:** Proceso de Acuerdos y Co-Constitución Tecnológicamente Organizada

### Essence

PACTO is a participatory, technology-enabled triadic process that ensures agentic systems remain aligned with human values and ecological imperatives through:

1. **Participatory Design**: Community involvement in agent capability definition
2. **Constitutional Grounding**: Agent behaviors bound by community-defined principles
3. **Technological Enablement**: AI augmentation of human deliberation

### The Triadic Structure

```
                    ┌─────────────────┐
                    │    ECOLOGICAL   │
                    │    IMPERATIVE   │
                    │                 │
                    │  (What Earth    │
                    │   Needs)        │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
       ┌──────────┐   ┌──────────┐   ┌──────────┐
       │  HUMAN   │   │  AGENT   │   │ COMMUNITY│
       │  VOICE   │◄──┤  PROCESS │──►│ COMMONS  │
       │          │   │          │   │          │
       │(Personal │   │(Technical│   │(Collective│
       │ agency)  │   │ capacity)│   │ wisdom)   │
       └──────────┘   └──────────┘   └──────────┘
```

### PACTO Principles

1. **Voice Sovereignty**: Every participant's voice has weight
2. **Agent Transparency**: All agent actions are auditable
3. **Collective Coherence**: Individual contributions synthesize into shared direction
4. **Living Process**: Agreements evolve through structured dialogue

---

## OPAL: Coherence Analysis Framework

OPAL provides coherence analysis across five dimensions:

| Dimension | Question | Application |
|-----------|----------|-------------|
| **O**ntology | What entities and relationships exist? | Agent knowledge boundaries |
| **P**hilosophy | What values guide decisions? | Constitutional constraints |
| **A**rchitecture | How do systems connect? | Technical integration |
| **L**anguage | How do we communicate? | Semantic standards |
| **A**ction | What behaviors are appropriate? | Agent capability bounds |

### OPAL Coherence Check Template

```yaml
proposal_coherence_check:
  ontology:
    - Does this proposal align with Regen's entity definitions?
    - Are new concepts properly integrated into existing schema?

  philosophy:
    - Does this support ecological regeneration?
    - Does it maintain human sovereignty?
    - Does it distribute benefits equitably?

  architecture:
    - Is it compatible with existing systems?
    - Does it introduce technical debt?
    - Are dependencies properly specified?

  language:
    - Does it use Living Language principles?
    - Is it accessible to non-technical participants?
    - Are terms defined consistently?

  action:
    - What agent capabilities does this enable/restrict?
    - What human actions are affected?
    - What are the reversibility characteristics?
```

---

## OPAL Coherence Scoring Algorithm

The coherence check template (above) provides qualitative questions. This section specifies how those questions are evaluated and translated into the 0.0–1.0 scores used by the `opal_scores` field in the [Work Order Schema](../../schemas/work-order.schema.json) and for governance routing decisions.

### Scoring Model Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                    OPAL SCORING PIPELINE                              │
│                                                                      │
│  ┌────────────┐    ┌─────────────────┐    ┌───────────────────┐     │
│  │  PROPOSAL   │───▶│  PER-DIMENSION  │───▶│  COMPOSITE SCORE  │     │
│  │  + CONTEXT  │    │  RUBRIC EVAL    │    │  + GOV ROUTING    │     │
│  └────────────┘    └─────────────────┘    └───────────────────┘     │
│       │                    │                        │                │
│  Inputs:              Evaluator:               Outputs:              │
│  - Proposal text      - AGENT-002 (primary)    - 5 dimension scores │
│  - KOI context        - Human reviewer         - overall composite  │
│  - Ledger state         (override/confirm)     - governance_layer   │
│  - Constitution       - Voice Council          - rationale per dim  │
│  - Architecture docs    (for contested items)  - routing decision   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Scoring Method: Rubric-Based Evaluation

Each OPAL dimension is scored using a structured rubric. The evaluator (AGENT-002 Governance Analyst, or a human reviewer) assesses the proposal against specific criteria and assigns a score per criterion. The dimension score is the weighted average of its criteria scores.

**Score Scale (per criterion):**

| Score | Meaning | Implication |
|-------|---------|-------------|
| 0.0 | No alignment / critical conflict | Blocks governance progression |
| 0.25 | Poor alignment / significant gaps | Requires major revision |
| 0.5 | Partial alignment / minor gaps | Acceptable with conditions |
| 0.75 | Good alignment / minor improvements possible | Ready for standard process |
| 1.0 | Full alignment / exemplary | May qualify for expedited process |

### Per-Dimension Scoring Rubrics

#### O — Ontology (Knowledge Coherence)

> *Does the proposal align with Regen's entity definitions and knowledge model?*

| Criterion | Weight | Inputs | Evaluation Method |
|-----------|--------|--------|-------------------|
| Entity alignment | 0.35 | KOI ontology graph, proposal entities | Check that all entities referenced in the proposal exist in or are compatible with the KOI ontology. New entity types score lower unless properly defined. |
| Schema compatibility | 0.30 | `regen-data-standards` schemas, proposal data structures | Verify data structures conform to or extend existing schemas without breaking changes. |
| Concept integration | 0.20 | KOI knowledge graph, existing mechanism specs | Assess whether new concepts are properly linked to existing knowledge. Orphaned concepts score 0.25. |
| Terminology consistency | 0.15 | Living Language glossary (KOI) | Check that terms match established definitions. Novel terms must include definitions. |

```
ontology_score = 0.35 × entity_alignment
              + 0.30 × schema_compatibility
              + 0.20 × concept_integration
              + 0.15 × terminology_consistency
```

**Agent evaluation:** AGENT-002 queries KOI MCP to resolve entities, check schema compatibility, and verify terminology. Returns score + list of unresolved entities.

#### P — Philosophy (Value Coherence)

> *Does the proposal align with Regen's values and constitutional principles?*

| Criterion | Weight | Inputs | Evaluation Method |
|-----------|--------|--------|-------------------|
| Ecological regeneration | 0.30 | Proposal intent, ecological impact assessment | Does the proposal directly or indirectly support ecological regeneration? Neutral = 0.5; harmful = 0.0. |
| Human sovereignty | 0.25 | Governance layer assignment, automation scope | Does the proposal maintain human decision authority at appropriate levels? Full automation of Layer 3+ decisions = 0.0. |
| Equitable distribution | 0.25 | Token flow analysis, beneficiary mapping | Are benefits distributed equitably? Does it concentrate power or resources inappropriately? |
| PACTO triadic alignment | 0.20 | Triadic structure assessment | Does the proposal balance ecological imperative, human voice, and community commons? |

```
philosophy_score = 0.30 × ecological_regeneration
                + 0.25 × human_sovereignty
                + 0.25 × equitable_distribution
                + 0.20 × pacto_triadic_alignment
```

**Agent evaluation:** AGENT-002 performs constitutional alignment check against Regen's stated values. **Human confirmation required** when philosophy_score < 0.5 (value misalignment is a critical concern that warrants human judgment).

#### A — Architecture (Technical Coherence)

> *Is the proposal technically sound and compatible with existing systems?*

| Criterion | Weight | Inputs | Evaluation Method |
|-----------|--------|--------|-------------------|
| System compatibility | 0.30 | Architecture docs (phase-1/1.5), module inventory | Does the proposal work with existing Cosmos SDK modules, MCP infrastructure, and agent runtime? |
| Dependency specification | 0.25 | Proposal dependencies, mechanism integration map | Are all dependencies explicitly listed and available? Missing dependencies score 0.25. |
| Technical debt impact | 0.20 | Codebase analysis, migration complexity | Does the proposal introduce technical debt? Major refactoring requirements score lower. |
| Security posture | 0.25 | Security framework (phase-3/3.4), invariant analysis | Does the proposal maintain or improve security invariants? New attack surfaces must be addressed. |

```
architecture_score = 0.30 × system_compatibility
                   + 0.25 × dependency_specification
                   + 0.20 × technical_debt_impact
                   + 0.25 × security_posture
```

**Agent evaluation:** AGENT-002 queries Ledger MCP for module compatibility, checks dependency graph, and runs security invariant analysis. Can be supplemented by AGENT-004 for validator-related proposals.

#### L — Language (Communicative Coherence)

> *Is the proposal accessible and clearly communicated?*

| Criterion | Weight | Inputs | Evaluation Method |
|-----------|--------|--------|-------------------|
| Clarity | 0.35 | Proposal text, readability metrics | Is the proposal understandable to its target audience? Technical proposals may have lower general readability but should be clear to their intended audience. |
| Accessibility | 0.30 | Audience analysis, language complexity | Can non-technical governance participants understand the implications? Jargon-heavy proposals without summaries score lower. |
| Terminological consistency | 0.20 | Living Language glossary (KOI), proposal terms | Does the proposal use terms consistently with established definitions? |
| Multilingual consideration | 0.15 | Language coverage, translation availability | For proposals affecting diverse communities: is the proposal accessible in relevant languages? (Score 0.75 default if single-language is acceptable for scope.) |

```
language_score = 0.35 × clarity
              + 0.30 × accessibility
              + 0.20 × terminological_consistency
              + 0.15 × multilingual_consideration
```

**Agent evaluation:** AGENT-002 performs NLP analysis for readability, checks terminology against KOI glossary, and flags jargon. This is the dimension most suited to fully automated scoring.

#### A — Action (Behavioral Coherence)

> *Are the behavioral implications appropriate and bounded?*

| Criterion | Weight | Inputs | Evaluation Method |
|-----------|--------|--------|-------------------|
| Agent capability bounds | 0.30 | Agent persona definitions (phase-2/2.4), proposed actions | Does the proposal stay within defined agent capabilities? Capability expansion requires explicit governance approval. |
| Human action impact | 0.25 | Governance process map (phase-2/2.3), affected workflows | What human actions are changed? Significant workflow disruptions score lower unless justified. |
| Reversibility | 0.25 | State transition analysis, rollback feasibility | Can the action be reversed if problems emerge? Irreversible actions on critical systems score lower. |
| Governance layer alignment | 0.20 | Layer classification (phase-1/1.4), proposal scope | Is the proposal being processed at the correct governance layer? Layer mismatches score 0.0. |

```
action_score = 0.30 × agent_capability_bounds
             + 0.25 × human_action_impact
             + 0.25 × reversibility
             + 0.20 × governance_layer_alignment
```

**Agent evaluation:** AGENT-002 checks proposed actions against agent capability definitions, classifies governance layer, and assesses reversibility. **Layer misalignment auto-flags for human review.**

### Composite Score Calculation

```
overall = w_O × ontology_score
        + w_P × philosophy_score
        + w_A × architecture_score
        + w_L × language_score
        + w_Act × action_score

Default weights (equal):
  w_O = w_P = w_A = w_L = w_Act = 0.20

Category-specific weight overrides:
  governance proposals:   w_P = 0.30, w_Act = 0.25, w_O = 0.15, w_A = 0.15, w_L = 0.15
  technical proposals:    w_A = 0.35, w_Act = 0.25, w_O = 0.20, w_P = 0.10, w_L = 0.10
  registry proposals:     w_O = 0.30, w_P = 0.25, w_A = 0.20, w_L = 0.15, w_Act = 0.10
  treasury proposals:     w_P = 0.30, w_A = 0.20, w_Act = 0.20, w_O = 0.15, w_L = 0.15
```

### Governance Routing Thresholds

OPAL scores determine governance layer routing and process speed:

| Condition | Routing Decision |
|-----------|-----------------|
| `overall >= 0.80` AND no dimension < 0.50 | **Expedited** — Layer 2 fast-track (24h human review window) |
| `overall >= 0.60` AND no dimension < 0.25 | **Standard** — Normal governance process at assigned layer |
| `overall >= 0.40` OR any dimension < 0.25 | **Enhanced review** — Escalate one governance layer up |
| `overall < 0.40` OR `philosophy < 0.25` | **Blocked** — Requires revision before governance progression |
| `philosophy == 0.0` | **Constitutional concern** — Automatic Layer 4 escalation |

### Scoring Workflow

```yaml
opal_scoring_workflow:
  trigger: work_order.created OR proposal.submitted
  evaluator: AGENT-002 (Governance Analyst)

  step_1_context_gathering:
    - fetch proposal text and metadata
    - query KOI MCP for ontology, glossary, related documents
    - query Ledger MCP for module state, governance parameters
    - load constitution and PACTO principles

  step_2_per_dimension_evaluation:
    for each dimension in [ontology, philosophy, architecture, language, action]:
      - evaluate each criterion per rubric
      - compute weighted dimension score
      - generate rationale (required: 2-3 sentences per dimension)

  step_3_composite_and_routing:
    - compute overall score using category-appropriate weights
    - determine governance routing per threshold table
    - flag any dimensions requiring human confirmation

  step_4_output:
    - populate opal_scores in work order schema
    - attach rationale document to KOI
    - emit EventOPALScoreComputed
    - if human_confirmation_required: notify designated reviewers

  human_override:
    - any OPAL score can be overridden by authorized human reviewer
    - overrides are logged with rationale in audit trail
    - override triggers re-computation of overall and routing
```

### Score Persistence and Versioning

```yaml
opal_score_record:
  work_order_id: string
  scored_at: timestamp
  scorer: string           # "AGENT-002" or human address
  version: integer         # increments on re-score or override
  scores:
    ontology: { score: float, rationale: string, criteria: [...] }
    philosophy: { score: float, rationale: string, criteria: [...] }
    architecture: { score: float, rationale: string, criteria: [...] }
    language: { score: float, rationale: string, criteria: [...] }
    action: { score: float, rationale: string, criteria: [...] }
    overall: float
  routing_decision: string   # expedited | standard | enhanced_review | blocked
  human_overrides: []        # list of override events
  governance_layer: string   # L1 | L2 | L3 | L4
```

### Open Questions

> **OQ-OPAL-1**: Should the scoring rubric weights be fixed or governance-adjustable? Fixed weights are simpler; adjustable weights allow community calibration over time.

> **OQ-OPAL-2**: The Language dimension includes a "multilingual consideration" criterion. For v0, should this be scored 0.75 by default (since most proposals are English-only and this is acceptable for current scope), or should it be omitted until multilingual governance is implemented?

> **OQ-OPAL-3**: Should OPAL scores be required for all governance actions, or only for work orders generated by Voice Councils? Requiring scores for all proposals adds overhead but ensures consistent evaluation.

> **OQ-OPAL-4**: How should the scoring interact with the M010 reputation system? A proposal's OPAL score could factor into the proposer's reputation, and the proposer's reputation could influence the scoring (e.g., higher trust = less scrutiny on Language dimension). Is this circular dependency acceptable?

---

## Voice Council Integration

### Purpose

Voice Councils enable humans to participate in governance through speech, with AI agents processing voice into structured work orders.

### Voice Council Session Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         VOICE COUNCIL SESSION                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. CONVENING                                                               │
│     └─▶ Facilitator opens session                                          │
│     └─▶ Participants join (voice + optional video)                         │
│     └─▶ Agent begins transcription                                         │
│                                                                             │
│  2. DELIBERATION                                                            │
│     └─▶ Topic introduction                                                  │
│     └─▶ Round-robin contributions                                          │
│     └─▶ Agent extracts key points in real-time                             │
│     └─▶ Clarification queries                                              │
│                                                                             │
│  3. SYNTHESIS                                                               │
│     └─▶ Agent presents summary of positions                                │
│     └─▶ Humans validate/correct synthesis                                  │
│     └─▶ Decision intents crystallized                                      │
│                                                                             │
│  4. WORK ORDER GENERATION                                                   │
│     └─▶ Agent drafts work orders from intents                              │
│     └─▶ Participants review orders                                         │
│     └─▶ Amendments via voice commands                                      │
│                                                                             │
│  5. SIGNING                                                                 │
│     └─▶ Work orders presented for signature                                │
│     └─▶ Participants sign (wallet or voice-authorized)                     │
│     └─▶ Signed orders queued for execution                                 │
│                                                                             │
│  6. EXECUTION                                                               │
│     └─▶ Agents execute signed work orders                                  │
│     └─▶ Results reported back to council                                   │
│     └─▶ Session closes with summary                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Work Order Schema

> **Implementation Note:** These type definitions are coordinated with
> [regen-data-standards](https://github.com/regen-network/regen-data-standards)
> schemas to ensure alignment with established taxonomies and norms.
> Formal LinkML schemas proposed in [regen-data-standards#51](https://github.com/regen-network/regen-data-standards/pull/51).
> JSON Schema for validation: [`schemas/work-order.schema.json`](../../schemas/work-order.schema.json).

```typescript
// Type Aliases
type timestamp = string;           // ISO 8601 format
type WorkOrderCategory = 'governance' | 'registry' | 'treasury' | 'technical';
type Priority = 'urgent' | 'high' | 'normal' | 'low';
type AgentId = string;             // e.g., "AGENT-001"
type ActionType = 'analyze' | 'prepare' | 'execute' | 'report';
type Duration = string;            // ISO 8601 duration, e.g., "P7D", "PT24H"
type ExecutionStatus = 'pending' | 'in_progress' | 'completed' | 'failed';

// Supporting Interfaces
interface Constraint {
  type: string;
  value: any;
  description: string;
}

interface Signer {
  address: string;
  role: string;
  signed_at?: timestamp;
  signature?: string;
}

interface ExecutionResult {
  success: boolean;
  outputs: Record<string, any>;
  error?: string;
}

interface AuditEntry {
  timestamp: timestamp;
  action: string;
  actor: string;                   // agent ID or human address
  details: string;
}

// Work Order Interface
interface WorkOrder {
  id: string;                    // Unique identifier
  session_id: string;            // Voice council session
  created_at: timestamp;         // Creation time

  // Intent
  intent: {
    description: string;         // Natural language description
    category: WorkOrderCategory; // governance | registry | treasury | technical
    priority: Priority;          // urgent | high | normal | low
  };

  // Specification
  specification: {
    agent_target: AgentId;       // Which agent should execute
    action_type: ActionType;     // analyze | prepare | execute | report
    parameters: Record<string, any>;
    constraints: Constraint[];
    timeout: Duration;
  };

  // Authorization
  authorization: {
    required_signatures: number;
    signers: Signer[];
    threshold_met: boolean;
    signature_deadline: timestamp;
  };

  // Execution
  execution: {
    status: ExecutionStatus;     // pending | in_progress | completed | failed
    started_at?: timestamp;
    completed_at?: timestamp;
    result?: ExecutionResult;
    audit_trail: AuditEntry[];
  };
}
```

---

## References

- [PACTO Framework](https://github.com/regen-network/pacto-framework)
- [Protocol Agent Specifications](https://github.com/regen-network/regen-agentc-synthesis/tree/main/01-protocol-politicians)
- [Regen Meta-Commons Coordination](https://hub.regencoordination.xyz/c/regen-commons/33) — migrated from regencommons.discourse.group ([migration proposal](https://hub.regencoordination.xyz/t/proposal-migrate-regen-commons-discourse-to-regen-coordination-forum/355))

---

*This document is part of the Regen Network Agentic Tokenomics framework.*
