export function median(nums) {
  if (!nums.length) return null;
  const s = [...nums].sort((a, b) => a - b);
  const mid = Math.floor(s.length / 2);
  return s.length % 2 ? s[mid] : (s[mid - 1] + s[mid]) / 2;
}

/**
 * Compute KPI metrics for credit class approval proposals.
 *
 * @param {Object} opts
 * @param {string} opts.as_of - ISO-8601 timestamp for point-in-time evaluation
 * @param {Array} opts.proposals - Array of proposal objects with status, agent_score, submit_time, decision_time
 * @returns {Object} KPI block matching m001_kpi.schema.json
 */
export function computeM001KPI({ as_of, proposals }) {
  const props = proposals ?? [];

  const proposals_submitted = props.length;
  const proposals_approved = props.filter(p => p.status === "APPROVED").length;
  const proposals_rejected = props.filter(p =>
    p.status === "REJECTED" || p.outcome?.result === "AUTO_REJECTED"
  ).length;
  const proposals_expired = props.filter(p => p.status === "EXPIRED").length;

  const decided = proposals_approved + proposals_rejected + proposals_expired;
  const approval_rate = decided > 0
    ? Number((proposals_approved / decided).toFixed(4))
    : 0.0;

  // Agent scoring metrics
  const scored = props.filter(p => p.agent_score != null);
  const proposals_scored = scored.length;

  let agent_accuracy = null;
  if (proposals_scored > 0) {
    const decided_scored = scored.filter(p =>
      p.status === "APPROVED" || p.status === "REJECTED" || p.status === "EXPIRED"
    );
    if (decided_scored.length > 0) {
      let correct = 0;
      for (const p of decided_scored) {
        const rec = p.agent_score.recommendation;
        const outcome = p.status;
        if (
          (rec === "APPROVE" && outcome === "APPROVED") ||
          (rec === "REJECT" && (outcome === "REJECTED" || p.outcome?.result === "AUTO_REJECTED")) ||
          (rec === "CONDITIONAL" && (outcome === "APPROVED" || outcome === "REJECTED"))
        ) {
          correct++;
        }
      }
      agent_accuracy = Number((correct / decided_scored.length).toFixed(4));
    }
  }

  const avg_score = proposals_scored > 0
    ? Number((scored.reduce((s, p) => s + p.agent_score.score, 0) / proposals_scored).toFixed(1))
    : null;

  const auto_reject_count = props.filter(p => p.outcome?.result === "AUTO_REJECTED").length;
  const override_count = props.filter(p => p.override != null).length;

  // Time to decision
  const decisionTimes = [];
  for (const p of props) {
    if (p.submit_time && p.decision_time) {
      const hours = (new Date(p.decision_time) - new Date(p.submit_time)) / (1000 * 60 * 60);
      if (Number.isFinite(hours)) decisionTimes.push(hours);
    }
  }
  const avg_time_to_decision_hours = decisionTimes.length > 0
    ? Number((decisionTimes.reduce((a, b) => a + b, 0) / decisionTimes.length).toFixed(2))
    : null;

  return {
    mechanism_id: "m001-enh",
    scope: "v0_advisory",
    as_of,
    proposals_submitted,
    proposals_approved,
    proposals_rejected,
    proposals_expired,
    approval_rate,
    agent_scoring: {
      proposals_scored,
      agent_accuracy,
      avg_score,
      auto_reject_count,
      override_count
    },
    avg_time_to_decision_hours
  };
}
