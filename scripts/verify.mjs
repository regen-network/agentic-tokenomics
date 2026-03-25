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

function listFilesRecursive(absDir) {
  const out = [];
  for (const entry of fs.readdirSync(absDir, { withFileTypes: true })) {
    const abs = path.join(absDir, entry.name);
    if (entry.isDirectory()) {
      out.push(...listFilesRecursive(abs));
    } else if (entry.isFile()) {
      out.push(abs);
    }
  }
  return out;
}

function assert(condition, message, exitCode) {
  if (!condition) {
    console.error(message);
    process.exit(exitCode);
  }
}

function validateSchema(rel, schema) {
  assert(schema && typeof schema === "object" && !Array.isArray(schema), `${rel}: schema must be a JSON object`, 5);
  assert(typeof schema.$schema === "string" && schema.$schema.length > 0, `${rel}: missing $schema`, 5);
  assert(schema.type === "object", `${rel}: top-level type must be object`, 5);
  assert(schema.properties && typeof schema.properties === "object" && !Array.isArray(schema.properties), `${rel}: missing properties object`, 5);

  if (schema.required !== undefined) {
    assert(Array.isArray(schema.required), `${rel}: required must be an array`, 5);
    const seen = new Set();
    for (const key of schema.required) {
      assert(typeof key === "string" && key.length > 0, `${rel}: required entries must be non-empty strings`, 5);
      assert(!seen.has(key), `${rel}: duplicate required entry '${key}'`, 5);
      seen.add(key);
      assert(Object.prototype.hasOwnProperty.call(schema.properties, key), `${rel}: required key '${key}' missing from properties`, 5);
    }
  }
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

// m013 core files
requireFile("mechanisms/m013-value-based-fee-routing/SPEC.md");
requireFile("mechanisms/m013-value-based-fee-routing/README.md");
requireFile("mechanisms/m013-value-based-fee-routing/schemas/m013_kpi.schema.json");
requireFile("mechanisms/m013-value-based-fee-routing/schemas/m013_fee_event.schema.json");
requireFile("mechanisms/m013-value-based-fee-routing/schemas/m013_fee_config.schema.json");
requireFile("mechanisms/m013-value-based-fee-routing/datasets/fixtures/v0_sample.json");

// Basic schema sanity — m010
const kpiSchema = readJson("mechanisms/m010-reputation-signal/schemas/m010_kpi.schema.json");
if (!kpiSchema.required || !kpiSchema.required.includes("mechanism_id")) {
  console.error("m010 KPI schema missing required fields.");
  process.exit(4);
// Schema sanity for all canonical schema artifacts.
const allFiles = listFilesRecursive(repoRoot);
const schemaFiles = allFiles
  .map((abs) => path.relative(repoRoot, abs))
  .filter((rel) => rel.endsWith(".schema.json"))
  .sort();

assert(schemaFiles.length > 0, "No .schema.json files found.", 4);
for (const rel of schemaFiles) {
  validateSchema(rel, readJson(rel));
}

// Basic schema sanity — m013
const m013KpiSchema = readJson("mechanisms/m013-value-based-fee-routing/schemas/m013_kpi.schema.json");
if (!m013KpiSchema.required || !m013KpiSchema.required.includes("mechanism_id")) {
  console.error("m013 KPI schema missing required fields.");
  process.exit(4);
}

// m013 self-test
run("node", ["mechanisms/m013-value-based-fee-routing/reference-impl/m013_fee.js"]);
// Basic schema sanity — m012
const m012KpiSchema = readJson("mechanisms/m012-fixed-cap-dynamic-supply/schemas/m012_kpi.schema.json");
if (!m012KpiSchema.required || !m012KpiSchema.required.includes("mechanism_id")) {
  console.error("m012 KPI schema missing required fields.");
  process.exit(4);
}

// m012 self-test
run("node", ["mechanisms/m012-fixed-cap-dynamic-supply/reference-impl/m012_supply.js"]);

console.log("agentic-tokenomics verify: PASS");
