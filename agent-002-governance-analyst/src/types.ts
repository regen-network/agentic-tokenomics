// ============================================================
// Cosmos SDK / Regen Ledger governance types
// ============================================================

export interface Proposal {
  id: string;
  status: ProposalStatus;
  final_tally_result: TallyResult;
  submit_time: string;
  deposit_end_time: string;
  total_deposit: Coin[];
  voting_start_time: string;
  voting_end_time: string;
  content: ProposalContent;
}

export type ProposalStatus =
  | "PROPOSAL_STATUS_DEPOSIT_PERIOD"
  | "PROPOSAL_STATUS_VOTING_PERIOD"
  | "PROPOSAL_STATUS_PASSED"
  | "PROPOSAL_STATUS_REJECTED"
  | "PROPOSAL_STATUS_FAILED";

export interface ProposalContent {
  "@type": string;
  title: string;
  description: string;
  [key: string]: unknown;
}

export interface TallyResult {
  yes: string;
  abstain: string;
  no: string;
  no_with_veto: string;
}

export interface Coin {
  denom: string;
  amount: string;
}

export interface Vote {
  proposal_id: string;
  voter: string;
  option: string;
  options: { option: string; weight: string }[];
}

export interface StakingPool {
  bonded_tokens: string;
  not_bonded_tokens: string;
}

export interface Validator {
  operator_address: string;
  consensus_pubkey: { "@type": string; key: string };
  status: string;
  tokens: string;
  delegator_shares: string;
  description: {
    moniker: string;
    identity: string;
    website: string;
    security_contact: string;
    details: string;
  };
  commission: {
    commission_rates: {
      rate: string;
      max_rate: string;
      max_change_rate: string;
    };
  };
}

// ============================================================
// OODA loop types
// ============================================================

export interface OODAExecution<TObserve, TOrient, TDecide, TAct> {
  executionId: string;
  workflowId: string;
  status: "running" | "completed" | "failed" | "escalated";
  observations: TObserve;
  orientation: TOrient | null;
  decision: TDecide | null;
  actions: TAct | null;
  startedAt: Date;
  completedAt: Date | null;
  error: string | null;
}

// ============================================================
// Workflow-specific types
// ============================================================

export type ProposalCategory =
  | "parameter_change"
  | "software_upgrade"
  | "community_pool_spend"
  | "credit_class"
  | "currency_allowlist"
  | "text_signaling"
  | "other";

export interface ProposalAnalysis {
  proposalId: string;
  title: string;
  category: ProposalCategory;
  tldr: string;
  impactAssessment: {
    technical: string;
    economic: string;
    governance: string;
  };
  historicalContext: string;
  riskFactors: string[];
  stakeholderImpacts: string[];
}

export interface VotingStatus {
  proposalId: string;
  title: string;
  yes: bigint;
  no: bigint;
  abstain: bigint;
  noWithVeto: bigint;
  totalVoted: bigint;
  bondedTokens: bigint;
  turnoutPct: number;
  quorumMet: boolean;
  projectedOutcome: "PASS" | "FAIL" | "UNCERTAIN";
  confidence: number;
  timeRemaining: string;
  alertLevel: "NORMAL" | "HIGH" | "CRITICAL";
}

export interface PostVoteReport {
  proposalId: string;
  title: string;
  outcome: "PASSED" | "REJECTED" | "FAILED";
  finalTurnout: number;
  voteBreakdown: {
    yes: string;
    no: string;
    abstain: string;
    noWithVeto: string;
  };
  analysis: string;
  predictionAccuracy: number | null;
}

// ============================================================
// Output types
// ============================================================

export interface OutputMessage {
  workflow: string;
  proposalId: string;
  title: string;
  content: string;
  alertLevel: "NORMAL" | "HIGH" | "CRITICAL";
  timestamp: Date;
}
