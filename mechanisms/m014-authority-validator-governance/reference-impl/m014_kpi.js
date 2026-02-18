import { computeM014Score } from "./m014_score.js";

/**
 * m014 Authority Validator Governance — KPI Computation
 *
 * Computes aggregate KPI metrics for the authority validator set.
 *
 * @param {Object} opts
 * @param {string} opts.as_of - ISO 8601 timestamp for evaluation point-in-time
 * @param {Array}  opts.validators - Array of validator objects with address, moniker,
 *        category, status, and factors (uptime, governance_participation, ecosystem_contribution)
 * @param {number} [opts.validator_fund_balance] - Optional: total validator fund balance from M013
 * @returns {Object} KPI block conforming to m014_kpi.schema.json
 */
export function computeM014KPI({ as_of, validators, validator_fund_balance = null }) {
  const vals = validators ?? [];

  // Count by status
  const validators_by_status = {
    candidate: 0,
    approved: 0,
    active: 0,
    probation: 0,
    removed: 0,
    term_expired: 0
  };
  for (const v of vals) {
    const st = v.status ?? "candidate";
    if (st in validators_by_status) validators_by_status[st]++;
  }

  // Active validators only for category/performance analysis
  const activeVals = vals.filter(v => v.status === "active" || v.status === "probation");

  // Count active by category
  const validators_by_category = {
    infrastructure_builders: 0,
    trusted_refi_partners: 0,
    ecological_data_stewards: 0
  };
  for (const v of activeVals) {
    const cat = v.category;
    if (cat in validators_by_category) validators_by_category[cat]++;
  }

  // Compute performance scores for active validators
  const scores = [];
  let belowThreshold = 0;

  for (const v of activeVals) {
    const result = computeM014Score({
      validator: { address: v.address },
      factors: v.factors ?? {}
    });
    scores.push(result.performance_score);
    if (result.flags.includes("below_performance_threshold")) belowThreshold++;
  }

  const avg_performance_score = scores.length
    ? Number((scores.reduce((a, b) => a + b, 0) / scores.length).toFixed(4))
    : null;

  const min_performance_score = scores.length
    ? Number(Math.min(...scores).toFixed(4))
    : null;

  const max_performance_score = scores.length
    ? Number(Math.max(...scores).toFixed(4))
    : null;

  // Composition validity: each category must have >= 5 active validators
  const composition_valid =
    validators_by_category.infrastructure_builders >= 5 &&
    validators_by_category.trusted_refi_partners >= 5 &&
    validators_by_category.ecological_data_stewards >= 5;

  // Byzantine tolerance: active_count > 3f + 1
  // Include probation validators — they are still in the active set
  const active_count = activeVals.length;
  const max_byzantine_f = Math.floor((active_count - 1) / 3);
  const tolerance_met = active_count > 0 && active_count >= 3 * max_byzantine_f + 1;

  // Compensation stats
  let compensation = null;
  if (validator_fund_balance != null && active_count > 0) {
    const base_per_validator = Number(((validator_fund_balance * 0.90) / active_count).toFixed(2));
    const bonus_pool = Number((validator_fund_balance * 0.10).toFixed(2));
    compensation = {
      validator_fund_balance,
      base_per_validator,
      bonus_pool
    };
  }

  const kpi = {
    mechanism_id: "m014",
    scope: "v0",
    as_of,
    total_validators: vals.length,
    validators_by_status,
    validators_by_category,
    avg_performance_score,
    min_performance_score,
    max_performance_score,
    validators_below_threshold: belowThreshold,
    composition_valid,
    byzantine_tolerance: {
      active_count,
      max_byzantine_f,
      tolerance_met
    }
  };

  if (compensation) kpi.compensation = compensation;

  return kpi;
}
