# m012 — Fixed Cap Dynamic Supply (SPEC)

## 0. Header
- **ID:** m012
- **Name:** Fixed Cap Dynamic Supply
- **Status:** draft
- **Owner:** (unset)
- **Last updated:** 2026-02-18
- **Scope:** Protocol-level supply management; replaces inflationary PoS minting with hard-capped, algorithmically managed mint/burn

## 1. Problem
The current Regen Network supply model is inflationary proof-of-stake: validators are compensated through continuous token minting, which dilutes holders and provides no upper bound on supply. This creates long-term policy uncertainty and misaligns token economics with ecological outcomes. The network needs a supply model that provides both **long-term monetary policy certainty** (like gold's fixed supply) and **adaptive flexibility** (like managed fiat), tied to real ecological activity.

## 2. Target actor and action
- **Actors:** the Network (protocol-level supply controller), Fee Payers (source of burn input via M013), Validators (receive compensation from fee revenue, not inflation), Governance (token holders adjusting supply parameters).
- **Action being evaluated (one action):** per-period **algorithmic supply adjustment** via mint (regrowth) and burn (fee consumption).
- **Event source:** epoch-level supply computation executed by the `x/supply` module (or parameter update to `x/mint`).

## 3. Signal definition
- **Signal name:** Supply State
- **Unit:** uregen (1 REGEN = 1,000,000 uregen)
- **Directionality:** supply trends toward dynamic equilibrium where minting equals burning
- **Granularity:** per-period (1 epoch, ~7 days)
- **Key outputs:** current supply, minted amount, burned amount, regrowth rate, effective multiplier, cap headroom

## 4. Evidence inputs

| Input | Source | Fields | Validity rules | Anti-spoof assumptions | Refresh cadence |
|---|---|---|---|---|---|
| Current supply | Chain state (`x/supply` or `x/bank`) | `current_supply` (uregen) | Must be non-negative and <= hard_cap | Read from chain state (not self-reported) | Per period |
| Staked amount | Chain staking module | `staked_amount` (uregen) | Must be non-negative and <= current_supply | Read from chain state | Per period |
| Burn amount | M013 fee routing | `burn_amount` (uregen) | Sum of `burn_share * fee` for all transactions in period | Computed from on-chain fee events | Per period |
| Stability commitments | M015 (via M014) | `stability_committed` (uregen) | Non-negative; only used when M014 state != INACTIVE | Read from M015 module state | Per period |
| M014 state | M014 module | `m014_state` enum | One of: INACTIVE, TRANSITION, ACTIVE, EQUILIBRIUM | Read from M014 module state | Per period |
| Ecological metric | External oracle (v1) | `delta_co2` (ppm) | Non-negative; v0 uses 1.0 (disabled) | Oracle integration required; v0 disabled | Per period (when enabled) |

## 5. Scoring function

### 5.1 Supply algorithm

```
S[t+1] = S[t] + M[t] - B[t]

where:
  S[t]  = circulating supply at period t (uregen)
  M[t]  = tokens minted in period t (regrowth) (uregen)
  B[t]  = tokens burned in period t (from fees via M013) (uregen)
  C     = hard cap (221,000,000,000,000 uregen = 221,000,000 REGEN)
```

### 5.2 Minting (regrowth)

```
M[t] = r * (C - S[t])

where r = regrowth rate, dynamically adjusted:
  r = r_base * effective_multiplier * ecological_multiplier

  r_base = 0.02 (2% base regrowth rate per period)
```

### 5.3 Effective multiplier (phase-gated)

The effective multiplier depends on the M014 module state:

```
staking_multiplier = clamp(1 + (S_staked / S_total), 1.0, 2.0)

stability_multiplier = clamp(1 + (S_stability_committed / S_total), 1.0, 2.0)

Phase-gated behavior:
  - if m014.state == INACTIVE:     effective_multiplier = staking_multiplier
  - if m014.state == TRANSITION:   effective_multiplier = max(staking, stability)
  - if m014.state in {ACTIVE, EQUILIBRIUM}: effective_multiplier = stability_multiplier
```

The `max()` selection during TRANSITION prevents a regrowth discontinuity as `S_staked` declines during the unbonding period while `S_stability_committed` ramps up.

### 5.4 Ecological multiplier

```
ecological_multiplier = max(0, 1 - (delta_co2 / reference_value))
  range: [0.0, 1.0+]
  reference_value = 50 ppm (configurable)
```

- Improving ecological metrics increase regrowth; worsening metrics slow regrowth toward zero.
- Floored at 0 to prevent negative minting. Supply contraction occurs exclusively through burning.
- **v0:** ecological_multiplier = 1.0 (disabled) until reliable oracle integration exists.

### 5.5 Burning

```
B[t] = sum of burn_share * fee for all fee-generating transactions in period t
```

Burn routing is defined in M013. This module receives the aggregated burn amount per period.

### 5.6 Key properties

- As `S[t]` approaches `C`, minting naturally decelerates toward zero.
- Supply can only increase through minting; supply can only decrease through burning.
- System reaches equilibrium when `M[t] = B[t]`.

### 5.7 Normalization

All supply values are in uregen (1 REGEN = 1,000,000 uregen). The regrowth rate `r` is a dimensionless multiplier applied to the headroom `(C - S[t])`.

### 5.8 Controls

- `hard_cap`: absolute upper bound on supply (Layer 4 constitutional governance).
- `r_base`: base regrowth rate, bounded to `[0, 0.10]` (Layer 3).
- `ecological_multiplier_enabled`: boolean toggle (Layer 3).
- `min_burn_rate`: safety floor for burn rate (Layer 2).
- `period_length`: epoch cadence for mint/burn cycles (Layer 2).

## 6. State machine

```
States: {INFLATIONARY, TRANSITION, DYNAMIC, EQUILIBRIUM}

INFLATIONARY -> TRANSITION
  trigger: governance.approve(m012_activation_proposal)
  guard: M013 fee routing active, M014 PoA active
  action: set hard_cap, disable inflation module, enable mint/burn module

TRANSITION -> DYNAMIC
  trigger: first_burn_period_complete AND validators_compensated
  guard: fee_revenue > 0, burn_executed
  action: begin algorithmic mint/burn cycles

DYNAMIC -> EQUILIBRIUM
  trigger: abs(M[t] - B[t]) < threshold for N consecutive periods
  guard: N >= 12 (12 months of near-balance)
  action: log equilibrium_reached, reduce monitoring frequency
  note: equilibrium is not permanent -- external shocks can return to DYNAMIC

EQUILIBRIUM -> DYNAMIC
  trigger: abs(M[t] - B[t]) >= threshold
  guard: deviation persists for >= 1 period
  action: resume full monitoring frequency
```

## 7. Economic linkage

This mechanism directly manages the monetary supply of REGEN:

- **Minting** distributes new tokens into circulation as regrowth, replacing inflationary PoS rewards.
- **Burning** permanently removes tokens from circulation, funded by value-based fees (M013).
- **Validator compensation** shifts from inflation-funded to fee-revenue-funded (via M014).
- **Equilibrium** represents the steady state where network activity (fees/burns) matches regrowth, creating a self-sustaining economic loop.

**Carrying capacity metaphor:** the hard cap mirrors nature's carrying capacity -- an upper limit for population within an ecosystem. Supply can contract and expand within the cap but never exceed it.

## 8. On-chain vs off-chain boundary

- **On-chain:** supply state storage (`SupplyState`), per-period mint/burn execution, parameter governance, cap enforcement.
- **Off-chain (v0):** KPI computation, equilibrium monitoring, digest reporting, ecological multiplier oracle integration (when enabled).
- **Events:** `EventSupplyMint`, `EventSupplyBurn`, `EventParameterUpdate`, `EventEquilibriumReached`.
- **Storage:** `SupplyState` (current supply, cap, parameters), `MintBurnRecord` (per-period history).

## 9. Attack model

- **Cap violation:** invariant enforcement at protocol level; no transaction may cause `S[t] > C`.
- **Runaway minting:** `r_base` is bounded to `[0, 0.10]`; effective multiplier capped at 2.0; ecological multiplier capped at 1.0+. Maximum possible `r` is `0.10 * 2.0 * 1.0 = 0.20`.
- **Burn manipulation:** burn amounts are derived from M013 fee routing, which has its own anti-gaming measures. Mint and burn are computed independently (Invariant 4).
- **Oracle manipulation (v1):** ecological multiplier oracle could be gamed to inflate regrowth. Mitigated by: v0 disables ecological multiplier; v1 requires governance-approved oracle sources; floor at 0 prevents negative minting.
- **Governance capture:** `hard_cap` changes require Layer 4 (67% supermajority); `r_base` changes require Layer 3 (community deliberation). No single actor can unilaterally change supply parameters.
- **Supply oscillation:** the regrowth formula inherently dampens oscillation -- as supply approaches cap, minting decelerates; as supply drops, minting accelerates.

## 10. Integration points

- **M013 (Value-Based Fee Routing):** provides the per-period burn amount `B[t]`. M012 consumes the aggregated burn total.
- **M014 (Authority Validator Governance):** determines the phase gate for effective multiplier selection. M012 reads `m014.state` to choose between staking and stability multipliers.
- **M015 (Contribution Rewards):** provides `S_stability_committed` for the stability multiplier computation (post-M014 activation).
- **Staking module:** provides `S_staked` for the staking multiplier computation (pre-M014 or during transition).
- **Bank module:** provides `S_total` (current circulating supply).
- **Governance module:** parameter updates for `hard_cap`, `r_base`, `ecological_multiplier_enabled`, etc.
- **KOI MCP (knowledge):** digest publication, equilibrium monitoring reports.
- **Ledger MCP (chain data):** supply state queries, mint/burn event history.

## 11. Acceptance tests

**Supply algorithm:**
1) **Basic mint/burn:** given a supply state with known staking ratio, compute `M[t]` and `B[t]`; verify `S[t+1] = S[t] + M[t] - B[t]`.
2) **Cap enforcement:** if `S[t] + M[t] - B[t] > C`, then `S[t+1] = C` (cap inviolability).
3) **Non-negative supply:** if `B[t] > S[t] + M[t]`, then `S[t+1] = 0` (non-negative supply).
4) **Staking multiplier range:** with 0% staked, multiplier = 1.0; with 100% staked, multiplier = 2.0; with 50% staked, multiplier = 1.5.
5) **Near-cap deceleration:** when supply is at 99% of cap, minted amount is very small (1% of headroom * rate).

**Phase gating (M014 integration):**
6) **INACTIVE phase:** only staking_multiplier is used regardless of stability_committed.
7) **TRANSITION phase:** effective_multiplier = max(staking, stability); larger value wins.
8) **ACTIVE phase:** only stability_multiplier is used; staking_multiplier is ignored.

**Ecological multiplier:**
9) **Disabled (v0):** ecological_multiplier = 1.0 when disabled; no effect on regrowth rate.
10) **Enabled (v1):** with delta_co2 = 25 ppm and reference = 50 ppm, ecological_multiplier = 0.5.
11) **Floor:** with delta_co2 = 100 ppm and reference = 50 ppm, ecological_multiplier = 0 (not negative).

**State machine:**
12) **INFLATIONARY to TRANSITION:** requires M013 active and M014 active.
13) **TRANSITION to DYNAMIC:** requires first burn period complete and fee revenue > 0.
14) **DYNAMIC to EQUILIBRIUM:** requires 12 consecutive periods of near-balance.
15) **EQUILIBRIUM to DYNAMIC:** external shock returns to DYNAMIC state.

**Security invariants:**
16) **Cap inviolability:** `S[t] <= hard_cap` at all times.
17) **Non-negative supply:** `S[t] >= 0` at all times.
18) **Monotonic cap:** `hard_cap` cannot be changed without Layer 4 governance.
19) **Mint-burn independence:** minting and burning are computed independently.
20) **Parameter bound safety:** `r_base` is bounded to `[0, 0.10]`.

## 12. Rollout plan

### v0 checklist
- Define supply parameters (`hard_cap`, `r_base`, `period_length`).
- Implement reference computation for `computeSupplyPeriod` (this spec).
- Disable ecological multiplier (set to 1.0).
- Implement KPI computation and digest reporting.
- Validate with test vectors covering normal, near-cap, high-staking, and large-burn scenarios.
- Integrate with M013 burn routing (consume aggregated burn amounts).
- Integrate with M014 phase gate (read `m014.state` for multiplier selection).

### Optional v1 outline (non-binding)
- Deploy `x/supply` module or migrate `x/mint` parameters on Regen Ledger.
- Enable ecological multiplier with governance-approved oracle source.
- Implement on-chain equilibrium detection and state transitions.
- Add governance proposal types for parameter updates.
- Implement per-block granularity (EIP-1559 style) if WG resolves OQ-M012-3.

## 13. Governance parameters

| Parameter | Initial Value | Governance Authority | Rationale |
|-----------|--------------|---------------------|-----------|
| `hard_cap` | 221,000,000 REGEN (221,000,000,000,000 uregen) | Layer 4 (Constitutional) | Fundamental monetary policy; requires 67% supermajority |
| `base_regrowth_rate` | 0.02 (2%) | Layer 3 (Human-in-Loop) | Significant economic impact; needs community deliberation |
| `staking_multiplier_enabled` | true | Layer 3 | Affects incentive structure |
| `ecological_multiplier_enabled` | false (v0) | Layer 3 | Requires oracle dependency; enable when ready |
| `ecological_reference_value` | 50 ppm | Layer 3 | Ecological sensitivity parameter |
| `min_burn_rate` | 0 | Layer 2 (Agentic + Oversight) | Safety floor; can be adjusted operationally |
| `period_length` | 1 epoch (~7 days) | Layer 2 | Operational cadence |

---

## Appendix A -- Security Invariants

1. **Cap Inviolability**: `S[t] <= hard_cap` at all times; no transaction may cause supply to exceed cap.
2. **Non-Negative Supply**: `S[t] >= 0`; burn cannot reduce supply below zero.
3. **Monotonic Cap**: `hard_cap` can only be changed via Layer 4 constitutional governance (67% supermajority).
4. **Mint-Burn Independence**: Minting and burning are computed independently; neither can block the other.
5. **Parameter Bound Safety**: `r_base` is in `[0, 0.10]`; regrowth rate bounded to prevent runaway minting.

## Appendix B -- Open Questions (for WG Resolution)

> **OQ-M012-1**: The exact hard cap value. Token-economics-synthesis says "~221M" based on current total supply (~224M). Should the cap be set at current total supply, slightly below (to create immediate scarcity), or at a round number?

> **OQ-M012-2**: The ecological multiplier oracle. What data source provides delta_co2 or equivalent ecological metric? Is this sourced from on-chain attestation data (M008) or from an external oracle? The v0 spec disables this until resolved.

> **OQ-M012-3**: Period length for mint/burn cycles. Is per-epoch (weekly) the right cadence, or should it be per-block (like EIP-1559) for finer granularity?

> **OQ-M012-4**: Should burned tokens be permanently destroyed or sent to a reserve pool that can be re-minted under governance control?

> **OQ-M012-5**: Should the staking_multiplier be replaced by a stability_multiplier (from M015 commitments) or a validator_participation_multiplier (from M014 active set health)?

## Appendix C -- Source anchors

- `phase-2/2.6-economic-reboot-mechanisms.md` section "M012 -- Fixed Cap Dynamic Supply"
- [Fixed Cap, Dynamic Supply (forum/34)](https://forum.regen.network/t/fixed-cap-dynamic-supply/34)
- [Economic Reboot Roadmap v0.1 (forum/567)](https://forum.regen.network/t/regen-economic-reboot-roadmap-v0-1/567)
- [Token Economics Synthesis](../docs/economics/token-economics-synthesis.md)
- Blockscience carrying capacity model (referenced in design philosophy)
- Ethereum EIP-1559 burn mechanics (referenced in design philosophy)
