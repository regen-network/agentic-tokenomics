# Economic Reboot Governance Proposals

## Overview

This document provides governance-ready proposal templates for activating mechanisms M012 through M015 on Regen Ledger mainnet. Each proposal includes copy-paste-ready text, parameter justifications with risk assessments, revenue or impact projections, and community pool sustainability analysis.

These proposals translate the technical specifications in [phase-2/2.6-economic-reboot-mechanisms.md](../../phase-2/2.6-economic-reboot-mechanisms.md) into the governance artifacts required for on-chain submission. They are designed to be submitted sequentially, respecting mechanism dependencies established by the Tokenomics Working Group and formalized in Gregory's PRs (#32, #35, #36).

### Source Documents

- [Phase 2.6: Economic Reboot Mechanism Specifications](../../phase-2/2.6-economic-reboot-mechanisms.md)
- [Token Economics Synthesis](../economics/token-economics-synthesis.md)
- [Agentic Governance Roadmap](./agentic-governance-roadmap.md)
- [Economic Reboot Roadmap v0.1 (forum/567)](https://forum.regen.network/t/regen-economic-reboot-roadmap-v0-1/567)
- [Network Coordination Architecture (forum/19#67)](https://forum.regen.network/t/regen-tokenomics-wg/19/67)
- [PoA Consensus RFC (forum/70)](https://forum.regen.network/t/regen-network-proof-of-authority-consensus-rfc/70)
- [Max Semenchuk Model Comparison](https://maxsemenchuk.github.io/regen-model-comparison/)

---

## Proposal Sequence and Dependencies

Proposals must be submitted and passed in order. Each mechanism's activation depends on prior mechanisms being live and generating the data or revenue it requires.

```
PROPOSAL 1: M013 Fee Routing
  │  (no dependencies — can deploy independently)
  │
  ├──────────────────────────────────────────────────────┐
  │                                                      │
  ▼                                                      ▼
PROPOSAL 2: M014 PoA Governance                PROPOSAL 3: M012 Dynamic Supply
  │  (requires M013 for validator                  │  (requires M013 for burn input;
  │   compensation via Validator Fund)              │   requires M014 to disable PoS inflation)
  │                                                 │
  │  NOTE: M014 and M012 proposals                  │
  │  can be submitted concurrently                  │
  │  once M013 is active, but M012                  │
  │  activation is gated on M014                    │
  │  reaching TRANSITION state.                     │
  │                                                 │
  └──────────────────────┬──────────────────────────┘
                         │
                         ▼
              PROPOSAL 4: M015 Contribution Rewards
                │  (requires M013 Community Pool inflow
                │   AND M014 active — staking rewards replaced)
                │
                ▼
              PROPOSAL 5: Agent Infrastructure Spend (Optional)
                (community pool spend if Model A agent fund adopted)
```

### Estimated Timeline

| Proposal | Earliest Submission | Deposit Required | Voting Period |
|----------|-------------------|------------------|---------------|
| Proposal 1 (M013) | Q2 2026 | 500 REGEN | 14 days |
| Proposal 2 (M014) | Q3 2026 (after M013 active) | 500 REGEN | 14 days |
| Proposal 3 (M012) | Q3 2026 (after M013 active) | 500 REGEN | 14 days |
| Proposal 4 (M015) | Q4 2026 (after M013 + M014 active) | 500 REGEN | 14 days |
| Proposal 5 (Agent Infra) | Q1 2027 (after M013 active, if Model A adopted) | 500 REGEN | 14 days |

> **Note**: Deposit amounts are based on current Regen Network governance parameters. Verify the current `min_deposit` via `regen query gov params` before submission.

---

## PROPOSAL 1: Enable M013 Fee Routing

### Proposal Text (Copy-Paste Ready)

> **Title**: Enable Value-Based Fee Routing for Ecological Credit Transactions (M013)
>
> **Type**: Software Upgrade Proposal (binary upgrade deploying `x/feerouter` module), followed by Parameter Change Proposal (setting initial fee rates and distribution shares)
>
> **Deposit**: 500 REGEN
>
> **Description**:
>
> This proposal activates the Value-Based Fee Routing mechanism (M013) on Regen Ledger mainnet. M013 replaces the current flat gas fee model for ecological credit transactions with value-proportional fees, routing revenue to four purpose-specific pools: Burn Pool, Validator Fund, Community Pool, and Agent Infrastructure Fund.
>
> **What changes**:
> - Credit issuance (`MsgCreateBatch`), transfer (`MsgSend`), retirement (`MsgRetire`), and marketplace trades (`MsgBuySellOrder`) will incur percentage-based fees proportional to credit value.
> - Non-credit transactions (standard Cosmos SDK messages) remain on flat gas fees — unchanged.
> - Fee revenue is split across four destination pools according to governance-set distribution shares.
> - A 90-day TRANSITION phase runs dual fees (flat gas + value-based) before full activation, allowing UI/tooling to adapt.
>
> **Why this matters**:
> - The current flat gas model generates negligible protocol revenue (~0.01 REGEN per transaction regardless of value). A $10 credit and a $10,000 credit pay the same fee, disconnecting protocol economics from the ecological value the network facilitates.
> - M013 creates a sustainable funding model for validator compensation, community spending, supply management, and agent infrastructure — without inflationary token emission.
> - This is the foundational mechanism for the Economic Reboot. M012 (Dynamic Supply), M014 (PoA Governance), and M015 (Contribution Rewards) all depend on M013 fee revenue.
>
> **Impact**:
> - Credit transaction costs increase from flat gas to 0.1%–2% of value (see parameter table below).
> - Protocol begins accumulating revenue in Validator Fund, Community Pool, and Burn Pool.
> - No impact on non-credit transactions.
>
> **Risk mitigation**: The 90-day transition phase provides a window to observe marketplace behavior. If fee rates are set too high, a follow-up parameter change proposal can adjust rates without a binary upgrade.

### Parameter Justification Table

| Parameter | Proposed Value | On-Chain Representation | Justification | Risk | Alternative Considered |
|-----------|---------------|------------------------|---------------|------|----------------------|
| `issuance_fee_rate` | 2% | 200 bps | Primary revenue source. Below industry standard 3–5% registry fees (Verra charges 3.5%, Gold Standard charges 2–4%). Balances revenue generation with low friction for credit originators. | May reduce issuance volume if originators are price-sensitive at low margins. | 1% (lower revenue, minimal friction) or 3% (higher revenue, closer to industry norm). Starting at 2% allows upward adjustment if tolerated. |
| `transfer_fee_rate` | 0.1% | 10 bps | Minimal friction for internal transfers between wallets. Transfers are often operational (e.g., custodian-to-marketplace) rather than value-generating events. | Negligible revenue contribution; could arguably be 0%. | 0% (zero friction, zero revenue) or 0.5% (meaningful revenue but may discourage routine transfers). |
| `retirement_fee_rate` | 0.5% | 50 bps | Exit fee capturing value at the point of ecological impact. Retirement is the terminal, highest-value event in the credit lifecycle. Rate kept moderate because retirement is the desired end state — we want to encourage, not penalize, it. | Could discourage retirement if buyers are cost-sensitive at the margin. | 1% (parity with trade fee) or 0.25% (near-zero friction). 0.5% signals that the protocol values the impact event without over-taxing it. |
| `trade_fee_rate` | 1% | 100 bps | Standard marketplace fee rate. Comparable to major exchanges (Coinbase 0.6%, Uniswap 0.3%, traditional commodity exchanges 1–2%). | Fee avoidance via off-chain or OTC trading. | 0.5% (competitive with DeFi) or 2% (closer to traditional commodity markets). 1% balances competitiveness with revenue. |
| `min_fee` | 1 REGEN | 1,000,000 uregen | Prevents zero-fee transactions on extremely low-value credits. At current REGEN price (~$0.03), this is approximately $0.03 — well below any reasonable transaction friction threshold. | Floor may need adjustment if REGEN price changes significantly. | 0.5 REGEN (lower floor) or 5 REGEN (anti-dust). 1 REGEN is a round number that provides meaningful minimum without blocking micro-transactions. |
| `max_fee_rate_bps` | 1,000 (10%) | 1,000 bps | Safety cap preventing governance attack that sets fee rates to confiscatory levels. No legitimate use case exceeds 10%. | Cap may constrain emergency response options. | 500 bps (5%) is arguably sufficient; 1,000 bps provides more headroom for unusual circumstances. |
| `burn_share` | 28% | 2,800 bps | Compromise between Model A (30%) and Model B (25–35%). Provides meaningful deflationary pressure for M012 while preserving more revenue for active contributors. | If burn is too aggressive, less revenue reaches Community Pool and validators. | Model A pure (30%) or reduced burn (10%) with remainder to community. 28% is a starting point; adjustable after 90-day observation period. |
| `validator_share` | 25% | 2,500 bps | Compromise between Model A (40%) and Model B (15–25%). Sufficient to meaningfully compensate a 15–21 validator set. With $50K/month total fees, this yields ~$12.5K/month for validators — roughly $600–800/month per validator. | If too low, validators remain unprofitable and may exit. | Model A pure (40%) provides better validator economics but reduces community/burn pools. Starting at 25% with quarterly review. |
| `community_share` | 45% | 4,500 bps | Compromise between Model A (25%) and Model B (50–60%). Prioritizes the Community Pool as the primary distribution channel, funding M015 activity rewards and governance-directed spending. | Lower than Model B's 50–60% may not fully fund M015 stability tier commitments at scale. | Model B pure (55%) with reduced burn and validator shares. 45% is a meaningful community-first allocation while sustaining the other pools. |
| `agent_share` | 2% | 200 bps | Minimal initial allocation for agent infrastructure. If the community decides agents should be funded through Community Pool proposals instead (Model B approach), this can be set to 0% via parameter change. | May be insufficient if agent operations scale faster than expected. | 5% (Model A original) or 0% (Model B, agents funded via proposals). 2% provides bootstrap funding without over-committing to an unproven cost center. |

> **Note on distribution share compromise**: The proposed {28%, 25%, 45%, 2%} split represents a middle ground between Model A {30%, 40%, 25%, 5%} and Model B {30%, 20%, 50%, 0%}. The sum equals 100%. The Tokenomics WG should evaluate this starting point and adjust via parameter change proposal after 90 days of data.

### Revenue Projection

Based on available on-chain data and conservative estimates:

| Metric | Conservative | Moderate | Optimistic |
|--------|-------------|----------|------------|
| Monthly credit issuance volume | $100K | $500K | $2M |
| Monthly credit trade volume | $200K | $1M | $5M |
| Monthly credit retirement volume | $150K | $750K | $3M |
| Monthly credit transfer volume | $50K | $250K | $1M |
| **Monthly issuance fee revenue** (2%) | $2,000 | $10,000 | $40,000 |
| **Monthly trade fee revenue** (1%) | $2,000 | $10,000 | $50,000 |
| **Monthly retirement fee revenue** (0.5%) | $750 | $3,750 | $15,000 |
| **Monthly transfer fee revenue** (0.1%) | $50 | $250 | $1,000 |
| **Total monthly fee revenue** | **$4,800** | **$24,000** | **$106,000** |

**Projected pool balances after 6 months** (moderate scenario, $24K/month):

| Pool | Monthly Inflow | 6-Month Balance |
|------|---------------|-----------------|
| Burn Pool (28%) | $6,720 | $40,320 (burned — reduces circulating supply) |
| Validator Fund (25%) | $6,000 | $36,000 ($1,714–$2,400/validator over 6 months) |
| Community Pool (45%) | $10,800 | $64,800 (available for M015 rewards + governance spending) |
| Agent Infra (2%) | $480 | $2,880 |
| **Total** | **$24,000** | **$144,000** |

### Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Reduced marketplace activity due to new fees | Medium | Medium | Start with TRANSITION phase (90 days dual-fee). If volume drops >20%, submit parameter change to reduce rates. Monitor weekly via AGENT-003. |
| Fee avoidance via off-chain/OTC trading | Medium | Medium | Fees are competitive with traditional registries (2–5%). Off-chain trades lose Regen Network provenance and verifiability — a meaningful cost for ESG-regulated buyers. |
| Credit value estimation inaccuracy (for non-marketplace transactions) | Medium | High | Use most recent marketplace price per credit class as reference. If no marketplace price exists, fall back to governance-set reference price. Flag as a known limitation in the upgrade documentation. |
| Fee denomination complexity (multi-denom collection) | Low | Medium | v1 collects fees in REGEN only. Fee payer must hold REGEN. Multi-denom collection deferred to v2. |
| Distribution share imbalance discovered post-launch | Low | High | Expected. Shares are governance parameters adjustable via standard parameter change proposal. Plan a 90-day review proposal. |

---

## PROPOSAL 2: Activate M014 PoA Governance

### Proposal Text (Copy-Paste Ready)

> **Title**: Activate Proof-of-Authority Validator Governance (M014)
>
> **Type**: Software Upgrade Proposal (binary upgrade deploying `x/authority` module and modifying `x/staking` parameters)
>
> **Deposit**: 500 REGEN
>
> **Description**:
>
> This proposal activates the Authority Validator Governance mechanism (M014) on Regen Ledger mainnet, beginning the transition from Proof of Stake to Proof of Authority consensus.
>
> **What changes**:
> - A new `x/authority` module is deployed for managing a curated validator set of 15–21 validators.
> - Validators are organized into three composition categories: Infrastructure Builders, Trusted ReFi Partners, and Ecological Data Stewards (minimum 5 per category).
> - Validator compensation shifts from inflationary staking rewards to fee-based compensation via the M013 Validator Fund.
> - A 12-month term structure with 99.5% uptime requirements and 30-day probation periods is enforced.
> - The transition proceeds in three phases: (1) curate existing set, (2) enable authority module with PoS coexistence, (3) disable PoS inflation.
>
> **Why this matters**:
> - The current validator set is unstable, sometimes dropping below 21 active validators. All validators operate at a loss, participating for mission alignment rather than economic sustainability.
> - PoS creates structural mismatches: security cost drops with token price, passive holders receive equal rewards as contributors, and security is funded by inflation that dilutes all holders.
> - PoA replaces this with a compensated, accountable, mission-aligned validator set whose authority derives from demonstrated ecological contribution.
>
> **Impact**:
> - The active validator set transitions from open staking to curated authority selection.
> - Staking inflation is phased out over the transition window (estimated 6 months).
> - Currently staked REGEN must be unbonded during the transition (see Staker Impact Analysis below).
> - Validator compensation funded by M013 fee revenue instead of new token emission.
>
> **Prerequisites**: Proposal 1 (M013 Fee Routing) must be active and generating Validator Fund revenue.

### Parameter Justification Table

| Parameter | Proposed Value | On-Chain Representation | Justification | Risk | Alternative Considered |
|-----------|---------------|------------------------|---------------|------|----------------------|
| `max_validators` | 21 | 21 | Sufficient for Byzantine fault tolerance (BFT requires >3f+1; with f=6, minimum is 19). 21 is an established Tendermint set size used by Cosmos Hub and others. Manageable governance overhead. | Larger sets provide more decentralization but increase coordination cost. | 15 (minimal viable), 25 (more representation). 21 balances fault tolerance with governance efficiency. |
| `min_validators` | 15 | 15 | Safety minimum ensuring BFT (with f=4, minimum is 13; 15 provides margin). Below this threshold, emergency governance escalation triggers. | If the qualified applicant pool is thin, maintaining 15 may require relaxed criteria. | 13 (strict BFT minimum) or 18 (higher safety margin). 15 provides reasonable buffer without over-constraining the qualified pool. |
| `term_length` | 365 days | 31,536,000 seconds | Annual accountability cycle. Long enough for validators to invest in infrastructure and relationships; short enough for regular re-evaluation. Aligns with typical organizational planning cycles. | Annual terms may be too infrequent if a validator underperforms. Mitigated by probation mechanism. | 6 months (more accountability, more overhead) or 24 months (less overhead, less responsiveness). 12 months is the conventional standard. |
| `min_uptime` | 99.5% | 9,950 bps | Translates to approximately 44 hours of downtime per year. Reasonable for compensated validators running professional infrastructure. Below typical enterprise SLA (99.9%) but above hobbyist levels. | May exclude smaller organizations without dedicated DevOps. | 99.0% (more inclusive, 87 hours downtime) or 99.9% (enterprise-grade, 8.7 hours downtime). 99.5% is achievable for mission-aligned organizations with reasonable infrastructure. |
| `composition_ratios` | 5/5/5 minimum per category | Array [5, 5, 5] | Ensures minimum representation from each stakeholder group: Infrastructure Builders, Trusted ReFi Partners, Ecological Data Stewards. Remaining 0–6 slots are flexible. | Hard minimums may be difficult to fill initially for some categories. | 3/3/3 (lower minimum, more flexibility) or equal thirds (7/7/7, fully prescribed). 5/5/5 with 6 flexible slots balances representation with pragmatism. |
| `probation_period` | 30 days | 2,592,000 seconds | Standard improvement period in employment and governance contexts. Long enough for meaningful remediation; short enough to resolve issues promptly. | 30 days of degraded performance from a validator affects network reliability. | 14 days (faster resolution) or 60 days (more generous). 30 days is conventional and balances speed with fairness. |
| `performance_bonus_share` | 10% of Validator Fund | 1,000 bps | Allocates 10% of total validator compensation to a performance-based bonus pool distributed by composite score (uptime 40%, governance participation 30%, ecosystem contribution 30%). Incentivizes operational excellence without creating excessive competition. | Performance metrics may be gamed. AGENT-004 monitoring mitigates this. | 0% (pure equal distribution — simpler, less gaming) or 20% (stronger performance incentive). 10% provides meaningful incentive without destabilizing base compensation. |

### Seed Validator Set

The initial authority validator set must be bootstrapped before the `x/authority` module can activate. The following template is provided for the Validator Working Group to populate:

| Slot | Category | Validator Name | Address | Qualifying Criteria | Nominated By |
|------|----------|---------------|---------|--------------------|--------------|
| 1 | Infrastructure Builder | _[To be filled by WG]_ | regen1... | _[Active development contributions]_ | _[Nominator]_ |
| 2 | Infrastructure Builder | _[To be filled by WG]_ | regen1... | _[Active development contributions]_ | _[Nominator]_ |
| 3 | Infrastructure Builder | _[To be filled by WG]_ | regen1... | _[Active development contributions]_ | _[Nominator]_ |
| 4 | Infrastructure Builder | _[To be filled by WG]_ | regen1... | _[Active development contributions]_ | _[Nominator]_ |
| 5 | Infrastructure Builder | _[To be filled by WG]_ | regen1... | _[Active development contributions]_ | _[Nominator]_ |
| 6 | Trusted ReFi Partner | _[To be filled by WG]_ | regen1... | _[Ecosystem participation record]_ | _[Nominator]_ |
| 7 | Trusted ReFi Partner | _[To be filled by WG]_ | regen1... | _[Ecosystem participation record]_ | _[Nominator]_ |
| 8 | Trusted ReFi Partner | _[To be filled by WG]_ | regen1... | _[Ecosystem participation record]_ | _[Nominator]_ |
| 9 | Trusted ReFi Partner | _[To be filled by WG]_ | regen1... | _[Ecosystem participation record]_ | _[Nominator]_ |
| 10 | Trusted ReFi Partner | _[To be filled by WG]_ | regen1... | _[Ecosystem participation record]_ | _[Nominator]_ |
| 11 | Ecological Data Steward | _[To be filled by WG]_ | regen1... | _[Verification/data quality record]_ | _[Nominator]_ |
| 12 | Ecological Data Steward | _[To be filled by WG]_ | regen1... | _[Verification/data quality record]_ | _[Nominator]_ |
| 13 | Ecological Data Steward | _[To be filled by WG]_ | regen1... | _[Verification/data quality record]_ | _[Nominator]_ |
| 14 | Ecological Data Steward | _[To be filled by WG]_ | regen1... | _[Verification/data quality record]_ | _[Nominator]_ |
| 15 | Ecological Data Steward | _[To be filled by WG]_ | regen1... | _[Verification/data quality record]_ | _[Nominator]_ |
| 16–21 | Flexible (any category) | _[To be filled by WG]_ | regen1... | _[Meets any category criteria]_ | _[Nominator]_ |

> **Process**: Candidate validators should submit applications to the Tokenomics Working Group with documentation of their qualifying criteria. The WG evaluates candidates against composition requirements and recommends a slate for community vote. The seed set approval is bundled with the M014 activation proposal.

### Staker Impact Analysis

The transition from PoS to PoA directly affects all REGEN delegators and validators. This section details the impact and mitigation plan.

**What happens to currently staked REGEN**:

| Phase | Staked REGEN Status | Action Required |
|-------|-------------------|-----------------|
| M014 Phase 1 (Curate) | Staking remains active. Existing validators who meet authority criteria continue; others are gradually rotated out. | Delegators to non-qualifying validators should redelegate to qualifying validators. 21-day unbonding period applies as normal. |
| M014 Phase 2 (Coexistence) | PoS and PoA modules run simultaneously. Staking inflation begins to ramp down (50% reduction in first quarter, 75% in second). | Delegators should begin planning for full unstaking. Staking rewards decline but remain nonzero. |
| M014 Phase 3 (PoS Disabled) | `x/staking` inflation set to 0%. All remaining delegations begin forced unbonding (standard 21-day period). | All staked REGEN is returned to delegators. No further staking rewards. Token holders can participate in M015 (Contribution Rewards) or Stability Tier for returns. |

**Unbonding timeline**:
- Phase 2 duration: estimated 6 months (allows gradual transition)
- Phase 3 forced unbonding: 21 days (standard Cosmos SDK unbonding period)
- Total transition window from Phase 1 to full PoA: approximately 9–12 months

**Communication plan**:
1. **T-90 days**: Forum post announcing M014 proposal timeline and staker impact
2. **T-60 days**: Governance discussion thread; validator applications open
3. **T-30 days**: Proposal submitted on-chain; detailed FAQ published
4. **T-0**: Proposal passes; Phase 1 begins
5. **Monthly**: Transition status updates via governance digest (AGENT-002)

### Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Insufficient qualified validator candidates for 5/5/5 composition | High | Medium | Begin outreach 6 months before proposal. Accept 4/4/4 minimum for initial set if necessary, with a 6-month deadline to reach 5/5/5. |
| Staker confusion or panic selling during PoS phase-out | Medium | High | Comprehensive communication plan (above). Gradual inflation reduction rather than abrupt cutoff. Emphasize M015 as replacement income opportunity. |
| Authority validator collusion (reduced decentralization) | High | Low | Composition requirements prevent single-category dominance. 12-month terms with re-evaluation. AGENT-004 monitoring for anomalous voting patterns. |
| Validator Fund insufficient to cover compensation at low fee volumes | High | Medium | Model validator economics at conservative fee projections before activating. If Validator Fund balance falls below 3 months of projected compensation, trigger emergency governance review. |

---

## PROPOSAL 3: Activate M012 Dynamic Supply

### Proposal Text (Copy-Paste Ready)

> **Title**: Activate Fixed Cap Dynamic Supply Management (M012)
>
> **Type**: Software Upgrade Proposal (binary upgrade deploying `x/supply` module and modifying `x/mint`)
>
> **Deposit**: 500 REGEN
>
> **Description**:
>
> This proposal activates the Fixed Cap Dynamic Supply mechanism (M012) on Regen Ledger mainnet, replacing the current inflationary proof-of-stake supply model with a hard-capped, algorithmically managed supply that ties token minting and burning to real ecological activity.
>
> **What changes**:
> - A hard cap of 221,000,000 REGEN is established. Total supply can never exceed this amount.
> - The current `x/mint` inflation schedule is disabled.
> - A new `x/supply` module implements algorithmic mint/burn cycles:
>   - **Minting (regrowth)**: New tokens are minted each period as `M[t] = r * (C - S[t])`, where r is the regrowth rate and (C - S[t]) is the gap between cap and current supply. As supply approaches the cap, minting naturally decelerates toward zero.
>   - **Burning**: Tokens from the M013 Burn Pool are permanently destroyed each period.
> - Supply is recalculated every 7 days (one period).
> - The system targets equilibrium where minting equals burning (`M[t] = B[t]`).
>
> **Why this matters**:
> - The current inflationary model creates persistent sell pressure as staking rewards are emitted regardless of network activity or revenue.
> - A hard cap provides long-term monetary policy certainty (like gold's fixed supply) while dynamic supply management provides short-term adaptability (like managed monetary policy).
> - Supply contraction from fee-funded burning ties deflation to real economic activity rather than artificial tokenomic mechanisms.
>
> **Impact**:
> - Circulating supply gains a hard ceiling at approximately current total supply levels.
> - Inflationary staking rewards cease (replaced by M014 fee compensation and M015 activity rewards).
> - Net supply direction depends on the balance between regrowth minting and fee-based burning.
>
> **Prerequisites**: Proposal 1 (M013 Fee Routing) must be active (provides burn input). Proposal 2 (M014 PoA Governance) must be in at least TRANSITION state (inflation phase-out underway).

### Parameter Justification Table

| Parameter | Proposed Value | On-Chain Representation | Justification | Risk | Alternative Considered |
|-----------|---------------|------------------------|---------------|------|----------------------|
| `hard_cap` | 221,000,000 REGEN | 221,000,000,000,000 uregen | Approximately current total supply (~224M minus tokens already burned or lost). Sets the cap slightly below total supply to create an immediate scarcity signal — the supply is already at or near the cap, so regrowth is minimal from day one and net burning begins immediately. | If set below actual circulating supply, the system cannot mint at all until burning reduces supply below the cap. This is the intended behavior but may surprise stakeholders expecting some regrowth. | 224M (current total supply — no immediate scarcity), 200M (aggressive scarcity — significant regrowth headroom), or 250M (headroom for future growth). 221M was recommended by the WG as a balance between signal and function. |
| `base_regrowth_rate` | 2% per period | 200 bps | Conservative regrowth rate. Models show that at 2% base rate with moderate fee burning, the system reaches equilibrium in approximately 18–24 months. The 2% rate means that in each 7-day period, the protocol mints 2% of the gap between cap and current supply. | If too high, minting outpaces burning and the system trends inflationary (though still capped). If too low, supply contracts faster than intended, potentially creating liquidity issues. | 1% (slower regrowth, faster contraction) or 5% (faster regrowth, stronger inflationary pressure within cap). 2% matches Blockscience modeling recommendations for ecological carrying capacity. |
| `staking_multiplier_enabled` | true | boolean true | Enabled initially to maintain continuity during the PoS-to-PoA transition. When M014 reaches ACTIVE state, this multiplier is superseded by `stability_multiplier` (from M015 commitments). During the transition window, the effective multiplier is `max(staking, stability)` to prevent regrowth discontinuity. | Maintains a dependency on staking metrics that the system is phasing out. Mitigated by automatic phase-gating tied to M014 state. | false (immediate stability multiplier only — but M015 may not have enough commitment data yet). true with automatic transition is the safe default. |
| `ecological_multiplier_enabled` | false | boolean false | Disabled in v0. The ecological multiplier requires a reliable oracle for environmental metrics (e.g., atmospheric CO2 delta). No such oracle is currently integrated or validated. Enabling an unvalidated oracle would be a critical security risk. | Missing ecological coupling means supply dynamics are purely economic in v0, not ecologically responsive. This is acceptable for launch; ecological coupling is a v2 enhancement. | true with a placeholder oracle (dangerous — introduces attack surface). false is the only safe v0 option. |
| `period_length` | 7 days | 604,800 seconds | Weekly mint/burn cycles align with the existing epoch structure on Regen Ledger. Weekly granularity provides meaningful observation windows for governance while being responsive enough to reflect changing activity levels. | Weekly is coarser than per-block (like EIP-1559). Fee revenue accumulates for a week before being burned, creating a delay between activity and supply impact. | Per-block (maximum responsiveness, highest gas cost), daily (more responsive, higher computational overhead), monthly (less responsive, lower overhead). Weekly is the WG-recommended cadence. |

### Supply Model Projections

Assuming the proposed parameters and a starting supply of 221M REGEN (at or near the cap):

```
Supply Trajectory Over 24 Months

Scenario A: Low Activity (Conservative — $4.8K/month fees)
─────────────────────────────────────────────────────────
Period    Supply (M)    Minted    Burned    Net Change
Month 1   221.00        ~0*       ~1,344    -1,344
Month 6   220.99        ~40       ~8,064    -8,024
Month 12  220.98        ~100      ~16,128   -16,028
Month 24  220.95        ~400      ~32,256   -31,856

  * Minting near zero because supply ≈ cap, so (C - S) ≈ 0.
  Net effect: Very slow contraction. Burn is modest. ~0.02% reduction/year.

Scenario B: Moderate Activity ($24K/month fees)
─────────────────────────────────────────────────────────
Period    Supply (M)    Minted     Burned     Net Change
Month 1   221.00        ~0*        ~6,720     -6,720
Month 6   220.96        ~320       ~40,320    -40,000
Month 12  220.88        ~960       ~80,640    -79,680
Month 24  220.55        ~5,400     ~161,280   -155,880

  Net effect: Gradual contraction toward equilibrium.
  Projected equilibrium: ~18-24 months at ~220.3M.

Scenario C: High Activity ($106K/month fees)
─────────────────────────────────────────────────────────
Period    Supply (M)    Minted      Burned      Net Change
Month 1   221.00        ~0*         ~29,680     -29,680
Month 6   220.82        ~1,440      ~178,080    -176,640
Month 12  220.46        ~4,320      ~356,160    -351,840
Month 24  219.49        ~12,120     ~712,320    -700,200

  Net effect: Meaningful deflationary pressure.
  Projected equilibrium: ~12-15 months at ~219M.
  Supply reduction of ~0.7% in 2 years.

Note: All burn figures assume 28% of total fees go to Burn Pool.
      Minting figures assume 2% base regrowth of (cap - supply) gap.
      Values in REGEN tokens, not uregen.
```

> **Key insight**: Because the cap is set near current supply, regrowth minting is minimal initially. The dominant supply dynamic in the first 12 months is fee-driven burning. This is intentional — it creates an immediate deflationary signal that strengthens as network activity grows. Equilibrium emerges naturally as the gap between cap and supply widens (increasing regrowth) until it matches the burn rate.

### Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Regrowth rate too high, creating inflationary pressure | Medium | Low | At 2% of a near-zero gap (cap - supply), initial minting is negligible. Rate can be reduced via Layer 3 governance proposal if needed. Safety bound: r_base capped at 10% maximum. |
| Burning outpaces regrowth, creating deflation spiral and liquidity crisis | Medium | Low | At current volumes, burn is modest. Deflationary pressure is self-limiting: as supply decreases, regrowth gap increases, increasing minting. The system has a natural equilibrium-seeking dynamic. |
| Hard cap set incorrectly (too low or too high) | High | Low | Requires Layer 4 (Constitutional) governance to change — intentionally difficult. Extensive modeling supports 221M. Community should reach broad consensus before submission. |
| Ecological multiplier enabled prematurely with unreliable oracle | Critical | Low | Disabled in v0. Enabling requires a separate governance proposal with oracle validation evidence. |
| Period length too long — burns delayed relative to activity | Low | Medium | 7-day periods mean maximum 7-day delay between fee collection and burn execution. Acceptable for a first version; can be shortened via Layer 2 governance. |

---

## PROPOSAL 4: Activate M015 Contribution Rewards

### Proposal Text (Copy-Paste Ready)

> **Title**: Activate Contribution-Weighted Reward Distribution (M015)
>
> **Type**: Software Upgrade Proposal (binary upgrade deploying `x/rewards` module), followed by Parameter Change Proposal (setting initial activity weights and stability tier parameters)
>
> **Deposit**: 500 REGEN
>
> **Description**:
>
> This proposal activates the Contribution-Weighted Rewards mechanism (M015) on Regen Ledger mainnet, replacing passive staking rewards with an activity-based distribution system where participants earn from the Community Pool proportional to their ecological and governance contributions.
>
> **What changes**:
> - A new `x/rewards` module is deployed for tracking activity scores and distributing rewards.
> - The Community Pool (funded by M013 fee routing) becomes the source of all participant rewards.
> - Rewards are distributed based on measurable on-chain activity: credit purchases (30%), credit retirements (30%), platform facilitation (20%), and governance participation (20%).
> - An optional Stability Tier allows long-term holders to commit REGEN for a fixed 6% annual return (capped at 30% of Community Pool inflow per period).
> - A 90-day tracking-only phase collects activity data before distributions begin, allowing calibration.
>
> **Why this matters**:
> - The current model rewards passive staking — locking tokens generates yield regardless of contribution to the network's ecological mission.
> - M015 creates a direct link: generate ecological value on the network, earn proportional rewards. Investment returns emerge as a consequence of network activity, not speculative dynamics.
> - The Stability Tier provides a transition path for holders accustomed to staking yields, offering predictable returns tied to real revenue rather than inflation.
>
> **Impact**:
> - Participants who actively use the network (buying, retiring, facilitating credits; voting on governance) earn rewards proportional to their contribution.
> - Passive holders can opt into the Stability Tier for a fixed 6% annual return (subject to capacity cap).
> - Reward amounts are constrained by actual Community Pool revenue — no inflation, no deficit spending.
>
> **Prerequisites**: Proposal 1 (M013 Fee Routing) must be active (Community Pool receiving fee revenue). Proposal 2 (M014 PoA Governance) must be active (staking rewards disabled; contribution rewards replace them).

### Parameter Justification Table

| Parameter | Proposed Value | On-Chain Representation | Justification | Risk | Alternative Considered |
|-----------|---------------|------------------------|---------------|------|----------------------|
| `weight_purchase` | 30% | 3,000 bps | Credit purchases represent the primary demand signal for ecological credits. Buyers are the fundamental driver of marketplace revenue and ecological outcome funding. Equal weight with retirement reflects that purchase and retirement are the two most important lifecycle events. | Over-weighting purchase may incentivize wash trading. Mitigated by M013 fees — circular transactions cost more in fees than they earn in rewards. | 25% (spread weight to other activities) or 40% (stronger purchase incentive). 30% parity with retirement emphasizes the full credit lifecycle. |
| `weight_retirement` | 30% | 3,000 bps | Credit retirement is the terminal impact event — the moment ecological value is permanently claimed. This is the strongest ecological commitment a participant can make and the mechanism that generates retirement certificates. | Retirement is a one-time event per credit; high weighting may disproportionately reward large one-time retirers over consistent smaller participants. | 20% (reduced emphasis) or 40% (dominant weight). 30% parity with purchase avoids over-indexing on either end of the lifecycle. |
| `weight_facilitation` | 20% | 2,000 bps | Platform facilitation rewards infrastructure builders — brokers, marketplace operators, and tools that enable transactions to occur. This rewards the ecosystem layer that grows the network's capacity. | Facilitation credit requires identifying the facilitating platform (metadata field or API key). Gaming risk if facilitation credit is loosely attributed. | 10% (minimal facilitation incentive) or 30% (strong ecosystem builder incentive). 20% provides meaningful reward without over-weighting intermediaries relative to direct participants. |
| `weight_governance` | 20% | 2,000 bps | Combined weight for governance voting (10%) and proposal submission (10%). Maintains governance participation incentive that previously came from staking requirements. Proposal credit is conditional on quorum attainment to prevent spam. | Governance weight may incentivize uninformed or rubber-stamp voting. Mitigated by the qualitative nature of governance — voting on everything without reading proposals yields no additional credit versus selective participation. | 10% (minimal governance weight) or 30% (strong governance incentive). 20% signals that governance participation matters without dominating the activity score. |
| `stability_tier_annual_return` | 6% | 600 bps | Sustainable if Community Pool receives $10K+/month (moderate scenario). At 6% annual return with 30% cap, the system can support approximately $600K in total stability commitments at $10.8K/month Community Pool inflow (moderate scenario). If Community Pool inflow grows, capacity grows proportionally. | If fee revenue is lower than projected, the 30% cap protects the activity pool but stability tier may fill quickly and create a waitlist. | 4% (more conservative, supports more total commitments) or 8% (more attractive, supports fewer total commitments). 6% is competitive with DeFi staking yields while being sustainable from fee revenue. |
| `max_stability_share` | 30% | 3,000 bps | Hard cap protecting the activity-based reward pool. At most 30% of each period's Community Pool inflow goes to stability tier commitments; the remaining 70%+ is distributed via activity scoring. This prevents stability tier demand from crowding out active contributors. | 30% may be too generous to passive holders at the expense of active contributors. Or too restrictive if there is high demand for stable yields. | 20% (stronger activity-pool protection) or 40% (more generous to stability holders). 30% is the WG's recommended balance point. |
| `tracking_period` | 90 days | 7,776,000 seconds | Three-month tracking-only period before distributions begin. Collects baseline activity data, allows the community to verify that scoring is working correctly, and provides time to identify any gaming patterns before real rewards are at stake. | 90 days of tracking with no payouts may frustrate early adopters expecting immediate rewards. Communicate clearly that tracking earns retroactive credit — first distribution includes all tracked activity. | 30 days (faster activation, less calibration data) or 180 days (more data, longer wait). 90 days provides a full quarter of data — sufficient for meaningful baselines. |
| `min_commitment` | 100 REGEN | 100,000,000 uregen | Minimum stability tier commitment. Prevents dust commitments that create administrative overhead with minimal economic effect. At current prices (~$0.03), this is approximately $3 — a very low barrier. | May need adjustment if REGEN price changes significantly. | 10 REGEN (near-zero barrier) or 1,000 REGEN (higher barrier, fewer commitments to manage). 100 REGEN is a round number providing minimal friction. |

### Community Pool Impact Analysis

**Current state** (pre-M013):
- Community Pool balance: check via `regen query distribution community-pool`
- Current inflow: minimal (governance tax on staking rewards only)
- Current outflow: governance-approved spend proposals (sporadic)

**Projected state** (with M013 active, moderate scenario):

| Metric | Monthly | Annual |
|--------|---------|--------|
| Community Pool inflow (45% of $24K fees) | $10,800 | $129,600 |
| Stability Tier allocation (up to 30%) | $3,240 max | $38,880 max |
| Activity reward pool (remaining 70%+) | $7,560 min | $90,720 min |
| Governance-directed spending budget | Separate — existing Community Pool balance + any unallocated inflow | Determined by governance proposals |

**Stability Tier sustainability analysis**:

```
At 6% annual return and $10,800/month Community Pool inflow:

  Maximum monthly stability payout = $10,800 * 30% = $3,240/month
  Maximum annual stability payout   = $3,240 * 12 = $38,880/year

  Maximum sustainable stability commitments at 6% annual:
    $38,880 / 0.06 = $648,000 worth of REGEN

  At current REGEN price (~$0.03):
    $648,000 / $0.03 = ~21.6M REGEN in stability tier

  This represents approximately 9.8% of total supply (221M) —
  a reasonable and sustainable level of stability commitment.

If fee revenue doubles (optimistic scenario):
  Maximum stability commitments = ~43.2M REGEN (~19.5% of supply)

If fee revenue halves (conservative scenario):
  Maximum stability commitments = ~10.8M REGEN (~4.9% of supply)
```

**Key finding**: The 30% cap on stability allocation ensures that even with high demand for stable yields, the majority of Community Pool revenue flows to active contributors. The system is self-regulating — stability capacity scales linearly with fee revenue.

### Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Activity score gaming via wash trading | Medium | Medium | M013 fees provide natural friction — each wash trade costs 1% in fees but earns only a fraction of that in rewards (since rewards are shared across all participants). Net-negative return for wash traders. Additional anti-gaming: minimum transaction size for reward eligibility (configurable). |
| Stability Tier demand exceeds capacity | Low | Medium | Commitments are queued and filled in order. Clear communication about capacity limits. If demand consistently exceeds capacity, governance can increase the cap or reduce the annual return rate. |
| Activity weights miscalibrated | Medium | High | Expected — weights are first estimates. The 90-day tracking period provides calibration data. Follow-up parameter change proposal planned for 6 months post-activation. |
| Community Pool inflow too low to fund meaningful rewards | Medium | Medium | Revenue constraint is a feature, not a bug — no deficit spending. If rewards are too small to be meaningful, the system signals that network activity needs to grow. This creates aligned incentives. |
| Sybil attacks via multiple addresses | Medium | Low | Activity scoring based on transaction value, not address count. Splitting activity across addresses does not increase total score. M013 fees make Sybil attacks costly. |

---

## PROPOSAL 5: Community Pool Spend — Agent Infrastructure (Optional)

> **Note**: This proposal is relevant only if the community adopts a fee distribution model that includes a dedicated Agent Infrastructure Fund (Model A or the compromise model in Proposal 1). If agents are funded exclusively through standard Community Pool proposals (Model B), this proposal serves as the template for the first such spend proposal.

### Proposal Text (Copy-Paste Ready)

> **Title**: Community Pool Spend — Bootstrap Agent Infrastructure Operations
>
> **Type**: Community Pool Spend Proposal
>
> **Deposit**: 500 REGEN
>
> **Requested Amount**: 50,000 REGEN (approximately $1,500 at current prices; covers estimated 6 months of minimal operations)
>
> **Recipient**: `regen1...` _(Agent Infrastructure multisig address — to be established by WG)_
>
> **Description**:
>
> This proposal allocates Community Pool funds to bootstrap the agent infrastructure that supports M012–M015 mechanism monitoring and governance assistance. Regen Network's agentic governance roadmap (Phase A, Q1–Q2 2026) establishes AI agents as read-only observers that analyze network state, generate governance digests, and monitor validator performance.
>
> **What this funds**:
> - LLM API access for AGENT-001 (Registry Reviewer), AGENT-002 (Governance Analyst), AGENT-003 (Market Monitor), and AGENT-004 (Validator Monitor).
> - Infrastructure hosting for agent runtimes (Kubernetes cluster, databases, monitoring).
> - MCP (Model Context Protocol) integration maintenance for Regen Ledger and KOI connections.
>
> **Why this matters**:
> - Agents provide real-time monitoring of the newly activated M012–M015 mechanisms, detecting anomalies and generating reports that inform governance decisions.
> - Without agent infrastructure, governance monitoring relies entirely on manual analysis — feasible but slower and less comprehensive.
> - This is a bootstrap allocation. Once M013 fee revenue begins flowing to the Agent Infrastructure Fund (if adopted), ongoing operations are self-funding.
>
> **Accountability**:
> - Monthly expense reports published to the governance forum.
> - Quarterly review with option for community to discontinue funding.
> - All agent actions are logged and auditable.
> - Agents operate in read-only mode (Phase A) — no on-chain write access.

### Budget Breakdown

| Line Item | Monthly Cost | 6-Month Total | Notes |
|-----------|-------------|---------------|-------|
| LLM API (Claude/GPT-4 class) | $150 | $900 | Estimated 4 agents, moderate usage. Cost sensitive to query volume; budget assumes governance-cadence (weekly digest) not real-time monitoring. |
| Kubernetes hosting (shared cluster) | $50 | $300 | Minimal deployment: 2–3 pods, shared namespace. Can run on existing RND infrastructure if available. |
| Database (PostgreSQL + vector store) | $20 | $120 | Small-scale: agent state, analysis history, embeddings for document retrieval. |
| Monitoring and alerting (Grafana/PagerDuty) | $15 | $90 | Basic dashboards and alerting for agent health and mechanism metrics. |
| Contingency (20%) | $47 | $282 | Buffer for API price changes, scaling, or unexpected costs. |
| **Total** | **$282/month** | **$1,692** | |

> **Note**: At current REGEN price (~$0.03), the requested 50,000 REGEN covers approximately $1,500 — close to the estimated 6-month cost. If REGEN price changes significantly before submission, the requested amount should be adjusted. The budget is intentionally conservative for bootstrap operations; a follow-up proposal can request additional funding based on demonstrated value.

### Sustainability Path

Once M013 is active, the Agent Infrastructure Fund receives 2% of fee revenue (under the proposed distribution). At the moderate scenario ($24K/month total fees), this yields approximately $480/month — sufficient to cover the estimated $282/month operational cost with a comfortable margin. At that point, Community Pool spend proposals for agent infrastructure become unnecessary.

---

## Summary

| Proposal | Mechanism | Type | Dependencies | Deposit | Estimated Submission |
|----------|-----------|------|-------------|---------|---------------------|
| 1 | M013 Fee Routing | Software Upgrade + Param Change | None | 500 REGEN | Q2 2026 |
| 2 | M014 PoA Governance | Software Upgrade | M013 active | 500 REGEN | Q3 2026 |
| 3 | M012 Dynamic Supply | Software Upgrade | M013 active, M014 transitioning | 500 REGEN | Q3 2026 |
| 4 | M015 Contribution Rewards | Software Upgrade + Param Change | M013 active, M014 active | 500 REGEN | Q4 2026 |
| 5 | Agent Infrastructure Spend | Community Pool Spend | M013 active (optional, for bootstrap) | 500 REGEN | Q1 2027 |

### Pre-Submission Checklist

Before submitting each proposal on-chain:

- [ ] Verify current governance parameters: `regen query gov params`
- [ ] Confirm deposit amount matches current `min_deposit`
- [ ] Verify voting period and quorum requirements
- [ ] Publish proposal text on [Regen Forum](https://forum.regen.network) for community discussion (minimum 2 weeks before on-chain submission)
- [ ] Collect informal sentiment via forum poll
- [ ] Confirm prerequisite mechanisms are active on mainnet
- [ ] Run the proposal through AGENT-002 (Governance Analyst) for impact analysis
- [ ] Coordinate with RND engineering on binary upgrade readiness (for software upgrade proposals)
- [ ] Prepare validator communication (especially for M014)

### Open Governance Questions

The following questions from the [M012–M015 specifications](../../phase-2/2.6-economic-reboot-mechanisms.md) must be resolved by community deliberation before the corresponding proposals are submitted:

| Question | Affects | Status |
|----------|---------|--------|
| OQ-M013-1: Distribution model (Model A vs B vs compromise) | Proposal 1 share parameters | Compromise proposed in this document; needs WG ratification |
| OQ-M013-2: Credit value estimation method | Proposal 1 fee calculation | Marketplace reference price recommended; needs validation |
| OQ-M013-3: Fee denomination (REGEN-only vs multi-denom) | Proposal 1 implementation | REGEN-only recommended for v1; needs community input |
| OQ-M013-5: Burn pool share (or elimination) | Proposal 1 burn_share parameter | 28% proposed; active debate ongoing |
| OQ-M014-1: Exact validator set size | Proposal 2 max_validators | 21 proposed; needs WG confirmation |
| OQ-M014-3: Seed validator set composition | Proposal 2 seed set | Template provided; WG must populate |
| OQ-M014-5: Delegated REGEN transition plan | Proposal 2 staker impact | Plan provided; needs community review |
| OQ-M012-1: Exact hard cap value | Proposal 3 hard_cap | 221M proposed; needs final modeling confirmation |
| OQ-M015-1: Stability tier return rate | Proposal 4 annual_return | 6% proposed; sustainability analysis provided |
| OQ-M015-3: Auto-distribution vs governance-directed split | Proposal 4 pool allocation | Automatic M015 from Community Pool inflow; governance-directed from existing balance |

---

*This document provides governance-ready templates for the Regen Economic Reboot. All parameter values represent the Tokenomics Working Group's best current estimates and should be refined through community deliberation before on-chain submission. Parameter change proposals can adjust values post-activation without requiring binary upgrades.*

*Prepared for the Regen Network Tokenomics Working Group, March 2026.*
