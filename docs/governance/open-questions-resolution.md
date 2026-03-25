# Open Questions Resolution: Phase 2 Specification Review

> **Status**: DRAFT FOR WG REVIEW
> **Date**: 2026-03-24
> **Scope**: All 31 open questions identified during Phase 2 specification
> **Resolution Summary**: 22 RESOLVED with specific recommendations, 9 NEEDS_GOVERNANCE

---

## Table of Contents

- [M001-ENH: Credit Class Approval Voting](#m001-enh-credit-class-approval-voting)
- [M009: Service Provision Escrow](#m009-service-provision-escrow)
- [M011: Marketplace Curation](#m011-marketplace-curation)
- [M012: Fixed Cap Dynamic Supply](#m012-fixed-cap-dynamic-supply)
- [M013: Value-Based Fee Routing](#m013-value-based-fee-routing)
- [M014: Authority Validator Governance](#m014-authority-validator-governance)
- [M015: Contribution-Weighted Rewards](#m015-contribution-weighted-rewards)
- [GOV-POA: Governance Transition](#gov-poa-governance-transition)
- [Summary Matrix](#summary-matrix)

---

## M001-ENH: Credit Class Approval Voting

### OQ-M001-1: Dual-Track Tally Weight Split

**Question**: Is 60/40 the right validator-to-holder weight split? Should registry-specific proposals weight ecological data steward validators higher?

**Analysis**:

The 60/40 split was proposed in M001-ENH to give authority validators -- who have been vetted through M014's admission process and bear direct operational responsibility for the chain -- a majority voice while preserving meaningful token-holder governance participation. The rationale is sound: validators have demonstrated commitment through the selection process, maintain infrastructure at their own cost, and face probation or removal for poor performance. They have more at stake operationally than a passive token holder.

However, a single fixed ratio across all governance processes creates a subtle misalignment. For registry-specific decisions -- credit class approvals, methodology validation, data quality standards -- ecological data steward validators (M014 category 3) possess domain expertise that infrastructure builders or ReFi partners may not. A blanket 60/40 does not distinguish between validators who deeply understand the ecological merits of a methodology and those who primarily contribute to infrastructure. Conversely, for treasury spending decisions, token holders who fund the ecosystem through purchases and stability commitments arguably deserve a stronger voice than they get at 40%.

The alternative of per-process weight variation has been explored in other PoA and hybrid governance systems (e.g., MakerDAO's multi-committee structure, Optimism's bicameral model). The tradeoff is complexity: more weight configurations mean more governance parameters to maintain and more surface area for political capture. However, the Regen governance taxonomy already distinguishes seven process categories (2.3), so the framework exists to support differentiated weights.

A pragmatic middle ground is to adopt 60/40 as the default while allowing specific governance tracks to override these weights via Layer 3 governance proposals. This keeps the initial system simple but creates a clear upgrade path. For registry-specific decisions specifically, a soft advisory mechanism -- where data steward validators' positions are highlighted in the voting guide but not mechanically weighted higher -- can capture domain expertise without complicating the tally algorithm.

**Recommendation**: Adopt 60/40 as the default dual-track weight. Introduce a governance parameter `process_weight_override` (Layer 3) that allows specific GOV processes to set custom weights within bounds [50/50, 80/20]. For v0, all processes use the default. Recommend the WG revisit after 6 months of operation with data on voting patterns. For registry-specific decisions, AGENT-001 should flag data steward validator positions in the voting summary to aid holder-track decision-making.

**Rationale**: Starting with a single default minimizes launch complexity while the override mechanism provides a clear path for specialization. The 60/40 balance reflects the PoA philosophy (validators have earned governance authority) while preventing validator oligarchy. The advisory highlight for data stewards captures domain expertise without creating a third governance track.

**Status**: RESOLVED

---

### OQ-M001-2: Agent Pre-Screening Score Governance Track Influence

**Question**: Should the agent pre-screening score influence which governance track is required? E.g., high-confidence agent approvals could use a lighter-touch validator-only fast track.

**Analysis**:

The current M001-ENH spec defines a three-tier agent scoring system: scores >= 0.7 advance to voting, scores < 0.3 with high confidence trigger rejection (with a human override window), and intermediate scores go to manual review. The question is whether a very high agent confidence score (e.g., >= 0.85) could bypass the full dual-track vote and proceed through a faster validator-only approval path.

The case for this is efficiency. Historical data from 2.3 shows that credit class creator allowlist proposals have a 92% pass rate with 58% average turnout. Many of these are routine approvals for well-established methodologies or organizations with strong track records. Requiring a full 7-day dual-track vote for every such proposal creates governance fatigue and delays legitimate applicants. A validator-only fast track (e.g., 48-hour window, validator track only) for high-confidence proposals would significantly reduce governance overhead.

The case against is that agent confidence is fundamentally a machine assessment, and credit class approvals have lasting economic implications -- a poorly approved credit class can damage the network's ecological credibility. Moreover, reducing holder participation in these decisions undermines the "voice" that the 40% holder track provides. If agent scores routinely fast-track proposals past holders, the holder track becomes decorative.

The critical safeguard is the human override window. If a high-confidence fast track is introduced, any single authority validator or any holder meeting a minimum threshold (e.g., 0.1% of circulating supply) should be able to escalate the proposal to the full dual-track process. This preserves the override guarantee while reducing unnecessary governance friction. The 48-hour window provides enough time for validators and engaged holders to review and escalate if needed.

Precedent from Cosmos SDK governance: the `expedited_proposal` feature in SDK v0.47+ provides a faster voting path with higher thresholds. This is analogous and already battle-tested.

**Recommendation**: Introduce a "Fast Track" governance path for proposals where agent_score >= 0.85 AND agent_confidence >= 0.9. Fast Track proposals go to a 48-hour validator-only vote (simple majority of active validators, 2/3 quorum). Any authority validator or any holder with >= 0.1% of circulating supply can escalate to the full 7-day dual-track process within the first 24 hours. Fast Track is limited to Layer 2 decisions (credit class allowlist, currency additions, minor parameter changes) and cannot be used for Layer 3-4 governance.

**Rationale**: Reduces governance fatigue for routine decisions while maintaining robust safeguards. The escalation mechanism prevents agent errors from having permanent consequences. The Layer 2 restriction ensures that significant decisions always receive full community deliberation.

**Status**: RESOLVED

---

## M009: Service Provision Escrow

### OQ-M009-1: Automatic Reputation Feedback on Agreement Completion

**Question**: Should provider reputation feed back into M010 automatically on agreement completion, or require a separate endorsement transaction?

**Analysis**:

M010 (Reputation Signal) tracks participant reputation scores across the network. M009 service agreements generate valuable signal about provider quality -- whether milestones were delivered on time, whether disputes arose, and the ultimate outcome. The question is whether this signal should flow automatically into M010 or require an explicit endorsement.

Automatic feedback creates a tight loop: complete an agreement successfully, and your reputation increases; fail, and it decreases. This is efficient and ensures that all agreement outcomes are captured. However, it creates a gaming vector: two colluding parties could create fake agreements, approve all milestones, and inflate each other's reputations. The M013 fee on escrow transactions provides some friction (the 1% platform fee), but for small agreements, this cost may be trivial relative to the reputation gain.

A separate endorsement transaction adds a deliberate step but introduces its own problems. Clients may forget to endorse, creating an asymmetry where only negative outcomes (disputes) reliably affect reputation. This "negativity bias" is common in review systems and discourages participation.

The best approach combines automatic baseline feedback with endorsement-gated bonuses. Successful agreement completion automatically generates a small, fixed reputation signal in M010 (e.g., +5 points). The client can then submit an optional endorsement that provides a larger, variable reputation adjustment (e.g., +1 to +20 points based on quality rating). For disputed agreements, the arbiter resolution type automatically generates the appropriate signal (negative for the losing party). This ensures all outcomes are captured while giving clients a mechanism to differentiate between adequate and excellent providers.

Anti-gaming measures: (1) Minimum escrow size for reputation-eligible agreements (e.g., 500 REGEN), preventing micro-agreement spam. (2) Diminishing returns from repeated agreements between the same two addresses. (3) AGENT-003 pricing monitor flags outlier patterns (e.g., same two addresses creating many small agreements).

**Recommendation**: Implement automatic baseline reputation feedback on agreement completion (+5 fixed for provider on successful completion, proportional negative signal on dispute loss). Enable optional client endorsement for additional reputation adjustment (+1 to +20 variable). Set minimum escrow of 500 REGEN for reputation-eligible agreements. Apply diminishing returns (50% reduction) for each successive agreement between the same address pair within a 90-day window.

**Rationale**: Captures all outcomes while rewarding clients who provide detailed feedback. The minimum escrow and diminishing returns prevent the most obvious gaming vectors without adding prohibitive complexity.

**Status**: RESOLVED

---

### OQ-M009-2: Partial Milestone Payments

**Question**: Should partial milestone payments be supported (e.g., 70% on submission, 30% on final approval) for long-running milestones?

**Analysis**:

The current M009 spec defines milestones as atomic units: the provider submits a deliverable, the client approves, and the full milestone payment is released. For short milestones (1-4 weeks), this works well. For long-running milestones -- such as a 6-month monitoring campaign or a multi-phase methodology development -- the provider bears significant financial risk and cash flow pressure while waiting for the full payment.

Partial payments address this by releasing a portion of the milestone payment upon submission (or at intermediate checkpoints) and the remainder upon final approval. This better aligns cash flow with work progress and reduces provider risk. Construction contracts and software development agreements routinely use this structure.

The complexity cost is real but manageable. The escrow contract must track partial release state per milestone, the platform fee calculation must account for partial releases, and the dispute resolution logic must handle the case where a dispute arises after partial payment (the remaining escrow may be less than the full milestone value). However, the M009 spec already supports a rich milestone lifecycle with multiple states, so adding a `partial_release` state is an incremental change.

The key design choice is whether partial release percentages are per-agreement configurable or fixed by the protocol. Per-agreement flexibility is more useful (different services have different risk profiles), but adds UI complexity. A reasonable compromise is to offer a small set of templates: 100/0 (current, default), 70/30 (standard partial), and 50/50 (equal split), with the client and provider agreeing on the template at agreement creation.

**Recommendation**: Support partial milestone payments with three templates: (A) 100/0 -- full payment on approval (default, backward-compatible); (B) 70/30 -- 70% released on submission, 30% on approval; (C) 50/50 -- equal split between submission and approval. Template is selected by mutual agreement at PROPOSED stage and immutable once FUNDED. Platform fee is calculated on total milestone payment, collected proportionally with each partial release. In dispute scenarios, the arbiter resolves over the remaining (unreleased) portion only.

**Rationale**: Templates balance flexibility with implementation simplicity. The 70/30 and 50/50 options cover the most common partial payment patterns. Making the template immutable after funding prevents mid-agreement disputes over payment terms.

**Status**: RESOLVED

---

### OQ-M009-3: Authority Validator Override for Stuck Disputes

**Question**: Under PoA (M014), should authority validators have an override capability for stuck disputes, or should all dispute resolution remain with the Arbiter DAO?

**Analysis**:

Stuck disputes are a genuine operational risk. The current spec routes all M009 disputes to the Arbiter DAO (an M008 subDAO). If the Arbiter DAO fails to reach quorum, becomes unresponsive, or is itself subject to a conflict of interest, the dispute -- and the locked escrow funds -- can remain frozen indefinitely. The M009 spec includes a `resolution_deadline_not_expired` guard, but does not specify what happens if the deadline passes without resolution.

Giving authority validators override capability provides a backstop. Under M014, the validator set is curated and compensated, meaning they have both the legitimacy and incentive to act as a court of last resort. However, this introduces a centralization risk: validators could theoretically collude to resolve disputes in favor of a connected party, and the power to override an independent dispute body undermines the purpose of having that body.

The resolution lies in designing the override as an emergency mechanism, not a routine one. The override should only be available after the Arbiter DAO has failed to resolve within its deadline, should require a supermajority of authority validators (e.g., 75%), and should be subject to a transparency requirement (all override votes and rationale published on-chain). Additionally, the override should be limited to three outcomes: (1) force the Arbiter DAO to re-vote with a new deadline, (2) release funds to the client (conservative default), or (3) split funds 50/50. Validators should not have the authority to award funds to the provider, which prevents the most dangerous form of capture.

This creates a hierarchy: Arbiter DAO is the primary resolution mechanism; validator override is the emergency backstop. The restriction on pro-provider resolutions means validators can only return or split funds, not extract value for a favored party.

**Recommendation**: Authority validators gain emergency override capability for disputes that remain unresolved past the Arbiter DAO's resolution deadline (default: 30 days). Override requires 75% supermajority of active validators. Override options are limited to: (A) extend Arbiter DAO deadline by 14 days (one-time), (B) return remaining escrow to client minus arbiter fee, or (C) split remaining escrow 50/50 minus arbiter fee. Validators cannot award full funds to the provider through this mechanism. All override votes and rationale are recorded on-chain.

**Rationale**: Prevents indefinite fund lockup while limiting centralization risk through supermajority requirement and restricted resolution options. The inability to award full provider payment prevents the most dangerous capture vector.

**Status**: RESOLVED

---

### OQ-M009-4: Escrow Interaction with M013 Fee Routing

**Question**: How should escrow interact with the fee routing mechanism (M013)? Should platform fees follow the M013 distribution model or have a separate path?

**Analysis**:

M009 currently specifies a 1% platform fee on milestone payments and a 1% completion fee, both routed to the community pool. M013 defines a four-pool distribution model for all fee revenue (burn/validator/community/agent). The question is whether M009's platform fees should follow M013's distribution ratios or maintain their own path.

Routing M009 fees through M013 creates consistency: all protocol revenue flows through the same distribution mechanism, making the system easier to reason about and audit. It also means M009 fees contribute to burn, validator compensation, and agent infrastructure -- not just the community pool. Since validators and agents both play roles in the M009 ecosystem (AGENT-001 reviews milestones, validators may override stuck disputes), directing revenue to their pools is appropriate.

A separate path could be justified if M009 service fees serve a fundamentally different purpose than M013's credit transaction fees. However, both are platform fees extracted from economic activity on the network. The distinction is that M009 fees come from service provision agreements rather than credit transactions, but the distribution needs are the same: fund the infrastructure that supports the activity.

One consideration is the burn share. M009 escrow fees are relatively small in absolute terms (1% of individual service agreements vs. 1-3% of credit issuance value). Burning a portion of these small fees has negligible supply impact but adds implementation complexity. However, consistency in routing outweighs this concern -- the `x/feerouter` module should handle all protocol fees uniformly.

**Recommendation**: Route M009 platform fees through M013's fee distribution mechanism. The 1% milestone fee and 1% completion fee are collected and distributed according to the same burn/validator/community/agent shares as credit transaction fees. This requires M009's CosmWasm contract to send collected fees to the M013 fee collector module rather than directly to the community pool. No changes to fee rates; only the distribution path changes.

**Rationale**: Consistency in fee routing simplifies auditing, ensures all revenue streams contribute proportionally to all network functions, and avoids the need to maintain separate distribution logic for each revenue source.

**Status**: RESOLVED

---

## M011: Marketplace Curation

### OQ-M011-1: Curator Credit Holdings Requirement

**Question**: Should curators be required to hold the credit types they curate, or is the bond sufficient alignment? Requiring holdings could improve skin-in-the-game but limits curator diversity.

**Analysis**:

The M011 curation mechanism requires curators to post a bond that can be slashed for poor curation quality. The question is whether curators should also be required to hold credits of the types they curate, creating a direct financial alignment between curation quality and credit value.

The case for requiring holdings is strong on paper: if a curator holds credits in a class they curate, they are financially harmed when their curation decisions degrade the class's market value. This aligns incentives. The classic parallel is the requirement for fund managers to invest in their own funds.

However, the practical implications for Regen are problematic. Ecological credit markets are thin and illiquid. Many credit classes have few active market participants. Requiring curators to hold credits creates three problems: (1) curators must acquire credits before they can curate, creating a chicken-and-egg problem for new credit classes; (2) curators from the ecological data steward community may lack capital to purchase credits; (3) curators holding credits have a financial interest in inflating quality scores for credits they hold, creating a conflict of interest that is the opposite of the intended alignment.

The bond mechanism already provides meaningful skin-in-the-game. A curator who consistently provides poor quality scores will be challenged, and their bond will be slashed. This is a more direct and less conflicted alignment mechanism than requiring credit holdings. The bond penalizes bad curation; holdings reward good curation but also reward score manipulation.

**Recommendation**: Do not require curators to hold the credit types they curate. The curation bond (as specified in M011) is sufficient for alignment. Instead, require curators to disclose any holdings in credit classes they curate. This disclosure is recorded on-chain (via a `CuratorDisclosure` object linked to their curation position) and displayed in the marketplace UI. Undisclosed holdings discovered post-hoc are grounds for bond slashing.

**Rationale**: Avoids limiting curator diversity and prevents the perverse incentive of score inflation for personal holdings. Disclosure provides transparency without creating a barrier to entry. The bond mechanism is the primary enforcement tool.

**Status**: RESOLVED

---

### OQ-M011-2: Quality Score Storage Model

**Question**: Should quality scores be fully on-chain (transparent but costly) or anchored on-chain with details in KOI (cheaper but requires trust in the off-chain layer)?

**Analysis**:

Quality scores are the core output of M011 curation. They inform marketplace pricing, collection composition, and buyer confidence. The storage model determines the balance between transparency, cost, and trust.

Fully on-chain storage means every quality score, its breakdown by dimension (additionality, permanence, co-benefits, etc.), and the supporting evidence are stored as chain state. This is maximally transparent -- anyone can verify any score and its basis by querying the chain. However, it is expensive. Quality score assessments can include detailed narrative rationale, methodology references, and comparative analysis. Storing all of this on-chain could cost 10-100x more gas than anchoring a hash.

KOI anchoring stores only a hash (content identifier) and the aggregate numeric score on-chain, with the full assessment document stored in the Knowledge Object Infrastructure (KOI) off-chain layer. This is dramatically cheaper and supports richer assessment documents (including images, graphs, and lengthy analysis). The tradeoff is trust: users must trust that the KOI-stored content matches the on-chain hash and that the KOI infrastructure remains available.

The hybrid approach is the clear winner here, and it is already the pattern used throughout the Regen ecosystem for IRIs and data anchoring. Regen Ledger's `x/data` module was specifically designed for this: anchor the hash on-chain, store the content off-chain. The M011 quality score can follow the same pattern with one addition: the numeric score itself (a single float or integer) should be stored on-chain alongside the hash, because the numeric score is what smart contracts and marketplace logic need to query. The detailed assessment rationale lives in KOI.

**Recommendation**: Hybrid model. Store on-chain: (1) aggregate quality score (uint8, 0-100), (2) per-dimension sub-scores (array of uint8), (3) KOI content hash (IRI) for the full assessment document, (4) curator address, (5) timestamp. Store in KOI: full assessment rationale, methodology references, comparative analysis, supporting data. The on-chain IRI enables anyone to retrieve and verify the full assessment against the anchored hash.

**Rationale**: Follows the established Regen Ledger data anchoring pattern. On-chain numeric scores enable efficient querying by marketplace contracts. KOI storage supports rich assessment content without prohibitive gas costs. The IRI link maintains verifiability.

**Status**: RESOLVED

---

### OQ-M011-3: Curation Fee Interaction with M013

**Question**: How should the curation fee interact with M013 fee routing? Should it be a separate fee or carved out of the existing trade fee?

**Analysis**:

M011 specifies a curation fee that compensates curators for their quality assessment work. M013 defines trade fees (1% of trade value) distributed across four pools. The question is whether the curation fee is an additional fee on top of M013's trade fee, or whether it is carved out of the existing trade fee.

An additional fee increases total transaction costs for marketplace participants. If M013 charges 1% and curation adds another 0.5%, the total fee is 1.5%. This may seem modest, but ecological credit markets are already thin, and every basis point of friction can discourage participation -- particularly for large institutional buyers who are price-sensitive. On the other hand, curation creates genuine value by signaling quality, and curators should be compensated from a dedicated revenue stream.

Carving the curation fee out of the existing trade fee means the total cost to the buyer does not increase, but the M013 pools receive less. If 0.25% of the 1% trade fee goes to curators, the remaining 0.75% is distributed across burn/validator/community/agent pools. This reduces revenue to those pools by 25%, which is significant.

The best approach recognizes that curation is a value-add service, not a base protocol function. Credit batches that are curated (included in a quality-scored collection) carry a price premium in the marketplace because buyers have confidence in their quality. This premium is the economic basis for the curation fee. Therefore, the curation fee should be an additional fee applied only to trades of curated credits, not an across-the-board cost. Uncurated credits trade at the base M013 rate (1%); curated credits trade at 1% + curation fee.

**Recommendation**: The curation fee is a separate, additional fee applied only to trades of credits within curated collections. Proposed rate: 0.25% of trade value, paid by the buyer. The curation fee is not routed through M013's four-pool distribution -- it goes directly to the curator's reward pool (pro-rata based on the curator's score contribution to the traded collection). M013's base trade fee (1%) remains unchanged and follows standard pool distribution. This creates a clear market signal: curated credits cost slightly more but carry quality assurance.

**Rationale**: Keeps M013 revenue streams intact while creating a separate, self-sustaining incentive for curation. Buyers who value quality assurance pay a modest premium; buyers who don't can trade uncurated credits at the base rate. The curation fee flows directly to curators, creating a tight incentive loop.

**Status**: RESOLVED

---

### OQ-M011-4: Curator Reputation via M010

**Question**: Should there be a "curator reputation" tracked via M010, and should poorly-performing curators face increasing bond requirements?

**Analysis**:

M010 (Reputation Signal) is designed as a general-purpose reputation layer for network participants. Extending it to track curator performance is a natural fit. Curator reputation can be derived from objective on-chain signals: how often a curator's quality scores are challenged and overturned, how their curated collections perform in the marketplace (trading volume, buyer complaints), and how their scores correlate with eventual credit retirement rates (a proxy for ecological impact realized).

The case for increasing bond requirements for low-reputation curators follows the logic of risk-based collateral: a curator who has demonstrated poor judgment should post more collateral to compensate for the higher risk of future poor decisions. This is analogous to how insurance premiums increase for riskier profiles. It creates a natural exit ramp -- a consistently poor curator will eventually face bond requirements that exceed the economic value of curation, and they will voluntarily exit.

The counterargument is that increasing bond requirements can create a "death spiral" for curators who make a single honest mistake. If one overturned score significantly increases the bond requirement, it could force out a generally competent curator. The mechanism needs a forgiveness component -- reputation should decay toward neutral over time, and the bond increase should be proportional to the severity and frequency of infractions, not triggered by isolated incidents.

The implementation cost is modest. M010 already tracks reputation scores; adding a curator dimension requires defining the input signals and weights. The bond adjustment mechanism can be a simple multiplier: `required_bond = base_bond * max(1.0, 2.0 - curator_reputation_score)`, where reputation ranges from 0 to 1. A perfect curator (1.0) pays the base bond; a curator with zero reputation pays double.

**Recommendation**: Track curator reputation via M010. Input signals: (1) challenge success rate against the curator (weight 0.4), (2) trading volume of curated collections vs. uncurated (weight 0.3), (3) buyer satisfaction signals -- repeat purchases, complaints (weight 0.3). Reputation score ranges from 0 to 1, computed per epoch. Bond requirement scales as `base_bond * (2.0 - reputation_score)`, clamped to [base_bond, 2 * base_bond]. Reputation decays toward 0.5 (neutral) at a rate of 10% per epoch (approximately quarterly) to prevent permanent stigma from isolated incidents.

**Rationale**: Creates accountability for curators using the existing reputation infrastructure. The decay mechanism prevents permanent penalization. The bond multiplier caps at 2x to avoid making curation economically impossible for recovering curators.

**Status**: RESOLVED

---

### OQ-M011-5: Basket Token Handling in Collections

**Question**: How to handle basket tokens (from `x/ecocredit/basket`) in collections -- should the quality score apply to the basket or to its constituent batches?

**Analysis**:

Basket tokens in Regen Ledger represent fungible tokens backed by a pool of ecological credit batches. When a user deposits credits into a basket, they receive fungible basket tokens; when they redeem, they receive credits from the basket pool. Baskets are designed to increase liquidity by making heterogeneous credits fungible within a basket's acceptance criteria.

Applying a quality score at the basket level is simpler but less informative. A basket may contain batches of varying quality, and a single aggregate score obscures this variation. A basket with one excellent batch and one poor batch might receive a mediocre aggregate score, which misrepresents both constituent batches.

Applying quality scores to constituent batches is more accurate but creates a conceptual problem: basket tokens are fungible. When a user holds basket tokens, they do not own specific batches -- they own a claim on the pool. If individual batches within the basket have different quality scores, the basket token's quality is indeterminate until redemption. This undermines the purpose of fungibility.

The resolution is a layered approach. Quality scores are assessed and stored at the batch level (this is where ecological quality is meaningful -- individual batches from specific projects with specific vintages). The basket itself receives a computed score that is the weighted average of its constituent batch scores, weighted by the credit quantity of each batch in the basket. This basket-level score updates dynamically as batches are added to or removed from the basket. Collections can include either individual batches (with their direct quality scores) or basket tokens (with their computed aggregate scores).

For curators, this means they assess batches, not baskets. The basket score is a derived metric. This is computationally manageable because basket composition changes are on-chain events that can trigger a score recomputation.

**Recommendation**: Quality scores are assessed at the batch level. Baskets receive a dynamically computed aggregate score: `basket_score = sum(batch_score_i * batch_quantity_i) / sum(batch_quantity_i)` for all batches in the basket. Collections can include basket tokens using their aggregate score. When a batch is added to or removed from a basket, the basket's aggregate score is recomputed. The aggregate score is stored on-chain alongside the basket state for efficient querying. Curators who curate individual batches indirectly influence basket scores through their batch-level assessments.

**Rationale**: Preserves the ecological meaningfulness of quality assessment (at the batch level where provenance and methodology matter) while supporting basket tokens in the collection framework through deterministic aggregation. Dynamic recomputation ensures the basket score reflects current composition.

**Status**: RESOLVED

---

## M012: Fixed Cap Dynamic Supply

### OQ-M012-1: Exact Hard Cap Value

**Question**: The exact hard cap value. Token-economics-synthesis says "approximately 221M" based on current total supply (~224M). Should the cap be set at current total supply, slightly below (to create immediate scarcity), or at a round number?

**Analysis**:

The hard cap is M012's most consequential parameter -- it is a Layer 4 Constitutional governance parameter requiring supermajority to change. Getting it right at launch is critical because changing it later will be politically difficult and could undermine credibility.

Setting the cap below current total supply (~224M) at 221M creates immediate burn pressure: approximately 3M REGEN must be burned before any new minting can occur. This sends a strong deflationary signal and creates urgency for fee-generating economic activity. However, it also means the network is in a "deficit" from day one, which could create anxiety among holders and pressure to reduce the burn share to reach equilibrium faster. It also raises questions about whose tokens are "above the cap" -- the answer is no one's specifically, since it is circulating supply that exceeds the cap, but the perception matters.

Setting the cap at current total supply (~224M) means M012 activates neutrally: supply is at the cap, minting is initially zero (since M[t] = r * (C - S[t]) = r * 0 = 0), and the system begins as burn-only until supply drops below the cap. This is the most conservative option and the easiest to explain: "we cap at where we are today, and supply can only grow back toward today's level after burning." The downside is that there is no immediate minting, which means M012's regrowth algorithm is dormant until enough burning occurs.

A round number (e.g., 225M) has cosmetic appeal and provides a small mint buffer from the start. But choosing a round number above current supply would appear to sanction future dilution, which contradicts the "fixed cap" narrative.

The strongest option is to set the cap at current total supply at the moment of M012 activation, rounded to the nearest million. This is approximately 224M (the exact number will depend on inflation between now and activation). It starts the system neutrally, avoids the perception issues of an arbitrary number, and links the cap to a concrete observable fact rather than an abstract target.

**Recommendation**: Set `hard_cap` to the total supply at the block height of M012 activation, rounded down to the nearest 1,000,000 REGEN. Based on current supply projections (accounting for ongoing inflation until M012 activates), this is expected to be approximately 224,000,000 REGEN. The exact value is determined programmatically during the migration handler, not hardcoded in the governance proposal. This removes any ambiguity and makes the cap a direct function of observable state.

**Rationale**: Anchoring the cap to actual supply at activation is the most defensible choice -- it requires no arbitrary selection and starts the system in a neutral state. Rounding down to the nearest million provides a trivial initial burn requirement (< 1M REGEN) that demonstrates the mechanism works without creating material scarcity shock. Programmatic determination during migration removes the need to predict future supply.

**Status**: RESOLVED

---

### OQ-M012-2: Ecological Multiplier Oracle Source

**Question**: The ecological multiplier oracle. What data source provides the ecological metric? Is this sourced from on-chain attestation data (M008) or from an external oracle? The v0 spec disables this until resolved.

**Analysis**:

The ecological multiplier in M012's regrowth formula ties minting rates to real-world ecological outcomes -- a distinctive feature that differentiates Regen's supply model from purely financial algorithmic supply mechanisms. However, it requires a reliable, manipulation-resistant data source for ecological metrics.

External oracles (e.g., Chainlink-style price feeds adapted for ecological data) have the advantage of being general-purpose and battle-tested for price data. But ecological metrics are fundamentally different from financial prices: they are slow-moving (annual or seasonal), measurement-methodology-dependent, and lack the deep market-based verification that price feeds enjoy. There is no "ecological metric market" that creates consensus on the number. External oracle manipulation is therefore easier for ecological data than for high-liquidity financial data.

M008 attestation data is the more natural fit for Regen. M008 is specifically designed to create bonded attestations of ecological data quality. Attesters stake REGEN tokens and face slashing for false claims. This creates an economic incentive layer around data quality that is missing from external oracles. Moreover, M008 attestation data is already on-chain and integrated with the broader Regen ecosystem (KOI for evidence storage, Arbiter DAO for dispute resolution).

The v0 decision to disable the ecological multiplier (set to 1.0) is correct. M008 itself is being specified in Phase 2 and will not have sufficient data density until well after M012 activation. The ecological multiplier should be enabled only after M008 has been operational for a minimum period (e.g., 12 months) and sufficient attestation data exists to compute a meaningful aggregate metric.

When enabled, the metric should be derived from aggregate M008 attestation data: the volume and quality of ecological claims attested and validated on the network. This is a measure of "ecological activity passing through Regen" rather than a measure of global ecological outcomes, which is more appropriate for an on-chain supply mechanism.

**Recommendation**: v0: ecological_multiplier = 1.0 (disabled), as specified. v1 (targeted 12+ months after M008 activation): derive the ecological multiplier from M008 aggregate attestation metrics. Specifically: `ecological_multiplier = min(2.0, 1.0 + (attested_ecological_value_this_period / reference_value))`, where `attested_ecological_value` is the sum of ecological credit value attested via M008 in the period, and `reference_value` is a governance-set target (Layer 3). This replaces the original CO2 delta formula with an on-chain-native metric. Do not use external oracles for v1.

**Rationale**: M008 attestation data is native to the ecosystem, economically bonded (resistant to manipulation), and already on-chain. The reformulated multiplier measures "ecological activity on Regen" rather than global ecological state, which is both more measurable and more directly relevant to the network's supply dynamics. The 12-month minimum operational period for M008 ensures sufficient data density before the multiplier is enabled.

**Status**: RESOLVED

---

### OQ-M012-3: Period Length for Mint/Burn Cycles

**Question**: Period length for mint/burn cycles. Is per-epoch (weekly) the right cadence, or should it be per-block (like EIP-1559) for finer granularity?

**Analysis**:

Ethereum's EIP-1559 adjusts the base fee per-block, creating highly responsive supply dynamics. Regen's M012 could adopt a similar approach, computing mint/burn at every block. However, the two systems face fundamentally different conditions.

Ethereum processes thousands of transactions per block, each paying gas fees. The per-block data is rich and statistically meaningful. Regen's ecological credit transactions are far less frequent -- possibly dozens to hundreds per day in early operation. Per-block computation on low-frequency data creates noisy, volatile supply adjustments. A single large credit issuance in one block could cause a disproportionate burn, followed by many blocks of near-zero activity. This volatility does not serve the "homeostatic equilibrium" design philosophy of M012.

Weekly (7-day epoch) computation smooths these fluctuations. It accumulates a meaningful sample of transactions before computing mint/burn, producing more stable supply dynamics. It is also simpler to implement, monitor, and debug. Validators and governance participants can observe each epoch's mint/burn outcome and understand the supply trajectory.

The middle ground -- daily computation -- offers more responsiveness than weekly without the noise of per-block. However, it adds six additional computation events per week with minimal benefit over weekly in a low-frequency transaction environment.

As network activity grows, the cadence can be shortened. This is a Layer 2 operational parameter that can be adjusted without major governance overhead. Starting with weekly and reducing to daily or per-block as transaction volume justifies it is the prudent approach.

**Recommendation**: Set `period_length` to 1 epoch (7 days) for launch. Add a governance parameter `min_period_transactions` (Layer 2, initial value: 100) that defines the minimum number of fee-generating transactions required in a period for mint/burn to execute. If a period has fewer transactions than this threshold, the mint/burn computation is deferred to the next period (with accumulated fees). This prevents noisy adjustments during low-activity periods. Provide a governance path to reduce period_length to 1 day (Layer 2) or per-block (Layer 3) as transaction volume grows.

**Rationale**: Weekly epochs match the current transaction volume reality. The minimum transaction threshold prevents degenerate behavior during quiet periods. The clear upgrade path to shorter periods ensures the system can evolve with network growth without requiring a fundamental redesign.

**Status**: RESOLVED

---

### OQ-M012-4: Burned Tokens -- Permanent Destruction vs. Reserve Pool

**Question**: Should burned tokens be permanently destroyed or sent to a reserve pool that can be re-minted under governance control?

**Analysis**:

Permanent burn means tokens sent to the burn address are irreversibly removed from circulation. The total supply (as tracked by the supply module) decreases permanently. The only way to increase supply is through M012's minting algorithm, which is capped by the hard cap and governed by the regrowth formula.

A reserve pool sends "burned" tokens to a governance-controlled address where they exist but are not in circulation. Governance could vote to release tokens from the reserve, effectively bypassing the mint algorithm. This provides an emergency valve -- if the network needs to inject liquidity (e.g., during a crisis or to fund a critical initiative), it can do so from the reserve rather than waiting for the mint algorithm to produce tokens.

The reserve pool approach fatally undermines M012's credibility. The hard cap and mint algorithm create a predictable, rules-based monetary policy. A governance-accessible reserve pool introduces discretionary monetary policy by another name. Holders cannot trust the supply trajectory if governance can override it by releasing reserves. Every governance proposal to release reserves would become a contentious, speculative event. The reserve pool also makes the burn mechanism less meaningful -- "burning" to a reserve that can be unburned is not really burning.

The minting algorithm already provides the emergency valve. If supply drops too low (S[t] is far below the hard cap), the regrowth formula M[t] = r * (C - S[t]) produces increasingly large mints to restore supply toward the cap. This is the designed response to over-burning. A reserve pool is an unnecessary override of this designed mechanism.

Permanent burn is simpler to implement (just send to a burn address with no keys), easier to verify (anyone can check the burn address balance), and more credible to external observers (exchanges, investors, regulators).

**Recommendation**: Burned tokens are permanently destroyed. No reserve pool. The burn address is a module account with no signing keys from which tokens can never be recovered. The M012 regrowth algorithm is the sole mechanism for increasing supply. If the community determines that burn rates are too aggressive, the governance recourse is to adjust M013 burn_share (Layer 2-3) or M012 base_regrowth_rate (Layer 3), not to unburn tokens.

**Rationale**: Permanent burn is the only credible form of burn. A reserve pool creates discretionary monetary policy that undermines the rules-based system M012 is designed to be. The existing regrowth algorithm and governance-adjustable parameters provide sufficient flexibility without a reserve.

**Status**: RESOLVED

---

### OQ-M012-5: Staking vs. Stability vs. Validator Participation Multiplier

**Question**: Should the staking_multiplier be replaced by a stability_multiplier (from M015 commitments) or a validator_participation_multiplier (from M014 active set health)?

**Analysis**:

The M012 spec already defines phase-gated behavior for the multiplier transition, but the question of which replacement is best deserves explicit resolution. The current staking_multiplier (range [1.0, 2.0]) increases regrowth when more tokens are staked, rewarding commitment. Under PoA, staking is disabled, so a new signal is needed.

The stability_multiplier derives from M015 stability tier commitments: holders who lock tokens for 6-24 months signal long-term commitment. Using this as the multiplier preserves the original intent -- reward commitment with faster regrowth -- while aligning with the new economic model. It creates a positive feedback loop: more stability commitments increase the multiplier, leading to more minting, leading to more tokens available for distribution (via M015), potentially leading to more stability commitments.

The validator_participation_multiplier derives from M014 active set health: how many validators are active, their uptime scores, their governance participation. This links supply health to network security health. If validators are underperforming (low uptime, missed governance), regrowth slows. This is conceptually elegant but creates a problematic dependency: supply dynamics should not be sensitive to the operational performance of 15-21 specific entities. A single validator going down for maintenance should not measurably affect the token supply algorithm.

The spec's existing phase-gated logic -- using `max(staking, stability)` during transition and `stability` alone post-transition -- is well-designed. The stability_multiplier is the correct replacement because it measures broad holder commitment (potentially thousands of participants) rather than a small validator set's performance. It preserves the "reward commitment" semantics of the original staking_multiplier in a post-staking world.

**Recommendation**: Confirm the spec's existing design: the stability_multiplier (from M015 stability tier commitments) replaces the staking_multiplier post-PoA. The phase-gated transition logic remains as specified: `max(staking, stability)` during M014 transition, `stability` alone after PoS is disabled. The validator_participation_multiplier is not used for M012. Instead, validator participation is governed through M014's own mechanisms (probation, removal, compensation adjustments).

**Rationale**: The stability_multiplier preserves the original "reward commitment" intent, draws from a broad participant base (not a small validator set), and aligns with M015's contribution-weighted reward model. The validator_participation_multiplier would create an inappropriate coupling between a small set of operators and the network's monetary policy.

**Status**: RESOLVED

---

## M013: Value-Based Fee Routing

### OQ-M013-1: Distribution Model A vs. B

**Question**: Which distribution model should be adopted? Model A provides a dedicated Agent Infrastructure fund; Model B routes a larger share through governance.

**Analysis**:

Model A ({30% burn, 40% validator, 25% community, 5% agent}) comes from the token-economics-synthesis and emphasizes validator compensation (40%) and a dedicated agent fund. Model B ({25-35% burn, 15-25% validator, 50-60% community}) comes from Gregory's Network Coordination Architecture and emphasizes community-directed distribution with no separate agent fund.

The fundamental tension is between operational efficiency (Model A) and governance control (Model B). Model A pre-commits to specific allocations, reducing governance overhead but limiting flexibility. Model B routes the majority through the community pool, giving governance maximum control but requiring more active governance to direct funds effectively.

PR #49's compromise proposal of {28% burn, 25% validator, 45% community, 2% agent} attempts to bridge the gap. It reduces the burn share from Model A's 30% to 28%, significantly reduces validator share from 40% to 25%, increases community share from 25% to 45%, and reduces the agent fund from 5% to 2%. However, 25% for validators may be insufficient. With a 15-21 validator set, the validator fund needs to cover operational costs (hardware, bandwidth, monitoring), not just be a nominal allocation. At $50K/month total fee revenue, 25% = $12.5K/month divided among 15 validators = $833/validator/month, which barely covers infrastructure costs. At 40%, each validator receives ~$1,333/month, which is more sustainable though still modest.

The agent infrastructure fund at 2% is likely too small to be operationally meaningful. At $50K/month revenue, 2% = $1K/month for all agent operations. Agent infrastructure includes compute, API costs, model inference, and monitoring. This is almost certainly insufficient. Either the agent fund should be meaningful (5-8%) or it should be eliminated and funded from the community pool via governance proposals, as in Model B.

**Recommendation**: Adopt a modified compromise: {15% burn, 28% validator, 45% community, 12% agent}. This reflects the resolution of OQ-M013-5 (reduced burn share at 15%, see below), maintains a meaningful validator allocation, provides the community pool with the largest share for M015 distribution and governance-directed spending, and gives the agent infrastructure fund a viable operating budget. All shares are Layer 2 governance parameters adjustable within bounds (each share can range +/- 10% from initial value, Layer 2; beyond that range requires Layer 3). The sum must always equal 100%.

**Rationale**: The 15% burn share represents the compromise position from OQ-M013-5 analysis below. The 28% validator share ensures each of 15-21 validators receives meaningful compensation. The 45% community share gives M015 a strong revenue base. The 12% agent fund provides viable operational funding without requiring constant governance proposals. Adjustable parameters allow the community to fine-tune as real revenue data becomes available.

**Status**: NEEDS_GOVERNANCE -- The exact distribution ratios represent significant economic policy. This recommendation provides a starting position, but the WG should discuss and the community should ratify via governance vote.

---

### OQ-M013-2: Credit Value Determination for Non-Marketplace Transactions

**Question**: How is credit value determined for non-marketplace transactions (issuance, transfer, retirement)?

**Analysis**:

For marketplace trades, value is explicit: the sell order price multiplied by quantity. For issuance, transfer, and retirement, there is no explicit price in the transaction. Yet M013 needs a value to compute the percentage-based fee. Three options are on the table:

Option A (most recent marketplace price) uses the last trade price for that credit class as the reference. This is the Time-Weighted Average Price (TWAP) approach. It is fully on-chain and requires no external data. However, for credit classes with thin or no trading history, there is no price to reference. Early issuances of a new credit class would have no reference price.

Option B (governance-set reference price) has governance define a reference price per credit type. This is reliable but requires active governance maintenance and may not reflect market reality. It could become a political tool (setting low reference prices to reduce fees).

Option C (external oracle via KOI) uses off-chain market data (e.g., prices from voluntary carbon markets). This is the most accurate for widely-traded credit types but introduces oracle dependency and is unavailable for novel credit types.

The practical solution is a waterfall: try TWAP first (most market-accurate), fall back to governance-set reference price (reliable), and use a minimum fee floor (ensures some fee collection even without a price reference).

For TWAP computation, the spec should use a 7-day TWAP rather than the last trade price. A single trade price is manipulable (a single wash trade at an extreme price would set the reference). A 7-day TWAP requires sustained manipulation and is much more robust.

**Recommendation**: Implement a price reference waterfall:
1. **Primary**: 7-day TWAP from `x/ecocredit/marketplace` for the credit class. Requires at least 3 trades in the 7-day window to be considered valid.
2. **Secondary**: Governance-set reference price per credit class (governance parameter, Layer 2). Updated quarterly or on significant market events.
3. **Tertiary**: Minimum fee floor (1 REGEN per transaction, as specified in M013).

For new credit classes with no trading history and no governance-set reference price, issuance fees default to the minimum fee floor until either trades occur or governance sets a reference price. This ensures fee collection begins immediately at launch even for untested credit classes, while transitioning to market-based pricing as trading activity emerges.

**Rationale**: The waterfall approach handles all edge cases -- active markets, thin markets, and no-market credit classes -- without requiring external oracle dependency. The 7-day TWAP with a 3-trade minimum prevents manipulation. Governance-set prices provide a fallback that the community controls. The minimum fee floor ensures M013 always generates some revenue.

**Status**: RESOLVED

---

### OQ-M013-3: Fee Denomination (REGEN-only vs. Native Denom vs. Hybrid)

**Question**: In what denomination should fees be collected and distributed?

**Analysis**:

This is one of the most consequential design decisions for the token's economic role. The three options have fundamentally different implications for REGEN's position in the ecosystem.

REGEN-only collection means every fee must be paid in REGEN. This forces all marketplace participants to acquire REGEN, creating natural buy pressure and ensuring the token is an essential medium of exchange. However, most credit marketplace transactions settle in stablecoins (USDC, EEUR). Requiring REGEN creates friction: buyers must swap stablecoins to REGEN to pay fees, adding a step, introducing slippage, and creating an exposure to REGEN price volatility that institutional buyers may find unacceptable.

Native denom collection accepts fees in whatever currency the transaction uses. This is zero-friction for users but creates a multi-denomination treasury. The burn pool would contain stablecoins, which cannot be "burned" in the M012 sense (burning USDC does not reduce REGEN supply). The validator fund and community pool would hold a mix of denoms, complicating accounting and distribution.

The hybrid approach collects fees in the native transaction denom and auto-converts the burn share to REGEN via DEX (e.g., Osmosis pools) before burning, while distributing the remaining shares in the collected denom. This is the most practical path. Users pay fees in their preferred currency (no friction), the burn mechanism functions correctly (REGEN is bought and burned, creating deflationary pressure), and recipients receive fees in the currency that was actually paid (stable value for operational expenses).

On the distribution side: validators and agent infrastructure operators have real-world costs denominated in fiat. Distributing in stablecoins is immediately useful for covering these costs. Forcing conversion to REGEN and then requiring recipients to sell REGEN to cover costs creates unnecessary sell pressure. The hybrid approach of distributing in the native fee denom (mostly stablecoins) with an optional "choose REGEN" bonus (e.g., 5% premium for electing REGEN distribution) is most aligned with the ecosystem's needs.

**Recommendation**: Hybrid fee denomination:
- **Collection**: Fees are collected in the native transaction denomination (stablecoin or REGEN).
- **Burn share**: Auto-converted to REGEN via on-chain DEX (Osmosis IBC) and burned. This is the primary source of REGEN buy pressure.
- **Validator/Community/Agent shares**: Distributed in the collected denomination. Recipients receive stablecoins when fees are paid in stablecoins, REGEN when fees are paid in REGEN.
- **REGEN election bonus**: Recipients of Community Pool (M015) and Agent Fund distributions can elect to receive their share in REGEN instead of stablecoins, with a 5% premium (funded from Community Pool reserves). This incentivizes REGEN accumulation without forcing it.

The DEX integration for burn-share conversion requires an IBC swap module or integration with Osmosis's concentrated liquidity pools. This should be implemented as a standalone `x/feerouter` sub-module that can be upgraded independently.

**Rationale**: Minimizes friction for marketplace users (pay in whatever you are already using), maintains REGEN deflationary mechanics (burn share is always REGEN), provides stable-value compensation to operators (validators and agents can cover costs), and creates an opt-in REGEN accumulation incentive. This positions REGEN as a governance and coordination token rather than a mandatory medium of exchange.

**Status**: RESOLVED

---

### OQ-M013-4: Agent Infrastructure Fund Governance

**Question**: How should the Agent Infrastructure fund be governed?

**Analysis**:

The Agent Infrastructure fund receives a dedicated share of M013 fee revenue (proposed 12% per OQ-M013-1 resolution) to cover operational costs of the agent system: compute, model inference, API access, monitoring, and development. The question is whether this fund should be a separate module account with its own spending authority or a tagged allocation within the Community Pool subject to standard governance proposals.

A separate module account provides operational independence: agent infrastructure operators can draw from the fund without submitting governance proposals for each expense. This is important because agent operations are continuous -- compute bills arrive monthly, not on governance proposal schedules. Requiring a governance proposal for each expense would create cash flow gaps and operational disruptions.

However, a fully autonomous module account with no oversight is a blank check. The fund could be misused or spent inefficiently without accountability. The balance is to create a separate module account with constrained spending authority and periodic governance oversight.

The recommended governance model is a multisig initially (during the early phase when the agent system is being built by a small team) with a transition plan to full governance control as the agent ecosystem matures. The multisig should include representatives from the core development team, the validator set, and community-elected members.

**Recommendation**: Create the Agent Infrastructure Fund as a separate module account (`x/agent_infra_fund`). Governance model phases:
- **Phase 1 (0-12 months)**: 3-of-5 multisig controls spending. Multisig members: 2 from core development team (RND), 1 from authority validator set (elected by validators), 2 from community (elected via governance proposal). Monthly spending reports published on-chain (anchored via KOI).
- **Phase 2 (12+ months)**: Transition to governance-approved annual budgets. The multisig submits an annual spending plan via governance proposal (Layer 3). Once approved, the multisig can execute within the approved budget without per-expense governance. Unspent budget rolls over. Exceeding the approved budget requires a new governance proposal.
- **Guardrail**: Maximum single expenditure of 25% of current fund balance without a separate governance approval. This prevents fund depletion from a single decision.

**Rationale**: The phased approach balances operational agility (multisig can respond quickly) with accountability (community oversight, published reports, annual budgets). The 3-of-5 multisig with diverse representation prevents capture by any single constituency. The spending cap prevents catastrophic misuse.

**Status**: RESOLVED

---

### OQ-M013-5: Should the Burn Pool Exist?

**Question**: Should the Burn Pool exist at all, and if so, at what share?

**Analysis**:

This is the most philosophically contentious question in the entire specification suite. It gets to the heart of what REGEN is: a coordination mechanism for ecological regeneration, or a capital formation instrument that incidentally funds regeneration?

The contributor-first argument against burning is compelling in its values alignment. Every token burned is a token that could have been distributed to someone actively building the network -- a verifier, a credit originator, a dApp developer, a governance participant. Burning transfers value to passive holders through supply reduction, which is effectively a subsidy to speculators. For a mission-driven network, maximizing resources directed to active contributors seems more aligned than creating scarcity dynamics.

The capital formation argument for burning is compelling in its pragmatism. Early-stage networks need external capital to fund infrastructure, and speculative interest -- driven partly by deflationary tokenomics -- is a powerful bootstrap mechanism. Token price appreciation (partly from burn pressure) increases the purchasing power of REGEN-denominated rewards, benefiting contributors indirectly. Moreover, burn creates a credible commitment that the protocol will not inflate away value, which is important for attracting long-term holders who provide governance stability.

The resolution is not binary. The question is not "burn or no burn" but "how much burn, and should it change over time?"

A 25-35% burn share (as in both Models A and B) is aggressive. At $50K/month revenue, that is $12.5K-$17.5K/month in burned tokens -- meaningful but modest in absolute terms. The deflationary signal to speculators is present but unlikely to materially affect token price at that scale. Meanwhile, those same tokens directed to contributors could fund multiple additional grants or operational budgets.

A 0% burn share eliminates the deflationary mechanism entirely, relying solely on M012's regrowth formula to manage supply dynamics. This is conceptually clean but removes a credible commitment to value preservation that some stakeholders (particularly early investors and large holders) value.

A moderate burn share (10-15%) provides a meaningful deflationary signal while keeping the majority of fee revenue in productive use. It represents a clear compromise position. The burn share can be dynamically adjusted via governance (Layer 2 within +/- 10%, Layer 3 beyond) as the network matures and fee revenue grows. Starting moderate and adjusting based on data is more prudent than starting aggressive and needing to reduce.

**Recommendation**: Establish the Burn Pool at 15% of fee revenue. This provides a meaningful but modest deflationary mechanism while directing 85% of fee revenue to productive uses (validators, community, agents). The burn share is a Layer 2 governance parameter within the range [5%, 25%]; changes outside this range require Layer 3 governance. This allows the community to increase burn during periods of excess revenue or decrease it during periods when contributor funding is scarce, without requiring constitutional-level governance.

**Rationale**: 15% represents the pragmatic middle ground. It is high enough to provide a credible deflationary signal and support M012's burn-side dynamics, but low enough that 85% of revenue flows to people doing productive work. The [5%, 25%] governance range gives the community significant flexibility to adjust without constitutional governance. If the network reaches high fee revenue levels ($500K+/month), even 15% provides substantial burn; if revenue is low, the community can reduce it further.

**Status**: NEEDS_GOVERNANCE -- While 15% is a well-reasoned starting point, the burn share involves deep value judgments about the network's purpose. The WG should deliberate and the community should ratify.

---

## M014: Authority Validator Governance

### OQ-M014-1: Exact Validator Set Size

**Question**: Exact validator set size. The WG discusses 15-21. What is the right target, and should it be fixed or allowed to float within the range?

**Analysis**:

The validator set size determines three things: security (Byzantine fault tolerance requires > 2/3 honest validators), operational resilience (more validators = more redundancy), and governance agility (fewer validators = faster coordination).

Tendermint BFT requires > 2/3 honest validators. For 15 validators, the network tolerates up to 4 Byzantine validators. For 21, it tolerates up to 6. In a PoA model where validators are vetted and compensated, the probability of Byzantine behavior is significantly lower than in an open PoS system, so the practical difference between 15 and 21 is minimal for security.

The composition requirements in M014 specify minimums of 5 infrastructure builders, 5 ReFi partners, and 5 ecological data stewards, totaling 15 minimum. This is already the `min_validators` parameter. Having exactly 15 leaves zero slack -- if any category loses one validator, the composition guarantee is violated. A target of 18 (6/6/6) provides one slot of buffer per category.

A floating range [15, 21] with `min_validators = 15` and `max_validators = 21` as Layer 4 parameters allows organic growth. The network starts at whatever size the seed set provides (likely 15-18 based on qualifying applicants) and can grow to 21 as new qualified validators apply. This avoids the need to artificially fill all 21 slots at launch while providing room to grow.

**Recommendation**: Set `min_validators = 15` and `max_validators = 21` as Layer 4 Constitutional parameters. Set `target_validators = 18` as a Layer 3 parameter representing the desired operational set size. The composition minimum per category is 5. Seed the network at whatever size the initial qualifying set supports (expected 15-18). New validators are admitted through M014's standard governance process as slots become available. The target of 18 provides one buffer slot per category beyond the minimum of 5.

**Rationale**: The floating range avoids the need to find exactly 21 qualified validators at launch (which may be difficult) while ensuring the set can grow as the ecosystem matures. The target of 18 provides composition resilience. The Layer 4 designation for min/max ensures structural changes require supermajority, while the Layer 3 target allows operational adjustment.

**Status**: RESOLVED

---

### OQ-M014-2: Performance Bonus Existence

**Question**: Should a performance bonus exist, or should all validators receive equal compensation?

**Analysis**:

The M014 spec proposes a 10% performance bonus pool based on uptime (0.4), governance participation (0.3), and ecosystem contribution (0.3). The question is whether this complexity is worth the benefit.

Arguments for the bonus: it incentivizes operational excellence, prevents "coasting" by validators who meet minimum requirements but do not invest in improvement, and creates a measurable framework for distinguishing validators during re-application.

Arguments against: in a curated 15-21 validator set, all validators have already been vetted for commitment and capability. The bonus pool is small (10% of the validator fund, so perhaps $100-200/month per validator at maximum). This amount is unlikely to meaningfully change behavior for organizations operating validators as a mission-aligned activity. Meanwhile, the ecosystem contribution metric (weight 0.3) is subjective and introduces gaming -- validators could optimize for measurable but low-impact contributions.

The stronger approach is to use performance metrics for accountability (probation, removal) rather than compensation. Validators who fall below minimum uptime or governance participation face probation per M014's lifecycle. Validators who consistently exceed expectations have a strong record for re-application. This achieves the same behavioral alignment without the compensation complexity and gaming surface.

**Recommendation**: Do not implement the performance bonus for v0. All active authority validators receive equal base compensation from the validator fund (`validator_fund_balance / active_validator_count / period`). Use M014's existing performance metrics for accountability purposes: validators below minimum thresholds face probation; validators with strong records receive streamlined re-application. Revisit the performance bonus after 12 months of operation with data on actual validator behavior patterns.

**Rationale**: In a small, curated, mission-aligned validator set, the accountability mechanism (probation, removal) is more effective than a modest financial bonus. Equal compensation simplifies the system, eliminates gaming incentives, and reinforces the PoA philosophy that all authority validators contribute equally. The 12-month revisit window allows the community to introduce bonuses later if data shows a need.

**Status**: RESOLVED

---

### OQ-M014-3: Initial Trusted Partner Determination

**Question**: How is "trusted partner" status determined during the initial transition? Who constitutes the seed set?

**Analysis**:

The seed set of authority validators is the bootstrap problem: you need a validator set to run governance, but governance is how you admit validators. This circular dependency must be resolved by an external mechanism.

The most legitimate approach is to bootstrap from the existing active validator set. The current Regen Network has approximately 75 validators, of whom a subset are actively maintaining infrastructure, participating in governance, and contributing to the ecosystem. These validators have a demonstrated track record that can be evaluated against M014's composition criteria.

The process should be:

1. **Application**: Open a formal application window where current validators (and external organizations meeting M014 criteria) can apply for authority validator status. The application includes: organization name, category (infrastructure/ReFi/data steward), track record evidence, infrastructure specifications, and commitment statement.

2. **Evaluation**: A temporary selection committee -- composed of current core contributors, active governance participants, and the Tokenomics WG -- evaluates applications against M014's published criteria. This committee is not permanent governance; it is a one-time bootstrap mechanism.

3. **Ratification**: The proposed seed set is published for community review (14 days) and ratified via a standard governance proposal under the current PoS model. This ensures the transition has democratic legitimacy.

4. **Activation**: Upon ratification, the M014 module is activated with the seed set.

This process uses the existing governance infrastructure (PoS governance) to legitimize the transition to the new model (PoA), avoiding any perception of a unilateral power grab.

**Recommendation**: Bootstrap the seed set through a formal process:
1. 30-day open application window for authority validator candidacy.
2. Evaluation by a temporary Selection Committee (3 core contributors, 2 Tokenomics WG members, 2 community-elected representatives) against M014's published criteria.
3. 14-day community review of the proposed seed set.
4. Ratification via governance proposal under current PoS model (standard threshold: simple majority, 40% quorum).
5. Upon ratification, M014 activates with the seed set.

Priority consideration (but not automatic inclusion) for current active validators who meet M014 criteria and have maintained > 95% uptime over the preceding 6 months.

**Rationale**: Using the existing PoS governance to ratify the seed set provides democratic legitimacy. The Selection Committee provides expert evaluation while the community review and governance vote provide broad accountability. Priority consideration for existing validators honors their contribution without bypassing the evaluation process.

**Status**: NEEDS_GOVERNANCE -- The process is well-defined, but the Selection Committee composition and specific evaluation criteria require WG deliberation and community consensus.

---

### OQ-M014-4: PoA Socialization Timeline

**Question**: PoA socialization timeline. What is the target activation date?

**Analysis**:

Gregory noted that PoA was first socialized approximately 18 months ago (mid-2024). The Economic Reboot Roadmap v0.1 suggests Q3-Q4 2026 for a pilot and 2027 for full migration. The question is whether this timeline is still realistic given the current state of specification and implementation work.

As of March 2026, Phase 2 specifications are nearing completion (this document resolves the remaining open questions). Phase 3 (implementation specifications, smart contract specs, testing plans, security framework) has begun but is not complete. The implementation itself -- new Cosmos SDK modules (`x/authority`, `x/feerouter`, `x/supply`, `x/rewards`), CosmWasm contracts, agent infrastructure, and migration code -- has not started.

A realistic implementation timeline, assuming work begins Q2 2026:
- Q2 2026: Phase 3 completion (implementation specs, testing plans, security framework).
- Q3 2026: Module development and unit testing.
- Q4 2026: Testnet deployment and integration testing.
- Q1 2027: Security audit, bug fixes, community testing.
- Q2 2027: Mainnet governance proposal for activation.

This places the pilot at Q4 2026 (testnet) and mainnet activation at Q2 2027, which is approximately 6 months later than the Economic Reboot Roadmap's optimistic Q3-Q4 2026 target. However, this timeline is achievable if implementation resources are secured in Q2 2026.

The pilot should specifically mean M013 activation (fee routing) on mainnet, which can operate independently of M014 (PoA). M013 can collect fees under the existing PoS model. M014 activation is the more complex step that follows.

**Recommendation**: Updated timeline:
- **Q2 2026**: Complete Phase 3 specifications. Begin implementation. Open seed set application window.
- **Q3 2026**: M013 (fee routing) testnet deployment. Module development for M014.
- **Q4 2026**: M013 mainnet activation via governance proposal (pilot). M014 testnet deployment.
- **Q1 2027**: M014 seed set selection process. Security audit for M014 + M012.
- **Q2 2027**: M014 mainnet activation (PoA goes live). M012 + M015 testnet.
- **Q3 2027**: M012 + M015 mainnet activation. Full economic reboot operational.

M013 is the priority for early activation because it begins generating fee revenue that demonstrates the economic model before the more controversial PoA transition.

**Rationale**: Sequencing M013 before M014 de-risks the transition by establishing fee revenue flows before changing the consensus model. The 6-month delay from the original roadmap reflects realistic implementation timelines. Each mechanism activation is a separate governance proposal, allowing the community to pause if any step reveals issues.

**Status**: RESOLVED

---

### OQ-M014-5: Delegated REGEN Handling When PoS Disabled

**Question**: What happens to delegated REGEN when PoS is disabled?

**Analysis**:

When M014 activates and PoS is eventually disabled, all staked (delegated) REGEN must be unbonded. Currently, the Regen Network unbonding period is 21 days (standard Cosmos SDK). At the time of PoS disablement, all delegations are forced into unbonding simultaneously.

The immediate concern is a "cliff effect": all staked tokens becoming liquid at the same time could create massive sell pressure. As of current data, a significant portion of REGEN supply is staked. If all staked tokens unbond simultaneously, the circulating supply could increase dramatically over a short window.

The mitigation is a phased approach with extended notice:

1. **90-day advance notice**: A governance proposal announcing the PoS sunset date is passed at least 90 days before the target date. This gives delegators time to begin voluntary unbonding.

2. **Staggered unbonding**: Rather than forcing all delegations to unbond on the same date, implement a staggered unbonding schedule. Delegations are grouped into cohorts (e.g., by validator) and each cohort's unbonding begins on a different date, spread over 30 days. Each cohort still has the standard 21-day unbonding period, so the total window from first cohort start to last cohort completion is 51 days.

3. **Stability tier incentive**: During the advance notice period, aggressively promote the M015 stability tier as an alternative commitment mechanism. Holders who move from staking to stability tier commitments maintain a form of token lockup, reducing the circulating supply increase.

4. **No inflation during wind-down**: Once the PoS sunset governance proposal passes, staking rewards should be reduced (or eliminated) to prevent perverse incentives to remain staked until the last moment. This encourages early voluntary unbonding.

**Recommendation**: Implement a structured PoS sunset process:
1. **T-90 days**: Governance proposal passes announcing PoS sunset date. Staking rewards reduced to 50% of current rate.
2. **T-60 days**: Staking rewards reduced to 25%. Stability tier (M015) opens for commitments with a 10% bonus for early adopters who commit within this window.
3. **T-30 days**: Staking rewards reduced to 0%. Voluntary unbonding strongly encouraged.
4. **T-0 (sunset)**: All remaining delegations enter forced unbonding. Staggered by validator: top 1/3 of validators (by delegation) unbond days 1-21, middle 1/3 days 11-31, bottom 1/3 days 21-41. Total wind-down: 41 days.
5. **T+41 days**: All tokens fully liquid. PoS module disabled.

Communication plan: the 90-day advance notice should be accompanied by a dedicated communication campaign (forum posts, social media, direct validator notification) explaining the timeline, the stability tier alternative, and the rationale.

**Rationale**: The 90-day notice with graduated reward reduction incentivizes early voluntary unbonding, smoothing the liquidity increase. Staggered forced unbonding prevents a single-day cliff. The stability tier promotion channels committed holders into the new economic model. The 41-day total forced unbonding window is manageable and predictable.

**Status**: RESOLVED

---

## M015: Contribution-Weighted Rewards

### OQ-M015-1: Sustainability of 6% Stability Tier Return

**Question**: Is 6% the right stability tier return? At $50K/month fee revenue, 30% of the community share supports approximately $3M in committed stability deposits at 6%. Is this realistic?

**Analysis**:

The sustainability analysis requires working backward from expected fee revenue.

At $50K/month total fee revenue (conservative early estimate based on current Regen marketplace volume):
- Community Pool share (45% per OQ-M013-1 resolution): $22,500/month = $270,000/year.
- Stability tier cap (30% of Community Pool inflow): $6,750/month = $81,000/year.
- At 6% annual return: this supports $81,000 / 0.06 = $1,350,000 in stability commitments.

At $200K/month (optimistic growth scenario):
- Community Pool share: $90,000/month = $1,080,000/year.
- Stability tier cap: $27,000/month = $324,000/year.
- At 6%: supports $324,000 / 0.06 = $5,400,000 in stability commitments.

Is $1.35M-$5.4M in stability commitments realistic? At a REGEN price of $0.05-$0.10, this represents 13.5M-108M REGEN in the stability tier. Current total supply is approximately 224M. Locking 6-48% of total supply in stability commitments seems plausible for a long-term-oriented community but ambitious at the high end.

The 6% rate is attractive in the current market (higher than most DeFi stablecoin yields) but could become unsustainable if fee revenue grows slowly. The 30% cap on Community Pool inflow provides a natural brake -- if stability demand exceeds the cap, new commitments are queued. This prevents over-commitment.

A dynamic rate would be more robust: set the stability rate to be a function of available Community Pool revenue rather than a fixed 6%. However, variable rates undermine the "predictable returns" value proposition of the stability tier. The compromise is to set 6% as a target rate with a "revenue adequacy" check: if the stability allocation (30% of Community Pool inflow) cannot cover 6% returns for all committed tokens, the effective rate decreases proportionally. This makes 6% a maximum, not a guarantee.

**Recommendation**: Set the stability tier target return at 6% per annum with a revenue adequacy constraint. Effective rate = min(6%, (stability_allocation / total_stability_commitments)). The 30% cap on Community Pool inflow for stability remains as specified. If demand exceeds capacity, new commitments are queued (as specified) and the effective rate for existing commitments remains 6% until the cap is hit. Communicate this clearly: "up to 6% annual return, subject to network fee revenue." Review the target rate after 12 months of M015 operation with actual revenue data.

**Rationale**: The "up to 6%" framing is honest about the revenue dependency while maintaining an attractive target. The revenue adequacy constraint prevents the protocol from promising returns it cannot deliver. The 30% cap and queuing mechanism provide natural backpressure. The 12-month review allows adjustment based on actual data.

**Status**: RESOLVED

---

### OQ-M015-2: Platform Facilitation Identification

**Question**: Should platform facilitation credit use a metadata field on transactions to identify the facilitating platform, or should it be determined by the originating API key / registered dApp address?

**Analysis**:

Platform facilitation (weight 0.20 in M015) rewards entities that enable marketplace transactions -- brokers, dApp frontends, integration partners. The question is how to identify which platform facilitated a given transaction.

Transaction metadata field: The facilitating platform includes a `facilitator_address` field in the transaction metadata. This is transparent (anyone can see who claimed facilitation credit), simple to implement (just an optional field in the message), and flexible (any platform can claim credit). The risk is false claims -- a user could include their own address as the facilitator to claim facilitation rewards on their own transactions.

Registered dApp address: Platforms register their addresses through governance or an allowlist. Only transactions originating from registered addresses receive facilitation credit. This prevents false claims but creates a gatekeeping bottleneck. New platforms must go through a registration process before they can earn facilitation credit, which slows ecosystem growth.

The best approach combines both: a transaction metadata field for identification, with a registered facilitator allowlist for validation. Platforms register their facilitator address through a lightweight governance process (similar to GOV-002 currency allowlist). When a transaction includes a `facilitator_address`, M015 only awards facilitation credit if that address is on the registered list. This prevents self-facilitation claims while keeping the identification mechanism simple and transparent.

The registration process should be lightweight (Layer 2, similar to currency allowlist) to avoid slowing ecosystem growth. Requirements: the platform must demonstrate it has facilitated at least $10,000 in credit value (or equivalent) within a 90-day period, verified by on-chain transaction history.

**Recommendation**: Implement both mechanisms in combination:
1. **Identification**: Optional `facilitator_address` field in credit transaction metadata (MsgBuySellOrder, MsgSend, MsgRetire).
2. **Validation**: Registered Facilitator Allowlist (governance-managed, Layer 2). Only addresses on the allowlist receive M015 facilitation credit.
3. **Registration**: Lightweight process -- submit evidence of $10K+ facilitated credit value in prior 90 days. AGENT-003 verifies on-chain. Governance fast-track (48h validator vote) if agent score >= 0.8.
4. **Anti-gaming**: Facilitator cannot be the same address as buyer, seller, or retirer in the same transaction. Facilitation credit per address capped at the facilitator's proportional share of total facilitated volume (prevents a single platform from dominating rewards).

**Rationale**: The allowlist prevents self-facilitation fraud while the low registration barrier ($10K facilitated volume) ensures legitimate platforms can quickly begin earning facilitation credit. The metadata field provides transparent attribution. The anti-gaming rules prevent the most obvious exploits.

**Status**: RESOLVED

---

### OQ-M015-3: Community Pool Split (Auto vs. Governance)

**Question**: How does M015 automatic distribution interact with the existing Community Pool spend proposal process (GOV-004)? What share of the Community Pool goes to automatic distribution vs. governance-directed spending?

**Analysis**:

The Community Pool under M013+M015 serves two distinct purposes: (1) automatic activity-based distribution to contributors (M015), and (2) governance-directed spending via proposals (GOV-004). These purposes can conflict -- M015 automatically distributes tokens that governance might want to direct toward specific initiatives.

A 100% automatic (M015) allocation leaves nothing for governance-directed spending, eliminating the community's ability to fund strategic initiatives, grants, or emergency measures. This is clearly undesirable.

A 100% governance-directed allocation (no M015 automation) returns to the current model where all spending requires proposals, which is slow and governance-intensive. M015's value proposition is precisely that it automatically rewards contributors without governance overhead.

The split should heavily favor automatic distribution (M015) because the automatic rewards drive the core economic model: people who create ecological value on the network receive proportional rewards without waiting for governance proposals. However, a meaningful governance-directed share is essential for strategic initiatives that the automatic model cannot capture -- ecosystem grants, R&D funding, emergency responses, and novel programs.

A 70% automatic / 30% governance-directed split gives M015 a strong base for contributor rewards while preserving meaningful governance discretionary spending. At $22.5K/month Community Pool inflow (per OQ-M013-1 resolution), this means ~$15.75K/month automatic and ~$6.75K/month governance-directed. The $6.75K/month governance pot accumulates over time, so quarterly spending proposals could deploy $20K+ per initiative.

**Recommendation**: Split the Community Pool inflow 70% automatic (M015 distribution) / 30% governance-directed (available for GOV-004 proposals). Within the 70% automatic share, the stability tier (up to 30% of the 70% = 21% of total Community Pool) is first priority, with the remainder going to activity-based distribution. The 30% governance-directed share accumulates in a sub-account and is available for standard GOV-004 proposals. This split is a Layer 2 governance parameter, adjustable within [60/40, 80/20]. Changes outside this range require Layer 3 governance.

**Rationale**: 70/30 gives M015 a strong revenue base for automatic contributor rewards while preserving meaningful governance discretionary spending. The accumulation model for the governance share allows for periodic larger disbursements rather than many small ones. The adjustable range provides flexibility as the community learns how much governance-directed spending is needed.

**Status**: RESOLVED

---

### OQ-M015-4: Anti-Gaming Measures

**Question**: What prevents a participant from gaming the system by making many small transactions to inflate their activity score?

**Analysis**:

The primary gaming vector is wash trading: making circular transactions (buy and immediately re-sell, or self-transfer) to inflate activity scores and claim disproportionate M015 rewards. The M013 fee on each transaction provides natural friction -- a wash trader pays 1%+ fees on each round-trip, so they must earn more in M015 rewards than they pay in fees for gaming to be profitable.

Let us formalize this: for wash trading to be profitable, the expected M015 reward from a fake transaction must exceed the M013 fee paid. If a transaction of value V incurs a fee of 1% (= 0.01V), the wash trader earns M015 reward proportional to their activity score increase relative to total network activity. In a network with substantial legitimate activity, the marginal activity score from one fake transaction is small relative to the total, making the reward much less than 0.01V. Only if the wash trader represents a large share of total network activity does gaming become profitable -- but at that point, the fees paid are also large.

However, there are edge cases. In the early phase when legitimate activity is low, a wash trader could represent a large share of total activity. The minimum fee floor (1 REGEN per transaction) helps here, but additional measures are warranted.

Minimum transaction value for M015 eligibility ensures that micro-transactions (which have the most favorable gaming economics due to the minimum fee floor) do not contribute to activity scores. Address correlation analysis (detecting patterns of circular transactions between related addresses) can flag suspicious activity for investigation.

A cooling-off period between buying and re-selling credits (e.g., credits purchased must be held for 24 hours before they count toward the buyer's activity score) prevents high-frequency wash trading.

**Recommendation**: Implement the following anti-gaming measures:
1. **M013 fees as primary defense**: No change -- the existing fee structure makes wash trading costly.
2. **Minimum transaction value**: Transactions below 100 REGEN equivalent (or the min_fee floor, whichever is higher) do not contribute to M015 activity scores. This prevents micro-transaction spam.
3. **Holding period**: Credit purchases must be held (not transferred or sold) for 7 days to count toward the buyer's M015 activity score. Credits retired immediately count at full weight (retirement is terminal and cannot be gamed).
4. **Address correlation**: AGENT-003 monitors for circular transaction patterns (A buys from B, B buys from A within the same period). Flagged addresses are referred to governance for investigation. Confirmed wash trading results in M015 activity score zeroing for the relevant period.
5. **Diminishing returns**: Activity score contribution per address is logarithmically scaled beyond a threshold. Specifically: the first $100K in activity per period receives linear credit; activity above $100K receives log-scaled credit. This prevents whales from dominating the reward pool.

**Rationale**: Multi-layered defense prevents gaming across different strategies. The 7-day holding period is the most impactful single measure, as it makes wash trading capital-intensive (the trader must hold positions, bearing price risk). Logarithmic scaling prevents domination without capping participation. AGENT-003 monitoring provides an adaptive layer that can respond to novel gaming strategies.

**Status**: RESOLVED

---

## GOV-POA: Governance Transition

### OQ-GOV-POA-1: Should Dual-Track Weights Differ by Governance Process?

**Question**: Should the dual-track tally weights (60/40) be the same for all governance processes, or should different processes have different splits?

**Analysis**:

This question is closely related to OQ-M001-1 but focuses specifically on whether different governance processes should have structurally different weight splits, rather than whether the default should be modified.

The intuition is sound: software upgrades require validators to execute the upgrade, so validator buy-in is more critical than for, say, a treasury spending decision. Conversely, treasury spending affects all token holders' economic interests, so holder voice should be stronger there. The governance taxonomy in 2.3 already recognizes seven distinct process categories, each with different risk profiles and stakeholder impacts.

However, the complexity cost of per-process weights is significant. With seven process categories and two tracks, there are 14 weight parameters to govern (or 7, since each pair sums to 100%). Each parameter is potentially contentious. Changes to these weights would themselves be governance proposals, creating meta-governance complexity. Validators would have an incentive to increase their weight on treasury decisions; holders would have an incentive to increase their weight on technical decisions. This creates a perpetual tug-of-war.

The pragmatic approach from OQ-M001-1 applies here with one extension: rather than per-process custom weights, define three weight tiers that processes map to.

- **Technical tier** (70/30 validator/holder): Software upgrades, parameter changes to consensus-critical modules, emergency operations. These require validator execution and technical competence.
- **Standard tier** (60/40 validator/holder): Credit class approvals, currency allowlist, routine parameter changes, validator set management. The default for most governance.
- **Economic tier** (50/50 validator/holder): Community Pool spending, M013/M015 economic parameter changes, fee rate changes. These directly affect token holder economics.

Three tiers is manageable, distinct enough to be meaningful, and maps cleanly to the existing governance layer framework.

**Recommendation**: Define three weight tiers:
- **Technical**: 70/30 (validator/holder). Applies to: GOV-003 (software upgrades), consensus parameter changes, emergency operations.
- **Standard**: 60/40 (validator/holder). Applies to: GOV-001 (credit class allowlist), GOV-002 (currency allowlist), GOV-005 (routine parameter changes), M014 validator set management.
- **Economic**: 50/50 (validator/holder). Applies to: GOV-004 (Community Pool spending), M013 fee rate changes, M015 reward parameter changes.

Each process's tier assignment is a Layer 3 governance parameter. The tier weights themselves are Layer 4 Constitutional parameters. This creates a two-level system: the community can reassign a process to a different tier (Layer 3) without changing the tier weights themselves (Layer 4).

**Rationale**: Three tiers capture the meaningful distinctions between technical, standard, and economic governance without the complexity of per-process weights. The tier system is simple enough for participants to understand and reduces governance overhead compared to managing individual process weights. The Layer 3 / Layer 4 split allows tier assignments to evolve without touching Constitutional parameters.

**Status**: NEEDS_GOVERNANCE -- The tier structure is well-defined, but the specific process-to-tier assignments should be deliberated by the WG and ratified by the community.

---

### OQ-GOV-POA-2: How Long Should PoS/PoA Run in Parallel?

**Question**: During the overlap period, how long should both PoS and PoA governance models run in parallel? Should there be a formal "graduation" criteria for switching to dual-track tally?

**Analysis**:

The overlap period is when M014 is active (authority validators are producing blocks) but governance still uses the legacy PoS tally model. This is Phase 2 in the governance transition plan (2.3). The question is how long this phase should last before transitioning to the dual-track tally (Phase 3).

A long overlap period (12+ months) provides extensive data on authority validator behavior under the new model before granting them governance authority. However, it creates confusion: stakeholders see authority validators but vote under the old model, leading to ambiguity about who actually governs the network. A prolonged overlap may also reduce urgency for completing the transition.

A short overlap (1-3 months) minimizes confusion and accelerates the transition. However, it provides little time to identify issues with the authority validator set or its governance dynamics. If a problematic validator was admitted, a short overlap gives little time to detect and address this before they gain governance authority.

6 months is the sweet spot. It provides two full quarterly review cycles (enough to assess validator performance), allows time for the community to adapt to the new model conceptually, and aligns with standard software deployment practices (release, observe for two quarters, promote).

Rather than a fixed duration, the overlap should end when graduation criteria are met, with a minimum of 3 months and a maximum of 9 months. If graduation criteria are met at 3 months, the transition can proceed; if not met by 9 months, there is a problem that needs addressing.

**Recommendation**: The overlap period runs for a minimum of 3 months and a maximum of 9 months, ending when all of the following graduation criteria are met:
1. **Validator stability**: Authority validator set has maintained >= min_validators (15) continuously for 90+ days with no emergency governance escalations.
2. **Uptime compliance**: All authority validators have maintained >= 99.0% uptime (slightly relaxed from 99.5% target during early operation) for 90+ days.
3. **Governance participation**: Authority validators have participated in >= 3 governance proposals during the overlap period.
4. **No composition violations**: The composition guarantee (5/5/5 minimum per category) has not been violated.
5. **Community readiness**: A governance proposal to activate dual-track tally passes under the current PoS model (simple majority, 40% quorum).

If criteria are not met by 9 months, governance must vote on whether to: (A) extend the overlap by 3 months, (B) address the failing criteria and re-evaluate, or (C) roll back M014.

**Rationale**: Criteria-based graduation is more robust than a fixed timeline because it ensures the authority validator set has demonstrated readiness. The 3-month minimum prevents premature transition; the 9-month maximum prevents indefinite limbo. The final governance vote requirement ensures the community actively consents to the transition.

**Status**: RESOLVED

---

### OQ-GOV-POA-3: Handling Existing ~75 Validators' Governance Participation

**Question**: How should the existing approximately 75 validators' governance participation be handled during transition?

**Analysis**:

When M014 activates, the active validator set shrinks from approximately 75 to 15-21. The remaining 54-60 validators lose block production rights and validator-associated governance weight. However, they retain their REGEN holdings and, in many cases, significant community standing and operational expertise.

Under the dual-track model, these former validators participate through the holder track. Their votes count proportional to their contribution-weighted scores (M015) or token holdings (pre-M015). This is a significant reduction in governance influence -- from validator-weight (which is disproportionately large in Cosmos PoS) to ordinary holder-weight.

This creates a legitimate grievance: validators who have supported the network for years, often at a financial loss, see their governance role diminished. If handled poorly, this could create a vocal opposition bloc that undermines the PoA transition.

Several mitigation strategies:

1. **Holder track participation**: Former validators participate in the holder track with their full token holdings. Their operational experience informs their voting but does not give them special weight.

2. **Priority consideration for re-admission**: If the authority validator set expands (up to max_validators = 21), former validators who meet M014 criteria receive priority consideration in the application process. Their operational track record is a strong qualification.

3. **M015 activity credit**: Former validators who continue to contribute (governance participation, ecosystem development, platform facilitation) earn M015 activity-based rewards. Their expertise is rewarded through the contribution model, not through validator compensation.

4. **Transition recognition**: A one-time "transition appreciation" distribution from the Community Pool (governance proposal) recognizing the contribution of long-serving validators. This is symbolic but signals respect for their service.

5. **Advisory role**: Establish a non-binding "Validator Alumni Advisory" group that is consulted (but not granted governance authority) on technical decisions. This leverages their expertise without creating a formal governance role.

**Recommendation**: Former validators (those not selected for the M014 authority set) transition to the holder track with full token-holding-based participation. Specific measures:
1. **Holder track participation**: Full participation with all token holdings. Pre-M015, this is 1-token-1-vote in the holder track. Post-M015, contribution-weighted.
2. **Application priority**: Former validators with >= 12 months of active service and >= 95% uptime during that period receive streamlined re-application (reduced evaluation period) if authority set slots open.
3. **Transition recognition**: Recommend (but do not mandate) a governance proposal for a one-time Community Pool distribution of 50-100 REGEN per month of active service, capped at 2,400 REGEN (2 years equivalent), to former validators who served during the transition period.
4. **No special governance weight**: Former validators do not receive elevated weight in the holder track. Their influence comes from their token holdings and M015 activity scores, same as any other holder.

**Rationale**: Treating former validators as standard holders is necessary for the integrity of the dual-track model -- creating a third track or special holder class would undermine the simplicity and fairness of the system. The application priority and transition recognition measures honor their service without creating permanent privileges. The community can decide whether the transition recognition distribution is appropriate via governance proposal.

**Status**: NEEDS_GOVERNANCE -- The transition recognition distribution (point 3) requires a community decision on whether and how much to allocate. The other points are structural recommendations that follow from the M014 design.

---

## Summary Matrix

| ID | Question | Status | Key Recommendation |
|-----|----------|--------|-------------------|
| OQ-M001-1 | 60/40 tally weight split | RESOLVED | 60/40 default with `process_weight_override` parameter; advisory highlight for data stewards on registry decisions |
| OQ-M001-2 | Agent score governance track influence | RESOLVED | Fast Track for agent_score >= 0.85: 48h validator-only vote, escalatable by any validator or 0.1% holder |
| OQ-M009-1 | Automatic reputation feedback | RESOLVED | Auto baseline (+5) on completion, optional endorsement (+1 to +20), 500 REGEN minimum, diminishing returns per address pair |
| OQ-M009-2 | Partial milestone payments | RESOLVED | Three templates: 100/0 (default), 70/30, 50/50; selected at PROPOSED stage, immutable after FUNDED |
| OQ-M009-3 | Validator override for stuck disputes | RESOLVED | Emergency override after 30-day Arbiter timeout; 75% supermajority; limited to extend, return-to-client, or 50/50 split |
| OQ-M009-4 | Escrow/M013 fee interaction | RESOLVED | Route M009 platform fees through M013 distribution; same pool shares as credit transaction fees |
| OQ-M011-1 | Curator credit holdings | RESOLVED | No requirement; disclosure required and enforced via bond slashing for undisclosed holdings |
| OQ-M011-2 | Quality score storage | RESOLVED | Hybrid: numeric scores on-chain, full assessment in KOI with IRI anchor |
| OQ-M011-3 | Curation fee / M013 interaction | RESOLVED | Separate 0.25% fee on curated credit trades; flows to curator reward pool, not through M013 |
| OQ-M011-4 | Curator reputation via M010 | RESOLVED | Track via M010; bond scales as `base_bond * (2.0 - reputation_score)`; reputation decays toward neutral |
| OQ-M011-5 | Basket tokens in collections | RESOLVED | Scores at batch level; baskets get quantity-weighted average; dynamically recomputed on composition change |
| OQ-M012-1 | Hard cap value | RESOLVED | Total supply at M012 activation block, rounded down to nearest 1M (expected ~224M) |
| OQ-M012-2 | Ecological multiplier oracle | RESOLVED | v0 disabled (1.0); v1 from M008 attestation data after 12+ months M008 operation |
| OQ-M012-3 | Mint/burn period length | RESOLVED | 7-day epoch; min 100 transactions per period; governance path to shorten as volume grows |
| OQ-M012-4 | Permanent burn vs. reserve pool | RESOLVED | Permanent burn; no reserve pool; governance adjusts burn rate via M013 share parameters |
| OQ-M012-5 | Multiplier replacement | RESOLVED | Stability multiplier (from M015) per existing phase-gated spec; no validator participation multiplier |
| OQ-M013-1 | Distribution model A vs. B | NEEDS_GOVERNANCE | Proposed: {15% burn, 28% validator, 45% community, 12% agent}; adjustable +/- 10% at Layer 2 |
| OQ-M013-2 | Credit value for non-marketplace tx | RESOLVED | Waterfall: 7-day TWAP (3-trade minimum) then governance-set reference then min fee floor |
| OQ-M013-3 | Fee denomination | RESOLVED | Hybrid: collect in native denom; auto-convert burn share to REGEN; distribute remainder in collected denom; 5% REGEN election bonus |
| OQ-M013-4 | Agent fund governance | RESOLVED | Separate module account; Phase 1 multisig (3-of-5), Phase 2 governance-approved annual budgets; 25% single-expenditure cap |
| OQ-M013-5 | Burn pool existence | NEEDS_GOVERNANCE | Proposed: 15% burn share; Layer 2 adjustable [5%, 25%]; balances deflationary signal with contributor funding |
| OQ-M014-1 | Validator set size | RESOLVED | Floating [15, 21]; min=15 (Layer 4), max=21 (Layer 4), target=18 (Layer 3); 5 minimum per composition category |
| OQ-M014-2 | Performance bonus | RESOLVED | No bonus for v0; equal compensation; performance metrics used for accountability (probation/removal); revisit after 12 months |
| OQ-M014-3 | Initial trusted partner determination | NEEDS_GOVERNANCE | 30-day application, Selection Committee evaluation, 14-day community review, PoS governance ratification |
| OQ-M014-4 | PoA socialization timeline | RESOLVED | M013 mainnet Q4 2026; M014 mainnet Q2 2027; M012+M015 mainnet Q3 2027; M013 first to demonstrate economic model |
| OQ-M014-5 | Delegated REGEN when PoS disabled | RESOLVED | 90-day notice with graduated reward reduction; staggered forced unbonding over 41 days; stability tier promotion |
| OQ-M015-1 | 6% stability tier sustainability | RESOLVED | "Up to 6%" with revenue adequacy constraint; 30% cap on Community Pool inflow; review after 12 months |
| OQ-M015-2 | Platform facilitation identification | RESOLVED | Metadata field + registered Facilitator Allowlist; $10K facilitated volume for registration; anti-self-facilitation rules |
| OQ-M015-3 | Community Pool split | RESOLVED | 70% automatic (M015) / 30% governance-directed (GOV-004); adjustable [60/40, 80/20] at Layer 2 |
| OQ-M015-4 | Anti-gaming measures | RESOLVED | 100 REGEN minimum transaction, 7-day holding period, address correlation monitoring, logarithmic scaling above $100K/period |
| OQ-GOV-POA-1 | Per-process tally weights | NEEDS_GOVERNANCE | Three tiers: Technical (70/30), Standard (60/40), Economic (50/50); process-tier assignment is Layer 3 |
| OQ-GOV-POA-2 | PoS/PoA parallel duration | RESOLVED | 3-9 month overlap with graduation criteria: validator stability, uptime, governance participation, composition, community vote |
| OQ-GOV-POA-3 | Existing validators' participation | NEEDS_GOVERNANCE | Holder track participation; application priority for re-admission; recommended transition recognition distribution |

---

## Resolution Statistics

- **Total Open Questions**: 31
- **RESOLVED**: 22 (71%)
- **NEEDS_GOVERNANCE**: 9 (29%)

### NEEDS_GOVERNANCE Items Requiring WG Action

1. **OQ-M013-1** (Distribution Model): The exact pool share ratios require community deliberation and vote.
2. **OQ-M013-5** (Burn Pool): Burn share involves deep value judgments about the network's purpose.
3. **OQ-M014-3** (Seed Set): Selection Committee composition and evaluation criteria need WG deliberation.
4. **OQ-GOV-POA-1** (Per-Process Weights): Tier structure and process-to-tier assignments need community consensus.
5. **OQ-GOV-POA-3** (Former Validators): Transition recognition distribution requires community decision.

Items 1 and 2 are linked (burn share determines the remaining shares). Items 3, 4, and 5 are transition-specific decisions that should be resolved during Q2 2026 per the proposed timeline.

### Cross-Cutting Dependencies

Several resolutions are interdependent:

- **OQ-M013-1 + OQ-M013-5**: Burn share directly affects all other pool shares.
- **OQ-M012-1 + OQ-M012-5 + OQ-M015-1**: Hard cap, stability multiplier, and stability tier return are linked through M012's regrowth algorithm and M015's stability tier.
- **OQ-M013-3 + OQ-M015-3**: Fee denomination affects how Community Pool accumulates value, which affects M015 distribution.
- **OQ-M014-4 + OQ-GOV-POA-2**: The PoA timeline determines when the overlap period begins and ends.

These dependencies should be validated through simulation modeling (Phase 3 activity) before mainnet activation.
