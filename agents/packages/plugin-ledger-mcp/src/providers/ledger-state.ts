/**
 * Ledger State Provider
 *
 * Injects current Regen Ledger state into the agent's context on every
 * message. Provides: active proposal count, credit class count, REGEN supply.
 *
 * Based on phase-3/3.2-agent-implementation.md §ledgerStateProvider.
 */

import type { LedgerMCPClient } from "../client.js";

export interface LedgerStateSnapshot {
  creditClassCount: number;
  activeProposalCount: number;
  activeProposals: Array<{ id: string; title: string }>;
  totalSupply: string;
}

export async function getLedgerState(
  client: LedgerMCPClient
): Promise<LedgerStateSnapshot> {
  const [classes, proposals, supply] = await Promise.all([
    client.listClasses({ limit: 100 }),
    client.listProposals({
      limit: 10,
      proposal_status: "PROPOSAL_STATUS_VOTING_PERIOD",
    }),
    client.getTotalSupply({ limit: 5 }),
  ]);

  const classesArr = (classes as any)?.classes ?? [];
  const proposalsArr = (proposals as any)?.proposals ?? [];
  const supplyArr = (supply as any)?.supply ?? [];

  const regenSupply = supplyArr.find(
    (s: any) => s.denom === "uregen"
  );
  const totalRegen = regenSupply
    ? `${(BigInt(regenSupply.amount) / BigInt(1_000_000)).toLocaleString()} REGEN`
    : "Unknown";

  return {
    creditClassCount: classesArr.length,
    activeProposalCount: proposalsArr.length,
    activeProposals: proposalsArr.map((p: any) => ({
      id: p.id ?? p.proposal_id,
      title: p.content?.title ?? "Untitled",
    })),
    totalSupply: totalRegen,
  };
}

export function formatLedgerState(state: LedgerStateSnapshot): string {
  const lines = [
    "## Regen Ledger State",
    `- Credit Classes: ${state.creditClassCount} active`,
    `- Active Proposals: ${state.activeProposalCount} in voting`,
    `- Total REGEN Supply: ${state.totalSupply}`,
  ];

  if (state.activeProposalCount > 0) {
    lines.push("", "### Active Proposals");
    for (const p of state.activeProposals) {
      lines.push(`- #${p.id}: ${p.title}`);
    }
  }

  return lines.join("\n");
}
