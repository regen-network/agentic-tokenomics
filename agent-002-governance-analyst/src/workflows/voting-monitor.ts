import { LedgerClient } from "../ledger.js";
import { store } from "../store.js";
import { assessVotingStatus } from "../analyst.js";
import { output } from "../output.js";
import type { OODAWorkflow } from "../ooda.js";
import type { Proposal, TallyResult } from "../types.js";
import { config } from "../config.js";

/**
 * WF-GA-02: Voting Outcome Prediction & Alerts
 *
 * Trigger: Periodic (every poll cycle) during voting period
 * Layer: 1 (fully automated, informational only)
 * SLA: Updates every 6h (normal), 1h (final 48h), 15m (final 6h)
 *
 * OODA:
 *   Observe — Fetch current tally and voting power for active proposals
 *   Orient  — Calculate turnout, quorum status, time remaining
 *   Decide  — Project outcome via Claude, determine alert level
 *   Act     — Save snapshot, output alerts if needed
 */

interface Observations {
  votingProposals: Proposal[];
  tallies: Map<string, TallyResult>;
  bondedTokens: string;
}

interface Orientation {
  proposalsToUpdate: {
    proposal: Proposal;
    tally: TallyResult;
    hoursRemaining: number;
    turnoutPct: number;
    quorumMet: boolean;
    previousSnapshot: string | null;
    shouldUpdate: boolean;
  }[];
}

interface Decision {
  updates: {
    proposalId: string;
    title: string;
    statusReport: string;
    alertLevel: "NORMAL" | "HIGH" | "CRITICAL";
  }[];
}

interface Actions {
  snapshotsSaved: number;
  alertsSent: number;
}

export function createVotingMonitorWorkflow(
  ledger: LedgerClient
): OODAWorkflow<Observations, Orientation, Decision, Actions> {
  return {
    id: "WF-GA-02",
    name: "Voting Outcome Prediction & Alerts",

    async observe(): Promise<Observations> {
      const [votingProposals, pool] = await Promise.all([
        ledger.getVotingProposals(),
        ledger.getStakingPool(),
      ]);

      const tallies = new Map<string, TallyResult>();
      await Promise.all(
        votingProposals.map(async (p) => {
          try {
            tallies.set(p.id, await ledger.getTally(p.id));
          } catch {
            tallies.set(p.id, p.final_tally_result);
          }
        })
      );

      return {
        votingProposals,
        tallies,
        bondedTokens: pool.bonded_tokens,
      };
    },

    async orient(obs: Observations): Promise<Orientation> {
      const now = Date.now();

      const proposalsToUpdate = obs.votingProposals.map((proposal) => {
        const tally = obs.tallies.get(proposal.id)!;
        const votingEnd = new Date(proposal.voting_end_time).getTime();
        const hoursRemaining = Math.max(0, (votingEnd - now) / 3_600_000);

        const totalVoted =
          BigInt(tally.yes) +
          BigInt(tally.no) +
          BigInt(tally.abstain) +
          BigInt(tally.no_with_veto);
        const bonded = BigInt(obs.bondedTokens);
        const turnoutPct =
          bonded > 0n ? Number(totalVoted * 10000n / bonded) / 100 : 0;
        const quorumMet = turnoutPct >= config.governance.quorumThreshold * 100;

        const prev = store.getLatestSnapshot(proposal.id);

        // Determine update frequency based on time remaining
        const snapshotCount = store.getSnapshotCount(proposal.id);
        let shouldUpdate: boolean;

        if (hoursRemaining <= 6) {
          // Final 6h: always update
          shouldUpdate = true;
        } else if (hoursRemaining <= 48) {
          // Final 48h: update every cycle
          shouldUpdate = true;
        } else {
          // Normal: update if this is the first snapshot or enough time has passed
          shouldUpdate = snapshotCount === 0 || !prev;
          if (prev) {
            const lastTime = new Date(prev.captured_at).getTime();
            const hoursSinceLast = (now - lastTime) / 3_600_000;
            shouldUpdate = shouldUpdate || hoursSinceLast >= 6;
          }
        }

        return {
          proposal,
          tally,
          hoursRemaining,
          turnoutPct,
          quorumMet,
          previousSnapshot: prev?.snapshot || null,
          shouldUpdate,
        };
      });

      return { proposalsToUpdate };
    },

    async decide(orientation: Orientation): Promise<Decision> {
      const updates: Decision["updates"] = [];

      for (const item of orientation.proposalsToUpdate) {
        if (!item.shouldUpdate) continue;

        console.log(
          `  Monitoring proposal #${item.proposal.id}: ${item.proposal.content.title} (${item.hoursRemaining.toFixed(1)}h left)`
        );

        const statusReport = await assessVotingStatus(
          item.proposal,
          item.tally,
          "0", // passed via tally context
          item.previousSnapshot
        );

        // Determine alert level
        let alertLevel: "NORMAL" | "HIGH" | "CRITICAL" = "NORMAL";
        if (item.hoursRemaining <= 24 && !item.quorumMet) {
          alertLevel = "CRITICAL";
        } else if (item.hoursRemaining <= 48 && !item.quorumMet) {
          alertLevel = "HIGH";
        } else if (item.hoursRemaining <= 6) {
          alertLevel = "HIGH";
        }

        updates.push({
          proposalId: item.proposal.id,
          title: item.proposal.content.title,
          statusReport,
          alertLevel,
        });
      }

      return { updates };
    },

    async act(decision: Decision): Promise<Actions> {
      let snapshotsSaved = 0;
      let alertsSent = 0;

      for (const update of decision.updates) {
        // Save snapshot
        store.saveVotingSnapshot(update.proposalId, update.statusReport);
        snapshotsSaved++;

        // Output (all updates go out; alerts get elevated)
        await output({
          workflow: "WF-GA-02",
          proposalId: update.proposalId,
          title: update.title,
          content: update.statusReport,
          alertLevel: update.alertLevel,
          timestamp: new Date(),
        });

        if (update.alertLevel !== "NORMAL") {
          alertsSent++;
        }
      }

      if (decision.updates.length === 0) {
        console.log("  No proposals in voting period.");
      }

      return { snapshotsSaved, alertsSent };
    },
  };
}
