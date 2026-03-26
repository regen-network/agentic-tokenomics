# Proposal: Hybrid Fee Collection with Buy-and-Burn

**Status**: Draft — for Working Group review
**Resolves**: OQ-M013-3 (fee denomination), OQ-M013-5 (burn pool rationale)
**Author**: ToknWrks
**Date**: 2026-03-26

---

## Summary

This proposal resolves OQ-M013-3 and OQ-M013-5 jointly. They are not independent questions — the case for or against burn depends entirely on *what is being burned*. Burning tokens already in circulation is a supply game. Burning tokens purchased with real fee revenue is value accrual.

**Recommended position:**

- **OQ-M013-3**: Hybrid collection — fees collected in native denomination (USDC/stablecoins), auto-converted to REGEN via Osmosis DEX, then burned
- **OQ-M013-5**: Maintain burn at 30-40%, implemented as buy-and-burn not direct burn

---

## The Core Argument

Most credit marketplace transactions settle in **USDC**, not REGEN. This is the current reality and is unlikely to change soon — stablecoins reduce friction for credit buyers.

This means the burn pool debate is not really about "should we burn REGEN" — it's about **what do we do with the USDC fees we collect**. The options are:

1. Hold USDC in treasury
2. Distribute USDC to stakeholders
3. Use USDC to buy REGEN on open market, then burn it

Option 3 — **buy-and-burn** — is the only option that creates a direct, mechanical link between ecological credit market activity and REGEN token value.

---

## Why Buy-and-Burn Satisfies Both Audiences

The OQ-M013-5 debate presents a false dichotomy between "contributor-first" and "capital formation" views. Buy-and-burn resolves it:

**For investors and token holders:**
Every dollar of credit market activity creates two compounding effects:
1. **Buy pressure** — USDC enters the REGEN market on Osmosis
2. **Supply reduction** — purchased REGEN is permanently burned

This is not a speculative supply game. It is real revenue from real ecological activity purchasing and removing REGEN from circulation. The mechanism is identical to Binance's BNB burn program, which is among the most credible token value accrual mechanisms in crypto.

**For contributors and regenerators:**
The remaining 60-70% of fees (distributed to validators, community pool, agent infrastructure) continues to fund active contributors. Nothing is taken from contributor rewards — the burn share is additive deflationary pressure on top of the existing distribution.

**The unified narrative:**

> *"Every tonne of carbon sequestered, every hectare of biodiversity restored, every credit retired — buys and burns REGEN forever. The more the planet regenerates, the scarcer REGEN becomes."*

Regeneration and token appreciation are the same transaction.

---

## Alignment with Regen Compute

Regen Compute (compute.regen.network) is driving credit retirements from AI developers. Every retirement via the API:

```
AI session ends
→ POST /api/v1/retire called
→ 0.5% retirement fee collected in USDC
→ 30-40% of fee routes to Burn Pool
→ USDC used to buy REGEN on Osmosis
→ REGEN permanently burned
→ Supply contracts
```

This creates a direct flywheel: AI compute growth → ecological credit demand → REGEN buy pressure → token appreciation → more capital into Regen ecosystem → more credit projects funded → more retirements → more burn.

---

## Proposed Fee Flow

```
Transaction fee collected (USDC)
    │
    ├── 35% → Burn Pool
    │         └── auto-convert to REGEN via Osmosis
    │               └── burn permanently (x/bank SendCoins to null address)
    │
    ├── 35% → Validator Fund (USDC or REGEN, validator's choice)
    │
    ├── 25% → Community Pool → M015 activity rewards
    │         └── distributed in REGEN (converted at claim time)
    │
    └── 5%  → Agent Infrastructure Fund (USDC for operational stability)
```

**On distribution denomination (OQ-M013-3, distribution side):**
- Burn Pool: always converted to REGEN (required for burn)
- Validator Fund: recipient choice — USDC for operational stability or REGEN for governance weight
- Community Pool / M015: converted to REGEN at claim time, with optional 5% bonus for recipients who choose REGEN over USDC
- Agent Infrastructure: USDC (agents need operational stability, not governance weight)

---

## Volume Modeling

At current REGEN price (~$0.0013) and realistic near-term volumes:

| Weekly Credit Volume | Fees Collected | Burn Pool (35%) | REGEN Bought & Burned |
|---|---|---|---|
| $1,000 | $5–$30 | $1.75–$10.50 | ~1,350–8,100 REGEN |
| $10,000 | $50–$300 | $17.50–$105 | ~13,500–81,000 REGEN |
| $100,000 | $500–$3,000 | $175–$1,050 | ~135,000–808,000 REGEN |
| $1,000,000 | $5,000–$30,000 | $1,750–$10,500 | ~1.35M–8.1M REGEN |

*Fee range reflects 0.5% (retirement) to 3% (issuance). Burn assumes full conversion via Osmosis.*

At $10K/week in credit volume — a realistic near-term target for Regen Compute — the network burns roughly **0.3–1.9% of total supply per year** (~700K–4.2M REGEN). Modest today, meaningful at scale, and directionally correct from day one.

---

## Impact on M012 (Supply Cap)

With buy-and-burn as the primary supply mechanism, M012's minting formula becomes secondary. We recommend:

- **Hard cap: 225M REGEN** (at or just above current supply — a clean, permanent ceiling)
- **Minting: minimal or zero** — supply only moves downward through burn
- **Ecological incentives: funded from Community Pool (M015)**, not new minting

This simplifies M012 significantly: the cap is a hard historical maximum, never approached again. Supply contracts as the credit market grows.

---

## Response to OQ-M013-5 Objections

**"Burn benefits passive holders over active contributors"**

The burn share (35%) is carved from fee revenue — money that didn't exist before. It does not reduce contributor rewards, which are funded from the remaining 65%. The question is not "burn vs. contributors" but "burn vs. treasury accumulation."

**"Ecological mission should rest on real impact, not tokenomic supply games"**

Agreed. That's exactly why buy-and-burn is preferable to direct REGEN burn. Real USDC revenue from real credit market activity is used to purchase REGEN. The supply reduction is a consequence of real economic activity, not a supply game.

**"Speculative interest is pragmatic for bootstrapping"**

Buy-and-burn is the most credible mechanism for aligning speculative interest with ecological outcomes. Speculators who understand the mechanism become advocates for more credit retirements — their investment thesis and the mission are identical.

---

## Implementation Notes

**DEX routing**: Osmosis is the natural venue — REGEN/USDC liquidity exists there, and the swap can be executed atomically via IBC. Slippage protection parameters should be set conservatively given current low liquidity.

**Osmosis liquidity risk**: REGEN/USDC liquidity on Osmosis is currently thin. At low weekly volumes ($1K–$10K), slippage could consume a meaningful portion of the burn value, reducing effectiveness. Mitigations:
- Set a minimum weekly accumulation threshold (e.g., $500 USDC) before executing a swap — below this, accumulate to the next epoch
- Set a maximum slippage parameter (e.g., 3%) — if the swap would exceed this, defer to next epoch
- As credit market volume grows, slippage becomes proportionally less significant
- Long-term: deeper REGEN/USDC liquidity is a natural consequence of the buy pressure this mechanism generates — the mechanism is self-reinforcing over time

**Burn address**: Cosmos SDK `x/bank` module supports sending to a burn address (`regen1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqnrums9`). Burned tokens are provably gone — verifiable on-chain.

**Frequency**: Weekly epoch aligns with M012's proposed period cadence. Accumulate fees for 7 days, execute single batch buy-and-burn. Reduces gas overhead and DEX impact vs. per-transaction burns.

**Oracle dependency**: None — this mechanism requires no price oracle. USDC is stable by definition; REGEN price is discovered at swap time on Osmosis.

---

## Recommended WG Positions

| Open Question | Recommended Resolution |
|---|---|
| OQ-M013-3 (collection) | Hybrid: collect in native denom (USDC), auto-convert burn share to REGEN via Osmosis |
| OQ-M013-3 (distribution) | Burn Pool: REGEN only. Validators: choice. Community Pool: REGEN with 5% bonus option. Agent Infra: USDC |
| OQ-M013-5 (burn share) | Option A — maintain burn at 30-40%, implemented as buy-and-burn |
| OQ-M012-1 (hard cap) | 225M — at current supply, clean permanent ceiling, no future minting needed |

---

## OPAL Coherence Self-Assessment

| Dimension | Score | Notes |
|---|---|---|
| Ontology | 0.85 | Consistent with M012/M013/M015 entity model. Adds buy-and-burn as sub-mechanism of Burn Pool. |
| Philosophy | 0.80 | Aligns ecological activity with token value. Contributor-first framing preserved via 65% non-burn distribution. |
| Architecture | 0.75 | Osmosis DEX dependency is new. Slippage/liquidity risk requires parameter governance. |
| Language | 0.90 | Resolves ambiguity in OQ-M013-3/5. Introduces "buy-and-burn" as defined term. |
| Action | 0.80 | Weekly epoch burn is reversible via governance. Cap change requires Layer 4. |
| **Composite** | **0.82** | Qualifies for expedited Layer 2 fast-track review |

---

## Next Steps

1. WG discussion in Discord #agentic-governance
2. Forum post at forum.regen.network for broader community input
3. If consensus reached: PR to update `phase-2/2.6-economic-reboot-mechanisms.md` resolving OQ-M013-3 and OQ-M013-5
4. Follow-on PR to update M012 spec with simplified supply cap (225M, burn-only)
