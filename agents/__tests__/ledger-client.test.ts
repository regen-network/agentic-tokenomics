import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { LedgerMCPClient } from "@regen/plugin-ledger-mcp";

describe("LedgerMCPClient", () => {
  let client: LedgerMCPClient;
  let fetchSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    client = new LedgerMCPClient({
      baseUrl: "http://localhost:3001",
      apiKey: "test-key",
    });

    fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ proposals: [] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      })
    );
  });

  afterEach(() => {
    fetchSpy.mockRestore();
  });

  it("calls the correct MCP endpoint for listProposals", async () => {
    await client.listProposals({ limit: 5 });

    expect(fetchSpy).toHaveBeenCalledWith(
      "http://localhost:3001/mcp/tools/list_governance_proposals",
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ limit: 5 }),
      })
    );
  });

  it("includes authorization header when API key is set", async () => {
    await client.listProposals({});

    const callArgs = fetchSpy.mock.calls[0][1] as RequestInit;
    expect((callArgs.headers as Record<string, string>)["Authorization"]).toBe(
      "Bearer test-key"
    );
  });

  it("omits authorization header when API key is empty", async () => {
    const noAuthClient = new LedgerMCPClient({
      baseUrl: "http://localhost:3001",
      apiKey: "",
    });
    await noAuthClient.listProposals({});

    const callArgs = fetchSpy.mock.calls[0][1] as RequestInit;
    expect(
      (callArgs.headers as Record<string, string>)["Authorization"]
    ).toBeUndefined();
  });

  it("throws on non-200 response", async () => {
    fetchSpy.mockResolvedValueOnce(
      new Response("Not Found", { status: 404, statusText: "Not Found" })
    );

    await expect(client.getProposal(999)).rejects.toThrow(
      "Ledger MCP call get_governance_proposal failed: 404 Not Found"
    );
  });

  it("strips trailing slash from baseUrl", async () => {
    const slashClient = new LedgerMCPClient({
      baseUrl: "http://localhost:3001/",
      apiKey: "",
    });
    await slashClient.listClasses({});

    expect(fetchSpy).toHaveBeenCalledWith(
      "http://localhost:3001/mcp/tools/list_classes",
      expect.anything()
    );
  });

  it("calls getProposal with correct params", async () => {
    await client.getProposal(62);

    expect(fetchSpy).toHaveBeenCalledWith(
      "http://localhost:3001/mcp/tools/get_governance_proposal",
      expect.objectContaining({
        body: JSON.stringify({ proposal_id: 62 }),
      })
    );
  });
});
