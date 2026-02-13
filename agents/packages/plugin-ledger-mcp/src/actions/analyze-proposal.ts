/**
 * Analyze Proposal Action
 *
 * Core action for AGENT-002 (Governance Analyst). Fetches a governance
 * proposal from chain, retrieves voting data, and builds a structured
 * analysis with impact assessment.
 *
 * Based on phase-3/3.2-agent-implementation.md §analyzeProposalAction
 * and workflow WF-GA-01 from phase-2/2.2-agentic-workflows.md.
 */

import type { LedgerMCPClient } from "../client.js";

export interface ProposalAnalysis {
  proposalId: number;
  title: string;
  type: string;
  status: string;
  submitTime: string;
  votingEndTime: string;
  description: string;
  tally: {
    yes: string;
    no: string;
    abstain: string;
    noWithVeto: string;
    totalVoted: string;
    yesPercent: string;
    noPercent: string;
    abstainPercent: string;
    vetoPercent: string;
  };
  impact: {
    technical: string;
    economic: string;
    governance: string;
  };
}

export function extractProposalId(text: string): number | null {
  const match = text.match(/proposal\s*#?(\d+)/i);
  return match ? parseInt(match[1], 10) : null;
}

export async function analyzeProposal(
  client: LedgerMCPClient,
  proposalId: number
): Promise<ProposalAnalysis | null> {
  const proposal = (await client.getProposal(proposalId)) as any;
  if (!proposal) return null;

  const content = proposal.content ?? {};
  const tallyResult = proposal.final_tally_result ?? {};

  const yes = BigInt(tallyResult.yes ?? "0");
  const no = BigInt(tallyResult.no ?? "0");
  const abstain = BigInt(tallyResult.abstain ?? "0");
  const noWithVeto = BigInt(tallyResult.no_with_veto ?? "0");
  const totalVoted = yes + no + abstain + noWithVeto;

  const pct = (val: bigint) =>
    totalVoted > 0n
      ? ((Number(val) / Number(totalVoted)) * 100).toFixed(1)
      : "0.0";

  const fmt = (val: bigint) => `${(Number(val) / 1_000_000).toFixed(1)}M`;

  return {
    proposalId,
    title: content.title ?? "Untitled",
    type: content["@type"] ?? "Unknown",
    status: proposal.status ?? "UNKNOWN",
    submitTime: proposal.submit_time ?? "N/A",
    votingEndTime: proposal.voting_end_time ?? "N/A",
    description: content.description ?? "No description available.",
    tally: {
      yes: fmt(yes),
      no: fmt(no),
      abstain: fmt(abstain),
      noWithVeto: fmt(noWithVeto),
      totalVoted: fmt(totalVoted),
      yesPercent: pct(yes),
      noPercent: pct(no),
      abstainPercent: pct(abstain),
      vetoPercent: pct(noWithVeto),
    },
    impact: {
      technical: assessTechnicalImpact(content),
      economic: assessEconomicImpact(content),
      governance: assessGovernanceImpact(content),
    },
  };
}

export function formatProposalAnalysis(a: ProposalAnalysis): string {
  return `## Proposal #${a.proposalId} Analysis

**Title**: ${a.title}
**Type**: ${a.type}
**Status**: ${a.status}
**Submit Time**: ${a.submitTime}
**Voting End**: ${a.votingEndTime}

### Summary
${a.description}

### Current Voting
| Vote | Amount | % |
|------|--------|---|
| Yes | ${a.tally.yes} | ${a.tally.yesPercent}% |
| No | ${a.tally.no} | ${a.tally.noPercent}% |
| Abstain | ${a.tally.abstain} | ${a.tally.abstainPercent}% |
| No w/ Veto | ${a.tally.noWithVeto} | ${a.tally.vetoPercent}% |

### Impact Assessment
- **Technical**: ${a.impact.technical}
- **Economic**: ${a.impact.economic}
- **Governance**: ${a.impact.governance}`;
}

function assessTechnicalImpact(content: Record<string, unknown>): string {
  const type = String(content["@type"] ?? "");
  if (type.includes("SoftwareUpgrade")) return "High - Chain upgrade required";
  if (type.includes("Parameter")) return "Low - Configuration change";
  if (type.includes("ClientUpdate")) return "Medium - IBC client update";
  return "Medium";
}

function assessEconomicImpact(content: Record<string, unknown>): string {
  const type = String(content["@type"] ?? "");
  if (type.includes("CommunityPoolSpend"))
    return "Direct - Treasury disbursement";
  if (type.includes("AllowDenom")) return "Medium - Market expansion";
  return "Low";
}

function assessGovernanceImpact(content: Record<string, unknown>): string {
  const type = String(content["@type"] ?? "");
  if (type.includes("UpdateParams")) return "High - Governance rules change";
  if (type.includes("CancelSoftwareUpgrade"))
    return "Medium - Upgrade cancellation";
  return "Low";
}
