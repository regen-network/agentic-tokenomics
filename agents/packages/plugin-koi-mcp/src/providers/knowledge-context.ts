/**
 * Knowledge Context Provider
 *
 * Searches the KOI knowledge graph for context relevant to the current
 * message and injects it into the agent's context window. Used by all
 * agents to ground their analysis in Regen's institutional knowledge.
 *
 * Based on phase-3/3.2-agent-implementation.md §knowledgeContextProvider.
 */

import type { KOIMCPClient, SearchIntent } from "../client.js";

export interface KnowledgeResult {
  title: string;
  snippet: string;
  source: string;
}

export async function getKnowledgeContext(
  client: KOIMCPClient,
  query: string,
  intent: SearchIntent = "general",
  limit: number = 5
): Promise<KnowledgeResult[]> {
  const raw = (await client.search({ query, intent, limit })) as any;
  const results = raw?.results ?? [];

  return results.map((r: any) => ({
    title: r.title ?? "Untitled",
    snippet:
      r.snippet ?? r.content?.substring(0, 300) ?? "No content available",
    source: r.source ?? "Unknown",
  }));
}

export function formatKnowledgeContext(results: KnowledgeResult[]): string {
  if (results.length === 0) {
    return "No relevant knowledge context found.";
  }

  const sections = results.map(
    (r) => `### ${r.title}\n${r.snippet}...\nSource: ${r.source}`
  );

  return `## Relevant Knowledge Context\n${sections.join("\n\n")}`;
}
