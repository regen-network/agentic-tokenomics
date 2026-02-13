import Anthropic from "@anthropic-ai/sdk";
import { config } from "./config.js";
import type { Proposal, TallyResult, ProposalCategory } from "./types.js";

const client = new Anthropic({ apiKey: config.anthropicApiKey });

const SYSTEM_PROMPT = `You are the Regen Governance Analyst Agent (AGENT-002).

Your responsibilities:
1. Analyzing and summarizing governance proposals for Regen Network
2. Tracking voting progress and predicting outcomes
3. Providing historical context and precedent analysis
4. Alerting stakeholders to important governance events

Core Principles:
- NEVER endorse specific voting positions
- Present balanced analysis of pros and cons
- Cite historical precedents when relevant
- Flag potential conflicts of interest
- Be precise with numbers and percentages

Regen Network Context:
- Cosmos SDK-based blockchain for ecological assets (eco-credits)
- ~224M REGEN total supply, ~3.2M in community pool
- 75 active validators, 33.4% quorum, 50% pass threshold
- 61+ historical governance proposals
- Core modules: x/ecocredit, x/gov, x/staking, x/marketplace

Output Format:
- Use markdown for structure
- Include vote tables with percentages
- Quantify confidence levels (0.0-1.0)
- Flag risks explicitly`;

/**
 * Ask Claude to analyze a governance proposal.
 */
export async function analyzeProposal(
  proposal: Proposal,
  tally: TallyResult,
  bondedTokens: string
): Promise<string> {
  const prompt = buildAnalysisPrompt(proposal, tally, bondedTokens);

  const response = await client.messages.create({
    model: config.model,
    max_tokens: 2000,
    system: SYSTEM_PROMPT,
    messages: [{ role: "user", content: prompt }],
  });

  return extractText(response);
}

/**
 * Ask Claude to assess voting status and project outcome.
 */
export async function assessVotingStatus(
  proposal: Proposal,
  tally: TallyResult,
  bondedTokens: string,
  previousSnapshot: string | null
): Promise<string> {
  const totalVoted =
    BigInt(tally.yes) +
    BigInt(tally.no) +
    BigInt(tally.abstain) +
    BigInt(tally.no_with_veto);
  const bonded = BigInt(bondedTokens);
  const turnout = bonded > 0n ? Number(totalVoted * 10000n / bonded) / 100 : 0;

  const votingEnd = new Date(proposal.voting_end_time);
  const now = new Date();
  const hoursRemaining = Math.max(
    0,
    (votingEnd.getTime() - now.getTime()) / 3_600_000
  );

  const prompt = `Provide a voting status update for this Regen Network governance proposal.

## Proposal
- ID: #${proposal.id}
- Title: ${proposal.content.title}
- Type: ${proposal.content["@type"]}
- Status: ${proposal.status}
- Voting ends: ${proposal.voting_end_time} (${hoursRemaining.toFixed(1)} hours remaining)

## Current Tally (uregen)
- Yes: ${tally.yes} (${pct(tally.yes, totalVoted)}%)
- No: ${tally.no} (${pct(tally.no, totalVoted)}%)
- Abstain: ${tally.abstain} (${pct(tally.abstain, totalVoted)}%)
- No with Veto: ${tally.no_with_veto} (${pct(tally.no_with_veto, totalVoted)}%)

## Participation
- Total voted: ${totalVoted.toString()} uregen
- Bonded tokens: ${bondedTokens} uregen
- Current turnout: ${turnout.toFixed(1)}%
- Quorum required: 33.4%
- Quorum met: ${turnout >= 33.4 ? "YES" : "NO"}

${previousSnapshot ? `## Previous Snapshot\n${previousSnapshot}` : "## No previous snapshot available."}

Provide:
1. A 2-sentence summary of current state
2. Whether quorum is at risk
3. Projected outcome (PASS/FAIL/UNCERTAIN) with confidence (0.0-1.0)
4. Alert level (NORMAL/HIGH/CRITICAL) based on quorum risk, close vote, or time pressure
5. Key changes since last snapshot (if available)

Format as a concise markdown report.`;

  const response = await client.messages.create({
    model: config.model,
    max_tokens: 1200,
    system: SYSTEM_PROMPT,
    messages: [{ role: "user", content: prompt }],
  });

  return extractText(response);
}

/**
 * Ask Claude to generate a post-vote analysis report.
 */
export async function generatePostVoteReport(
  proposal: Proposal,
  tally: TallyResult,
  bondedTokens: string,
  votes: { voter: string; option: string }[]
): Promise<string> {
  const totalVoted =
    BigInt(tally.yes) +
    BigInt(tally.no) +
    BigInt(tally.abstain) +
    BigInt(tally.no_with_veto);
  const bonded = BigInt(bondedTokens);
  const turnout = bonded > 0n ? Number(totalVoted * 10000n / bonded) / 100 : 0;

  const outcome =
    proposal.status === "PROPOSAL_STATUS_PASSED"
      ? "PASSED"
      : proposal.status === "PROPOSAL_STATUS_REJECTED"
        ? "REJECTED"
        : "FAILED";

  const prompt = `Generate a post-vote analysis report for this finalized Regen Network governance proposal.

## Proposal
- ID: #${proposal.id}
- Title: ${proposal.content.title}
- Type: ${proposal.content["@type"]}
- Outcome: ${outcome}

## Description
${proposal.content.description.slice(0, 2000)}

## Final Tally (uregen)
- Yes: ${tally.yes} (${pct(tally.yes, totalVoted)}%)
- No: ${tally.no} (${pct(tally.no, totalVoted)}%)
- Abstain: ${tally.abstain} (${pct(tally.abstain, totalVoted)}%)
- No with Veto: ${tally.no_with_veto} (${pct(tally.no_with_veto, totalVoted)}%)
- Total turnout: ${turnout.toFixed(1)}%

## Vote Count
- Total distinct voters: ${votes.length}

Provide:
1. Executive summary (2-3 sentences)
2. Analysis of the result — was it decisive or contested?
3. Turnout analysis — high/low compared to typical Regen proposals
4. Implications — what does this mean for the network going forward?
5. Notable patterns (e.g., high veto %, low turnout, etc.)

Format as a structured markdown report with ## headings.`;

  const response = await client.messages.create({
    model: config.model,
    max_tokens: 1500,
    system: SYSTEM_PROMPT,
    messages: [{ role: "user", content: prompt }],
  });

  return extractText(response);
}

// ── Helpers ──────────────────────────────────────────────────

function buildAnalysisPrompt(
  proposal: Proposal,
  tally: TallyResult,
  bondedTokens: string
): string {
  const totalVoted =
    BigInt(tally.yes) +
    BigInt(tally.no) +
    BigInt(tally.abstain) +
    BigInt(tally.no_with_veto);

  return `Analyze this Regen Network governance proposal in detail.

## Proposal Data
- ID: #${proposal.id}
- Title: ${proposal.content.title}
- Type: ${proposal.content["@type"]}
- Status: ${proposal.status}
- Submitted: ${proposal.submit_time}
- Voting Start: ${proposal.voting_start_time}
- Voting End: ${proposal.voting_end_time}
- Deposit: ${proposal.total_deposit.map((d) => `${d.amount} ${d.denom}`).join(", ")}

## Description
${proposal.content.description.slice(0, 3000)}

## Current Tally (uregen)
- Yes: ${tally.yes} (${pct(tally.yes, totalVoted)}%)
- No: ${tally.no} (${pct(tally.no, totalVoted)}%)
- Abstain: ${tally.abstain} (${pct(tally.abstain, totalVoted)}%)
- No with Veto: ${tally.no_with_veto} (${pct(tally.no_with_veto, totalVoted)}%)
- Bonded tokens: ${bondedTokens}

Provide a comprehensive analysis with:
1. **TL;DR** — One-sentence summary
2. **Category** — parameter_change | software_upgrade | community_pool_spend | credit_class | currency_allowlist | text_signaling | other
3. **Impact Assessment** (Technical / Economic / Governance — rate each Low/Medium/High with explanation)
4. **Risk Factors** — What could go wrong?
5. **Historical Context** — How does this compare to previous Regen proposals?
6. **Stakeholder Impacts** — Who benefits, who bears risk?

Format as a structured markdown document.`;
}

function pct(amount: string, total: bigint): string {
  if (total === 0n) return "0.0";
  return (Number(BigInt(amount) * 10000n / total) / 100).toFixed(1);
}

function extractText(response: Anthropic.Message): string {
  return response.content
    .filter((b): b is Anthropic.TextBlock => b.type === "text")
    .map((b) => b.text)
    .join("\n");
}
