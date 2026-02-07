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

```typescript
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

- [PACTO Framework](https://www.notion.so/PACTO-framework-28b25b77eda180a499dafbf71583057d)
- [Protocol Agent Specifications](https://www.notion.so/Protocol-Agent-9-1f325b77eda180ea8c10eb83327f5895)
- [Regen Meta-Commons Coordination](https://regencommons.discourse.group/t/the-regen-meta-commons-coordination-network-nation/79)

---

*This document is part of the Regen Network Agentic Tokenomics framework.*
