#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();

function readJson(rel) {
  const abs = path.join(repoRoot, rel);
  return JSON.parse(fs.readFileSync(abs, "utf8"));
}

async function loadModuleFromJs(rel) {
  const abs = path.join(repoRoot, rel);
  const src = fs.readFileSync(abs, "utf8");
  const dataUrl = `data:text/javascript;base64,${Buffer.from(src).toString("base64")}`;
  return import(dataUrl);
}

function assertSubset(actual, expected, label) {
  for (const [key, value] of Object.entries(expected)) {
    assert.deepStrictEqual(
      actual[key],
      value,
      `${label}: expected '${key}' to equal ${JSON.stringify(value)}, got ${JSON.stringify(actual[key])}`
    );
  }
}

function mean(nums) {
  if (!nums.length) return null;
  return nums.reduce((sum, n) => sum + n, 0) / nums.length;
}

function assertFiniteNumberInRange(value, min, max, label) {
  assert(typeof value === "number" && Number.isFinite(value), `${label}: expected finite number, got ${JSON.stringify(value)}`);
  assert(value >= min && value <= max, `${label}: expected in range [${min}, ${max}], got ${value}`);
}

function deriveChallengeKpis(input) {
  const chs = input.challenges ?? [];
  const evs = input.events ?? [];

  const resolvedValid = chs.filter((c) => c.status === "resolved_valid").length;
  const resolvedInvalid = chs.filter((c) => c.status === "resolved_invalid").length;
  const resolvedTotal = resolvedValid + resolvedInvalid;
  const escalated = chs.filter((c) => c.status === "escalated").length;

  const resolutionHours = chs
    .filter((c) => c.resolution?.resolved_at)
    .map((c) => (Date.parse(c.resolution.resolved_at) - Date.parse(c.timestamp)) / (1000 * 60 * 60))
    .filter((n) => Number.isFinite(n) && n >= 0);

  const avgResolution = mean(resolutionHours);

  return {
    challenges_filed: chs.length,
    challenge_rate: evs.length ? Number((chs.length / evs.length).toFixed(4)) : 0.0,
    avg_resolution_time_hours: avgResolution === null ? null : Number(avgResolution.toFixed(2)),
    challenge_success_rate: resolvedTotal ? Number((resolvedInvalid / resolvedTotal).toFixed(4)) : null,
    admin_resolution_timeout_rate: Number((escalated / chs.length).toFixed(4))
  };
}

function assertVectorInvariants(vectorName, input, actual) {
  const evs = input.events ?? [];
  const chs = input.challenges ?? [];
  const kpi = actual.kpi ?? {};
  const score = actual.score ?? {};

  assert.strictEqual(kpi.mechanism_id, "m010", `${vectorName}: KPI mechanism_id must be 'm010'`);
  assert.strictEqual(kpi.as_of, input.as_of, `${vectorName}: KPI as_of must match input`);
  assert.strictEqual(kpi.signals_emitted, evs.length, `${vectorName}: KPI signals_emitted must equal input event count`);

  const uniqueSubjects = new Set(evs.map((e) => `${e.subject_type}:${e.subject_id}`)).size;
  assert.strictEqual(kpi.subjects_touched, uniqueSubjects, `${vectorName}: KPI subjects_touched must equal unique subjects in input`);
  assertFiniteNumberInRange(kpi.evidence_coverage_rate, 0, 1, `${vectorName}: KPI evidence_coverage_rate`);

  if (kpi.median_event_latency_hours !== null) {
    assertFiniteNumberInRange(kpi.median_event_latency_hours, 0, Number.MAX_SAFE_INTEGER, `${vectorName}: KPI median_event_latency_hours`);
  }

  assertFiniteNumberInRange(score.reputation_score_0_1, 0, 1, `${vectorName}: score.reputation_score_0_1`);

  if (chs.length > 0) {
    assert(kpi.challenge_kpis && typeof kpi.challenge_kpis === "object", `${vectorName}: challenge_kpis must exist when challenges are provided`);

    const requiredKeys = [
      "challenges_filed",
      "challenge_rate",
      "avg_resolution_time_hours",
      "challenge_success_rate",
      "admin_resolution_timeout_rate"
    ];
    for (const key of requiredKeys) {
      assert(Object.prototype.hasOwnProperty.call(kpi.challenge_kpis, key), `${vectorName}: missing challenge_kpis.${key}`);
    }

    assert.strictEqual(kpi.challenge_kpis.challenges_filed, chs.length, `${vectorName}: challenge_kpis.challenges_filed must equal challenge count`);
    assertFiniteNumberInRange(kpi.challenge_kpis.challenge_rate, 0, 1, `${vectorName}: challenge_kpis.challenge_rate`);
    assertFiniteNumberInRange(kpi.challenge_kpis.admin_resolution_timeout_rate, 0, 1, `${vectorName}: challenge_kpis.admin_resolution_timeout_rate`);

    if (kpi.challenge_kpis.challenge_success_rate !== null) {
      assertFiniteNumberInRange(kpi.challenge_kpis.challenge_success_rate, 0, 1, `${vectorName}: challenge_kpis.challenge_success_rate`);
    }
    if (kpi.challenge_kpis.avg_resolution_time_hours !== null) {
      assertFiniteNumberInRange(kpi.challenge_kpis.avg_resolution_time_hours, 0, Number.MAX_SAFE_INTEGER, `${vectorName}: challenge_kpis.avg_resolution_time_hours`);
    }

    const derived = deriveChallengeKpis(input);
    assert.deepStrictEqual(
      kpi.challenge_kpis,
      derived,
      `${vectorName}: challenge_kpis mismatch with values derived from input fixture`
    );
  } else {
    assert(!Object.prototype.hasOwnProperty.call(kpi, "challenge_kpis"), `${vectorName}: challenge_kpis should be omitted when no challenges are provided`);
  }
}

function computeOutputs(input, computeM010KPI, computeM010Score) {
  return {
    kpi: computeM010KPI({
      as_of: input.as_of,
      events: input.events,
      challenges: input.challenges,
      scope: input.scope
    }),
    score: computeM010Score({
      as_of: input.as_of,
      events: input.events
    })
  };
}

async function main() {
  const { computeM010KPI } = await loadModuleFromJs("mechanisms/m010-reputation-signal/reference-impl/m010_kpi.js");
  const { computeM010Score } = await loadModuleFromJs("mechanisms/m010-reputation-signal/reference-impl/m010_score.js");

  const vectors = [
    {
      name: "v0_sample",
      inputRel: "mechanisms/m010-reputation-signal/reference-impl/test_vectors/vector_v0_sample.input.json",
      expectedRel: "mechanisms/m010-reputation-signal/reference-impl/test_vectors/vector_v0_sample.expected.json"
    },
    {
      name: "v0_challenge",
      inputRel: "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_sample.json",
      expectedRel: "mechanisms/m010-reputation-signal/reference-impl/test_vectors/vector_v0_challenge.expected.json",
      assertFixtureKpis: true
    },
    {
      name: "v0_challenge_escalated",
      inputRel: "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_escalated_sample.json",
      expectedRel: "mechanisms/m010-reputation-signal/reference-impl/test_vectors/vector_v0_challenge_escalated.expected.json",
      assertFixtureKpis: true
    },
    {
      name: "v0_challenge_edge_timing",
      inputRel: "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_edge_timing_sample.json",
      expectedRel: "mechanisms/m010-reputation-signal/reference-impl/test_vectors/vector_v0_challenge_edge_timing.expected.json",
      assertFixtureKpis: true
    }
  ];

  for (const vector of vectors) {
    const input = readJson(vector.inputRel);
    const expected = readJson(vector.expectedRel);
    const actual = computeOutputs(input, computeM010KPI, computeM010Score);
    assertVectorInvariants(vector.name, input, actual);

    try {
      assert.deepStrictEqual(actual, expected);
    } catch (err) {
      console.error(`m010 vector mismatch for '${vector.name}'`);
      console.error("Expected:");
      console.error(JSON.stringify(expected, null, 2));
      console.error("Actual:");
      console.error(JSON.stringify(actual, null, 2));
      throw err;
    }

    if (vector.assertFixtureKpis && input.expected_outputs?.challenge_kpis) {
      assertSubset(actual.kpi.challenge_kpis ?? {}, input.expected_outputs.challenge_kpis, `fixture expected_outputs.challenge_kpis (${vector.name})`);
    }
  }

  console.log("m010 reference-impl vectors: PASS");
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : String(err));
  process.exit(1);
});
