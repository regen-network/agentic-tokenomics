# Contributing to Regen Agentic Tokenomics

Welcome to the Regen Network Agentic Tokenomics & Governance System. This repository documents a framework for 65-75% automated governance, integrating AI agents with on-chain infrastructure to accelerate ecological regeneration.

## Table of Contents

1. [Vision](#vision)
2. [Contribution Types](#contribution-types)
3. [For Human Contributors](#for-human-contributors)
4. [For Agentic Contributors](#for-agentic-contributors)
5. [Repository Structure](#repository-structure)
6. [Progressive Access Tiers](#progressive-access-tiers)
7. [Getting Started](#getting-started)

---

## Vision

> "Accelerate ecological regeneration by linking economic and AI incentives to human stewardship."

This repository is part of a larger ecosystem transformation. Contributors—whether human or AI—participate in evolving:
- **Governance mechanisms** that balance automation with human oversight
- **Token economics** that align incentives with ecological outcomes
- **Agent workflows** that augment human decision-making
- **Semantic infrastructure** that enables verifiable ecological claims

---

## Contribution Types

| Type | Description | Examples |
|------|-------------|----------|
| **Specification** | Formal mechanism designs, protocol specs | Token utility mechanisms, governance processes |
| **Implementation** | Code, smart contracts, agent plugins | CosmWasm contracts, ElizaOS plugins |
| **Research** | Analysis, modeling, simulation | Tokenomics modeling, agent behavior analysis |
| **Documentation** | Guides, explanations, diagrams | Architecture docs, workflow diagrams |
| **Review** | Feedback, validation, testing | Security audits, specification reviews |

---

## For Human Contributors

### Quick Start

1. **Explore the Phases**: Start with `README.md` to understand the 5-phase structure
2. **Find Your Entry Point**:
   - Phase 1-2 for researchers and designers
   - Phase 3 for implementers
   - Phase 4-5 for operators and community builders
3. **Join the Conversation**: Engage on [forum.regen.network](https://forum.regen.network) or Discord

### Contribution Workflow

```
1. Fork the repository
2. Create a feature branch: feat/<description> or fix/<description>
3. Make changes following the style guide
4. Submit PR with clear description linking to relevant issues/discussions
5. Engage in review process
6. Merge after approval
```

### Style Guide

- Use clear, accessible language (see [Living Language Tone Guide](./docs/contributor-guide/living-language.md))
- Include diagrams for complex flows (Mermaid preferred)
- Cross-reference related specifications
- Maintain backlinking to prior context

### Where Humans Are Essential

Per the 4-layer governance model:
- **Layer 4 (Constitutional)**: All constitutional changes require human deliberation
- **Layer 3 (Human-in-Loop)**: Community pool spends, significant parameter changes
- **High-stakes decisions**: Security-critical changes, irreversible actions

---

## For Agentic Contributors

### Agent Types & Boundaries

This repository recognizes several agent personas with defined scopes:

| Agent | Scope | Governance Layer |
|-------|-------|------------------|
| Registry Reviewer (AGENT-001) | Credit class analysis, methodology review | Layer 2-3 |
| Governance Analyst (AGENT-002) | Proposal analysis, voting patterns | Layer 2-3 |
| Market Monitor (AGENT-003) | Price analysis, liquidity assessment | Layer 1-2 |
| Validator Monitor (AGENT-004) | Network health, validator performance | Layer 1-2 |

### Agentic Contribution Protocol

1. **Identification**: All agent contributions must be clearly labeled
2. **Provenance**: Include the agent ID and version in commit metadata
3. **Attestation**: Link to supporting evidence in KOI knowledge graph
4. **Human Checkpoint**: Flag items requiring human review

### Agent Commit Format

```
[AGENT-XXX] <type>: <description>

Agent: <agent-id>@<version>
Evidence: <koi-rid>
Human-Review-Required: yes|no
Confidence: <0.0-1.0>

<detailed description>

Co-Authored-By: <agent-name> <agent-email>
```

### Tool Access

Agents interact via MCP (Model Context Protocol):
- **KOI MCP**: Knowledge graph queries, document retrieval
- **Ledger MCP**: On-chain state queries, governance data
- **TX Builder MCP**: Transaction preparation (human-signed)

---

## Repository Structure

```
agentic-tokenomics/
├── README.md                    # Entry point and navigation
├── CONTRIBUTING.md              # This file
├── phase-1/                     # Discovery & Analysis
│   ├── 1.1-stakeholder-value-flow.md
│   ├── 1.2-tokenomic-mechanisms.md
│   ├── 1.3-agentic-services.md
│   ├── 1.4-governance-architecture.md
│   └── 1.5-system-architecture.md
├── phase-2/                     # Mechanism Design & Specification
│   ├── 2.1-token-utility-mechanisms.md
│   ├── 2.2-agentic-workflows.md
│   ├── 2.3-governance-processes.md
│   ├── 2.4-agent-orchestration.md
│   └── 2.5-data-schema-integration.md
├── phase-3/                     # Implementation & Testing
│   ├── 3.1-smart-contract-specs.md
│   ├── 3.2-agent-implementation.md
│   ├── 3.3-testing-plan.md
│   ├── 3.4-security-framework.md
│   └── 3.5-technical-docs.md
├── phase-4/                     # Deployment & Migration (planned)
├── phase-5/                     # Operations & Evolution (planned)
├── docs/
│   ├── contributor-guide/       # Contribution documentation
│   ├── integration/             # External system integrations
│   └── governance/              # Governance context and processes
└── schemas/                     # Data schemas and ontology references
```

---

## Progressive Access Tiers

Following the [HTTP Config Architecture v2](https://github.com/DarrenZal/koi-research/blob/regen-prod/docs/http-config-architecture-v2.md):

### Public Tier (Default)
- Access: All contributors
- Content: Core specifications, public documentation
- Capabilities: Read, propose changes, participate in discussions

### Partner Tier (Phase 2)
- Access: Verified ReFi ecosystem partners
- Content: Extended integrations, partner-specific workflows
- Capabilities: Direct collaboration, early access to specifications

### Core Tier
- Access: Internal Regen team, trusted validators
- Content: Security-sensitive specifications, operational procedures
- Capabilities: Direct merge rights, system administration

### Tier Progression

Contributors can progress through tiers via:
1. Sustained, quality contributions
2. Domain expertise demonstration
3. Community trust building
4. Formal partnership agreements (for organizations)

---

## Getting Started

### Prerequisites

- Familiarity with Regen Network ecosystem
- GitHub account
- (For agents) MCP tool access configured

### First Contribution Ideas

1. **Documentation improvements**: Clarify existing specifications
2. **Diagram creation**: Visualize complex workflows
3. **Review participation**: Provide feedback on open PRs
4. **Issue triage**: Help organize and label open issues

### Resources

- [Regen Registry Handbook](https://handbook.regen.network)
- [Technical Guides](https://guides.regen.network)
- [Forum Discussions](https://forum.regen.network)
- [KOI Knowledge Base](https://regen.gaiaai.xyz)
- [Regen Heartbeat Digests](https://gaiaaiagent.github.io/regen-heartbeat/digests/)

---

## Questions?

- Open a GitHub Discussion
- Ask in Discord #agentic-governance channel
- Post on [forum.regen.network](https://forum.regen.network)

---

*This document is part of the Regen Network Agentic Tokenomics framework.*
*Last updated: February 2026*
