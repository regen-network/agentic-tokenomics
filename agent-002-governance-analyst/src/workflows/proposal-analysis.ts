import { LedgerClient } from "../ledger.js";
import { store } from "../store.js";
import { analyzeProposal } from "../analyst.js";
import { output } from "../output.js";
import type { OODAWorkflow } from "../ooda.js";
import type { Proposal, TallyResult } from "../types.js";

/**
 * WF-GA-01: Proposal Analysis & Summarization
 *
 * Trigger: New proposal enters voting period (or deposit period)
 * Layer: 1 (fully automated, informational only)
 * SLA: Analysis within 2 hours of proposal submission
 *
 * OODA:
 *   Observe — Fetch proposal data and current tally from ledger
 *   Orient  — Classify proposal, assess impact, gather context
 *   Decide  — Generate comprehensive analysis via Claude
 *   Act     — Persist analysis, output to configured channels
 */

interface Observations {
  proposals: Proposal[];
  tallies: Map<string, TallyResult>;
  bondedTokens: string;
  newProposalIds: string[];
}

interface Orientation {
  proposalsToAnalyze: {
    proposal: Proposal;
    tally: TallyResult;
  }[];
}

interface Decision {
  analyses: { proposalId: string; title: string; analysis: string }[];
}

interface Actions {
  saved: number;
  output: number;
}

export function createProposalAnalysisWorkflow(
  ledger: LedgerClient
): OODAWorkflow<Observations, Orientation, Decision, Actions> {
  return {
    id: "WF-GA-01",
    name: "Proposal Analysis & Summarization",

    async observe(): Promise<Observations> {
      // Fetch all proposals in voting and deposit periods
      const [voting, deposit, pool] = await Promise.all([
        ledger.getVotingProposals(),
        ledger.getDepositProposals(),
        ledger.getStakingPool(),
      ]);

      const proposals = [...voting, ...deposit];
      const tallies = new Map<string, TallyResult>();

      // Fetch live tallies for voting proposals
      await Promise.all(
        voting.map(async (p) => {
          try {
            const tally = await ledger.getTally(p.id);
            tallies.set(p.id, tally);
          } catch {
            // Use final_tally_result as fallback
            tallies.set(p.id, p.final_tally_result);
          }
        })
      );

      // For deposit-period proposals, use zero tally
      for (const p of deposit) {
        if (!tallies.has(p.id)) {
          tallies.set(p.id, {
            yes: "0",
            no: "0",
            abstain: "0",
            no_with_veto: "0",
          });
        }
      }

      // Identify new proposals we haven't analyzed
      const newProposalIds = proposals
        .map((p) => p.id)
        .filter((id) => !store.hasAnalysis(id));

      return {
        proposals,
        tallies,
        bondedTokens: pool.bonded_tokens,
        newProposalIds,
      };
    },

    async orient(obs: Observations): Promise<Orientation> {
      const proposalsToAnalyze = obs.newProposalIds
        .map((id) => {
          const proposal = obs.proposals.find((p) => p.id === id);
          const tally = obs.tallies.get(id);
          if (!proposal || !tally) return null;
          return { proposal, tally };
        })
        .filter(Boolean) as Orientation["proposalsToAnalyze"];

      return { proposalsToAnalyze };
    },

    async decide(orientation: Orientation): Promise<Decision> {
      const analyses: Decision["analyses"] = [];

      for (const { proposal, tally } of orientation.proposalsToAnalyze) {
        console.log(
          `  Analyzing proposal #${proposal.id}: ${proposal.content.title}`
        );

        // This calls Claude
        const analysis = await analyzeProposal(
          proposal,
          tally,
          "0" // bondedTokens passed separately
        );

        analyses.push({
          proposalId: proposal.id,
          title: proposal.content.title,
          analysis,
        });
      }

      return { analyses };
    },

    async act(decision: Decision): Promise<Actions> {
      let saved = 0;
      let outputCount = 0;

      for (const { proposalId, title, analysis } of decision.analyses) {
        // Persist
        store.saveAnalysis(proposalId, analysis);
        saved++;

        // Output
        await output({
          workflow: "WF-GA-01",
          proposalId,
          title,
          content: analysis,
          alertLevel: "NORMAL",
          timestamp: new Date(),
        });
        outputCount++;
      }

      if (decision.analyses.length === 0) {
        console.log("  No new proposals to analyze.");
      }

      return { saved, output: outputCount };
    },
  };
}
