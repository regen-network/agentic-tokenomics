/**
 * @regen/plugin-koi-mcp
 *
 * ElizaOS plugin providing Regen KOI knowledge graph access via MCP.
 * Enables semantic search, entity resolution, SPARQL queries, and
 * code graph navigation.
 *
 * Based on phase-3/3.2-agent-implementation.md §@regen/plugin-koi-mcp.
 */

export { KOIMCPClient, type KOIMCPConfig, type SearchIntent } from "./client.js";
export {
  getKnowledgeContext,
  formatKnowledgeContext,
  type KnowledgeResult,
} from "./providers/knowledge-context.js";
