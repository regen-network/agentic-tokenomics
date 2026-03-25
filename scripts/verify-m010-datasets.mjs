#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();

function readJson(rel) {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, rel), "utf8"));
}

function assertIsoDate(value, label) {
  const ts = Date.parse(value);
  assert(Number.isFinite(ts), `${label}: invalid ISO date '${value}'`);
}

function parseSignalIdToken(entry) {
  const m = /^([^\s]+)\s+\(.+\)$/.exec(entry);
  return m ? m[1] : null;
}

function mean(nums) {
  if (!nums.length) return null;
  return nums.reduce((sum, n) => sum + n, 0) / nums.length;
}

function hasDuplicates(values) {
  return new Set(values).size !== values.length;
}

function deriveChallengeKpis(fixture) {
  const events = fixture.events ?? [];
  const challenges = fixture.challenges ?? [];

  const resolvedValid = challenges.filter((c) => c.status === "resolved_valid").length;
  const resolvedInvalid = challenges.filter((c) => c.status === "resolved_invalid").length;
  const resolvedTotal = resolvedValid + resolvedInvalid;
  const escalated = challenges.filter((c) => c.status === "escalated").length;

  const resolutionHours = challenges
    .filter((c) => c.resolution?.resolved_at)
    .map((c) => (Date.parse(c.resolution.resolved_at) - Date.parse(c.timestamp)) / (1000 * 60 * 60))
    .filter((n) => Number.isFinite(n) && n >= 0);

  const avgResolution = mean(resolutionHours);

  return {
    challenges_filed: challenges.length,
    challenge_rate: events.length ? Number((challenges.length / events.length).toFixed(4)) : 0.0,
    challenge_success_rate: resolvedTotal ? Number((resolvedInvalid / resolvedTotal).toFixed(4)) : null,
    avg_resolution_time_hours: avgResolution === null ? null : Number(avgResolution.toFixed(2)),
    admin_resolution_timeout_rate: challenges.length ? Number((escalated / challenges.length).toFixed(4)) : null
  };
}

function verifyChallengeFixture(rel) {
  const fixture = readJson(rel);
  const events = fixture.events ?? [];
  const challenges = fixture.challenges ?? [];

  assert(events.length > 0, `${rel}: events must not be empty`);
  assert(challenges.length > 0, `${rel}: challenge fixtures must include challenges`);

  const signalIdToEvent = new Map();
  for (const e of events) {
    assertIsoDate(e.timestamp, `${rel}: event ${e.signal_id ?? "<missing-signal-id>"} timestamp`);
    assert(typeof e.signal_id === "string" && e.signal_id.length > 0, `${rel}: all challenge replay events must include signal_id`);
    assert(!signalIdToEvent.has(e.signal_id), `${rel}: duplicate signal_id '${e.signal_id}'`);
    signalIdToEvent.set(e.signal_id, e);
  }

  const challengeIds = challenges.map((c) => c.challenge_id);
  assert(!hasDuplicates(challengeIds), `${rel}: duplicate challenge_id detected`);

  for (const c of challenges) {
    assert(typeof c.challenge_id === "string" && c.challenge_id.length > 0, `${rel}: challenge_id must be a non-empty string`);
    assertIsoDate(c.timestamp, `${rel}: challenge ${c.challenge_id} timestamp`);
    const target = signalIdToEvent.get(c.signal_id);
    assert(target, `${rel}: challenge '${c.challenge_id}' references unknown signal_id '${c.signal_id}'`);
    assert(c.category === target.category, `${rel}: challenge '${c.challenge_id}' category mismatch (challenge=${c.category}, signal=${target.category})`);
    const minEvidence = (c.evidence?.koi_links?.length ?? 0) + (c.evidence?.ledger_refs?.length ?? 0);
    assert(minEvidence > 0, `${rel}: challenge '${c.challenge_id}' must include at least one koi_links or ledger_refs evidence entry`);

    const resolved = c.status === "resolved_valid" || c.status === "resolved_invalid";
    if (resolved) {
      assert(c.resolution && typeof c.resolution === "object", `${rel}: resolved challenge '${c.challenge_id}' missing resolution`);
      assertIsoDate(c.resolution.resolved_at, `${rel}: challenge ${c.challenge_id} resolution.resolved_at`);
      const deltaHours = (Date.parse(c.resolution.resolved_at) - Date.parse(c.timestamp)) / (1000 * 60 * 60);
      assert(deltaHours >= 0, `${rel}: challenge '${c.challenge_id}' resolved before it was filed`);
      const expectedOutcome = c.status === "resolved_valid" ? "VALID" : "INVALID";
      assert(
        c.resolution.outcome === expectedOutcome,
        `${rel}: challenge '${c.challenge_id}' status '${c.status}' requires resolution.outcome '${expectedOutcome}', got '${c.resolution.outcome}'`
      );
    } else {
      assert(!c.resolution, `${rel}: unresolved challenge '${c.challenge_id}' must not include resolution`);
    }
  }

  const contributingStatuses = new Set(["active", "resolved_valid"]);
  const rawExpectedContrib = fixture.expected_outputs?.contributing_signals ?? [];
  const rawExpectedExcluded = fixture.expected_outputs?.excluded_signals ?? [];

  const contribTokens = rawExpectedContrib.map(parseSignalIdToken);
  const excludedTokens = rawExpectedExcluded.map(parseSignalIdToken);
  assert(!contribTokens.includes(null), `${rel}: expected_outputs.contributing_signals entries must be '<signal_id> (...)' tokens`);
  assert(!excludedTokens.includes(null), `${rel}: expected_outputs.excluded_signals entries must be '<signal_id> (...)' tokens`);
  assert(!hasDuplicates(contribTokens), `${rel}: expected_outputs.contributing_signals contains duplicates`);
  assert(!hasDuplicates(excludedTokens), `${rel}: expected_outputs.excluded_signals contains duplicates`);

  const expectedContrib = new Set(contribTokens);
  const expectedExcluded = new Set(excludedTokens);

  const derivedContrib = new Set(events.filter((e) => contributingStatuses.has(e.status)).map((e) => e.signal_id));
  const derivedExcluded = new Set(events.filter((e) => !contributingStatuses.has(e.status)).map((e) => e.signal_id));

  assert.deepStrictEqual(expectedContrib, derivedContrib, `${rel}: expected_outputs.contributing_signals do not match status-derived contributors`);
  assert.deepStrictEqual(expectedExcluded, derivedExcluded, `${rel}: expected_outputs.excluded_signals do not match status-derived exclusions`);

  for (const id of expectedContrib) {
    assert(!expectedExcluded.has(id), `${rel}: signal '${id}' appears in both contributing and excluded lists`);
  }

  if (fixture.expected_outputs?.challenge_kpis) {
    const derived = deriveChallengeKpis(fixture);
    for (const [key, expectedValue] of Object.entries(fixture.expected_outputs.challenge_kpis)) {
      assert.deepStrictEqual(
        derived[key],
        expectedValue,
        `${rel}: expected_outputs.challenge_kpis.${key} mismatch (expected ${JSON.stringify(expectedValue)}, derived ${JSON.stringify(derived[key])})`
      );
    }
  }
}

function assertFails(fn, expectedPattern, label) {
  let failed = false;
  try {
    fn();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    assert(expectedPattern.test(message), `${label}: expected error matching ${expectedPattern}, got '${message}'`);
    failed = true;
  }
  assert(failed, `${label}: expected verification to fail`);
}

function main() {
  const sampleRel = "mechanisms/m010-reputation-signal/datasets/fixtures/v0_sample.json";
  const validChallengeFixtures = [
    "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_sample.json",
    "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_escalated_sample.json",
    "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_edge_timing_sample.json"
  ];
  const invalidChallengeFixtures = [
    {
      rel: "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_invalid_resolution_sample.json",
      pattern: /must not include resolution/
    },
    {
      rel: "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_invalid_outcome_sample.json",
      pattern: /requires resolution\.outcome/
    }
  ];

  const sample = readJson(sampleRel);
  assert(sample.events?.length > 0, `${sampleRel}: events must not be empty`);
  for (const [idx, e] of sample.events.entries()) {
    assertIsoDate(e.timestamp, `${sampleRel}: event[${idx}] timestamp`);
  }

  for (const rel of validChallengeFixtures) {
    verifyChallengeFixture(rel);
  }
  for (const { rel, pattern } of invalidChallengeFixtures) {
    assertFails(() => verifyChallengeFixture(rel), pattern, rel);
  }
  console.log("m010 dataset integrity: PASS");
}

try {
  main();
} catch (err) {
  console.error(err instanceof Error ? err.message : String(err));
  process.exit(1);
}
