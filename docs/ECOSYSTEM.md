# Ecosystem Directory

This repository (`regen-network/agentic-tokenomics`) is the **coordination layer** for Regen Network's agentic tokenomics and governance system — specifications, schemas, and mechanism designs for a 65-75% automated governance framework.

Below are projects building on or alongside this framework. If you are working on something related, [add your project](#list-your-project).

## Coordination Layer

- **[regen-network/agentic-tokenomics](https://github.com/regen-network/agentic-tokenomics)** — This repo. Governance mechanism specs, token utility designs, agent persona definitions, and contributor coordination.

## Core Stack

The core development stack spans this repo plus three additional repositories with defined data flows and ownership boundaries. See [`architecture/STACK_MAP.md`](architecture/STACK_MAP.md) for the full diagram and routing guide.

- **[regen-network/koi-research](https://github.com/regen-network/koi-research)** — Research artifacts, datasets, extractors, ontologies, and tooling.
- **[gaiaaiagent/regen-ai-claude](https://github.com/gaiaaiagent/regen-ai-claude)** — Agent skills, Claude configurations, and MCP adapter interfaces.
- **[gaiaaiagent/regen-heartbeat](https://github.com/gaiaaiagent/regen-heartbeat)** — Signal emission, digests, and validators. Continuous metabolization of Regen Network, RegenAI, and Regen Commons. Contains `.claude/characters` and `.claude/output-styles` configuration.

## Community Projects

- **[CShear/regen-compute-credits](https://github.com/CShear/regen-compute-credits)** — An MCP agent that funds verified ecological regeneration from AI compute usage via Regen Network. Associated with [bridge.eco](https://bridge.eco).
  - Fork: [brawlaphant/regenerative-compute](https://github.com/brawlaphant/regenerative-compute) (unmerged)
- **[Eco-Wealth/netnet](https://github.com/Eco-Wealth/netnet)** — An operator agent for paid execution and ecological proof. Ships conservative primitives first; autonomy is intentionally gated. v0.3.0 testing release includes Bridge.eco retirement orchestration, ecoToken verification links, and an agent-callable Carbon API.

## Organizations

- **[agent-ecowealth](https://github.com/agent-ecowealth)** — GitHub organization.

## List Your Project

Open a PR adding your entry to the **Community Projects** section above. Include:

1. Repo link (GitHub `org/repo` format)
2. One-line description of what the project does
3. Any relevant version or status info

Follow the [PR format](CONTRIBUTOR_NAV.md) used in this repo:

- **Lands in:** `docs/`
- **Changes:** Add `<project-name>` to ecosystem directory
- **Validate:** docs-only
