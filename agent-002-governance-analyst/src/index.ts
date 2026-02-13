#!/usr/bin/env node
import { config, validateConfig } from "./config.js";
import { LedgerClient } from "./ledger.js";
import { executeOODA } from "./ooda.js";
import { store } from "./store.js";
import { createProposalAnalysisWorkflow } from "./workflows/proposal-analysis.js";
import { createVotingMonitorWorkflow } from "./workflows/voting-monitor.js";
import { createPostVoteReportWorkflow } from "./workflows/post-vote-report.js";

// ── Banner ────────────────────────────────────────────────────

function banner() {
  console.log(`
  ╔══════════════════════════════════════════════════════════════╗
  ║            REGEN GOVERNANCE ANALYST (AGENT-002)              ║
  ║                                                              ║
  ║  Layer 1 — Fully Automated, Informational Only               ║
  ║  Workflows: WF-GA-01, WF-GA-02, WF-GA-03                    ║
  ║                                                              ║
  ║  Regen Agentic Tokenomics Framework                          ║
  ╚══════════════════════════════════════════════════════════════╝
`);
}

// ── Main loop ─────────────────────────────────────────────────

async function runCycle(ledger: LedgerClient): Promise<void> {
  const ts = new Date().toISOString();
  console.log(`\n[${ts}] ═══ Starting governance analysis cycle ═══\n`);

  // WF-GA-01: Analyze any new proposals
  const wf01 = createProposalAnalysisWorkflow(ledger);
  await executeOODA(wf01);

  // WF-GA-02: Monitor voting on active proposals
  const wf02 = createVotingMonitorWorkflow(ledger);
  await executeOODA(wf02);

  // WF-GA-03: Report on recently finalized proposals
  const wf03 = createPostVoteReportWorkflow(ledger);
  await executeOODA(wf03);

  const execCount = store.getExecutionCount();
  console.log(
    `[${new Date().toISOString()}] ═══ Cycle complete (${execCount} total executions logged) ═══\n`
  );
}

async function main() {
  banner();
  validateConfig();

  const runOnce = process.argv.includes("--once");
  const ledger = new LedgerClient();

  console.log(`Configuration:`);
  console.log(`  LCD endpoint: ${config.lcdUrl}`);
  console.log(`  LLM model:    ${config.model}`);
  console.log(`  Discord:      ${config.discordWebhookUrl ? "configured" : "not configured"}`);
  console.log(`  Mode:         ${runOnce ? "single run" : `polling every ${config.pollIntervalMs / 1000}s`}`);
  console.log();

  // Verify LCD connectivity
  try {
    const pool = await ledger.getStakingPool();
    const bonded = BigInt(pool.bonded_tokens) / 1_000_000n;
    console.log(`Connected to Regen Ledger. Bonded: ${bonded.toLocaleString()} REGEN\n`);
  } catch (err) {
    console.error(
      `Failed to connect to Regen Ledger at ${config.lcdUrl}:`,
      err
    );
    process.exit(1);
  }

  if (runOnce) {
    // Single run mode (e.g., `npm run analyze`)
    await runCycle(ledger);
  } else {
    // Polling mode
    await runCycle(ledger);

    const interval = setInterval(() => {
      runCycle(ledger).catch((err) =>
        console.error(`Cycle failed:`, err)
      );
    }, config.pollIntervalMs);

    // Graceful shutdown
    const shutdown = () => {
      console.log("\nShutting down gracefully...");
      clearInterval(interval);
      store.close();
      process.exit(0);
    };

    process.on("SIGINT", shutdown);
    process.on("SIGTERM", shutdown);

    console.log("Agent running. Press Ctrl+C to stop.\n");
  }
}

main().catch((err) => {
  console.error("Fatal error:", err);
  process.exit(1);
});
