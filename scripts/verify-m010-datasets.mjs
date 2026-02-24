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

function verifyChallengeFixture(rel) {
  const fixture = readJson(rel);
  const events = fixture.events ?? [];
  const challenges = fixture.challenges ?? [];

  assert(events.length > 0, `${rel}: events must not be empty`);

  const signalIdToEvent = new Map();
  for (const e of events) {
    assertIsoDate(e.timestamp, `${rel}: event ${e.signal_id ?? "<missing-signal-id>"} timestamp`);
    assert(typeof e.signal_id === "string" && e.signal_id.length > 0, `${rel}: all challenge replay events must include signal_id`);
    assert(!signalIdToEvent.has(e.signal_id), `${rel}: duplicate signal_id '${e.signal_id}'`);
    signalIdToEvent.set(e.signal_id, e);
  }

  for (const c of challenges) {
    assertIsoDate(c.timestamp, `${rel}: challenge ${c.challenge_id} timestamp`);
    const target = signalIdToEvent.get(c.signal_id);
    assert(target, `${rel}: challenge '${c.challenge_id}' references unknown signal_id '${c.signal_id}'`);
    assert(c.category === target.category, `${rel}: challenge '${c.challenge_id}' category mismatch (challenge=${c.category}, signal=${target.category})`);

    const resolved = c.status === "resolved_valid" || c.status === "resolved_invalid";
    if (resolved) {
      assert(c.resolution && typeof c.resolution === "object", `${rel}: resolved challenge '${c.challenge_id}' missing resolution`);
      assertIsoDate(c.resolution.resolved_at, `${rel}: challenge ${c.challenge_id} resolution.resolved_at`);
      const deltaHours = (Date.parse(c.resolution.resolved_at) - Date.parse(c.timestamp)) / (1000 * 60 * 60);
      assert(deltaHours >= 0, `${rel}: challenge '${c.challenge_id}' resolved before it was filed`);
    } else {
      assert(!c.resolution, `${rel}: unresolved challenge '${c.challenge_id}' must not include resolution`);
    }
  }

  const contributingStatuses = new Set(["active", "resolved_valid"]);
  const expectedContrib = new Set((fixture.expected_outputs?.contributing_signals ?? []).map(parseSignalIdToken).filter(Boolean));
  const expectedExcluded = new Set((fixture.expected_outputs?.excluded_signals ?? []).map(parseSignalIdToken).filter(Boolean));

  const derivedContrib = new Set(events.filter((e) => contributingStatuses.has(e.status)).map((e) => e.signal_id));
  const derivedExcluded = new Set(events.filter((e) => !contributingStatuses.has(e.status)).map((e) => e.signal_id));

  assert.deepStrictEqual(expectedContrib, derivedContrib, `${rel}: expected_outputs.contributing_signals do not match status-derived contributors`);
  assert.deepStrictEqual(expectedExcluded, derivedExcluded, `${rel}: expected_outputs.excluded_signals do not match status-derived exclusions`);

  for (const id of expectedContrib) {
    assert(!expectedExcluded.has(id), `${rel}: signal '${id}' appears in both contributing and excluded lists`);
  }
}

function main() {
  const sampleRel = "mechanisms/m010-reputation-signal/datasets/fixtures/v0_sample.json";
  const challengeRel = "mechanisms/m010-reputation-signal/datasets/fixtures/v0_challenge_sample.json";

  const sample = readJson(sampleRel);
  assert(sample.events?.length > 0, `${sampleRel}: events must not be empty`);
  for (const [idx, e] of sample.events.entries()) {
    assertIsoDate(e.timestamp, `${sampleRel}: event[${idx}] timestamp`);
  }

  verifyChallengeFixture(challengeRel);
  console.log("m010 dataset integrity: PASS");
}

try {
  main();
} catch (err) {
  console.error(err instanceof Error ? err.message : String(err));
  process.exit(1);
}
