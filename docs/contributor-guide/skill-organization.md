# Skill Organization for Contributors

This document explains how skills, contexts, and agents are organized within the Regen ecosystem, enabling progressive capability disclosure and continuous learning.

## Directory Structure

```
contexts/     → Domain knowledge files
skills/       → Reusable procedures
agents/       → Multi-skill orchestrators
playbooks/    → Sequential workflows
```

## Skill Definition Pattern

Each skill is defined in a single `SKILL.md` file:

```markdown
# Skill: <name>
description: <when to use this skill>
version: <semver>
license: <license>

## Overview
<purpose and scope>

## Quick Start
<minimal viable usage>

## Full Specification
<complete template>

## Examples
<input/output pairs>

## Failure Modes
<what can go wrong>
```

## Context Hierarchical Merging

Contexts merge based on user tier:

1. **Public base** always loads
2. **Partner extensions** add conditionally
3. **Core additions** stack on top
4. **Personal additions** append last

This enables individual customization without forking entire configurations.

## Progressive Learning Integration

Skills connect to the continuous learning system via:

1. **Digest Integration**: Skills can query regen-heartbeat for recent context
2. **KOI Linkage**: Skills reference KOI knowledge graph for grounded information
3. **Feedback Loops**: Skill usage feeds back into improvement cycles

## Available Skill Categories

| Category | Examples | Purpose |
|----------|----------|---------|
| Ledger Query | `ledger-query` | On-chain data patterns |
| Analysis | `credit-analysis` | Portfolio and project analysis |
| Generation | `regen-brand-generation` | Visual and content generation |
| Validation | `code-review` | Review and validation |
| Research | `weekly-digest` | Synthesis and summarization |
| Meta | `metaprompt` | Prompt engineering and design |

## Creating New Skills

1. Identify a recurring task pattern
2. Draft the `SKILL.md` following the template
3. Test with multiple scenarios
4. Submit PR for review
5. After approval, skill becomes available

---

*See [http-config-architecture-v2.md](https://github.com/DarrenZal/koi-research/blob/regen-prod/docs/http-config-architecture-v2.md) for full architecture details.*
