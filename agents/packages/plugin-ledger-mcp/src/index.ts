/**
 * @regen/plugin-ledger-mcp
 *
 * ElizaOS plugin providing Regen Ledger on-chain data access via MCP.
 * Registers actions, providers, and evaluators for interacting with
 * the Regen Ledger blockchain.
 *
 * Based on phase-3/3.2-agent-implementation.md §@regen/plugin-ledger-mcp.
 */

export { LedgerMCPClient, type LedgerMCPConfig } from "./client.js";
export {
  analyzeProposal,
  formatProposalAnalysis,
  extractProposalId,
  type ProposalAnalysis,
} from "./actions/analyze-proposal.js";
export {
  getLedgerState,
  formatLedgerState,
  type LedgerStateSnapshot,
} from "./providers/ledger-state.js";
