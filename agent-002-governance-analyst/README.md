# AGENT-002: Regen Governance Analyst

**The first running agent in the Regen Agentic Tokenomics framework.**

A Layer 1 (fully automated, read-only, informational) agent that monitors Regen Network governance, analyzes proposals, tracks voting, and generates post-vote reports.

## What it does

| Workflow | Trigger | Output |
|----------|---------|--------|
| **WF-GA-01** Proposal Analysis | New proposal detected | Comprehensive analysis: TL;DR, impact assessment, risk factors, historical context |
| **WF-GA-02** Voting Monitor | Periodic polling during voting period | Voting status, turnout tracking, outcome projection, quorum alerts |
| **WF-GA-03** Post-Vote Report | Proposal finalized | Final analysis, turnout report, implications |

Each workflow follows the **OODA loop** (Observe → Orient → Decide → Act) as specified in the framework.

## Architecture

```
Regen Ledger (LCD REST API)
    ↓ observe
AGENT-002 (OODA engine)
    ↓ orient + decide (Claude)
Local SQLite (state)
    ↓ act
Console / Discord webhook
```

**No MCP dependency.** Talks directly to any Cosmos LCD endpoint. When Ledger MCP and KOI MCP become available, the `LedgerClient` can be swapped behind the same interface.

**No ElizaOS dependency.** Standalone Node.js process. Can be wrapped as an ElizaOS plugin later without changing the core logic.

## Quick start

```bash
# 1. Install
cd agent-002-governance-analyst
npm install

# 2. Configure
cp .env.example .env
# Edit .env — at minimum set ANTHROPIC_API_KEY

# 3. Run (single analysis pass)
npm run analyze

# 4. Run (continuous polling)
npm start

# 5. Run (dev mode with auto-reload)
npm run dev
```

## Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `ANTHROPIC_API_KEY` | Yes | — | Claude API key |
| `REGEN_LCD_URL` | No | `https://regen.api.chandrastation.com` | Cosmos LCD endpoint |
| `DISCORD_WEBHOOK_URL` | No | — | Discord webhook for posting analyses |
| `POLL_INTERVAL_SECONDS` | No | `300` | Polling interval (seconds) |
| `ANTHROPIC_MODEL` | No | `claude-sonnet-4-5-20250929` | Claude model to use |

## How it maps to the framework specs

| Framework Spec | Implementation |
|----------------|---------------|
| Phase 2.2 WF-GA-01 | `src/workflows/proposal-analysis.ts` |
| Phase 2.2 WF-GA-02 | `src/workflows/voting-monitor.ts` |
| Phase 2.2 WF-GA-03 | `src/workflows/post-vote-report.ts` |
| Phase 2.4 OODA executor | `src/ooda.ts` |
| Phase 2.4 Agent character | System prompt in `src/analyst.ts` |
| Phase 2.5 Workflow executions table | `src/store.ts` (SQLite) |
| Phase 3.2 Ledger MCP client | `src/ledger.ts` (direct LCD) |

## Design decisions

1. **Direct LCD over MCP**: MCP servers don't exist yet. Direct LCD calls work now and can be swapped later.

2. **SQLite over PostgreSQL**: Lowers the barrier to running the agent locally. The store interface is simple enough to swap to pg.

3. **Standalone over ElizaOS**: ElizaOS plugin API may change. A standalone process proves the workflow logic works independently of any runtime framework.

4. **Claude for Orient+Decide only**: The Observe and Act phases are deterministic. Only the analytical phases (Orient, Decide) use LLM calls, keeping costs bounded and behavior predictable.

## Governance layer

This agent operates at **Layer 1 only**:

- Read-only access to on-chain state
- Cannot submit proposals
- Cannot vote
- Cannot execute transactions
- Informational output only

This matches the framework's principle of starting with the lowest-risk, highest-value capability.
