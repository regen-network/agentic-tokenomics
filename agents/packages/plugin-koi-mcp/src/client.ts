/**
 * Regen KOI MCP Client
 *
 * Wraps HTTP calls to the KOI MCP server, which provides access to
 * Regen's knowledge graph (Apache Jena + Qdrant) via semantic search,
 * entity resolution, SPARQL queries, and code graph navigation.
 *
 * Based on phase-3/3.2-agent-implementation.md §KOIMCPClient.
 */

import type { MCPClientConfig } from "@regen/core";

export interface KOIMCPConfig extends MCPClientConfig {}

export type SearchIntent =
  | "general"
  | "person_activity"
  | "technical_howto";

export class KOIMCPClient {
  private baseUrl: string;
  private apiKey: string;
  private timeoutMs: number;

  constructor(config: KOIMCPConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.apiKey = config.apiKey;
    this.timeoutMs = config.timeoutMs ?? 30_000;
  }

  async search(params: {
    query: string;
    intent?: SearchIntent;
    limit?: number;
    source?: string;
  }) {
    return this.call("search", params);
  }

  async resolveEntity(label: string, typeHint?: string) {
    return this.call("resolve_entity", { label, type_hint: typeHint });
  }

  async queryCodeGraph(params: {
    query_type: string;
    entity_name?: string;
    repo_name?: string;
  }) {
    return this.call("query_code_graph", params);
  }

  async sparqlQuery(query: string, limit?: number) {
    return this.call("sparql_query", { query, limit });
  }

  async getEntityDocuments(uri: string, limit?: number) {
    return this.call("get_entity_documents", { uri, limit });
  }

  async call(
    tool: string,
    params: Record<string, unknown>
  ): Promise<unknown> {
    const controller = new AbortController();
    const timeout = setTimeout(
      () => controller.abort(),
      this.timeoutMs
    );

    try {
      const response = await fetch(
        `${this.baseUrl}/mcp/tools/${tool}`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            ...(this.apiKey
              ? { Authorization: `Bearer ${this.apiKey}` }
              : {}),
          },
          body: JSON.stringify(params),
          signal: controller.signal,
        }
      );

      if (!response.ok) {
        throw new Error(
          `KOI MCP call ${tool} failed: ${response.status} ${response.statusText}`
        );
      }

      return response.json();
    } finally {
      clearTimeout(timeout);
    }
  }
}
