import { describe, it, expect, vi } from "vitest";
import { OODAExecutor, type StepHandlers } from "@regen/core";
import type { OODAWorkflow, WorkflowDecision } from "@regen/core";
import { GovernanceLayer } from "@regen/core";

function makeWorkflow(overrides?: Partial<OODAWorkflow>): OODAWorkflow {
  return {
    id: "WF-TEST-01",
    name: "Test Workflow",
    agentId: "AGENT-002",
    governanceLayer: GovernanceLayer.Layer1_Automated,
    sla: { responseTimeMs: 60_000 },
    observe: [
      { id: "obs-1", type: "ledger_query", tool: "get_proposal", params: { id: 1 } },
    ],
    orient: [
      { id: "ori-1", type: "rule_check" },
    ],
    decide: {
      id: "dec-1",
      escalationThreshold: 0.7,
      rules: [
        { condition: "completeness >= 0.9", recommendation: "APPROVE", confidence: 0.95 },
      ],
    },
    act: [
      { id: "act-1", type: "post_message", params: { channel: "governance" } },
    ],
    ...overrides,
  };
}

function makeHandlers(overrides?: Partial<StepHandlers>): StepHandlers {
  return {
    observe: vi.fn().mockResolvedValue({ proposalId: 62, status: "VOTING" }),
    orient: vi.fn().mockResolvedValue({ completeness: 0.95, risk: 0.1 }),
    decide: vi.fn().mockResolvedValue({
      recommendation: "APPROVE",
      confidence: 0.92,
      rationale: "All checks passed",
      evidence: [],
    } satisfies WorkflowDecision),
    act: vi.fn().mockResolvedValue({ posted: true }),
    ...overrides,
  };
}

describe("OODAExecutor", () => {
  it("executes a complete workflow through all 4 phases", async () => {
    const handlers = makeHandlers();
    const executor = new OODAExecutor(handlers);
    const workflow = makeWorkflow();

    const result = await executor.execute(workflow);

    expect(result.status).toBe("completed");
    expect(result.workflowId).toBe("WF-TEST-01");
    expect(result.agentId).toBe("AGENT-002");
    expect(result.observations).toHaveLength(1);
    expect(result.orientation).toEqual({ completeness: 0.95, risk: 0.1 });
    expect(result.decision?.recommendation).toBe("APPROVE");
    expect(result.decision?.confidence).toBe(0.92);
    expect(result.actions).toHaveLength(1);
    expect(result.completedAt).toBeDefined();

    expect(handlers.observe).toHaveBeenCalledTimes(1);
    expect(handlers.orient).toHaveBeenCalledTimes(1);
    expect(handlers.decide).toHaveBeenCalledTimes(1);
    expect(handlers.act).toHaveBeenCalledTimes(1);
  });

  it("escalates when confidence is below threshold", async () => {
    const handlers = makeHandlers({
      decide: vi.fn().mockResolvedValue({
        recommendation: "UNCERTAIN",
        confidence: 0.55,
        rationale: "Insufficient data",
        evidence: [],
      }),
    });
    const executor = new OODAExecutor(handlers);
    const workflow = makeWorkflow();

    const result = await executor.execute(workflow);

    expect(result.status).toBe("escalated");
    expect(result.decision?.confidence).toBe(0.55);
    // Act phase should NOT have been executed
    expect(handlers.act).not.toHaveBeenCalled();
    expect(result.actions).toHaveLength(0);
  });

  it("marks execution as failed when an observe step throws", async () => {
    const handlers = makeHandlers({
      observe: vi.fn().mockRejectedValue(new Error("MCP server unreachable")),
    });
    const executor = new OODAExecutor(handlers);
    const workflow = makeWorkflow();

    const result = await executor.execute(workflow);

    expect(result.status).toBe("failed");
    expect(result.completedAt).toBeDefined();
  });

  it("skips act steps whose conditions are not met", async () => {
    const handlers = makeHandlers({
      decide: vi.fn().mockResolvedValue({
        recommendation: "REJECT",
        confidence: 0.88,
        rationale: "Invalid methodology",
        evidence: [],
      }),
    });
    const executor = new OODAExecutor(handlers);
    const workflow = makeWorkflow({
      act: [
        {
          id: "act-approve",
          type: "post_message",
          condition: "recommendation == APPROVE",
          params: { channel: "approved" },
        },
        {
          id: "act-reject",
          type: "post_message",
          condition: "recommendation == REJECT",
          params: { channel: "rejected" },
        },
      ],
    });

    const result = await executor.execute(workflow);

    expect(result.status).toBe("completed");
    expect(result.actions).toHaveLength(1);
    expect((result.actions[0] as any).stepId).toBe("act-reject");
  });

  it("runs multiple observe steps sequentially", async () => {
    const callOrder: string[] = [];
    const handlers = makeHandlers({
      observe: vi.fn().mockImplementation(async (step) => {
        callOrder.push(step.id);
        return { step: step.id };
      }),
    });
    const executor = new OODAExecutor(handlers);
    const workflow = makeWorkflow({
      observe: [
        { id: "obs-1", type: "ledger_query", tool: "proposals", params: {} },
        { id: "obs-2", type: "koi_search", tool: "search", params: {} },
        { id: "obs-3", type: "memory_recall", tool: "recall", params: {} },
      ],
    });

    const result = await executor.execute(workflow);

    expect(result.observations).toHaveLength(3);
    expect(callOrder).toEqual(["obs-1", "obs-2", "obs-3"]);
  });

  it("handles edge case: exactly at escalation threshold", async () => {
    const handlers = makeHandlers({
      decide: vi.fn().mockResolvedValue({
        recommendation: "CONDITIONAL",
        confidence: 0.7, // Exactly at threshold
        rationale: "Borderline case",
        evidence: [],
      }),
    });
    const executor = new OODAExecutor(handlers);
    const workflow = makeWorkflow(); // threshold is 0.7

    const result = await executor.execute(workflow);

    // 0.7 is NOT less than 0.7, so should proceed to act
    expect(result.status).toBe("completed");
    expect(handlers.act).toHaveBeenCalled();
  });
});
