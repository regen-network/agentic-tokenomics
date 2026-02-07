/**
 * v0 (advisory): Exponential decay with configurable half-life.
 * Default halfLifeHours = 14 * 24 = 336 hours
 * lambda = ln(2) / halfLifeHours
 *
 * v0 computes a decay-weighted average of endorsement levels (no stake weighting).
 * Stake parameter is accepted for forward-compatibility with v1 (on-chain, stake-weighted).
 * See SPEC.md section 5.2 for v0 vs target formula distinction.
 *
 * @param {Object} opts
 * @param {string} opts.as_of - ISO 8601 timestamp for scoring point-in-time
 * @param {Array} opts.events - Array of signal events with timestamp, endorsement_level, and optional stake
 * @param {number} [opts.halfLifeHours=336] - Decay half-life in hours (configurable)
 * @param {boolean} [opts.useStakeWeighting=false] - Reserved for v1: enable stake-weighted scoring
 * @returns {{ reputation_score_0_1: number }}
 */
export function computeM010Score({ as_of, events, halfLifeHours = 336, useStakeWeighting = false }) {
  const asOf = new Date(as_of);
  const evs = events ?? [];
  if (!evs.length) return { reputation_score_0_1: 0.0 };

  const lambda = Math.log(2) / halfLifeHours;

  let wSum = 0;
  let dSum = 0;

  for (const e of evs) {
    const ts = new Date(e.timestamp);
    const ageH = (asOf - ts) / (1000*60*60);
    const decay = Math.exp(-lambda * Math.max(0, ageH));
    const w = Math.max(0, Math.min(1, (e.endorsement_level ?? 0) / 5));
    // v1 (future): const stakeWeight = useStakeWeighting ? (e.stake ?? 1) : 1;
    wSum += w * decay;
    dSum += decay;
  }

  const score = dSum ? (wSum / dSum) : 0.0;
  return { reputation_score_0_1: Number(Math.max(0, Math.min(1, score)).toFixed(4)) };
}
