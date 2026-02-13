import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  analyzeProposal,
  formatProposalAnalysis,
  extractProposalId,
} from "@regen/plugin-ledger-mcp";
import type { LedgerMCPClient } from "@regen/plugin-ledger-mcp";

// Mock proposal data matching real Regen Ledger response shape
const MOCK_PROPOSAL = {
  id: "62",
  status: "PROPOSAL_STATUS_VOTING_PERIOD",
  submit_time: "2026-01-15T12:00:00Z",
  voting_end_time: "2026-01-29T12:00:00Z",
  content: {
    "@type": "/cosmos.params.v1beta1.ParameterChangeProposal",
    title: "Enable IBC Transfer Memo Field",
    description: "Enables memo field for IBC transfers, required for cross-chain integrations.",
  },
  final_tally_result: {
    yes: "45200000000000",
    no: "2100000000000",
    abstain: "10400000000000",
    no_with_veto: "0",
  },
};

function mockClient(): LedgerMCPClient {
  return {
    getProposal: vi.fn().mockResolvedValue(MOCK_PROPOSAL),
    listProposals: vi.fn(),
    listVotes: vi.fn(),
    listClasses: vi.fn(),
    listProjects: vi.fn(),
    listBatches: vi.fn(),
    listSellOrders: vi.fn(),
    listValidators: vi.fn(),
    getValidatorRewards: vi.fn(),
    getTotalSupply: vi.fn(),
    call: vi.fn(),
  } as unknown as LedgerMCPClient;
}

describe("extractProposalId", () => {
  it("extracts from 'proposal 62'", () => {
    expect(extractProposalId("analyze proposal 62")).toBe(62);
  });

  it("extracts from 'proposal #62'", () => {
    expect(extractProposalId("summarize proposal #62")).toBe(62);
  });

  it("extracts from 'Proposal 3'", () => {
    expect(extractProposalId("What is Proposal 3 about?")).toBe(3);
  });

  it("returns null when no proposal ID found", () => {
    expect(extractProposalId("what's new?")).toBeNull();
  });
});

describe("analyzeProposal", () => {
  let client: LedgerMCPClient;

  beforeEach(() => {
    client = mockClient();
  });

  it("returns structured analysis for a valid proposal", async () => {
    const analysis = await analyzeProposal(client, 62);

    expect(analysis).not.toBeNull();
    expect(analysis!.proposalId).toBe(62);
    expect(analysis!.title).toBe("Enable IBC Transfer Memo Field");
    expect(analysis!.status).toBe("PROPOSAL_STATUS_VOTING_PERIOD");
    expect(analysis!.tally.yesPercent).toBe("78.3");
    expect(analysis!.tally.vetoPercent).toBe("0.0");
    expect(analysis!.impact.technical).toBe("Low - Configuration change");
    expect(analysis!.impact.economic).toBe("Low");
  });

  it("returns null for non-existent proposal", async () => {
    (client.getProposal as any).mockResolvedValue(null);
    const analysis = await analyzeProposal(client, 999);
    expect(analysis).toBeNull();
  });

  it("handles software upgrade proposals", async () => {
    (client.getProposal as any).mockResolvedValue({
      ...MOCK_PROPOSAL,
      content: {
        "@type": "/cosmos.upgrade.v1beta1.SoftwareUpgradeProposal",
        title: "Regen Ledger v6.0",
        description: "Upgrade to v6.0 with CosmWasm support.",
      },
    });

    const analysis = await analyzeProposal(client, 63);
    expect(analysis!.impact.technical).toBe("High - Chain upgrade required");
  });

  it("handles community pool spend proposals", async () => {
    (client.getProposal as any).mockResolvedValue({
      ...MOCK_PROPOSAL,
      content: {
        "@type": "/cosmos.distribution.v1beta1.CommunityPoolSpendProposal",
        title: "Fund Ecosystem Development",
        description: "Spend 100k REGEN on ecosystem development.",
      },
    });

    const analysis = await analyzeProposal(client, 64);
    expect(analysis!.impact.economic).toBe("Direct - Treasury disbursement");
  });

  it("handles zero-vote proposals", async () => {
    (client.getProposal as any).mockResolvedValue({
      ...MOCK_PROPOSAL,
      final_tally_result: {
        yes: "0",
        no: "0",
        abstain: "0",
        no_with_veto: "0",
      },
    });

    const analysis = await analyzeProposal(client, 65);
    expect(analysis!.tally.yesPercent).toBe("0.0");
    expect(analysis!.tally.totalVoted).toBe("0.0M");
  });
});

describe("formatProposalAnalysis", () => {
  it("produces markdown with all sections", async () => {
    const client = mockClient();
    const analysis = await analyzeProposal(client, 62);
    const formatted = formatProposalAnalysis(analysis!);

    expect(formatted).toContain("## Proposal #62 Analysis");
    expect(formatted).toContain("**Title**: Enable IBC Transfer Memo Field");
    expect(formatted).toContain("### Current Voting");
    expect(formatted).toContain("| Yes |");
    expect(formatted).toContain("### Impact Assessment");
    expect(formatted).toContain("**Technical**:");
  });
});
