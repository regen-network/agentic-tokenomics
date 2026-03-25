export function computeM011KPI({ as_of, collections, scored_batches }) {
  const colls = collections ?? [];
  const batches = scored_batches ?? [];

  // Collection counts
  const collections_active = colls.filter(c =>
    ["ACTIVE", "PROPOSED"].includes(c.status)
  ).length;
  const collections_closed = colls.filter(c => c.status === "CLOSED").length;
  const collections_suspended = colls.filter(c => c.status === "SUSPENDED").length;
  const collections_under_review = colls.filter(c => c.status === "UNDER_REVIEW").length;

  // Batch scoring stats
  const batches_scored = batches.length;
  const scores = batches.map(b => b.quality_score).filter(s => s != null);
  const avg_quality_score = scores.length > 0
    ? Number((scores.reduce((s, q) => s + q, 0) / scores.length).toFixed(1))
    : null;

  const featured_batches = batches.filter(b =>
    b.quality_score != null && b.quality_score >= 800
  ).length;

  // Challenge stats
  const challenged = colls.filter(c =>
    c.challenge != null && c.challenge.outcome != null
  );
  const total_colls = colls.length;
  const challenge_rate = total_colls > 0
    ? Number((colls.filter(c => c.challenge != null).length / total_colls).toFixed(4))
    : 0.0;

  const challenge_outcome_breakdown = challenged.length > 0
    ? {
        curator_wins: challenged.filter(c => c.challenge.outcome === "CURATOR_WINS").length,
        challenger_wins: challenged.filter(c => c.challenge.outcome === "CHALLENGER_WINS").length
      }
    : null;

  // Curation economics
  const bonds = colls.map(c => parseInt(c.bond?.amount ?? "0", 10)).filter(b => b > 0);
  const total_bonded = bonds.reduce((s, b) => s + b, 0);

  const DEFAULT_SLASH_PERCENTAGE = 0.20; // governance parameter, see SPEC.md
  const slashed_colls = colls.filter(c =>
    c.challenge?.outcome === "CHALLENGER_WINS"
  );
  const total_slashed = slashed_colls.reduce((s, c) => {
    const bond = parseInt(c.bond?.amount ?? "0", 10);
    return s + Math.round(bond * DEFAULT_SLASH_PERCENTAGE);
  }, 0);

  const total_trade_volume = colls.reduce((s, c) =>
    s + parseInt(c.trade_volume ?? "0", 10), 0
  );
  const total_curator_rewards = colls.reduce((s, c) =>
    s + parseInt(c.total_rewards ?? "0", 10), 0
  );

  const member_counts = colls.filter(c => c.members).map(c => c.members.length);
  const avg_collection_size = member_counts.length > 0
    ? Number((member_counts.reduce((s, n) => s + n, 0) / member_counts.length).toFixed(1))
    : null;

  // Quality score distribution
  const quality_score_distribution = scores.length > 0
    ? {
        below_300: scores.filter(s => s < 300).length,
        "300_to_599": scores.filter(s => s >= 300 && s < 600).length,
        "600_to_799": scores.filter(s => s >= 600 && s < 800).length,
        "800_plus": scores.filter(s => s >= 800).length
      }
    : null;

  return {
    mechanism_id: "m011",
    scope: "v0_advisory",
    as_of,
    collections_active,
    collections_closed,
    collections_suspended,
    collections_under_review,
    batches_scored,
    avg_quality_score,
    featured_batches,
    challenge_rate,
    challenge_outcome_breakdown,
    curation_economics: {
      total_bonded: String(total_bonded),
      total_slashed: String(total_slashed),
      total_curator_rewards: String(total_curator_rewards),
      total_trade_volume: String(total_trade_volume),
      avg_collection_size
    },
    quality_score_distribution
  };
}
