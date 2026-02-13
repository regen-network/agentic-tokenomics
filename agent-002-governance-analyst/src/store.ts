import Database from "better-sqlite3";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DB_PATH = path.join(__dirname, "..", "agent-002.db");

/**
 * Local SQLite store for agent state.
 *
 * Tracks which proposals we've already analyzed, voting snapshots,
 * and workflow execution history. Intentionally lightweight —
 * can be replaced with PostgreSQL for production.
 */
class Store {
  private db: Database.Database;

  constructor() {
    this.db = new Database(DB_PATH);
    this.db.pragma("journal_mode = WAL");
    this.migrate();
  }

  private migrate() {
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS workflow_executions (
        execution_id TEXT PRIMARY KEY,
        workflow_id TEXT NOT NULL,
        agent_id TEXT NOT NULL,
        status TEXT NOT NULL,
        started_at TEXT NOT NULL,
        completed_at TEXT,
        result TEXT
      );

      CREATE TABLE IF NOT EXISTS proposal_analyses (
        proposal_id TEXT PRIMARY KEY,
        analysis TEXT NOT NULL,
        analyzed_at TEXT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS voting_snapshots (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        proposal_id TEXT NOT NULL,
        snapshot TEXT NOT NULL,
        captured_at TEXT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS post_vote_reports (
        proposal_id TEXT PRIMARY KEY,
        report TEXT NOT NULL,
        reported_at TEXT NOT NULL
      );

      CREATE INDEX IF NOT EXISTS idx_voting_proposal
        ON voting_snapshots(proposal_id);
      CREATE INDEX IF NOT EXISTS idx_exec_workflow
        ON workflow_executions(workflow_id);
    `);
  }

  // ── Proposal analyses ─────────────────────────────────

  hasAnalysis(proposalId: string): boolean {
    const row = this.db
      .prepare("SELECT 1 FROM proposal_analyses WHERE proposal_id = ?")
      .get(proposalId);
    return !!row;
  }

  saveAnalysis(proposalId: string, analysis: string): void {
    this.db
      .prepare(
        `INSERT OR REPLACE INTO proposal_analyses (proposal_id, analysis, analyzed_at)
         VALUES (?, ?, ?)`
      )
      .run(proposalId, analysis, new Date().toISOString());
  }

  getAnalysis(proposalId: string): string | null {
    const row = this.db
      .prepare("SELECT analysis FROM proposal_analyses WHERE proposal_id = ?")
      .get(proposalId) as { analysis: string } | undefined;
    return row?.analysis || null;
  }

  // ── Voting snapshots ──────────────────────────────────

  saveVotingSnapshot(proposalId: string, snapshot: string): void {
    this.db
      .prepare(
        `INSERT INTO voting_snapshots (proposal_id, snapshot, captured_at)
         VALUES (?, ?, ?)`
      )
      .run(proposalId, snapshot, new Date().toISOString());
  }

  getLatestSnapshot(
    proposalId: string
  ): { snapshot: string; captured_at: string } | null {
    const row = this.db
      .prepare(
        `SELECT snapshot, captured_at FROM voting_snapshots
         WHERE proposal_id = ? ORDER BY id DESC LIMIT 1`
      )
      .get(proposalId) as
      | { snapshot: string; captured_at: string }
      | undefined;
    return row || null;
  }

  getSnapshotCount(proposalId: string): number {
    const row = this.db
      .prepare(
        "SELECT COUNT(*) as cnt FROM voting_snapshots WHERE proposal_id = ?"
      )
      .get(proposalId) as { cnt: number };
    return row.cnt;
  }

  // ── Post-vote reports ─────────────────────────────────

  hasPostVoteReport(proposalId: string): boolean {
    const row = this.db
      .prepare("SELECT 1 FROM post_vote_reports WHERE proposal_id = ?")
      .get(proposalId);
    return !!row;
  }

  savePostVoteReport(proposalId: string, report: string): void {
    this.db
      .prepare(
        `INSERT OR REPLACE INTO post_vote_reports (proposal_id, report, reported_at)
         VALUES (?, ?, ?)`
      )
      .run(proposalId, report, new Date().toISOString());
  }

  // ── Workflow executions ───────────────────────────────

  logExecution(exec: {
    executionId: string;
    workflowId: string;
    agentId: string;
    status: string;
    startedAt: string;
    completedAt: string;
    result: string;
  }): void {
    this.db
      .prepare(
        `INSERT INTO workflow_executions
         (execution_id, workflow_id, agent_id, status, started_at, completed_at, result)
         VALUES (?, ?, ?, ?, ?, ?, ?)`
      )
      .run(
        exec.executionId,
        exec.workflowId,
        exec.agentId,
        exec.status,
        exec.startedAt,
        exec.completedAt,
        exec.result
      );
  }

  getExecutionCount(): number {
    const row = this.db
      .prepare("SELECT COUNT(*) as cnt FROM workflow_executions")
      .get() as { cnt: number };
    return row.cnt;
  }

  close(): void {
    this.db.close();
  }
}

export const store = new Store();
