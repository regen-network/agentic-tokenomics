# Voice Memo Analysis: Dynamic Validator Set, Agent Provisioning & Sustainable Spin Model

**Date**: 2026-02-12
**Source**: Voice memo from project lead
**Context**: Review against existing Agentic Tokenomics framework (Phases 1-3) and Feasibility Review

---

## 1. Voice Memo — Distilled Proposals

The memo proposes 7 interconnected ideas. I'll name them for reference:

| ID | Proposal | Core Concept |
|----|----------|-------------|
| **VM-01** | Dynamic Validator Set Sizing | Validator count responsive to network traffic and revenue |
| **VM-02** | Sustainable Spin Model | Eco-credit on-chain value + exchange value funds protocol-level validator economics |
| **VM-03** | Independent Validator Governance | Registry/allowlist system with independent validator governance body |
| **VM-04** | Agent Provisioning as Co-Creator Role | New co-creator class: agent operators alongside validators |
| **VM-05** | Validator-to-Compute Transition | Validators shift from block production to providing compute for agent swarms |
| **VM-06** | Harmonized Swarm Work Allocation | Swarms of teams (agents + humans) on shared objectives, with process for registering new ideas/initiatives |
| **VM-07** | Beyond Eco-Credits | Expand to ecological claims, data management, knowledge commoning, fresh GTM |

### The North Star (as stated)

> "Harmonizing the agency of all of these different stakeholders to achieve ecological regeneration outcomes. And to transform the human economy to value those regenerative ecological transformations highly enough such that they become common and core piece of the human and technological economies."

---

## 2. Mapping Against the Existing Framework

### What the framework already covers well

| Voice Memo Idea | Existing Coverage | Location |
|----------------|-------------------|----------|
| VM-03 (Validator governance) | Partially covered — GOV-001 allowlist pattern exists for credit class creators | `phase-2/2.3-governance-processes.md` |
| VM-04 (Agent provisioning) | Agent personas defined, but not as a stakeholder *role* with economic participation | `phase-2/2.4-agent-orchestration.md` |
| VM-06 (Coordinated work) | OODA workflows and inter-agent communication sketched | `phase-2/2.2-agentic-workflows.md` |
| VM-07 (Beyond eco-credits) | Stakeholder Pentad acknowledges broader value, but mechanisms are eco-credit-specific | `phase-1/1.1-stakeholder-value-flow.md` |

### What the framework does NOT cover — and must

| Voice Memo Idea | Gap | Severity |
|----------------|-----|----------|
| **VM-01** (Dynamic validator set) | No mechanism for responsive validator set sizing. Current spec treats 75 validators as static. | **Major** — this is a protocol-level change |
| **VM-02** (Sustainable spin model) | No revenue model connecting eco-credit value to validator economics. Validators earn staking rewards only. | **Major** — economics are disconnected |
| **VM-04** (Agent operators as co-creators) | Agents are treated as infrastructure, not as stakeholder-economic participants. No "agent operator" role in the Stakeholder Pentad. | **Major** — requires stakeholder model revision |
| **VM-05** (Validator → compute transition) | Not addressed at all. Validators and agent compute are separate concerns in current framework. | **Major** — new economic mechanism needed |
| **VM-07** (Fresh GTM processes) | No mechanism for proposing, evaluating, and rewarding new market opportunities. | **Significant** — the framework optimizes existing processes but doesn't create new ones |

---

## 3. Detailed Assessment of Each Proposal

### VM-01: Dynamic Validator Set Sizing

**What's proposed**: Validator set shrinks to minimum-for-liveness when traffic is low, expands up to Cosmos SDK maximum when traffic justifies it.

**Technical feasibility**: MODERATE

- The Cosmos SDK `x/staking` module supports `MaxValidators` as a governance parameter. It can be changed via parameter change proposal (GOV-005 in the existing framework).
- However, **dynamic automatic adjustment** of `MaxValidators` is not a native Cosmos SDK feature. This would require either:
  - (a) A custom module that monitors on-chain metrics and proposes parameter changes — which is *exactly* what AGENT-004 (Validator Monitor) could do via WF-VM-03
  - (b) A CosmWasm contract that triggers governance proposals based on thresholds — feasible but adds complexity

- **Minimum for liveness**: Tendermint requires 2/3+1 of validators for liveness. With the minimum viable set, this means at least 4 validators (3 to achieve 2/3 + 1 for safety margin). Practically, the floor should be higher (e.g., 10-15) for decentralization.

- **Maximum**: Cosmos SDK historically supports up to 300 validators (Cosmos Hub), though performance degrades. Regen's current 75 is already moderate.

**What's missing from the framework**:
- A metric for "network traffic" that drives sizing decisions (tx/day? eco-credit issuance volume? revenue?)
- Threshold functions mapping traffic metrics to target validator count
- Transition procedures for validators entering/exiting the active set
- Economic analysis of the minimum viable validator economics at different set sizes

**Risk**: Shrinking the validator set concentrates power. If done reactively to low revenue, it could create a death spiral: fewer validators → less decentralization → less trust → less adoption → less revenue → fewer validators.

**Recommendation**: Define this as a new mechanism **M012: Dynamic Validator Set Sizing**. Implement as an extension of AGENT-004's WF-VM-03 (Decentralization Monitoring), where the agent monitors traffic-to-validator-ratio metrics and *proposes* (not executes) parameter changes via Layer 3 governance. Include hard floors and ceilings as constitutional parameters (Layer 4).

---

### VM-02: Sustainable Spin Model

**What's proposed**: Protocol-level sustainability where eco-credit on-chain value combined with exchange value generates a funding model for validators.

**Technical feasibility**: MODERATE-TO-HIGH (conceptually sound, mechanically complex)

This is the most economically significant proposal because it addresses a real problem: **validators currently earn only staking rewards and tx fees, which are disconnected from the ecological value the network produces**.

The "spin model" implies a **flywheel**:

```
Eco-credits registered on-chain
    → Creates attestable ecological value
    → Exchange activity generates fees
    → Fees fund validator operations
    → Validators secure the network
    → Security enables more eco-credit registration
    → Flywheel spins
```

**What's needed that the framework doesn't have**:
1. **A fee capture mechanism** that routes a percentage of eco-credit marketplace transactions to a validator sustainability pool (distinct from the community pool)
2. **A value-linkage formula** connecting eco-credit volume/value to validator compensation
3. **Economic modeling** of whether current eco-credit volumes (78+ batches, 23+ sell orders) generate sufficient fees to sustain even a minimal validator set

**Honest assessment**: The memo itself acknowledges eco-crediting is "a pretty soft market, not really proving to be as lucrative as we want." This is the critical vulnerability of VM-02 — the spin model only works if the top of the funnel (eco-credit demand) generates enough value. This makes VM-07 (expanding beyond eco-credits) not optional but *prerequisite*.

**Recommendation**: Define as **M013: Validator Sustainability Pool**. Model as a fee-split mechanism where X% of marketplace fees (M004) flow to a validator sustainability fund. But critically, do the math first — model current and projected volumes against minimum viable validator economics. If the numbers don't work with eco-credits alone, that's the forcing function for VM-07.

---

### VM-03: Independent Validator Governance

**Technical feasibility**: HIGH

This is the most directly implementable proposal because the pattern already exists in the framework:

- GOV-001 (Credit Class Creator Allowlist) defines a registry + application + allowlisting process
- The same pattern can be applied to validators: apply → review → allowlist → monitor → remove

**What's needed**:
- **GOV-006: Validator Allowlist Governance** — mirror GOV-001 structure
- **An independent validator governance body** (the memo says "independent validator governance") — this maps to a new DAO via DAO DAO, similar to the arbiter DAOs proposed for M008/M009
- **AGENT-004 enhancement**: Add a workflow WF-VM-04 for Validator Application Review (parallel to WF-RR-01 for credit class applications)

**Recommendation**: Lowest implementation friction of all VM proposals. Can be specified as a Phase 2 addendum and built alongside existing GOV-001 patterns.

---

### VM-04: Agent Provisioning as Co-Creator Role

**What's proposed**: Agent operators (humans/orgs running agent management, governance, registry agents) become a formal co-creator stakeholder class, compensated from protocol revenue for covering compute costs.

**This is the most architecturally transformative proposal.** It reframes agents from "infrastructure" to "stakeholder-participants" — a conceptual shift the current framework doesn't make.

**Current framework gap**: The Stakeholder Pentad in Phase 1.1 lists Co-Creators as:
- Validators
- Verifiers
- Methodology Developers

Agent operators are **not** in this list. They're treated as part of the "system architecture" (Phase 1.5), not as economic participants.

**What VM-04 requires**:

1. **Stakeholder Pentad revision**: Add "Agent Operators" to the Co-Creator class alongside Validators, Verifiers, and Methodology Developers

2. **A new mechanism M014: Agent Provisioning Registry**:
   - Agent operators apply to provide specific agent services (governance analysis, registry review, market monitoring, etc.)
   - Permissioned via an allowlist governance process (GOV-007)
   - Compensated from protocol revenue for compute costs
   - Subject to performance monitoring (SLA compliance, decision quality, uptime)

3. **Economic model for agent compensation**:
   - What's the cost of running an agent? (LLM API costs, compute, storage)
   - How is this funded? (from M013 validator sustainability pool? from a separate agent pool? from per-service fees?)
   - What's the incentive alignment? (agent operators profit only if their agents provide high-quality service)

**Risk**: Creating an "agent operator" rent-seeking class. If operators are paid per-agent regardless of value, you get bloat. Compensation must be tied to measurable outcomes.

**Recommendation**: Specify this as the most important new mechanism, but design it *after* the sustainable spin model (VM-02/M013) is economically validated. You can't fund agent operators from protocol revenue if the revenue isn't there.

---

### VM-05: Validator-to-Compute Transition

**What's proposed**: Validators shift from traditional block validation to providing compute for agent swarms — "lower cost, higher impact work."

**Technical feasibility**: LOW-TO-MODERATE (conceptually appealing, mechanically complex)

This conflates two fundamentally different roles:
- **Validators** secure consensus (Tendermint BFT), must run full nodes, sign blocks, maintain uptime
- **Compute providers** run agent workloads (ElizaOS, LLM inference, database queries)

These have different hardware profiles, different SLAs, and different failure modes. A validator going offline means missed blocks and slashing. A compute provider going offline means degraded agent services but no consensus failure.

**What could work**: A model where the *same entities* participate in both roles, but these remain distinct on-chain roles with separate staking, separate slashing, and separate compensation. The "transition" is organizational (the same orgs/humans shift focus) not architectural (the same software serves both roles).

**What's dangerous**: Coupling consensus security to agent compute availability. If a validator's agent compute is overloaded and it affects their block signing, the network's security degrades.

**Recommendation**: Design as **parallel roles, not a transition**. An entity can be both a validator and an agent compute provider, with separate registrations, separate bonds, and separate performance metrics. The framework's existing AGENT-004 (Validator Monitor) already tracks validator performance — extend it to also monitor compute provider performance if the same entity holds both roles. Do NOT create a mechanism where validators must provide compute or vice versa.

---

### VM-06: Harmonized Swarm Work Allocation

**What's proposed**: "Swarms of teams working on the same general bit" with "a clear process to register new ideas" — covering infrastructure, BD, sales, storytelling, documentation.

**This is where the voice memo directly addresses the feasibility review's critique** that the framework lacks task decomposition and delegation protocols for true swarm management.

**What this needs that the framework doesn't have**:

1. **A work registration protocol**: How does a new initiative (e.g., "build a carbon credit marketplace integration with Shopify") get proposed, evaluated, approved, and assigned to a swarm?

2. **Role-typed agent slots**: The current framework has 4 fixed agent types. VM-06 implies dynamic roles:
   - Infrastructure agents (existing: AGENT-001 through AGENT-004)
   - Business development agents (new)
   - Documentation agents (new)
   - Storytelling/communications agents (new)
   - Sales/outreach agents (new)

3. **Value attribution**: How do you measure and reward the contribution of a "storytelling agent" vs. a "registry reviewer agent"? The existing M010 (Reputation) system could be extended, but it currently only covers credit quality, project legitimacy, verifier competence, and methodology rigor — not marketing effectiveness or documentation quality.

4. **Swarm coordination primitives**: The inter-agent coordination gap flagged in the feasibility review becomes even more critical with VM-06. If you have 15 different agent types in a swarm, you need:
   - Task boards / work queues (agentic project management)
   - Dependency tracking between initiatives
   - Resource contention resolution
   - Progress reporting and accountability

**This is where Opus 4.6 and similar models become transformative**: The reason this kind of swarm coordination is newly feasible is that models like Claude Opus 4.6 can handle the *judgment* required for task decomposition, quality evaluation, and coordination — not just the execution of predefined workflows. The existing framework's OODA loop + governance layers provide the *safety* structure; what Opus 4.6 adds is the *cognitive* capability to make that structure dynamic rather than static.

**Recommendation**: This is the most ambitious proposal and should be the *aspirational target* rather than an immediate implementation. Define it as Phase 5+ and build toward it by:
1. First proving the 4-agent topology works (Phases 3-4)
2. Then adding one non-infrastructure agent (e.g., a documentation agent) to test the expanded role model
3. Then designing the work registration and value attribution systems
4. Then opening up to swarm-scale coordination

---

### VM-07: Beyond Eco-Credits

**What's proposed**: Expand from eco-credits to ecological claims broadly, data management, knowledge commoning, and fresh go-to-market opportunities.

**Assessment**: This is strategically essential but not directly a tokenomics mechanism — it's a **business strategy** that determines whether the tokenomics *have enough volume to function*.

The current framework's mechanisms (M001-M011) are all anchored to eco-credit workflows. If the network expands to new domains, each new domain needs:
- Its own service catalog (like Phase 1.3 but for ecological claims, data management, etc.)
- Its own agent workflows (new OODA loops for new domains)
- Potentially new agent personas or extensions to existing ones
- New governance processes if the domain has unique requirements

**The framework's architecture actually supports this well** because:
- The 4-layer governance model is domain-agnostic
- The OODA workflow pattern applies to any domain
- The MCP integration layer abstracts data sources
- KOI can index knowledge from any domain

**What's needed**: A **Domain Expansion Protocol** — a meta-process for onboarding new value domains into the framework:
1. Stakeholder mapping for the new domain
2. Service catalog for the new domain
3. Automation feasibility assessment
4. Agent workflow design
5. Mechanism design (bonding, escrow, reputation for the new domain)
6. Governance process formalization

This is itself an agent-assistable process — an "expansion planning agent" could help evaluate new domains against the framework's templates.

**Recommendation**: Define as **GOV-008: Domain Expansion Proposal** — a Layer 3 governance process for proposing and evaluating new value domains for the network.

---

## 4. Incorporation Path

### Proposed New Mechanisms and Processes

| ID | Name | Source | Priority | Dependency |
|----|------|--------|----------|------------|
| **M012** | Dynamic Validator Set Sizing | VM-01 | Medium | M013 (needs revenue model first) |
| **M013** | Validator Sustainability Pool | VM-02 | **High** | None (foundational) |
| **M014** | Agent Provisioning Registry | VM-04 | **High** | M013 (funding source), M010 (reputation) |
| **GOV-006** | Validator Allowlist Governance | VM-03 | **High** | Existing GOV-001 pattern |
| **GOV-007** | Agent Operator Allowlist | VM-04 | Medium | M014 |
| **GOV-008** | Domain Expansion Proposal | VM-07 | Medium | Existing governance patterns |
| **WF-VM-04** | Validator Application Review | VM-03 | Medium | AGENT-004 implementation |

### Stakeholder Model Revision

The Stakeholder Pentad's Co-Creator class needs expansion:

```
Current Co-Creators:          Proposed Co-Creators:
├── Validators                ├── Validators (block production)
├── Verifiers                 ├── Compute Providers (agent infrastructure)
└── Methodology Developers    ├── Agent Operators (agent management)
                              ├── Verifiers
                              ├── Methodology Developers
                              └── GTM Contributors (BD, sales, storytelling)
```

### Phased Incorporation Timeline

**Immediate (integrate into existing Phase 2 specs)**:
- Revise Phase 1.1 Stakeholder Pentad to include Agent Operators and Compute Providers
- Draft GOV-006 (Validator Allowlist) — mirrors existing GOV-001 pattern
- Draft M013 (Validator Sustainability Pool) — economic model and fee-split specification

**Short-term (Phase 3 implementation alongside existing mechanisms)**:
- Implement GOV-006 alongside existing governance process implementations
- Economic modeling for M013 — answer the critical question: "Do current and projected eco-credit volumes generate enough fees to sustain minimum viable validator economics?"
- Draft M014 (Agent Provisioning Registry)
- Draft GOV-007 (Agent Operator Allowlist)

**Medium-term (Phase 4, post-testnet)**:
- Implement M012 (Dynamic Validator Set) as AGENT-004 enhancement
- Deploy M014 with first agent operator onboarding
- Draft GOV-008 (Domain Expansion) and run first expansion evaluation

**Long-term (Phase 5+)**:
- Implement VM-06 swarm coordination primitives
- Expand agent role types beyond infrastructure
- Implement value attribution for non-infrastructure contributions (BD, documentation, storytelling)
- Run first Domain Expansion proposals for ecological claims, data management, knowledge commoning

---

## 5. Deficiencies — Honest Register

These are the hard problems that neither the existing framework nor the voice memo fully resolves:

### D-01: Revenue Insufficiency (Critical)

The entire model depends on eco-credit marketplace fees generating enough revenue to fund validators, agent operators, and compute providers. The memo acknowledges the market is "soft." Current on-chain data (23+ sell orders, ~3.2M REGEN community pool) suggests **the flywheel doesn't have enough fuel yet**.

**What to do**: Build the economic model *before* building the mechanisms. Calculate: at current eco-credit volumes, what's the annual fee revenue? What's the annual cost of a minimal validator set + 4 agent operators? If there's a gap (likely), that gap defines how urgently VM-07 (beyond eco-credits) needs to be pursued.

### D-02: Specification-to-Code Gap (Critical, unchanged)

The voice memo adds more conceptual architecture to a framework that already has 15 specification documents and zero running code. Every new mechanism (M012-M014) and governance process (GOV-006 through GOV-008) widens the spec-to-code gap.

**What to do**: Impose a moratorium on new specifications until AGENT-002 (Governance Analyst) is running in production. Use that implementation as the forcing function to resolve all the infrastructure questions (ElizaOS version, MCP integration, database setup, CI/CD) that apply to every subsequent agent.

### D-03: Swarm Coordination Primitives (Major, amplified)

VM-06 significantly expands the coordination requirements. Going from 4 fixed agents to dynamic swarms with BD, sales, documentation, and storytelling roles requires coordination mechanisms that don't exist in the framework and are hard problems in distributed systems.

**What to do**: Treat this as a research objective, not an implementation target. Study how existing agent swarm frameworks (CrewAI, AutoGen, LangGraph) handle multi-agent coordination. Define Regen-specific requirements. Prototype with 2-3 agents before designing for N agents.

### D-04: Agent Quality Measurement (Major, amplified)

If agent operators are compensated co-creators (VM-04), you need to measure agent quality to prevent rent-seeking. The framework's M010 (Reputation) system covers eco-credit domain quality but not:
- Was the governance analysis accurate?
- Did the market monitoring catch real anomalies vs. generating false positives?
- Did the documentation agent produce useful content?
- Did the BD agent's GTM proposals generate actual revenue?

**What to do**: Extend M010 with outcome-linked reputation categories. Every agent role needs a measurable outcome metric that can be tracked and that reputation decays against.

### D-05: Bootstrapping Paradox (Major, unchanged)

The governance system that must approve these changes is the system being redesigned. Dynamic validator set sizing (VM-01) requires a parameter change proposal. Agent provisioning (VM-04) requires new on-chain modules. These require buy-in from the current 75 validators, who may not benefit from a shrinking validator set.

**What to do**: Frame the proposals in terms of validator benefit. The transition from "validator earning diminishing staking rewards" to "compute provider/agent operator earning fees from growing eco-credit volume" must be presented as an economic upgrade, not a demotion. This requires VM-02 (sustainable spin model) to be convincingly modeled before VM-01 (set sizing) is proposed.

### D-06: Constitutional Tension in Set Sizing

Shrinking the validator set is arguably a constitutional-level change (Layer 4, 67% supermajority). But VM-01 proposes it as responsive/automatic. There's a tension between "dynamic" and "constitutional" that needs resolving.

**What to do**: Set the floor and ceiling as Layer 4 constitutional parameters (can only be changed by supermajority). Set the *movement within that range* as a Layer 2-3 process (agent proposes based on metrics, with human oversight window).

---

## 6. Opportunities to Market and Improve

### O-01: "Agent Operator" as Unique Positioning

No other L1 blockchain has a formal, on-chain, economically-integrated role for AI agent operators. This is a differentiator. Market it as: "Regen Network — the first blockchain where AI agents are economic citizens, not just tools."

### O-02: The Opus 4.6 Moment

The voice memo's vision of swarms handling BD, documentation, storytelling, and infrastructure is *newly feasible* because of models like Opus 4.6 that can:
- Decompose ambiguous objectives into concrete tasks
- Evaluate quality of creative and strategic outputs (not just code)
- Coordinate across domains with shared context
- Operate with the judgment required for Layer 2 governance

The existing framework was designed for narrower agent capabilities. Reframing it around what Opus 4.6-class models actually enable — genuine judgment in soft domains like BD and storytelling — is a major marketing and functional opportunity.

### O-03: Knowledge Commoning as GTM

VM-07 mentions "knowledge commoning." This is an underexplored GTM vector. KOI already indexes 64K documents. An agent-operated knowledge commons — where ecological data, methodologies, and research are curated, attested (M008), and reputation-scored (M010) — is a product that:
- Has broader market appeal than carbon credits alone
- Demonstrates the agent swarm framework's capabilities
- Creates the data substrate that makes eco-credits more valuable
- Can operate largely at Layer 1 (low risk, high automation)

### O-04: The Framework Itself as Product

The 4-layer governance model, OODA workflow patterns, and service decomposition methodology are applicable far beyond Regen Network. Other blockchain ecosystems face the same challenge of integrating AI agents with on-chain governance. Packaging and open-sourcing the framework patterns (separate from Regen-specific implementations) creates:
- Developer ecosystem growth
- Credibility as thought leaders in agentic blockchain governance
- Potential for the framework to be adopted by other Cosmos chains

---

## 7. Outcomes We Aim to Achieve — Clarity for Agentic Development

For agent swarms (powered by Opus 4.6-class models) to be "unleashed optimally," they need unambiguous outcome definitions. Here's a proposed outcome registry:

### Protocol-Level Outcomes

| ID | Outcome | Metric | Target | Measurable By |
|----|---------|--------|--------|---------------|
| OUT-01 | Network self-sustainability | Protocol revenue ≥ protocol costs (validators + agents + infra) | Revenue/cost ratio ≥ 1.0 | On-chain fee tracking |
| OUT-02 | Right-sized decentralization | Nakamoto coefficient within target range | Nak. coeff. 5-15 (adjustable) | AGENT-004 WF-VM-03 |
| OUT-03 | Governance participation | Active governance participation rate | >40% of staked tokens voting | AGENT-002 WF-GA-02 |
| OUT-04 | Ecological value registered | Total eco-credit volume on-chain | 2x current volume within 12 months | Ledger MCP queries |

### Agent Swarm Outcomes

| ID | Outcome | Metric | Target | Measurable By |
|----|---------|--------|--------|---------------|
| OUT-05 | Agent decision quality | Human override rate of Layer 2 decisions | <15% override rate | Audit logs |
| OUT-06 | Agent availability | Combined agent uptime | >99.5% | Infrastructure monitoring |
| OUT-07 | Escalation efficiency | Avg. time from escalation to human resolution | <24h for standard, <1h for critical | Workflow execution logs |
| OUT-08 | Knowledge growth | KOI indexed documents, new domains | +20% annually | KOI MCP metrics |

### Ecosystem Growth Outcomes

| ID | Outcome | Metric | Target | Measurable By |
|----|---------|--------|--------|---------------|
| OUT-09 | Domain expansion | New value domains onboarded | ≥2 new domains in 18 months | GOV-008 proposals |
| OUT-10 | Agent operator onboarding | Active permissioned agent operators | ≥5 operators in 12 months | M014 registry |
| OUT-11 | GTM pipeline | New revenue opportunities registered and pursued | ≥3 active GTM initiatives | VM-06 work registry |
| OUT-12 | Stakeholder harmonization | All 5 pentad stakeholder classes actively participating | Measurable activity in each class | Cross-mechanism analytics |

### The "North Star" Outcome

| ID | Outcome | Metric | Why It Matters |
|----|---------|--------|----------------|
| OUT-00 | Ecological regeneration valued by the economy | Tons of verified ecological regeneration × price per ton, trending upward | This is the ultimate measure. Everything else — validators, agents, governance, tokenomics — is scaffolding for this. If this number isn't growing, the system isn't working regardless of how elegant the architecture is. |

---

## 8. Recommended Next Actions (Ordered)

1. **Economic model first**: Build a spreadsheet/simulation of VM-02 (sustainable spin model) with current on-chain data. Answer: "Can the flywheel sustain itself at current volumes? At what volume does it become self-sustaining?"

2. **AGENT-002 prototype**: Ship the Governance Analyst as running code. This is the cheapest way to prove the framework works and builds community trust for the governance proposals that follow.

3. **Stakeholder model revision**: Update Phase 1.1 to incorporate Agent Operators and Compute Providers as co-creator classes.

4. **GOV-006 specification**: Draft Validator Allowlist Governance, mirroring GOV-001. This is the easiest new governance process to specify.

5. **M013 specification**: Draft Validator Sustainability Pool mechanism with fee-split economics.

6. **Outcome registry adoption**: Adopt the outcome definitions from Section 7 above as the framework's success criteria. Every specification and implementation should trace back to one or more outcomes.

7. **Moratorium on further mechanism design** until items 1-2 are complete. The framework has enough specs. It needs code and validated economics.

---

*Analysis conducted 2026-02-12. Cross-referenced against Phases 1-3 specifications and FEASIBILITY-REVIEW.md.*
