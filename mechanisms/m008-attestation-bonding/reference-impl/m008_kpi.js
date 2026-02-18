export function computeM008KPI({ as_of, attestations }) {
  const atts = attestations ?? [];

  const attestations_submitted = atts.length;
  const attestations_active = atts.filter(a => a.status === "ACTIVE").length;
  const attestations_released = atts.filter(a => a.status === "RELEASED").length;
  const attestations_challenged = atts.filter(a =>
    ["CHALLENGED", "RESOLVED_VALID", "RESOLVED_INVALID", "SLASHED"].includes(a.status)
  ).length;
  const attestations_slashed = atts.filter(a =>
    a.status === "SLASHED" || a.status === "RESOLVED_INVALID"
  ).length;

  const challenge_rate = attestations_submitted > 0
    ? Number((attestations_challenged / attestations_submitted).toFixed(4))
    : 0.0;

  const slash_rate = attestations_challenged > 0
    ? Number((attestations_slashed / attestations_challenged).toFixed(4))
    : 0.0;

  // Bond economics
  const bonds = atts.map(a => parseInt(a.bond?.amount ?? "0", 10)).filter(b => b > 0);
  const total_bonded = bonds.reduce((s, b) => s + b, 0);
  const released = atts.filter(a => a.status === "RELEASED").map(a => parseInt(a.bond?.amount ?? "0", 10));
  const total_released = released.reduce((s, b) => s + b, 0);
  const slashed = atts.filter(a => a.status === "SLASHED" || a.status === "RESOLVED_INVALID")
    .map(a => parseInt(a.bond?.amount ?? "0", 10));
  const total_slashed = slashed.reduce((s, b) => s + b, 0);
  const avg_bond_amount = bonds.length > 0
    ? Number((total_bonded / bonds.length).toFixed(1))
    : null;

  // Quality scores
  const scores = atts.filter(a => a.quality_score != null).map(a => a.quality_score);
  const avg_quality_score = scores.length > 0
    ? Number((scores.reduce((s, q) => s + q, 0) / scores.length).toFixed(1))
    : null;

  // Type breakdown
  const types = { ProjectBoundary: 0, BaselineMeasurement: 0, CreditIssuanceClaim: 0, MethodologyValidation: 0 };
  for (const a of atts) {
    if (types[a.attestation_type] !== undefined) types[a.attestation_type]++;
  }

  return {
    mechanism_id: "m008",
    scope: "v0_advisory",
    as_of,
    attestations_submitted,
    attestations_active,
    attestations_released,
    attestations_challenged,
    attestations_slashed,
    challenge_rate,
    slash_rate,
    bond_economics: {
      total_bonded: String(total_bonded),
      total_released: String(total_released),
      total_slashed: String(total_slashed),
      avg_bond_amount
    },
    avg_quality_score,
    attestation_type_breakdown: types
  };
}
