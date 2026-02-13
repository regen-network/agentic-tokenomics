/**
 * Core types for the Regen Agentic Tokenomics system.
 *
 * Includes inter-agent message envelope (addresses the coordination gap
 * flagged in FEASIBILITY-REVIEW.md §2.3), workflow types, and governance
 * layer definitions.
 */

// ---------------------------------------------------------------------------
// Governance layers (from phase-1/1.4-governance-architecture.md)
// ---------------------------------------------------------------------------

export enum GovernanceLayer {
  /** Fully automated, no human oversight required */
  Layer1_Automated = 1,
  /** Agentic with 24-72h human override window */
  Layer2_AgenticOversight = 2,
  /** Human-in-the-loop, agent provides analysis */
  Layer3_HumanInLoop = 3,
  /** Constitutional — human-only decisions */
  Layer4_Constitutional = 4,
}

// ---------------------------------------------------------------------------
// OODA Workflow types (from phase-2/2.2-agentic-workflows.md)
// ---------------------------------------------------------------------------

export interface ObserveStep {
  id: string;
  type: "ledger_query" | "koi_search" | "memory_recall" | "external_oracle";
  tool: string;
  params: Record<string, unknown>;
}

export interface OrientStep {
  id: string;
  type: "llm_analysis" | "rule_check" | "similarity_search";
  prompt?: string;
  rules?: Array<{ condition: string; result: string }>;
}

export interface DecideStep {
  id: string;
  escalationThreshold: number;
  rules: Array<{
    condition: string;
    recommendation: string;
    confidence: number;
  }>;
}

export interface ActStep {
  id: string;
  type: "post_message" | "create_koi_object" | "escalate" | "log";
  condition?: string;
  params: Record<string, unknown>;
}

export interface OODAWorkflow {
  id: string;
  name: string;
  agentId: string;
  governanceLayer: GovernanceLayer;
  sla: { responseTimeMs: number };
  observe: ObserveStep[];
  orient: OrientStep[];
  decide: DecideStep;
  act: ActStep[];
}

// ---------------------------------------------------------------------------
// Workflow execution tracking (from phase-2/2.5-data-schema-integration.md)
// ---------------------------------------------------------------------------

export type WorkflowStatus =
  | "running"
  | "completed"
  | "failed"
  | "escalated";

export interface WorkflowExecution {
  executionId: string;
  workflowId: string;
  agentId: string;
  status: WorkflowStatus;
  governanceLayer: GovernanceLayer;
  observations: unknown[];
  orientation: Record<string, unknown> | null;
  decision: WorkflowDecision | null;
  actions: unknown[];
  startedAt: Date;
  completedAt?: Date;
}

export interface WorkflowDecision {
  recommendation: string;
  confidence: number;
  rationale: string;
  evidence: unknown[];
}

// ---------------------------------------------------------------------------
// Inter-agent message envelope
//
// This schema was flagged as missing in FEASIBILITY-REVIEW.md §2.3.
// It defines a standard format for all agent-to-agent communication.
// ---------------------------------------------------------------------------

export interface AgentMessageEnvelope {
  /** Unique message ID */
  id: string;
  /** ISO 8601 timestamp */
  timestamp: string;
  /** Sending agent identifier (e.g. "AGENT-002") */
  from: string;
  /** Receiving agent identifier, or "*" for broadcast */
  to: string;
  /** Communication pattern */
  pattern: "request" | "response" | "publish" | "delegate";
  /** What kind of payload this carries */
  type: string;
  /** The actual message content */
  payload: unknown;
  /** Correlation ID for request/response pairs */
  correlationId?: string;
  /** For conflict resolution: higher-confidence messages take precedence */
  confidence?: number;
  /** Workflow execution this message belongs to */
  workflowExecutionId?: string;
  /** Priority for queue ordering (0 = highest) */
  priority?: number;
  /** TTL in milliseconds — messages expire after this */
  ttlMs?: number;
}

// ---------------------------------------------------------------------------
// Agent decision audit record (from phase-2/2.5-data-schema-integration.md)
// ---------------------------------------------------------------------------

export interface AgentDecisionRecord {
  decisionId: string;
  executionId: string;
  agentId: string;
  decisionType: string;
  subjectType: string;
  subjectId: string;
  decision: string;
  confidence: number;
  rationale: string;
  evidence: unknown[];
  humanOverride: boolean;
  createdAt: Date;
}

// ---------------------------------------------------------------------------
// MCP client config
// ---------------------------------------------------------------------------

export interface MCPClientConfig {
  baseUrl: string;
  apiKey: string;
  timeoutMs?: number;
}
