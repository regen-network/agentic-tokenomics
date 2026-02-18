/**
 * m012 — Fixed Cap Dynamic Supply: KPI computation.
 *
 * Aggregates period records into a KPI block conforming to
 * schemas/m012_kpi.schema.json.
 *
 * @module m012_kpi
 */

/**
 * Compute m012 KPI from an array of period records.
 *
 * @param {Object} opts
 * @param {string} opts.as_of - ISO-8601 timestamp for KPI point-in-time
 * @param {Array} opts.periods - Array of period records with { supply_after, minted, burned, regrowth_rate, effective_multiplier }
 * @param {string} [opts.hard_cap] - Hard cap in uregen (string, default "221000000000000")
 * @returns {Object} KPI block conforming to m012_kpi.schema.json
 */
export function computeM012KPI({ as_of, periods, hard_cap }) {
  const cap = BigInt(hard_cap ?? "221000000000000");
  const ps = periods ?? [];

  if (!ps.length) {
    return {
      mechanism_id: "m012",
      scope: "v0_reference",
      as_of,
      current_supply: "0",
      hard_cap: cap.toString(),
      cap_headroom: cap.toString(),
      cap_utilization: 0,
      total_minted: "0",
      total_burned: "0",
      net_supply_change: "0",
      latest_regrowth_rate: 0,
      latest_effective_multiplier: 1.0,
      periods_evaluated: 0,
      equilibrium_status: "not_reached",
      equilibrium_gap_pct: null
    };
  }

  let totalMinted = 0n;
  let totalBurned = 0n;

  for (const p of ps) {
    totalMinted += BigInt(p.minted);
    totalBurned += BigInt(p.burned);
  }

  const last = ps[ps.length - 1];
  const currentSupply = BigInt(last.supply_after ?? last.next_supply);
  const headroom = cap - currentSupply;
  const capUtil = Number(currentSupply) / Number(cap);
  const netChange = totalMinted - totalBurned;

  // Equilibrium detection: gap as percentage
  const maxMB = totalMinted > totalBurned ? totalMinted : totalBurned;
  const absGap = netChange >= 0n ? netChange : -netChange;
  const gapPct = maxMB > 0n
    ? Number(absGap) / Number(maxMB) * 100
    : null;

  let equilibriumStatus = "not_reached";
  if (gapPct !== null && gapPct < 5) {
    equilibriumStatus = "approaching";
  }
  if (gapPct !== null && gapPct < 1) {
    equilibriumStatus = "reached";
  }

  return {
    mechanism_id: "m012",
    scope: "v0_reference",
    as_of,
    current_supply: currentSupply.toString(),
    hard_cap: cap.toString(),
    cap_headroom: headroom.toString(),
    cap_utilization: Number(capUtil.toFixed(6)),
    total_minted: totalMinted.toString(),
    total_burned: totalBurned.toString(),
    net_supply_change: netChange.toString(),
    latest_regrowth_rate: last.regrowth_rate,
    latest_effective_multiplier: last.effective_multiplier,
    periods_evaluated: ps.length,
    equilibrium_status: equilibriumStatus,
    equilibrium_gap_pct: gapPct !== null ? Number(gapPct.toFixed(4)) : null
  };
}
