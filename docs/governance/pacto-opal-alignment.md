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
