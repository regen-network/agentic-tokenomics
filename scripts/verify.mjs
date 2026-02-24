#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const repoRoot = process.cwd();

function requireFile(rel) {
  const p = path.join(repoRoot, rel);
  if (!fs.existsSync(p)) {
    console.error(`Missing required file: ${rel}`);
    process.exit(2);
  }
}

function run(cmd, args) {
  const res = spawnSync(cmd, args, { cwd: repoRoot, encoding: "utf8" });
  if (res.status !== 0) {
    console.error(res.stdout || "");
    console.error(res.stderr || "");
    process.exit(res.status ?? 3);
  }
}

function readJson(rel) {
  const p = path.join(repoRoot, rel);
  return JSON.parse(fs.readFileSync(p, "utf8"));
}

// Core files
requireFile("README.md");
requireFile("mechanisms/m010-reputation-signal/SPEC.md");
requireFile("mechanisms/m010-reputation-signal/README.md");
requireFile("mechanisms/m010-reputation-signal/schemas/m010_kpi.schema.json");
requireFile("mechanisms/m010-reputation-signal/schemas/m010_signal.schema.json");
requireFile("mechanisms/m010-reputation-signal/schemas/m010_challenge.schema.json");
requireFile("mechanisms/m010-reputation-signal/datasets/schema.json");
requireFile("mechanisms/m010-reputation-signal/datasets/fixtures/v0_sample.json");
requireFile("mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_sample.json");
requireFile("mechanisms/m010-reputation-signal/reference-impl/m010_kpi.js");
requireFile("mechanisms/m010-reputation-signal/reference-impl/m010_score.js");
requireFile("mechanisms/m010-reputation-signal/reference-impl/test_vectors/vector_v0_sample.input.json");
requireFile("mechanisms/m010-reputation-signal/reference-impl/test_vectors/vector_v0_sample.expected.json");
requireFile("mechanisms/m010-reputation-signal/reference-impl/test_vectors/vector_v0_challenge.expected.json");
requireFile("scripts/verify-m010-reference-impl.mjs");
requireFile("scripts/verify-m010-datasets.mjs");

// Mechanism index check
run("node", ["scripts/build-mechanism-index.mjs", "--check"]);
run("node", ["scripts/verify-m010-reference-impl.mjs"]);
run("node", ["scripts/verify-m010-datasets.mjs"]);

// Basic schema sanity
const kpiSchema = readJson("mechanisms/m010-reputation-signal/schemas/m010_kpi.schema.json");
if (!kpiSchema.required || !kpiSchema.required.includes("mechanism_id")) {
  console.error("KPI schema missing required fields.");
  process.exit(4);
}
const challengeKpiRequired = kpiSchema.properties?.challenge_kpis?.required ?? [];
if (!challengeKpiRequired.includes("challenge_rate")) {
  console.error("KPI schema missing required challenge KPI fields.");
  process.exit(4);
}

const signalSchema = readJson("mechanisms/m010-reputation-signal/schemas/m010_signal.schema.json");
const signalStatus = signalSchema.properties?.status?.enum ?? [];
if (!signalStatus.includes("escalated")) {
  console.error("Signal schema missing escalated status.");
  process.exit(5);
}

const challengeSchema = readJson("mechanisms/m010-reputation-signal/schemas/m010_challenge.schema.json");
const challengeGuards = challengeSchema.allOf ?? [];
if (!Array.isArray(challengeGuards) || challengeGuards.length < 4) {
  console.error("Challenge schema missing lifecycle guard clauses.");
  process.exit(6);
}

console.log("agentic-tokenomics verify: PASS");
