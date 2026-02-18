/**
 * v0 (advisory): 4-factor weighted composite scoring for credit class creator proposals.
 *
 * Factors:
 *   methodology_quality (weight 0.4): Methodology rigor across additionality, baseline, MRV, permanence
 *   admin_reputation    (weight 0.3): M010 reputation score for admin address
 *   novelty             (weight 0.2): 1 - max_similarity vs existing classes
 *   completeness        (weight 0.1): Application completeness checklist
 *
 * See SPEC.md section 5 for full formula.
 *
 * @param {Object} opts
 * @param {Object} opts.proposal - Proposal with methodology_iri, credit_type, admin_address
 * @param {Object} opts.factors - Pre-computed factor scores (each 0-1000)
 * @param {number} opts.factors.methodology_quality - Methodology rigor score
 * @param {number} opts.factors.admin_reputation - M010 reputation (or 500 default)
 * @param {number} opts.factors.novelty - (1 - max_similarity) * 1000
 * @param {number} opts.factors.completeness - Checklist score
 * @returns {{ score: number, confidence: number, recommendation: string, factors: Object }}
 */
export function computeM001Score({ proposal, factors }) {
  const W_METHODOLOGY = 0.4;
  const W_REPUTATION = 0.3;
  const W_NOVELTY = 0.2;
  const W_COMPLETENESS = 0.1;

  const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));

  const fMeth = clamp(factors.methodology_quality ?? 0, 0, 1000);
  const fRep = clamp(factors.admin_reputation ?? 500, 0, 1000);
  const fNov = clamp(factors.novelty ?? 0, 0, 1000);
  const fComp = clamp(factors.completeness ?? 0, 0, 1000);

  const score = Math.round(
    W_METHODOLOGY * fMeth +
    W_REPUTATION * fRep +
    W_NOVELTY * fNov +
    W_COMPLETENESS * fComp
  );

  const confidence = computeConfidence(factors);

  let recommendation;
  if (score >= 700) {
    recommendation = "APPROVE";
  } else if (score < 300 && confidence > 900) {
    recommendation = "REJECT";
  } else {
    recommendation = "CONDITIONAL";
  }

  return {
    score: clamp(score, 0, 1000),
    confidence,
    recommendation,
    factors: {
      methodology_quality: fMeth,
      admin_reputation: fRep,
      novelty: fNov,
      completeness: fComp
    }
  };
}

/**
 * Compute confidence based on data availability for each factor.
 *
 * @param {Object} factors
 * @param {boolean} [factors.reputation_available] - M010 score exists (not defaulted)
 * @param {boolean} [factors.methodology_resolvable] - IRI resolves to parseable document
 * @param {boolean} [factors.sufficient_classes] - >= 3 existing classes for similarity
 * @param {boolean} [factors.history_available] - Historical proposal data for admin
 * @returns {number} confidence (0-1000)
 */
function computeConfidence(factors) {
  let available = 0;
  const total = 4;

  if (factors.reputation_available) available++;
  if (factors.methodology_resolvable !== false) available++;
  if (factors.sufficient_classes !== false) available++;
  if (factors.history_available) available++;

  return Math.round((available / total) * 1000);
}

// --- Self-test: run with `node m001_score.js` ---
const isMain = typeof process !== "undefined" &&
  process.argv[1] &&
  (process.argv[1].endsWith("m001_score.js") || process.argv[1].endsWith("m001_score"));

if (isMain) {
  const fs = await import("node:fs");
  const path = await import("node:path");
  const url = await import("node:url");

  const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
  const inputPath = path.join(__dirname, "test_vectors", "vector_v0_sample.input.json");
  const expectedPath = path.join(__dirname, "test_vectors", "vector_v0_sample.expected.json");

  const input = JSON.parse(fs.readFileSync(inputPath, "utf8"));
  const expected = JSON.parse(fs.readFileSync(expectedPath, "utf8"));

  const results = input.proposals.map(p => computeM001Score({
    proposal: p.proposal,
    factors: p.factors
  }));

  let pass = true;
  for (let i = 0; i < results.length; i++) {
    const r = results[i];
    const e = expected.scores[i];
    if (r.score !== e.score || r.recommendation !== e.recommendation) {
      console.error(`FAIL proposal[${i}]: got score=${r.score} rec=${r.recommendation}, expected score=${e.score} rec=${e.recommendation}`);
      pass = false;
    }
  }

  if (pass) {
    console.log("m001_score self-test: PASS");
    console.log(JSON.stringify({ scores: results }, null, 2));
  } else {
    process.exit(1);
  }
}
