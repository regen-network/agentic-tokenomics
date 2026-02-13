import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { KOIMCPClient } from "@regen/plugin-koi-mcp";

describe("KOIMCPClient", () => {
  let client: KOIMCPClient;
  let fetchSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    client = new KOIMCPClient({
      baseUrl: "http://localhost:3002",
      apiKey: "test-koi-key",
    });

    fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ results: [] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      })
    );
  });

  afterEach(() => {
    fetchSpy.mockRestore();
  });

  it("calls search endpoint with correct params", async () => {
    await client.search({
      query: "regen governance proposal",
      intent: "general",
      limit: 5,
    });

    expect(fetchSpy).toHaveBeenCalledWith(
      "http://localhost:3002/mcp/tools/search",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({
          query: "regen governance proposal",
          intent: "general",
          limit: 5,
        }),
      })
    );
  });

  it("calls resolveEntity with label and type hint", async () => {
    await client.resolveEntity("Regen Network", "organization");

    expect(fetchSpy).toHaveBeenCalledWith(
      "http://localhost:3002/mcp/tools/resolve_entity",
      expect.objectContaining({
        body: JSON.stringify({
          label: "Regen Network",
          type_hint: "organization",
        }),
      })
    );
  });

  it("calls sparqlQuery with query and limit", async () => {
    const sparql = "SELECT ?s WHERE { ?s a <http://example.org/CreditClass> } LIMIT 10";
    await client.sparqlQuery(sparql, 10);

    expect(fetchSpy).toHaveBeenCalledWith(
      "http://localhost:3002/mcp/tools/sparql_query",
      expect.objectContaining({
        body: JSON.stringify({ query: sparql, limit: 10 }),
      })
    );
  });

  it("includes authorization header", async () => {
    await client.search({ query: "test" });

    const callArgs = fetchSpy.mock.calls[0][1] as RequestInit;
    expect((callArgs.headers as Record<string, string>)["Authorization"]).toBe(
      "Bearer test-koi-key"
    );
  });

  it("throws on server error", async () => {
    fetchSpy.mockResolvedValueOnce(
      new Response("Internal Server Error", {
        status: 500,
        statusText: "Internal Server Error",
      })
    );

    await expect(
      client.search({ query: "test" })
    ).rejects.toThrow("KOI MCP call search failed: 500 Internal Server Error");
  });
});
