export function median(nums) {
  if (!nums.length) return null;
  const s = [...nums].sort((a,b) => a-b);
  const mid = Math.floor(s.length/2);
  return s.length % 2 ? s[mid] : (s[mid-1] + s[mid]) / 2;
}

function mean(nums) {
  if (!nums.length) return null;
  return nums.reduce((sum, n) => sum + n, 0) / nums.length;
}

export function computeM010KPI({ as_of, events, challenges = [], scope = "v0_advisory" }) {
  const asOf = new Date(as_of);
  const evs = events ?? [];
  const signals_emitted = evs.length;
  const subjects_touched = new Set(evs.map(e => `${e.subject_type}:${e.subject_id}`)).size;
  const chs = challenges ?? [];

  let bothEvidence = 0;
  const latencies = [];

  for (const e of evs) {
    const koiOk = (e.evidence?.koi_links ?? []).length > 0;
    const ledOk = (e.evidence?.ledger_refs ?? []).length > 0;
    if (koiOk && ledOk) bothEvidence += 1;

    const ts = new Date(e.timestamp);
    const hours = (asOf - ts) / (1000*60*60);
    if (Number.isFinite(hours)) latencies.push(hours);
  }

  const evidence_coverage_rate = signals_emitted ? bothEvidence / signals_emitted : 0.0;
  const median_event_latency_hours = median(latencies);

  const out = {
    mechanism_id: "m010",
    scope,
    as_of,
    signals_emitted,
    subjects_touched,
    evidence_coverage_rate: Number(evidence_coverage_rate.toFixed(4)),
    median_event_latency_hours: median_event_latency_hours === null ? null : Number(median_event_latency_hours.toFixed(2))
  };

  if (chs.length) {
    const resolvedValid = chs.filter((c) => c.status === "resolved_valid").length;
    const resolvedInvalid = chs.filter((c) => c.status === "resolved_invalid").length;
    const resolvedTotal = resolvedValid + resolvedInvalid;
    const escalated = chs.filter((c) => c.status === "escalated").length;

    const resolutionHours = chs
      .filter((c) => c.resolution?.resolved_at)
      .map((c) => (new Date(c.resolution.resolved_at) - new Date(c.timestamp)) / (1000 * 60 * 60))
      .filter((n) => Number.isFinite(n) && n >= 0);

    const avgResolution = mean(resolutionHours);

    out.challenge_kpis = {
      challenges_filed: chs.length,
      challenge_rate: signals_emitted ? Number((chs.length / signals_emitted).toFixed(4)) : 0.0,
      avg_resolution_time_hours: avgResolution === null ? null : Number(avgResolution.toFixed(2)),
      challenge_success_rate: resolvedTotal ? Number((resolvedInvalid / resolvedTotal).toFixed(4)) : null,
      admin_resolution_timeout_rate: Number((escalated / chs.length).toFixed(4))
    };
  }

  return out;
}
