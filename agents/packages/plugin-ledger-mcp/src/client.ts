/**
 * Regen Ledger MCP Client
 *
 * Wraps HTTP calls to the Regen Ledger MCP server, which provides
 * on-chain data access (proposals, validators, credit classes, etc).
 *
 * Based on phase-3/3.2-agent-implementation.md §LedgerMCPClient.
 */

import type { MCPClientConfig } from "@regen/core";

export interface LedgerMCPConfig extends MCPClientConfig {}

export class LedgerMCPClient {
  private baseUrl: string;
  private apiKey: string;
  private timeoutMs: number;

  constructor(config: LedgerMCPConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, "");
    this.apiKey = config.apiKey;
    this.timeoutMs = config.timeoutMs ?? 30_000;
  }

  // -- Ecocredit queries --

  async listClasses(params: { limit?: number; offset?: number } = {}) {
    return this.call("list_classes", params);
  }

  async listProjects(params: { limit?: number; offset?: number } = {}) {
    return this.call("list_projects", params);
  }

  async listBatches(params: { limit?: number; offset?: number } = {}) {
    return this.call("list_batches", params);
  }

  // -- Governance queries --

  async getProposal(proposalId: number | string) {
    return this.call("get_governance_proposal", {
      proposal_id: proposalId,
    });
  }

  async listProposals(params: {
    limit?: number;
    proposal_status?: string;
  } = {}) {
    return this.call("list_governance_proposals", params);
  }

  async listVotes(params: { proposal_id: number | string; limit?: number }) {
    return this.call("list_governance_votes", params);
  }

  // -- Marketplace queries --

  async listSellOrders(params: { limit?: number; page?: number } = {}) {
    return this.call("list_sell_orders", params);
  }

  // -- Staking queries --

  async listValidators(params: { limit?: number; status?: string } = {}) {
    return this.call("list_validators", params);
  }

  async getValidatorRewards(validatorAddress: string) {
    return this.call("get_validator_outstanding_rewards", {
      validator_address: validatorAddress,
    });
  }

  // -- Supply --

  async getTotalSupply(params: { limit?: number } = {}) {
    return this.call("get_total_supply", params);
  }

  // -- Generic MCP tool call --

  async call(
    tool: string,
    params: Record<string, unknown>
  ): Promise<unknown> {
    const controller = new AbortController();
    const timeout = setTimeout(
      () => controller.abort(),
      this.timeoutMs
    );

    try {
      const response = await fetch(
        `${this.baseUrl}/mcp/tools/${tool}`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            ...(this.apiKey
              ? { Authorization: `Bearer ${this.apiKey}` }
              : {}),
          },
          body: JSON.stringify(params),
          signal: controller.signal,
        }
      );

      if (!response.ok) {
        throw new Error(
          `Ledger MCP call ${tool} failed: ${response.status} ${response.statusText}`
        );
      }

      return response.json();
    } finally {
      clearTimeout(timeout);
    }
  }
}
