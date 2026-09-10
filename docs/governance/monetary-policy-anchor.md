# Monetary policy anchor: fixed cap, dynamic supply — and how the 2026 interim measures relate to it

**Status:** DRAFT v0.1 (2026-09-10). Proposed as a standing reference for the Tokenomics Working Group and for any contributor or agent drafting forum posts, proposal text, RFC comments or digests about $REGEN monetary policy. Where this document states a position, it is RND PBC's stated direction as recorded publicly by its CEO on the forum; it is not a decision of on-chain governance or of the Regen Foundation.

**Why it exists.** Between July and September 2026 the chain moved fast: emissions went to zero, a burn stream went live, the validator set shrank to eleven, and community-pool spends began funding validators and stewardship work. Each step was taken with stock Cosmos SDK modules and no chain upgrade. None of them *is* the long-term design this repository specifies (M012–M015), and several of them are easy to describe in ways that contradict it. This page fixes the vocabulary so that comments, proposals and documentation stay consistent while the interim phase runs.

---

## 1. The long-term design in one page

The design is the fixed-cap, dynamic-supply model suggested by Michael Zargham (BlockScience) at a February 2025 retreat, written up on the forum by Will Szal (Regen Foundation), and championed publicly by RND: a hard supply cap that acts as carrying capacity, algorithmic minting toward the cap, and fee-funded burning, so that what the network mints and burns is tied to what the network actually does.

```
S[t+1] = S[t] + M[t] − B[t]
M[t]   = r · (C − S[t])                       regrowth toward the cap C; tends to 0 as S approaches C
r      = r_base · effective_multiplier · ecological_multiplier
B[t]   = Σ burn_share · fee                  burn input from value-based fee routing (M013)
```

- **Cap = carrying capacity.** Supply moves inside the cap and never above it. Changing the cap is a constitutional (Layer 4) decision.
- **Minting is regrowth, not inflation for security.** It provisions what the network values, accelerates with staking or stability commitment and, once an oracle exists, with verified ecological signal.
- **Burning is activity-linked.** Fees on credit issuance, transfer, retirement and trade (M013) feed the burn; validators are paid from the fee stream (M014); contribution rewards (M015) come from the community share.
- **Equilibrium** is where mint ≈ burn over many periods; shocks return the system to a dynamic state.

The specification is `mechanisms/m012-fixed-cap-dynamic-supply/SPEC.md` (draft), with the governance sequence in `docs/governance/economic-reboot-proposals.md` (M013 → M014 and M012 → M015). Its Appendix B leaves five questions to the working group: the cap value, the ecological oracle, the period length, destroy-versus-reserve for burned tokens, and which multiplier replaces staking after PoA.

**Sequencing principle (RND, working group, February 2026).** Be parsimonious about the consensus core — the transformation from proof-of-stake reward tokenomics to fee-based distribution and coordination of capital — and keep a separate experimental layer for cap size, burn function, minting details and further token utility. Move "in unison" with the PoA transition, incrementally and empirically validated.

Public record: [Fixed Cap, Dynamic Supply (forum 34)](https://forum.regen.network/t/fixed-cap-dynamic-supply/34) · [RND's response and the equation (forum 34, posts 2–3)](https://forum.regen.network/t/fixed-cap-dynamic-supply/34/2) · [Anchoring Ethical Capital Formation (Medium, 2025-02-25)](https://medium.com/@gregorylandua/anchoring-ethical-capital-formation-the-case-for-a-fixed-cap-dynamic-supply-in-regen-tokenomics-a8d2e0d1719d) · [Economic Reboot Roadmap, WS2 Monetary Policy (forum 567)](https://forum.regen.network/t/regen-economic-reboot-roadmap-v0-1/567) · [PoA Consensus RFC (forum 70)](https://forum.regen.network/t/regen-network-proof-of-authority-consensus-rfc/70).

---

## 2. What is on chain now

Verified 2026-09-10 14:37–14:43 UTC against the public LCD. Re-verify before quoting: every path below is relative to any Regen LCD endpoint (for example `https://lcd-regen.keplr.app`).

| Item | State | Verify at |
|---|---|---|
| Emissions | `inflation_min = inflation_max = inflation_rate_change = 0` | `/cosmos/mint/v1beta1/params`, `/cosmos/mint/v1beta1/inflation` |
| Total supply | ~239.83M REGEN, fixed since proposal #74 | `/cosmos/bank/v1beta1/supply/by_denom?denom=uregen` |
| Bonded validators | 11 (proposal #73); bonded ≈ 87.13M; quorum (40%) ≈ 34.85M | `/cosmos/staking/v1beta1/params`, `/cosmos/staking/v1beta1/pool` |
| Community pool | ~5.04M REGEN | `/cosmos/protocolpool/v1/community_pool` |
| Burn stream | one continuous fund: 15% of community-pool inflow to the keyless address `regen1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvptr3e`, no expiry (proposal #66) | `/cosmos/protocolpool/v1/continuous_funds` |
| Live proposals | #76 (LiquidityDAO 867,000 REGEN), #77 (100,000 REGEN VSH stewardship tranche), #78 (396,000 REGEN validator floor, six months) — all past quorum at the time of reading | `/cosmos/gov/v1/proposals/<id>`, `/cosmos/gov/v1/proposals/<id>/tally` |

Passed measures that define the interim phase: #64 (inflation 10% → 7%, July), #66 (burn stream, July), #72 (`blocks_per_year` corrected), #73 (`max_validators` 11, August), #74 (emissions to zero, September), #75 (VSH credit type, September).

---

## 3. How the interim measures map to the design

| On chain | Bridges toward | What it is | What it is not |
|---|---|---|---|
| #74 emissions to zero — reversible by a single `x/mint` parameter change, as its own text says | M012's mint side, in a degenerate state: supply sits at the (undecided) cap with zero regrowth | the design's prediction that minting tends to zero at the cap, reached early by parameter | **the fixed cap.** The cap value is undecided (spec OQ-M012-1); today's supply (~239.8M) is above the spec's 221M placeholder, so the placeholder cannot be cited as decided |
| #66 15% of community-pool inflow to a keyless address | M013 Phase 1 burn pool, using stock `x/protocolpool` | continuous, governance-dialled removal of REGEN from circulation | a `total_supply` burn: the proposal's own text says supply accounting changes only with the Phase 2 `x/feerouter` + `x/supply` upgrade. Since #74 the pool's inflow is fees only, so the stream is small |
| #73 eleven validators | M014 "Plan A" via stock staking parameters | a smaller, cheaper-to-fund set | the `x/authority` module; composition and term rules are still to come |
| #78 396,000 REGEN to eleven validators over six months | the validator share of M013/M014 | a bounded, expiring operating floor while fee routing does not yet pay validators | a solution to validator economics |
| #77 100,000 REGEN for verified stewardship hours | M015 contribution rewards | receipts before payment; a first tranche | a standing program; the independence of the acceptor is an open question |
| #76 867,000 REGEN to LiquidityDAO | liquidity provisioning (experimental layer) | a resubmission of a failed earlier proposal | monetary policy |
| Community pool ~5.04M REGEN | the stock every interim spend draws down | bridge financing with sunsets | a replenishing fund — inflow is near zero until fee routing exists |

**Reading.** The chain has walked the consensus core with stock modules: inflation off, a burn stream on, a small curated set. The experimental layer — cap value, mint rule, fee schedule, oracle — is still open and belongs in the RFC process ([PR #107](https://github.com/regen-network/agentic-tokenomics/pull/107) proposes the process), not in comment threads.

---

## 4. Consistency rules for public text

Run this list before a forum post, a proposal description, an RFC review comment or a digest leaves a draft.

1. **State the direction.** The text can be read as consistent with fixed cap + dynamic supply; if it cannot, say where it conflicts and route the question to an RFC.
2. **Do not cite a settled cap.** 221M is the specification's February 2026 placeholder and is below current supply. Re-basing it (above supply, or burning down to it) is spec question OQ-M012-1.
3. **State burns at their honest scope.** The #66 stream removes REGEN from circulation; total-supply accounting arrives with the Phase 2 upgrade. Do not attach magnitude claims without the current inflow figure.
4. **Emissions-zero is a bridge**, explicitly reversible, and predicted by the design. Do not propose re-enabling inflation to fund anything; in the design, minting returns only under M012 rules.
5. **Community-pool spends are drawdowns** of a stock that does not replenish until fee routing exists: bounded, expiring, receipt-bearing, and named for what they stand in for.
6. **Validators are paid from fees in the design**, not from emission. Any pool-funded floor is a bridge; the size and composition of the set is an M014 question for governance.
7. **Use the sequencing language:** consensus core versus experimental layer; in unison with PoA; incremental and empirically validated. Do not restate the timelines in older documents as commitments.
8. **Attribute correctly.** Design suggested by Michael Zargham (BlockScience); forum write-up by Will Szal (Regen Foundation); the regrowth equation is Zargham's; the "best of both worlds" phrasing is from the forum write-up.
9. **Re-read the chain and time-stamp figures.** Quote *marginal* vote weight (stake behind validators that have not voted), not total stake, when discussing whether a vote changes an outcome.
10. **Mark agent-drafted text.** Text drafted with an agent and posted by a person says so.
11. **Do not claim** that M012 is decided, that the design is unique, or any position of the Regen Foundation board.
12. **When a live item conflicts** — re-enabling inflation, a cap below supply with no burn path, a pool spend without a sunset — comment on the conflict against this anchor and ask for the RFC route rather than a bare objection.

---

## 5. Open questions this anchor does not settle

- **Cap value (OQ-M012-1).** Supply is ~239.8M against a 221M placeholder. Options on the table since 2025 include a cap at current supply, a cap in the low 300Ms so that regrowth headroom exists, or burning down to a lower cap. This is the first question the RFC process should take.
- **Burn accounting (OQ-M012-4).** Circulation removal now; supply-metric reduction requires the `x/supply` work.
- **Ecological multiplier oracle (OQ-M012-2).** Disabled in v0 by design.
- **Period length (OQ-M012-3)** and **post-PoA multiplier (OQ-M012-5).**
- **Sequencing with the PoA upgrade path.** The interim used stock parameters; the module-level M014 remains specified but not deployed.

---

## 6. Statements in this repository that this anchor supersedes when they conflict

- `README.md` on-chain context: supply and community-pool figures predate the interim measures (refreshed alongside this document).
- `docs/GLOSSARY.md`, `docs/governance/economic-reboot-proposals.md` (Proposal 3), `docs/integrators/m012.md`, `simulations/cadcad/README.md`: the 221M cap and ~224M starting supply are February 2026 placeholders. The "cap slightly below supply, burn-down in months" narrative in Proposal 3 no longer holds arithmetically at ~239.8M supply with a fee-only inflow; treat those numbers as scenario inputs, not decisions.
- Any stated proposal calendar (Q2–Q4 2026) is illustrative.

Changes to the specification itself go through the RFC process, not through this page.

---

## 7. Provenance and changelog

- v0.1 (2026-09-10): first draft. Chain figures read from the public LCD at 14:37–14:43 UTC; forum and proposal texts read the same day. Drafted with an agent; reviewed and published by a person.
