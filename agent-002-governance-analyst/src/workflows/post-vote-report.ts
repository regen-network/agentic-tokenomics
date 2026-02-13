import { LedgerClient } from "../ledger.js";
import { store } from "../store.js";
import { generatePostVoteReport } from "../analyst.js";
import { output } from "../output.js";
import type { OODAWorkflow } from "../ooda.js";
import type { Proposal, TallyResult, Vote } from "../types.js";

/**
 * WF-GA-03: Post-Vote Analysis & Reporting
 *
 * Trigger: Proposal finalized (passed, rejected, or failed)
 * Layer: 1 (fully automated, informational only)
 *
 * OODA:
 *   Observe — Fetch finalized proposals, final tallies, vote records
 *   Orient  — Identify proposals we haven't reported on yet
 *   Decide  — Generate post-vote analysis via Claude
 *   Act     — Persist report, output to channels
 */

interface Observations {
  finalizedProposals: Proposal[];
  tallies: Map<string, TallyResult>;
  votes: Map<string, Vote[]>;
  bondedTokens: string;
}

interface Orientation {
  proposalsToReport: {
    proposal: Proposal;
    tally: TallyResult;
    votes: Vote[];
  }[];
}

interface Decision {
  reports: {
    proposalId: string;
    title: string;
    outcome: string;
    report: string;
  }[];
}

interface Actions {
  saved: number;
  output: number;
}

export function createPostVoteReportWorkflow(
  ledger: LedgerClient
): OODAWorkflow<Observations, Orientation, Decision, Actions> {
  return {
    id: "WF-GA-03",
    name: "Post-Vote Analysis & Reporting",

    async observe(): Promise<Observations> {
      const [passed, rejected, pool] = await Promise.all([
        ledger.getPassedProposals(),
        ledger.getRejectedProposals(),
        ledger.getStakingPool(),
      ]);

      const finalizedProposals = [...passed, ...rejected];
      const tallies = new Map<string, TallyResult>();
      const votes = new Map<string, Vote[]>();

      // Only fetch details for proposals we haven't reported on
      const unreported = finalizedProposals.filter(
        (p) => !store.hasPostVoteReport(p.id)
      );

      // Limit to most recent 5 unreported to avoid overwhelming API
      const toProcess = unreported.slice(0, 5);

      await Promise.all(
        toProcess.map(async (p) => {
          try {
            tallies.set(p.id, p.final_tally_result);
            const v = await ledger.getVotes(p.id);
            votes.set(p.id, v);
          } catch {
            tallies.set(p.id, p.final_tally_result);
            votes.set(p.id, []);
          }
        })
      );

      return {
        finalizedProposals: toProcess,
        tallies,
        votes,
        bondedTokens: pool.bonded_tokens,
      };
    },

    async orient(obs: Observations): Promise<Orientation> {
      const proposalsToReport = obs.finalizedProposals
        .filter((p) => !store.hasPostVoteReport(p.id))
        .map((proposal) => ({
          proposal,
          tally: obs.tallies.get(proposal.id) || proposal.final_tally_result,
          votes: obs.votes.get(proposal.id) || [],
        }));

      return { proposalsToReport };
    },

    async decide(orientation: Orientation): Promise<Decision> {
      const reports: Decision["reports"] = [];

      for (const { proposal, tally, votes } of orientation.proposalsToReport) {
        const outcome =
          proposal.status === "PROPOSAL_STATUS_PASSED"
            ? "PASSED"
            : proposal.status === "PROPOSAL_STATUS_REJECTED"
              ? "REJECTED"
              : "FAILED";

        console.log(
          `  Generating post-vote report for #${proposal.id}: ${proposal.content.title} (${outcome})`
        );

        const simpleVotes = votes.map((v) => ({
          voter: v.voter,
          option: v.option || v.options?.[0]?.option || "unknown",
        }));

        const report = await generatePostVoteReport(
          proposal,
          tally,
          "0",
          simpleVotes
        );

        reports.push({
          proposalId: proposal.id,
          title: proposal.content.title,
          outcome,
          report,
        });
      }

      return { reports };
    },

    async act(decision: Decision): Promise<Actions> {
      let saved = 0;
      let outputCount = 0;

      for (const { proposalId, title, outcome, report } of decision.reports) {
        store.savePostVoteReport(proposalId, report);
        saved++;

        await output({
          workflow: "WF-GA-03",
          proposalId,
          title: `${title} — ${outcome}`,
          content: report,
          alertLevel: "NORMAL",
          timestamp: new Date(),
        });
        outputCount++;
      }

      if (decision.reports.length === 0) {
        console.log("  No finalized proposals need reporting.");
      }

      return { saved, output: outputCount };
    },
  };
}
