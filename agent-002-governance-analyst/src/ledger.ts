import { config } from "./config.js";
import type {
  Proposal,
  TallyResult,
  Vote,
  StakingPool,
  Validator,
} from "./types.js";

/**
 * Regen Ledger LCD (REST) client.
 *
 * Talks directly to a Cosmos LCD endpoint — no MCP dependency.
 * When Ledger MCP becomes available, this can be swapped out
 * behind the same interface.
 */
export class LedgerClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = (baseUrl || config.lcdUrl).replace(/\/$/, "");
  }

  // ── Governance ──────────────────────────────────────────────

  async listProposals(status?: string): Promise<Proposal[]> {
    const params = new URLSearchParams();
    if (status) params.set("proposal_status", status);
    params.set("pagination.limit", "100");
    params.set("pagination.reverse", "true");

    const data = await this.get(
      `/cosmos/gov/v1beta1/proposals?${params.toString()}`
    );
    return (data.proposals || []) as Proposal[];
  }

  async getProposal(proposalId: string | number): Promise<Proposal | null> {
    try {
      const data = await this.get(
        `/cosmos/gov/v1beta1/proposals/${proposalId}`
      );
      return (data.proposal || null) as Proposal | null;
    } catch {
      return null;
    }
  }

  async getVotingProposals(): Promise<Proposal[]> {
    return this.listProposals("2"); // PROPOSAL_STATUS_VOTING_PERIOD
  }

  async getPassedProposals(): Promise<Proposal[]> {
    return this.listProposals("3"); // PROPOSAL_STATUS_PASSED
  }

  async getRejectedProposals(): Promise<Proposal[]> {
    return this.listProposals("4"); // PROPOSAL_STATUS_REJECTED
  }

  async getDepositProposals(): Promise<Proposal[]> {
    return this.listProposals("1"); // PROPOSAL_STATUS_DEPOSIT_PERIOD
  }

  async getTally(proposalId: string | number): Promise<TallyResult> {
    const data = await this.get(
      `/cosmos/gov/v1beta1/proposals/${proposalId}/tally`
    );
    return data.tally as TallyResult;
  }

  async getVotes(
    proposalId: string | number,
    limit = 200
  ): Promise<Vote[]> {
    const params = new URLSearchParams();
    params.set("pagination.limit", String(limit));

    const data = await this.get(
      `/cosmos/gov/v1beta1/proposals/${proposalId}/votes?${params.toString()}`
    );
    return (data.votes || []) as Vote[];
  }

  // ── Staking ─────────────────────────────────────────────────

  async getStakingPool(): Promise<StakingPool> {
    const data = await this.get(`/cosmos/staking/v1beta1/pool`);
    return data.pool as StakingPool;
  }

  async getValidators(status = "BOND_STATUS_BONDED"): Promise<Validator[]> {
    const params = new URLSearchParams();
    params.set("status", status);
    params.set("pagination.limit", "200");

    const data = await this.get(
      `/cosmos/staking/v1beta1/validators?${params.toString()}`
    );
    return (data.validators || []) as Validator[];
  }

  // ── HTTP ────────────────────────────────────────────────────

  private async get(path: string): Promise<Record<string, unknown>> {
    const url = `${this.baseUrl}${path}`;
    const res = await fetch(url, {
      headers: { Accept: "application/json" },
      signal: AbortSignal.timeout(15_000),
    });

    if (!res.ok) {
      throw new Error(`LCD ${res.status}: ${res.statusText} — ${url}`);
    }

    return (await res.json()) as Record<string, unknown>;
  }
}
