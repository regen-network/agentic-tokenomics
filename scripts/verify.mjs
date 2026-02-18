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
requireFile("mechanisms/m010-reputation-signal/datasets/fixtures/v0_sample.json");

// m012 core files
requireFile("mechanisms/m012-fixed-cap-dynamic-supply/SPEC.md");
requireFile("mechanisms/m012-fixed-cap-dynamic-supply/README.md");
requireFile("mechanisms/m012-fixed-cap-dynamic-supply/schemas/m012_kpi.schema.json");
requireFile("mechanisms/m012-fixed-cap-dynamic-supply/schemas/m012_supply_state.schema.json");
requireFile("mechanisms/m012-fixed-cap-dynamic-supply/schemas/m012_period_record.schema.json");
requireFile("mechanisms/m012-fixed-cap-dynamic-supply/datasets/fixtures/v0_sample.json");
requireFile("mechanisms/m012-fixed-cap-dynamic-supply/reference-impl/m012_supply.js");
requireFile("mechanisms/m012-fixed-cap-dynamic-supply/reference-impl/m012_kpi.js");

// Mechanism index check
run("node", ["scripts/build-mechanism-index.mjs", "--check"]);

// Basic schema sanity — m010
const kpiSchema = readJson("mechanisms/m010-reputation-signal/schemas/m010_kpi.schema.json");
if (!kpiSchema.required || !kpiSchema.required.includes("mechanism_id")) {
  console.error("m010 KPI schema missing required fields.");
  process.exit(4);
}

// Basic schema sanity — m012
const m012KpiSchema = readJson("mechanisms/m012-fixed-cap-dynamic-supply/schemas/m012_kpi.schema.json");
if (!m012KpiSchema.required || !m012KpiSchema.required.includes("mechanism_id")) {
  console.error("m012 KPI schema missing required fields.");
  process.exit(4);
}

// m012 self-test
run("node", ["mechanisms/m012-fixed-cap-dynamic-supply/reference-impl/m012_supply.js"]);

console.log("agentic-tokenomics verify: PASS");
