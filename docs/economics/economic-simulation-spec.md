# Economic Simulation Specification for M012-M015 Validation

## 1. Overview

### Purpose

This document specifies a comprehensive simulation framework for validating that the economic reboot parameters defined in [Phase 2.6](../../phase-2/2.6-economic-reboot-mechanisms.md) produce sustainable, stable outcomes before mainnet deployment. The four mechanism specifications --- M012 (Fixed Cap Dynamic Supply), M013 (Value-Based Fee Routing), M014 (Authority Validator Governance), and M015 (Contribution-Weighted Rewards) --- define the mathematical relationships governing the post-reboot $REGEN economy. However, these formulas interact in non-obvious ways across time, and their combined behavior under realistic and adversarial conditions has not been validated.

The simulation addresses three critical gaps:

1. **Parameter sensitivity**: Many parameters (regrowth rate, fee shares, stability tier returns) were set by intuition or precedent. The simulation quantifies how sensitive system outcomes are to each parameter choice and identifies safe operating ranges.

2. **Dynamic stability**: The equilibrium analysis in M012 assumes M[t] = B[t] convergence, but the path to equilibrium --- and whether the system reaches it at all under realistic agent behavior --- requires simulation to validate.

3. **Adversarial resilience**: Wash trading, bank runs on the stability tier, fee avoidance, and oracle manipulation are real threats. The simulation must demonstrate that the economic design degrades gracefully under attack, or reveal failure modes that require mechanism redesign.

### Scope

- **In scope**: All economic flows defined by M012-M015, agent behavior modeling, parameter sweeps, stress testing, Monte Carlo uncertainty quantification
- **Out of scope**: Cosmos SDK implementation details, smart contract code generation, UI/UX, cross-chain bridge mechanics
- **Time horizon**: 5-year simulation (260 weekly epochs), with detailed analysis of the first 24 months (transition period)

### Success Criteria

The simulation produces a "green light" for mainnet deployment if:

| Criterion | Threshold |
|-----------|-----------|
| Supply remains within bounds | `S[t] in [100M, 221M]` for >99% of Monte Carlo runs |
| Validator income covers operational costs | Per-validator annual income > $15,000 in >90% of runs |
| Stability tier remains solvent | Stability payouts never exceed 30% of Community Pool inflow in >95% of runs |
| Fee revenue covers operating costs | Monthly fee revenue > $20K within 12 months in >80% of runs |
| System reaches equilibrium | `abs(M[t] - B[t]) / S[t] < 0.001` within 36 months in >70% of runs |
| No death spiral | Supply never drops below 50% of initial in any run |

---

## 2. Model Architecture

### Framework

The simulation uses a **cadCAD-compatible agent-based modeling** framework, combining system dynamics (state evolution equations) with heterogeneous behavioral agents that generate the transactions driving fee collection, supply changes, and reward distribution.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        SIMULATION ARCHITECTURE                         │
│                                                                        │
│  ┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐     │
│  │  Behavioral │────▶│  Policy Functions │────▶│  State Update   │     │
│  │  Agents     │     │  (Mechanisms)     │     │  (cadCAD SUFs)  │     │
│  └─────────────┘     └──────────────────┘     └─────────────────┘     │
│        │                     │                        │               │
│        │              ┌──────┴──────┐                 │               │
│        │              │ M012: Supply │                 ▼               │
│        │              │ M013: Fees   │          ┌───────────┐         │
│        │              │ M014: Valids │          │   State   │         │
│        │              │ M015: Rewards│          │ Variables │──┐      │
│        │              └─────────────┘          └───────────┘  │      │
│        │                                                       │      │
│        └───────────────────────────────────────────────────────┘      │
│                          (agents observe state)                       │
└─────────────────────────────────────────────────────────────────────────┘
```

### State Variables

| Variable | Symbol | Type | Initial Value | Updated By |
|----------|--------|------|---------------|------------|
| Circulating supply | `S[t]` | float | 224,000,000 (current) | M012 |
| Hard cap | `C` | float | 221,000,000 | Governance (Layer 4) |
| Burn pool balance | `pool_burn[t]` | float | 0 | M013 inflow, M012 burn |
| Validator fund balance | `pool_validator[t]` | float | 0 | M013 inflow, M014 distribution |
| Community pool balance | `pool_community[t]` | float | 0 | M013 inflow, M015 distribution |
| Agent infra fund balance | `pool_agent[t]` | float | 0 | M013 inflow, agent ops spend |
| Active validator count | `n_validators[t]` | int | 15 | M014 lifecycle |
| Total stability committed | `S_stability[t]` | float | 0 | M015 commitments |
| Total staked | `S_staked[t]` | float | ~100,000,000 | PoS (declining during transition) |
| Period fee revenue | `F[t]` | float | 0 | M013 collection |
| Period credit volume | `V_credit[t]` | float | varies | Agent behavior |
| Activity scores (per agent) | `score[p][t]` | float | 0 | M015 tracking |
| Ecological multiplier | `eco_mult[t]` | float | 1.0 (v0) | Oracle / scenario |
| M014 phase state | `m014_state` | enum | INACTIVE | M014 transition |
| Cumulative tokens burned | `total_burned[t]` | float | 0 | M012 |
| Cumulative tokens minted | `total_minted[t]` | float | 0 | M012 |
| REGEN price (USD) | `price[t]` | float | $0.03 (current) | External / endogenous model |
| Credit issuance volume (USD) | `V_issuance[t]` | float | varies | Agent behavior |
| Credit retirement volume (USD) | `V_retirement[t]` | float | varies | Agent behavior |
| Credit trade volume (USD) | `V_trade[t]` | float | varies | Agent behavior |
| Credit transfer volume (USD) | `V_transfer[t]` | float | varies | Agent behavior |

### Policy Functions

Each mechanism maps to one or more cadCAD policy functions:

#### P1: Fee Collection (M013)

```python
def fee_collection(params, substep, state_history, prev_state):
    """Compute fees from all credit transactions in the period."""
    V = prev_state['credit_volumes']  # dict of tx_type -> USD volume

    fees = {}
    for tx_type, volume in V.items():
        rate = params['fee_rates'][tx_type]
        fee_regen = (volume * rate) / prev_state['price']
        fee_regen = max(fee_regen, params['min_fee'] * volume_tx_count(volume))
        fees[tx_type] = fee_regen

    total_fee = sum(fees.values())

    return {
        'burn_inflow': total_fee * params['burn_share'],
        'validator_inflow': total_fee * params['validator_share'],
        'community_inflow': total_fee * params['community_share'],
        'agent_inflow': total_fee * params['agent_share'],
        'total_fee': total_fee
    }
```

#### P2: Supply Mint/Burn (M012)

```python
def supply_update(params, substep, state_history, prev_state):
    """Execute mint/burn algorithm."""
    S = prev_state['supply']
    C = params['hard_cap']

    # Effective multiplier depends on M014 phase
    if prev_state['m014_state'] == 'INACTIVE':
        eff_mult = 1 + (prev_state['S_staked'] / S)
    elif prev_state['m014_state'] == 'TRANSITION':
        staking_mult = 1 + (prev_state['S_staked'] / S)
        stability_mult = 1 + (prev_state['S_stability'] / S)
        eff_mult = max(staking_mult, stability_mult)
    else:  # ACTIVE or EQUILIBRIUM
        eff_mult = 1 + (prev_state['S_stability'] / S)

    r = params['r_base'] * eff_mult * prev_state['eco_mult']
    M = r * (C - S)
    M = max(0, M)  # No negative minting

    B = prev_state['pool_burn']  # Burn everything accumulated in burn pool

    new_supply = S + M - B
    new_supply = max(0, min(C, new_supply))  # Enforce bounds

    return {
        'supply': new_supply,
        'minted': M,
        'burned': B,
        'pool_burn': 0  # Reset after burn
    }
```

#### P3: Validator Compensation (M014)

```python
def validator_compensation(params, substep, state_history, prev_state):
    """Distribute validator fund to active validators."""
    fund = prev_state['pool_validator']
    n = prev_state['n_validators']

    if n == 0:
        return {'pool_validator': fund, 'per_validator_payment': 0}

    bonus_pool = fund * params['performance_bonus_share']
    base_pool = fund - bonus_pool

    base_per_validator = base_pool / n
    # Bonus distributed proportional to composite score
    # (simplified: uniform in base model, varied in agent model)
    bonus_per_validator = bonus_pool / n  # Uniform approximation

    return {
        'pool_validator': 0,  # Fully distributed
        'per_validator_payment': base_per_validator + bonus_per_validator
    }
```

#### P4: Reward Distribution (M015)

```python
def reward_distribution(params, substep, state_history, prev_state):
    """Distribute community pool to stability tier and activity participants."""
    inflow = prev_state['community_inflow_this_period']

    # 1. Stability tier allocation (first priority)
    annual_commitment_return = prev_state['S_stability'] * params['stability_rate']
    period_commitment_return = annual_commitment_return / params['periods_per_year']
    max_stability = inflow * params['max_stability_share']
    stability_payout = min(period_commitment_return, max_stability)

    # 2. Activity pool
    activity_pool = inflow - stability_payout

    # 3. Per-participant distribution (proportional to score)
    total_score = sum(prev_state['activity_scores'].values())
    rewards = {}
    if total_score > 0:
        for p, score in prev_state['activity_scores'].items():
            rewards[p] = activity_pool * (score / total_score)

    return {
        'stability_payout': stability_payout,
        'activity_pool_distributed': activity_pool,
        'rewards': rewards,
        'pool_community': prev_state['pool_community'] - inflow
    }
```

### Behavioral Agents

Each simulation run instantiates a population of heterogeneous agents. Agents observe state and make decisions each period.

| Agent Type | Count (baseline) | Behavior | Key Parameters |
|-----------|------------------|----------|----------------|
| **Credit Issuer** | 20-50 | Issues new credit batches with stochastic volume; responds to price signals | issuance_rate, batch_size_dist |
| **Credit Buyer** | 100-500 | Purchases credits on marketplace; volume driven by CSR budgets and credit price | purchase_budget, price_sensitivity |
| **Credit Retirer** | 50-200 | Retires credits for compliance or voluntary offset; retirement rate as % of holdings | retirement_propensity, compliance_deadline_driven |
| **Validator** | 15-21 | Operates node; may enter/exit based on compensation adequacy | min_acceptable_income, uptime, churn_probability |
| **Stability Tier Holder** | 50-500 | Commits REGEN to stability tier; exits if returns unsatisfactory or liquidity needed | lock_amount, lock_duration, exit_threshold |
| **Governance Participant** | 30-200 | Votes on proposals, submits proposals; activity driven by stake weight and engagement | vote_probability, proposal_frequency |
| **Wash Trader (adversarial)** | 0-10 | Executes circular trades to inflate activity scores; pays fees on each loop | attack_budget, loop_size, detection_avoidance |

#### Agent Decision Functions

**Credit Buyer** (representative):
```python
class CreditBuyer:
    def __init__(self, budget_mean, budget_std, price_elasticity):
        self.budget = np.random.lognormal(np.log(budget_mean), budget_std)
        self.price_elasticity = price_elasticity

    def decide(self, state):
        """How much credit to buy this period."""
        price_index = state['credit_price'] / self.reference_price
        volume = self.budget * (price_index ** (-self.price_elasticity))
        volume *= np.random.lognormal(0, 0.3)  # Stochastic noise
        return max(0, volume)
```

**Stability Tier Holder** (representative):
```python
class StabilityHolder:
    def __init__(self, amount, duration_months, exit_threshold):
        self.committed = amount
        self.duration = duration_months
        self.exit_threshold = exit_threshold  # Min acceptable realized APY

    def decide_exit(self, state):
        """Whether to exit early."""
        realized_apy = state['stability_realized_return'] / self.committed
        regen_price_change = state['price'] / state['price_at_commit'] - 1

        # Exit if realized return too low OR REGEN price dropping fast
        if realized_apy < self.exit_threshold or regen_price_change < -0.4:
            return 'EARLY_EXIT'
        return 'HOLD'
```

**Wash Trader** (adversarial):
```python
class WashTrader:
    def __init__(self, budget, accounts):
        self.budget = budget  # Total REGEN allocated to attack
        self.accounts = accounts  # Number of sybil accounts

    def execute(self, state):
        """Execute circular trades across sybil accounts."""
        # Each loop: account A sells to B, B sells to C, ..., N sells to A
        # Each trade pays M013 fees
        loop_volume = self.budget / self.accounts
        fees_paid = loop_volume * state['fee_rate_trade'] * self.accounts
        # Net loss to attacker = fees_paid (value destroyed)
        # Activity score gained = loop_volume * accounts * weight_purchase
        score_gained = loop_volume * self.accounts * 0.30

        # Profitable if: reward_share * activity_pool > fees_paid
        return {
            'volume': loop_volume * self.accounts,
            'fees_paid': fees_paid,
            'score_gained': score_gained
        }
```

---

## 3. Parameter Space

### Complete Parameter Table

Every tunable parameter in M012-M015 with baseline values, bounds, and sensitivity ratings.

#### M012 Parameters (Fixed Cap Dynamic Supply)

| Parameter | Symbol | Baseline | Min | Max | Unit | Sensitivity | Rationale |
|-----------|--------|----------|-----|-----|------|-------------|-----------|
| Hard cap | `C` | 221,000,000 | 200,000,000 | 250,000,000 | REGEN | **High** | Determines long-term supply ceiling; affects all equilibrium calculations |
| Base regrowth rate | `r_base` | 0.02 | 0.005 | 0.10 | per period | **High** | Primary lever controlling mint rate; directly determines equilibrium supply |
| Period length | `period_length` | 7 days | 1 day | 30 days | days | Medium | Cadence of mint/burn; affects volatility of supply changes |
| Ecological multiplier | `eco_mult` | 1.0 (v0 disabled) | 0.0 | 1.5 | dimensionless | **High** | Can halt minting entirely (=0) or accelerate it; oracle-dependent |
| Ecological reference value | `eco_ref` | 50 | 10 | 200 | ppm CO2 | Medium | Sensitivity of eco multiplier to real-world data |
| Min burn rate | `min_burn` | 0 | 0 | 0.001 | per period | Low | Safety floor; rarely binding under normal conditions |
| Regrowth rate upper bound | `r_max` | 0.10 | 0.05 | 0.20 | per period | Medium | Safety cap preventing runaway minting |
| Initial supply | `S_0` | 224,000,000 | 220,000,000 | 230,000,000 | REGEN | Medium | Starting above cap creates immediate net-burn pressure |

#### M013 Parameters (Value-Based Fee Routing)

| Parameter | Symbol | Baseline | Min | Max | Unit | Sensitivity | Rationale |
|-----------|--------|----------|-----|-----|------|-------------|-----------|
| Issuance fee rate | `fee_issuance` | 0.02 (2%) | 0.01 | 0.03 | fraction | **High** | Primary revenue source; too high discourages issuance |
| Transfer fee rate | `fee_transfer` | 0.001 (0.1%) | 0.0005 | 0.005 | fraction | Low | Minimal impact due to low rate |
| Retirement fee rate | `fee_retirement` | 0.005 (0.5%) | 0.002 | 0.01 | fraction | Medium | Exit fee; affects retirement propensity |
| Trade fee rate | `fee_trade` | 0.01 (1%) | 0.005 | 0.02 | fraction | **High** | Affects marketplace competitiveness and volume |
| Burn share | `burn_share` | 0.30 | 0.00 | 0.35 | fraction | **High** | Directly controls supply deflation rate |
| Validator share | `validator_share` | 0.40 | 0.15 | 0.50 | fraction | **High** | Determines validator economic viability |
| Community share | `community_share` | 0.25 | 0.20 | 0.60 | fraction | **High** | Funds rewards and governance spending |
| Agent infra share | `agent_share` | 0.05 | 0.00 | 0.10 | fraction | Low | Small allocation; limited system impact |
| Minimum fee | `min_fee` | 1 REGEN | 0.1 | 10 | REGEN | Low | Floor for low-value transactions |

#### M014 Parameters (Authority Validator Governance)

| Parameter | Symbol | Baseline | Min | Max | Unit | Sensitivity | Rationale |
|-----------|--------|----------|-----|-----|------|-------------|-----------|
| Max validators | `n_max` | 21 | 15 | 50 | count | Medium | Affects per-validator income (inverse relationship) |
| Min validators | `n_min` | 15 | 7 | 21 | count | Medium | Security threshold |
| Performance bonus share | `bonus_share` | 0.10 | 0.00 | 0.30 | fraction | Low | Small effect on total economics; affects incentive alignment |
| Term length | `term_length` | 12 | 6 | 24 | months | Low | Governance parameter; limited economic impact |
| Min uptime | `min_uptime` | 0.995 | 0.990 | 0.999 | fraction | Low | Operational; affects churn rate indirectly |
| Probation period | `probation_days` | 30 | 14 | 90 | days | Low | Operational |
| Compensation review frequency | `review_freq` | 4 | 2 | 12 | per year | Low | Administrative |
| Uptime weight (score) | `w_uptime` | 0.4 | 0.2 | 0.6 | weight | Low | Performance score composition |
| Governance weight (score) | `w_governance` | 0.3 | 0.1 | 0.5 | weight | Low | Performance score composition |
| Ecosystem weight (score) | `w_ecosystem` | 0.3 | 0.1 | 0.5 | weight | Low | Performance score composition |

#### M015 Parameters (Contribution-Weighted Rewards)

| Parameter | Symbol | Baseline | Min | Max | Unit | Sensitivity | Rationale |
|-----------|--------|----------|-----|-----|------|-------------|-----------|
| Stability tier annual return | `stability_rate` | 0.06 (6%) | 0.02 | 0.12 | annual fraction | **High** | Too high risks insolvency; too low attracts no capital |
| Max stability share of pool | `max_stability_share` | 0.30 | 0.10 | 0.50 | fraction | **High** | Caps stability tier exposure; protects activity rewards |
| Min lock period | `min_lock` | 6 | 3 | 12 | months | Medium | Shorter lock = more liquid = more bank run risk |
| Max lock period | `max_lock` | 24 | 12 | 48 | months | Low | Upper bound; rarely binding |
| Early exit penalty | `exit_penalty` | 0.50 | 0.25 | 1.00 | fraction forfeited | Medium | Higher penalty deters runs but reduces attractiveness |
| Min commitment | `min_commit` | 100 | 10 | 1,000 | REGEN | Low | Access threshold |
| Purchase weight | `w_purchase` | 0.30 | 0.10 | 0.50 | weight | Medium | Activity score composition |
| Retirement weight | `w_retirement` | 0.30 | 0.10 | 0.50 | weight | Medium | Activity score composition |
| Facilitation weight | `w_facilitation` | 0.20 | 0.05 | 0.40 | weight | Low | Activity score composition |
| Governance vote weight | `w_gov_vote` | 0.10 | 0.05 | 0.30 | weight | Low | Activity score composition |
| Proposal weight | `w_proposal` | 0.10 | 0.00 | 0.20 | weight | Low | Activity score composition |

---

## 4. Sensitivity Analysis

### Methodology

For each key parameter, we fix all other parameters at baseline and sweep the target parameter across its range. We measure the impact on five key system outputs at month 24.

### M012: Supply Dynamics Sensitivity

#### `r_base` (Base Regrowth Rate)

| Scenario | `r_base` | Eq. Supply | Net Mint (Mo. 24) | Time to Eq. | Supply at Mo. 24 |
|----------|----------|-----------|-------------------|-------------|------------------|
| Low | 0.005 | ~219.5M | +22K/wk | >60 months | ~221.8M |
| **Baseline** | **0.02** | **~215M** | **+86K/wk** | **~36 months** | **~218M** |
| High | 0.05 | ~205M | +200K/wk | ~18 months | ~212M |
| Extreme | 0.10 | ~185M | +350K/wk | ~12 months | ~200M |

**Impact analysis**: `r_base` is the single most influential parameter on supply trajectory. At 0.005, the system barely mints and relies almost entirely on burn-side dynamics. At 0.10, aggressive minting counteracts burning, and the system oscillates more before settling. The equilibrium supply level decreases significantly with higher `r_base` because faster regrowth maintains higher minting even as burn rates stabilize.

**Derivation of equilibrium supply**: At equilibrium, M[t] = B[t]:
```
r_base * eff_mult * eco_mult * (C - S_eq) = B_eq
```
Where `B_eq` depends on fee volume (see Section 6).

#### `burn_share` (M013 Burn Pool Fraction)

| Scenario | `burn_share` | Fee Rev. (Mo.) | Burn/Period | Eq. Supply | Validator Income |
|----------|-------------|----------------|-------------|-----------|-----------------|
| No burn | 0.00 | $50K | 0 | Cap (221M) | Higher (+33%) |
| Low | 0.10 | $50K | ~5K REGEN | ~219M | Higher (+25%) |
| **Baseline** | **0.30** | **$50K** | **~15K REGEN** | **~215M** | **Baseline** |
| High | 0.35 | $50K | ~17.5K REGEN | ~213M | Lower (-8%) |

**Impact analysis**: Burn share directly trades off supply deflation against validator/community funding. Eliminating burn maximizes funding for validators and rewards but removes the deflationary mechanism that supports price. This is the core tension identified in OQ-M013-5.

#### `fee_trade` (Marketplace Trade Fee Rate)

| Scenario | `fee_trade` | Monthly Rev. | Annual Validator Inc. | Activity Pool | Volume Effect |
|----------|-----------|-------------|----------------------|---------------|--------------|
| Low | 0.005 | $32K | $18K/validator | $6.5K/mo | Volume +20% |
| **Baseline** | **0.01** | **$50K** | **$28K/validator** | **$10K/mo** | **Baseline** |
| High | 0.02 | $58K | $33K/validator | $12K/mo | Volume -25% |

**Impact analysis**: Higher trade fees increase per-transaction revenue but suppress volume. The relationship is approximately: revenue = fee_rate * volume(fee_rate), where volume has elasticity of approximately -0.8 to -1.2 with respect to fee rate. Revenue is roughly maximized around 1-1.5% for ecological credit markets (which have limited off-chain alternatives).

### M015: Reward System Sensitivity

#### `stability_rate` (Annual Return for Stability Tier)

| Scenario | `stability_rate` | Stability Demand | Cap Binding? | Activity Pool Squeeze | Sustainability Limit |
|----------|-----------------|-----------------|-------------|----------------------|---------------------|
| Low | 0.02 (2%) | ~$500K committed | No | Minimal | Sustainable at $15K/mo |
| **Baseline** | **0.06 (6%)** | **~$2M committed** | **Near cap** | **Moderate** | **Requires $40K/mo** |
| High | 0.12 (12%) | ~$5M committed | Cap binding | Severe | Requires $100K/mo |

**Impact analysis**: At 6%, with 30% cap, the stability tier is sustainable if monthly Community Pool inflow exceeds `(stability_committed * 0.06) / (12 * 0.30)`. For $2M committed: $33K/month Community Pool inflow required, or $133K total fee revenue (since Community Pool gets 25%). At projected $50K/month fee revenue, stability commitments are limited to approximately $750K before the 30% cap binds.

#### `max_stability_share` (Cap on Stability Tier Allocation)

| Scenario | `max_stability_share` | Max Stability Payout | Activity Pool Floor | Risk Level |
|----------|----------------------|---------------------|--------------------|-----------|
| Conservative | 0.10 | 10% of Community Pool | 90% for activity | Low |
| **Baseline** | **0.30** | **30% of Community Pool** | **70% for activity** | **Medium** |
| Aggressive | 0.50 | 50% of Community Pool | 50% for activity | High |

**Impact analysis**: Higher cap allows more stability commitments but squeezes the activity reward pool. At 50%, a bank run on the stability tier would drain half the Community Pool, and activity-based participants would see rewards drop sharply, potentially triggering an engagement death spiral.

### Cross-Parameter Sensitivity Matrix

Pairwise interaction effects (High / Medium / Low / Negligible):

| | `r_base` | `burn_share` | `fee_trade` | `stability_rate` | `validator_share` | `n_validators` |
|---|---|---|---|---|---|---|
| `r_base` | --- | **High** | Medium | Low | Low | Negligible |
| `burn_share` | **High** | --- | **High** | Medium | **High** | Medium |
| `fee_trade` | Medium | **High** | --- | Medium | Medium | Medium |
| `stability_rate` | Low | Medium | Medium | --- | Low | Negligible |
| `validator_share` | Low | **High** | Medium | Low | --- | **High** |
| `n_validators` | Negligible | Medium | Medium | Negligible | **High** | --- |

The strongest interaction is between `burn_share` and `validator_share` (zero-sum constraint) and between `r_base` and `burn_share` (jointly determine equilibrium supply).

---

## 5. Stress Test Scenarios

### SC-001: Low Credit Volume

**Premise**: The voluntary carbon market contracts, or Regen fails to onboard new credit classes, causing a 90% decline in credit transaction volume from projected baseline.

| Dimension | Value |
|-----------|-------|
| **Initial conditions** | Baseline state at month 6 (post-M013 activation) |
| **Adversary model** | None (exogenous market shock) |
| **Volume trajectory** | Drop from $500K/month to $50K/month over 2 months; remain low for 12 months |
| **Expected trajectory** | Monthly fee revenue drops to ~$5K. Validator fund receives ~$2K/month. Per-validator income drops to ~$1,600/year (below operational costs). Community Pool inflow insufficient for stability tier obligations. Supply barely burns; minting continues, pushing supply toward cap. |
| **Failure threshold** | Per-validator income < $5,000/year for 3 consecutive months |
| **Key metrics to track** | Fee revenue, validator churn, stability tier solvency, supply trajectory |
| **Mitigation** | (1) Governance reduces validator set size to concentrate compensation. (2) Emergency burn share reduction to redirect fees to validators. (3) Community Pool reserve fund (pre-funded during good months). (4) Cross-chain fee capture from IBC transfers. |

### SC-002: High Validator Churn

**Premise**: Validator operators, dissatisfied with compensation or facing operational costs, leave the active set at high rates.

| Dimension | Value |
|-----------|-------|
| **Initial conditions** | 21 active validators, baseline fee revenue |
| **Adversary model** | None (rational exit behavior) |
| **Churn trajectory** | 50% of validators exit per quarter; new validators join at 25% replacement rate |
| **Expected trajectory** | Active set drops from 21 to ~11 in Q1, then oscillates between 11-15 as remaining validators receive higher compensation (fund/fewer validators). If set drops below 15, emergency governance triggers. Compensation per remaining validator increases, creating stabilizing feedback. |
| **Failure threshold** | Active set < `n_min` (15) for > 7 days; active set < 7 (Byzantine tolerance for 21-set) |
| **Key metrics to track** | Active validator count, per-validator compensation, block production, governance quorum |
| **Mitigation** | (1) Compensation floor guarantee from Community Pool reserve. (2) Automated recruitment agent (AGENT-004) activating standby validators. (3) Reduce `n_min` temporarily via emergency governance. |

### SC-003: Wash Trading Attack

**Premise**: An adversary controlling 30% of marketplace volume executes circular trades across sybil accounts to capture disproportionate M015 activity rewards.

| Dimension | Value |
|-----------|-------|
| **Initial conditions** | Baseline state; adversary has 500K REGEN budget |
| **Adversary model** | 10 sybil accounts executing buy-sell loops. Each loop: Account A sells to B at price P, B sells to C at P, ..., all accounts trade identical credit batches. Adversary pays 1% trade fee on each leg. |
| **Attack economics** | Per loop (10 legs at $10K each): fees paid = 10 * $10K * 0.01 = $1,000 in fees. Activity score generated = 10 * $10K * 0.30 (purchase weight) = $30K score-equivalent. If adversary captures 30% of activity pool ($3K/month) while paying $4K/month in fees: **attack is unprofitable**. |
| **Expected trajectory** | At baseline parameters, wash trading is net-negative due to M013 fees. However, if the adversary can claim sufficient activity rewards AND drive up REGEN price through volume appearance, combined returns may be positive. |
| **Failure threshold** | Adversary profit > 0 (attack is self-sustaining); adversary captures >50% of activity rewards |
| **Key metrics to track** | Wash trader P&L, legitimate participant reward dilution, fee revenue impact, activity score concentration (Gini coefficient) |
| **Mitigation** | (1) M013 fees as natural friction (current design). (2) Minimum transaction size for reward eligibility. (3) Graph analysis detecting circular flows (AGENT-003). (4) Quadratic scoring: score = sqrt(volume) to reduce large-volume advantage. |

**Critical finding from analytical model**: For wash trading to be unprofitable, the following must hold:
```
fee_rate_trade > (w_purchase * attacker_share * activity_pool) / (attack_volume * (1 - attacker_share_of_total_score))
```
At baseline: 0.01 > 0.30 * 0.30 * $10K / ($100K * 0.70) = 0.0129. This is **barely profitable at 30% volume capture**. Increasing `fee_trade` to 1.5% or reducing `w_purchase` to 0.20 restores the safety margin.

### SC-004: Stability Tier Bank Run

**Premise**: A price crash or negative news triggers 80% of stability tier holders to exit early in a single period.

| Dimension | Value |
|-----------|-------|
| **Initial conditions** | $2M in stability commitments; Community Pool balance = $50K |
| **Adversary model** | Panic behavior (not adversarial; rational response to perceived crisis) |
| **Exit trajectory** | 80% of committed holders invoke early exit in one epoch |
| **Expected trajectory** | $1.6M unlocked from stability tier. 50% early exit penalty applied: $48K in accrued rewards forfeited (returned to Community Pool). Stability multiplier (M012) drops sharply, reducing regrowth rate. If exits trigger REGEN selling, price drops, triggering more exits (reflexive feedback). |
| **Failure threshold** | Community Pool insolvent (stability obligations > balance); supply enters deflationary spiral (burn >> mint for >6 months) |
| **Key metrics to track** | Stability tier TVL, early exit rate, Community Pool solvency, regrowth rate, REGEN price |
| **Mitigation** | (1) 50% early exit penalty (current design) discourages panic exits. (2) 30% cap on stability allocation ensures Community Pool is never fully committed. (3) Queue-based exit processing (max exits per epoch) to prevent instantaneous drain. (4) Stability tier cooldown: no re-entry for 90 days after early exit. |

### SC-005: Fee Avoidance

**Premise**: 50% of credit trading volume migrates to off-chain or cross-chain venues to avoid M013 fees, reducing on-chain fee revenue.

| Dimension | Value |
|-----------|-------|
| **Initial conditions** | Baseline state with $500K/month on-chain volume |
| **Adversary model** | Rational market participants seeking lowest transaction costs; OTC desks and alternative registries capture volume |
| **Migration trajectory** | On-chain volume drops from $500K to $250K over 6 months |
| **Expected trajectory** | Fee revenue halves to ~$25K/month. Validator income drops to ~$14K/year (marginal viability). Community Pool inflow reduces activity rewards, lowering participation incentive. Burn rate decreases, supply drifts toward cap. Off-chain transactions miss retirement tracking and verification benefits. |
| **Failure threshold** | On-chain volume < $100K/month (protocol economically non-viable) |
| **Key metrics to track** | On-chain vs estimated total market volume, fee revenue, validator retention, reward pool adequacy |
| **Mitigation** | (1) Governance reduces fee rates to be competitive (but reduces per-transaction revenue). (2) Exclusive value proposition: only on-chain retirements count for verified impact claims. (3) On-chain retirement NFTs, impact certificates, and composability as non-replicable features. (4) KOI integration capturing cross-chain fees. |

### SC-006: Governance Deadlock

**Premise**: Political gridlock prevents any governance proposals from passing for 3 months. No parameter updates, no Community Pool spending proposals, no validator set changes.

| Dimension | Value |
|-----------|-------|
| **Initial conditions** | Baseline state; governance quorum requires 33% participation |
| **Adversary model** | Fragmented stakeholder interests; no coalition reaches quorum or majority |
| **Deadlock trajectory** | 3 months with zero passed proposals |
| **Expected trajectory** | M012-M015 continue operating on existing parameters (algorithmic, no governance input required for operation). Community Pool accumulates unspent funds. Validator set cannot be modified (no new approvals, no removals). Parameters cannot adapt to changing conditions. Stability tier commitments mature normally. Activity rewards distribute automatically. |
| **Failure threshold** | Critical parameter adjustment needed (e.g., fee rate change due to market shift) but cannot be enacted; validator set drops below minimum with no governance process to replenish |
| **Key metrics to track** | Governance participation rate, proposal passage rate, Community Pool accumulation, parameter staleness |
| **Mitigation** | (1) Mechanisms designed to operate autonomously --- M012/M013/M015 continue without governance input. (2) Layer 2 agentic governance (AGENT-004) can handle operational adjustments without full governance votes. (3) Emergency proposals with lower quorum thresholds for critical parameter changes. (4) Time-locked parameter adjustments that auto-execute if governance does not override. |

### SC-007: Supply Shock (Hard Cap Hit)

**Premise**: Supply reaches the hard cap through aggressive minting, then sustained burning with zero minting pushes supply into rapid contraction.

| Dimension | Value |
|-----------|-------|
| **Initial conditions** | `S[t] = C = 221M` (cap reached); high fee revenue driving burn |
| **Adversary model** | None (natural dynamics when cap is reached) |
| **Shock trajectory** | Minting falls to zero (C - S = 0). Burning continues at full rate. Supply contracts rapidly. |
| **Expected trajectory** | Once supply drops below cap, minting resumes: `M[t] = r * (C - S[t])`. Small gap (C - S) means small mint; large burn rate means net deflation continues. Supply undershoots equilibrium, then minting exceeds burning, creating dampened oscillation toward equilibrium. The system is inherently self-correcting but may oscillate for 6-12 months. |
| **Failure threshold** | Oscillation amplitude > 10% of supply; convergence time > 24 months |
| **Key metrics to track** | Supply trajectory, mint-burn ratio, oscillation amplitude, convergence rate |
| **Mitigation** | (1) The M012 formula is inherently stabilizing (negative feedback). (2) `r_base` can be tuned via governance to control oscillation damping. (3) If `S_0 > C` at launch (current supply 224M > 221M cap), the system starts in net-burn mode by design --- this IS the intended initial trajectory. |

**Analytical note**: The supply dynamics form a linear first-order system:
```
S[t+1] = S[t] + r*(C - S[t]) - B
       = (1 - r)*S[t] + r*C - B
```
With constant B, this converges to `S_eq = C - B/r` with time constant `tau = -1/ln(1-r)`. At r=0.02, tau ~= 50 periods (about 1 year of weekly epochs). Oscillation occurs only if B varies with S (which it does indirectly through price effects), but the system is never unstable for `0 < r < 1`.

### SC-008: Oracle Manipulation

**Premise**: The ecological multiplier oracle is compromised. Attacker manipulates the ecological data to set `eco_mult = 0` (halt all minting) or `eco_mult = 1.5` (maximize minting).

| Dimension | Value |
|-----------|-------|
| **Initial conditions** | Baseline state with eco_mult enabled (post-v0) |
| **Adversary model** | Oracle data source compromised; attacker can set arbitrary ecological metric values |
| **Attack vector A** | Set eco_mult = 0: minting halts entirely. Supply only burns. Deflation accelerates. Token price may increase (supply reduction), but minting stop removes ecosystem growth coupling. |
| **Attack vector B** | Set eco_mult = 1.5 (above reference): minting rate increases 50%. Supply inflates faster than burn rate can compensate. If sustained, supply approaches cap and dilutes existing holders. |
| **Failure threshold** | Manipulated eco_mult persists for > 2 periods (14 days) without detection; supply deviation > 5% from expected trajectory |
| **Key metrics to track** | eco_mult value, mint rate anomaly, supply trajectory deviation from forecast, oracle data integrity checks |
| **Mitigation** | (1) v0 launches with eco_mult = 1.0 (disabled) until oracle is battle-tested. (2) eco_mult bounded to [0.0, 1.5] at protocol level. (3) Rate-of-change limit: eco_mult cannot change by more than 0.1 per period. (4) Multi-oracle median with outlier rejection. (5) Governance circuit-breaker: automatic eco_mult freeze if value deviates > 2 standard deviations from 30-period moving average. |

### Stress Test Summary Matrix

| Scenario | Probability | Severity | Detection Difficulty | Recovery Time | Residual Risk |
|----------|------------|----------|---------------------|---------------|---------------|
| SC-001: Low Volume | Medium | High | Low (observable) | 6-12 months | Medium |
| SC-002: Validator Churn | Medium | High | Low (observable) | 1-3 months | Low |
| SC-003: Wash Trading | High | Medium | Medium (requires analysis) | Immediate (param change) | Medium |
| SC-004: Bank Run | Low | High | Low (observable) | 3-6 months | Medium |
| SC-005: Fee Avoidance | Medium | Medium | Medium (off-chain) | 6-12 months | High |
| SC-006: Gov. Deadlock | Low | Low-Medium | Low (observable) | Variable | Low |
| SC-007: Supply Shock | Medium | Low | Low (observable) | 6-12 months | Low |
| SC-008: Oracle Attack | Low | High | High (subtle) | Days-weeks | Low (with mitigations) |

---

## 6. Expected Equilibrium Ranges

### Supply Equilibrium

At equilibrium, `M[t] = B[t]`:

```
r * eff_mult * eco_mult * (C - S_eq) = B_eq
```

**Solving for S_eq**:

B_eq depends on fee revenue:
```
B_eq = burn_share * total_fees_in_REGEN_per_period
     = burn_share * (sum over tx_types: V_type * fee_rate_type) / price
```

Let `F_USD` = total monthly fee revenue in USD. Then:
```
B_eq (weekly) = burn_share * F_USD / (price * periods_per_month)
              = 0.30 * F_USD / (price * 4.33)
```

For the full equilibrium:
```
S_eq = C - B_eq / (r * eff_mult * eco_mult)
```

**Numerical examples** (assuming eco_mult = 1.0, eff_mult = 1.5 mid-range):

| Monthly Fee Rev. (USD) | REGEN Price | B_eq (weekly REGEN) | S_eq (REGEN) | Gap from Cap |
|------------------------|-------------|--------------------|--------------|--------------|
| $20,000 | $0.03 | 46,189 | 219,461,000 | 1.54M below cap |
| $50,000 | $0.03 | 115,473 | 217,153,000 | 3.85M below cap |
| $50,000 | $0.10 | 34,642 | 219,845,000 | 1.15M below cap |
| $100,000 | $0.05 | 138,568 | 216,381,000 | 4.62M below cap |
| $200,000 | $0.10 | 138,568 | 216,381,000 | 4.62M below cap |
| $500,000 | $0.30 | 115,473 | 217,153,000 | 3.85M below cap |

**Key insight**: At low price and high volume, the equilibrium supply sits further below the cap (more tokens burned per period). At higher price, each USD of fees buys fewer REGEN to burn, so equilibrium supply sits closer to cap. The system has a natural price-stabilizing property: price increases reduce burn pressure, allowing supply to grow, which puts downward pressure on price.

### Fee Revenue Equilibrium

Given projected credit market volumes:

| Volume Scenario | Monthly Credit Volume (USD) | Weighted Avg Fee Rate | Monthly Fee Revenue |
|----------------|----------------------------|-----------------------|--------------------|
| Bear | $1M | 1.2% | $12,000 |
| Base | $5M | 1.2% | $60,000 |
| Bull | $20M | 1.2% | $240,000 |
| Moonshot | $100M | 1.2% | $1,200,000 |

The weighted average fee rate of 1.2% assumes a transaction mix of 20% issuance (2%), 10% transfer (0.1%), 30% retirement (0.5%), and 40% trade (1%).

### Validator Income

Per-validator annual income = `(validator_share * annual_fee_revenue) / n_validators`:

| Annual Fee Revenue | Validator Share | n_validators | Per-Validator Annual | Monthly |
|-------------------|----------------|-------------|---------------------|---------|
| $144K (bear) | 40% | 15 | $3,840 | $320 |
| $144K (bear) | 40% | 21 | $2,743 | $229 |
| $720K (base) | 40% | 15 | $19,200 | $1,600 |
| $720K (base) | 40% | 21 | $13,714 | $1,143 |
| $2.88M (bull) | 40% | 15 | $76,800 | $6,400 |
| $2.88M (bull) | 40% | 21 | $54,857 | $4,571 |

**Critical finding**: In the bear case, per-validator income is below operational costs (~$15K/year for a professional validator). This confirms SC-001 as a real risk. Validator viability requires either: (a) fee revenue > $562K/year at 40% share and 15 validators, or (b) reducing the validator set to concentrate compensation, or (c) supplementary funding from Community Pool during ramp-up.

### Stability Tier Capacity

The stability tier is capped at `max_stability_share` (30%) of Community Pool inflow. Maximum sustainable stability commitments:

```
max_committed = (community_share * annual_fee_revenue * max_stability_share) / stability_rate
```

| Annual Fee Revenue | Community Share | Max Stability Share | Stability Rate | Max Committed |
|-------------------|----------------|--------------------|---------|---------|
| $144K | 25% | 30% | 6% | $180,000 |
| $720K | 25% | 30% | 6% | $900,000 |
| $2.88M | 25% | 30% | 6% | $3,600,000 |

**Key finding**: At base-case fee revenue ($720K/year), the stability tier can support up to $900K in committed REGEN (at current prices ~$0.03, that is 30M REGEN or ~13.5% of supply). This is a meaningful but not excessive commitment level. At bear-case revenue, only $180K can be committed, which is too small to attract meaningful participation.

### Activity Reward Per Participant

Activity pool = Community Pool inflow - stability allocation:

| Scenario | Monthly Activity Pool | Active Participants | Avg Monthly Reward |
|----------|----------------------|--------------------|--------------------|
| Bear, few participants | $2,100 | 50 | $42 |
| Base, moderate | $8,750 | 200 | $44 |
| Base, many | $8,750 | 500 | $18 |
| Bull, moderate | $35,000 | 200 | $175 |
| Bull, many | $35,000 | 1,000 | $35 |

**Key finding**: Activity rewards are modest even in the bull case. The primary value proposition is not direct reward income but rather governance participation rights and ecological impact attribution. Rewards serve as a supplementary incentive, not primary income.

---

## 7. Monte Carlo Inputs

For each stochastic input, we specify a probability distribution, rationale, and sampling method.

### Credit Trading Volume

| Property | Value |
|----------|-------|
| **Distribution** | Log-normal |
| **Monthly mean (USD)** | $5,000,000 |
| **Monthly sigma (log-scale)** | 0.8 |
| **Rationale** | Credit markets exhibit fat-tailed volume distributions; log-normal captures the right-skew observed in voluntary carbon markets. The mean reflects Regen's target market share of the VCM. |
| **Growth trend** | 5% monthly compounding growth overlay (reflecting VCM expansion) |
| **Seasonal factor** | 1.3x in Q4 (corporate offset purchases), 0.8x in Q1 |
| **Correlation** | Positively correlated with REGEN price (r=0.3) --- higher price signals ecosystem health, attracting more volume |

```python
def sample_credit_volume(month, price, rng):
    base_mean = 5_000_000 * (1.05 ** month)
    seasonal = [0.8, 0.9, 1.0, 1.3][month % 4]  # Q1-Q4
    price_factor = (price / 0.03) ** 0.3  # Mild positive correlation
    mean = base_mean * seasonal * price_factor
    return rng.lognormal(np.log(mean) - 0.5 * 0.8**2, 0.8)
```

### Credit Retirement Rate

| Property | Value |
|----------|-------|
| **Distribution** | Beta(2, 5) scaled to [0.05, 0.40] |
| **Mean retirement fraction** | ~0.15 (15% of outstanding credits retired per month) |
| **Rationale** | Retirement is driven by compliance deadlines and voluntary commitments. Beta distribution captures the bounded nature (cannot exceed holdings) with right-skew toward lower rates. |
| **Shock events** | 5% chance per month of a compliance deadline spike (retirement rate jumps to 0.50 for one month) |

### New Credit Issuance Rate

| Property | Value |
|----------|-------|
| **Distribution** | Poisson arrivals, log-normal batch size |
| **Arrival rate** | 8 new batches per month (lambda=8) |
| **Batch size mean** | $200,000 |
| **Batch size sigma** | 1.0 (log-scale) |
| **Rationale** | Credit issuance is project-driven with irregular timing. Poisson captures the random arrival; log-normal batch size reflects wide variation from small community projects to large forestry programs. |
| **Growth trend** | Lambda increases by 0.5 per quarter (new project onboarding) |

### Validator Application Rate

| Property | Value |
|----------|-------|
| **Distribution** | Poisson(lambda=0.5/month) |
| **Acceptance probability** | 0.6 (conditional on slot availability) |
| **Exit rate** | 0.02/month baseline; increases if compensation < $1,000/month |
| **Rationale** | Validator applications are rare events. Exit rate is compensation-sensitive --- validators are mission-aligned but need minimum viable economics. |

```python
def validator_dynamics(state, params, rng):
    # Applications
    n_applications = rng.poisson(0.5)
    slots_available = params['n_max'] - state['n_validators']
    new_validators = min(n_applications, slots_available) * (rng.random() < 0.6)

    # Exits
    monthly_income = state['per_validator_payment'] * 4.33  # weekly to monthly
    exit_prob = 0.02 if monthly_income > 1000 else 0.02 + 0.15 * (1 - monthly_income/1000)
    exits = sum(rng.random(state['n_validators']) < exit_prob)

    return state['n_validators'] + new_validators - exits
```

### Stability Tier Adoption

| Property | Value |
|----------|-------|
| **Distribution** | Logistic growth curve with noise |
| **Saturation level** | 20% of liquid supply (in REGEN terms) |
| **Growth rate** | k = 0.15/month (half-saturation at month 12) |
| **Noise** | Multiplicative log-normal noise, sigma=0.3 |
| **Exit rate** | 2%/month baseline; spike to 20%/month under price shock (>30% price decline) |
| **Rationale** | Adoption follows S-curve as awareness grows. Saturation is limited by willingness to lock tokens. Exit correlated with price volatility. |

```python
def stability_adoption(month, supply, price, price_history, rng):
    max_committed = 0.20 * supply  # 20% saturation
    logistic = max_committed / (1 + np.exp(-0.15 * (month - 12)))
    noise = rng.lognormal(0, 0.3)
    target = logistic * noise

    # Price shock exits
    if len(price_history) > 1:
        price_change = price / price_history[-1] - 1
        if price_change < -0.30:
            exit_rate = 0.20
        else:
            exit_rate = 0.02

    return target, exit_rate
```

### Governance Participation Rate

| Property | Value |
|----------|-------|
| **Distribution** | Beta(3, 7) --- mean ~0.30 |
| **Current baseline** | ~5% (per token-economics-synthesis.md) |
| **12-month target** | 15% |
| **24-month target** | 30% |
| **Proposal frequency** | Poisson(lambda=3/month) |
| **Rationale** | Governance participation follows a Beta distribution (bounded 0-1) with current low engagement gradually increasing as M015 activity rewards incentivize voting. |

### REGEN Token Price

| Property | Value |
|----------|-------|
| **Model** | Geometric Brownian Motion with mean-reversion |
| **Initial price** | $0.03 |
| **Drift** | 0.5%/month (mild appreciation reflecting network growth) |
| **Volatility** | 40%/month (crypto-typical) |
| **Mean-reversion target** | Fundamental value = f(fee_revenue, supply) |
| **Mean-reversion speed** | 0.05/month |
| **Rationale** | Token price is driven by both speculative dynamics (GBM) and fundamental value (fee revenue / supply). Mean-reversion reflects eventual market pricing of fundamentals. |

```python
def price_model(prev_price, fee_revenue, supply, rng):
    fundamental = (fee_revenue * 12 * 20) / supply  # P/E-like fundamental
    drift = 0.005 + 0.05 * (fundamental / prev_price - 1)  # Mean-revert
    shock = rng.normal(0, 0.40)
    new_price = prev_price * np.exp(drift + shock - 0.5 * 0.40**2)
    return max(0.001, new_price)  # Floor at $0.001
```

### Summary of Distributions

| Input | Distribution | Mean | Key Parameters | Correlation |
|-------|-------------|------|----------------|-------------|
| Credit volume | Log-normal | $5M/mo | sigma=0.8, growth=5%/mo | Price (+0.3) |
| Retirement rate | Beta(2,5) | 15%/mo | range [5%, 40%] | Compliance events |
| Issuance rate | Poisson + Log-normal | 8 batches, $200K each | lambda growth +0.5/Q | Independent |
| Validator apps | Poisson | 0.5/mo | acceptance=0.6 | Independent |
| Stability adoption | Logistic + noise | 20% saturation | k=0.15, sigma=0.3 | Price (-0.5 for exits) |
| Governance | Beta(3,7) | 30% | evolving over time | Reward level (+0.2) |
| Token price | GBM + mean-reversion | $0.03 initial | vol=40%, drift=0.5% | Fee revenue, supply |

---

## 8. Simulation Outputs

### Time Series Outputs

Generated for every simulation run (per-epoch resolution, 260 epochs for 5-year run):

| Output | Description | Alert Threshold |
|--------|-------------|-----------------|
| `supply[t]` | Circulating supply of REGEN | < 100M or > 221M |
| `minted[t]` | Tokens minted this period | Sudden spike > 3x rolling average |
| `burned[t]` | Tokens burned this period | Drop to 0 for > 4 periods |
| `fee_revenue_usd[t]` | Total fee revenue in USD | < $5K/month for > 3 months |
| `fee_revenue_regen[t]` | Total fee revenue in REGEN | N/A (price-dependent) |
| `pool_burn[t]` | Burn pool balance | Accumulation > 1M REGEN (not being burned) |
| `pool_validator[t]` | Validator fund balance | < min required for compensation |
| `pool_community[t]` | Community pool balance | < stability tier obligations |
| `pool_agent[t]` | Agent infra fund balance | < operational minimum |
| `n_validators[t]` | Active validator count | < 15 (n_min) |
| `per_validator_income[t]` | Per-validator compensation | < $1,250/month |
| `S_stability[t]` | Total stability committed | Exceeds sustainable capacity |
| `stability_payout[t]` | Stability tier distribution | Hits 30% cap |
| `activity_pool[t]` | Activity reward distribution | < $2K/month |
| `price[t]` | REGEN token price | < $0.005 or > $1.00 |
| `mint_burn_ratio[t]` | M[t] / B[t] | > 2.0 or < 0.5 for extended periods |

### Distribution Outputs (Across Monte Carlo Runs)

Generated by aggregating N=1,000 Monte Carlo runs at key time points (month 6, 12, 18, 24, 36, 48, 60):

| Output | Description | Visualization |
|--------|-------------|---------------|
| Equilibrium supply distribution | Distribution of S[t] across runs at each time point | Histogram + confidence intervals |
| Validator income distribution | Per-validator annual income across runs | Box plot by time point |
| Activity reward distribution | Per-participant monthly reward across runs | Histogram |
| Fee revenue distribution | Monthly fee revenue across runs | Fan chart (percentile bands) |
| Time-to-equilibrium distribution | Months until M[t] ~= B[t] | Survival curve / CDF |
| Stability tier solvency | % of runs where stability tier is solvent at each time | Line chart |
| Supply floor events | % of runs where supply < X at any point | Exceedance probability curve |

### Heat Maps (Parameter Sweep)

Two-dimensional parameter sweeps showing system behavior across parameter pairs:

| Heat Map | X-axis | Y-axis | Color (z-axis) |
|----------|--------|--------|----------------|
| Supply equilibrium | `r_base` [0.005, 0.10] | `burn_share` [0.00, 0.35] | Equilibrium supply level |
| Validator viability | `fee_trade` [0.005, 0.02] | `n_validators` [10, 25] | Per-validator annual income |
| Stability sustainability | `stability_rate` [0.02, 0.12] | `max_stability_share` [0.10, 0.50] | Max sustainable commitments |
| Wash trade safety | `fee_trade` [0.005, 0.02] | `w_purchase` [0.10, 0.50] | Attacker profit margin |
| Revenue vs. volume | `fee_issuance` [0.01, 0.03] | `fee_trade` [0.005, 0.02] | Monthly fee revenue (with elasticity) |
| Time to equilibrium | `r_base` [0.005, 0.10] | Monthly volume [$1M, $20M] | Months to M=B convergence |

Each heat map uses a 20x20 grid (400 parameter combinations) with 100 Monte Carlo runs per cell.

### Failure Probability Outputs

Critical safety metrics aggregated across all Monte Carlo runs:

| Failure Mode | Definition | Acceptable Threshold |
|-------------|------------|---------------------|
| Supply collapse | `S[t] < 100M` at any point in 5 years | < 1% of runs |
| Validator exodus | `n_validators < n_min` for > 30 days | < 5% of runs |
| Stability insolvency | Stability obligations > Community Pool balance | < 5% of runs |
| Revenue failure | Monthly fee revenue < $10K for > 6 consecutive months | < 10% of runs |
| Equilibrium failure | System does not reach equilibrium within 48 months | < 30% of runs |
| Death spiral | Supply, price, and volume all declining for > 6 months | < 2% of runs |
| Wash trade profitability | Adversary achieves > 0 profit from wash trading | < 10% of runs |

**Composite safety score**: Weighted average of (1 - failure_probability) across all failure modes. Target: > 0.90.

---

## 9. Implementation Notes

### Environment Setup

```bash
# Python 3.10+ required
python -m venv sim-env
source sim-env/bin/activate

pip install cadcad==0.5.3
pip install numpy pandas scipy matplotlib seaborn plotly
pip install jupyter  # For interactive analysis
```

### Project Structure

```
simulation/
├── config/
│   ├── baseline.yaml          # Baseline parameter values
│   ├── stress_tests/
│   │   ├── sc001_low_volume.yaml
│   │   ├── sc002_validator_churn.yaml
│   │   ├── sc003_wash_trading.yaml
│   │   ├── sc004_bank_run.yaml
│   │   ├── sc005_fee_avoidance.yaml
│   │   ├── sc006_governance_deadlock.yaml
│   │   ├── sc007_supply_shock.yaml
│   │   └── sc008_oracle_manipulation.yaml
│   └── sweeps/
│       ├── supply_equilibrium.yaml
│       ├── validator_viability.yaml
│       └── stability_sustainability.yaml
├── model/
│   ├── __init__.py
│   ├── state.py               # State variable definitions
│   ├── policies/
│   │   ├── fee_collection.py   # M013 policy function
│   │   ├── supply_update.py    # M012 policy function
│   │   ├── compensation.py     # M014 policy function
│   │   └── rewards.py          # M015 policy function
│   ├── agents/
│   │   ├── credit_issuer.py
│   │   ├── credit_buyer.py
│   │   ├── credit_retirer.py
│   │   ├── validator.py
│   │   ├── stability_holder.py
│   │   ├── governance_participant.py
│   │   └── wash_trader.py
│   └── price_model.py          # Exogenous/endogenous price dynamics
├── experiments/
│   ├── baseline_run.py         # Single baseline trajectory
│   ├── monte_carlo.py          # N=1000 Monte Carlo runs
│   ├── parameter_sweep.py      # 2D parameter sweep heat maps
│   ├── stress_test_runner.py   # Run all 8 stress scenarios
│   └── sensitivity_analysis.py # Sobol/Morris sensitivity indices
├── analysis/
│   ├── equilibrium_analysis.py # Closed-form equilibrium computations
│   ├── failure_detection.py    # Failure mode classification
│   └── report_generator.py     # Generate summary tables and charts
├── visualization/
│   ├── time_series.py          # Supply, revenue, pool balance plots
│   ├── distributions.py        # Histograms, box plots, fan charts
│   ├── heat_maps.py            # Parameter sweep heat maps
│   └── dashboard.py            # Interactive Plotly dashboard
└── README.md
```

### cadCAD Configuration

```python
from cadcad.configuration import Experiment
from cadcad.configuration.utils import config_sim

sim_config = config_sim({
    'N': 1,           # Number of runs per config (use MC wrapper for multiple)
    'T': range(260),  # 260 weekly epochs = 5 years
    'M': baseline_params
})

partial_state_update_blocks = [
    {
        'label': 'Agent Behavior',
        'policies': {'agent_actions': agent_behavior_policy},
        'variables': {
            'credit_volumes': update_credit_volumes,
            'activity_scores': update_activity_scores,
            'n_validators': update_validator_count,
            'S_stability': update_stability_commitments
        }
    },
    {
        'label': 'Fee Collection (M013)',
        'policies': {'fees': fee_collection},
        'variables': {
            'pool_burn': update_burn_pool,
            'pool_validator': update_validator_pool,
            'pool_community': update_community_pool,
            'pool_agent': update_agent_pool,
            'total_fees': update_total_fees
        }
    },
    {
        'label': 'Reward Distribution (M015)',
        'policies': {'rewards': reward_distribution},
        'variables': {
            'pool_community': update_community_post_rewards,
            'stability_payout': update_stability_payout,
            'activity_rewards': update_activity_rewards
        }
    },
    {
        'label': 'Validator Compensation (M014)',
        'policies': {'compensation': validator_compensation},
        'variables': {
            'pool_validator': update_validator_post_compensation,
            'per_validator_payment': update_per_validator_payment
        }
    },
    {
        'label': 'Supply Update (M012)',
        'policies': {'supply': supply_update},
        'variables': {
            'supply': update_supply,
            'total_minted': update_total_minted,
            'total_burned': update_total_burned,
            'pool_burn': reset_burn_pool
        }
    },
    {
        'label': 'Price Update',
        'policies': {'price': price_update_policy},
        'variables': {
            'price': update_price
        }
    }
]

experiment = Experiment()
experiment.append_configs(
    initial_state=initial_state,
    partial_state_update_blocks=partial_state_update_blocks,
    sim_configs=sim_config
)
```

### Running Experiments

```bash
# Baseline single run (quick validation)
python experiments/baseline_run.py --config config/baseline.yaml --output results/baseline/

# Full Monte Carlo (1000 runs, ~2-4 hours on 8-core machine)
python experiments/monte_carlo.py --config config/baseline.yaml --n-runs 1000 --output results/monte_carlo/

# Parameter sweep (400 cells x 100 runs = 40,000 runs; parallelizable)
python experiments/parameter_sweep.py --sweep config/sweeps/supply_equilibrium.yaml --n-runs 100 --output results/sweeps/

# All stress tests
python experiments/stress_test_runner.py --scenarios config/stress_tests/ --n-runs 500 --output results/stress/

# Global sensitivity analysis (Sobol indices)
python experiments/sensitivity_analysis.py --config config/baseline.yaml --n-samples 4096 --output results/sensitivity/
```

### Output Visualization

```bash
# Generate all standard plots
python visualization/dashboard.py --input results/monte_carlo/ --output reports/figures/

# Interactive dashboard (opens browser)
python visualization/dashboard.py --input results/monte_carlo/ --interactive
```

### Computational Requirements

| Experiment | Runs | Time per Run | Total Time (8 cores) | Memory |
|------------|------|-------------|---------------------|--------|
| Baseline | 1 | ~2 seconds | 2 seconds | 100 MB |
| Monte Carlo | 1,000 | ~2 seconds | ~4 minutes | 2 GB |
| Parameter Sweep | 40,000 | ~2 seconds | ~3 hours | 8 GB |
| Stress Tests (all 8) | 4,000 | ~2 seconds | ~17 minutes | 4 GB |
| Sensitivity Analysis | 4,096 | ~2 seconds | ~17 minutes | 4 GB |
| **Full suite** | **~50,000** | --- | **~4 hours** | **8 GB peak** |

---

## 10. Summary

### Key Analytical Findings

1. **The supply dynamics are inherently stable.** The M012 formula `M[t] = r * (C - S[t])` creates negative feedback: as supply approaches the cap, minting decelerates. Combined with fee-driven burning, the system converges to a well-defined equilibrium. The time constant is approximately `1/r_base` periods (~50 weeks at r=0.02). There are no unstable fixed points or chaotic attractors in the deterministic model.

2. **Validator economics are the tightest constraint.** At baseline fee revenue projections ($50-60K/month), per-validator income is $13-19K/year. This is marginal for professional operators. The system is one bad quarter away from validator exodus in the bear case. Mitigation: consider a higher validator share (Model B's 15-25% may be too low; Model A's 40% is more prudent for the ramp-up phase), or establish a validator compensation floor funded by a Community Pool reserve.

3. **Wash trading is near-profitable at baseline parameters.** The analytical wash trade model shows the attack is marginal at 1% trade fee and 0.30 purchase weight. Small parameter changes (reducing purchase weight to 0.20, or increasing trade fee to 1.5%) restore a clear safety margin. This should be a priority calibration target for the simulation.

4. **Stability tier capacity is limited but sustainable.** At base-case revenue, the 30% cap on Community Pool allocation supports approximately $900K in committed REGEN. This provides a meaningful staking alternative but will not accommodate demand from large holders. The 6% return is sustainable only if fee revenue meets projections.

5. **The burn share vs. contributor funding tradeoff is real.** Eliminating burn (as discussed in OQ-M013-5) increases validator income by ~33% and community rewards by ~43% but removes the deflationary price support mechanism. The simulation should test both Model A (30% burn) and the zero-burn variant to quantify the long-term price and ecosystem health implications.

6. **Oracle risk is the highest-severity, lowest-probability threat.** The ecological multiplier, once enabled, has the power to halt all minting (eco_mult=0) or accelerate it dramatically. The v0 approach of disabling it (eco_mult=1.0) is prudent. When activated, it requires robust multi-oracle infrastructure with circuit breakers.

### Recommended Simulation Priority

| Priority | Experiment | Question Answered |
|----------|-----------|-------------------|
| P0 | Baseline Monte Carlo (1,000 runs) | Does the system converge to equilibrium under expected conditions? |
| P0 | SC-001 (Low Volume) | What is the minimum viable fee revenue? |
| P0 | SC-003 (Wash Trading) | Are baseline parameters safe against wash trading? |
| P1 | Parameter sweep: `r_base` x `burn_share` | What is the safe operating envelope for supply dynamics? |
| P1 | Parameter sweep: `fee_trade` x `n_validators` | What fee level sustains validator economics? |
| P1 | SC-004 (Bank Run) | Does the stability tier survive a panic event? |
| P2 | SC-005 (Fee Avoidance) | How sensitive is the system to off-chain migration? |
| P2 | SC-008 (Oracle Manipulation) | What circuit breakers are needed for eco_mult? |
| P2 | Global sensitivity analysis (Sobol) | Which parameters require the most careful governance? |
| P3 | Full stress test suite | Comprehensive resilience assessment |

### Open Questions for Simulation Resolution

These questions from M012-M015 can be directly answered by simulation results:

| Open Question | Simulation Approach |
|--------------|---------------------|
| OQ-M012-1: Exact hard cap value | Sweep C in [200M, 250M]; measure equilibrium behavior |
| OQ-M012-3: Period length | Compare weekly vs. daily epochs; measure supply volatility |
| OQ-M013-1: Distribution model (A vs B) | Run both; compare validator income, reward pool, supply trajectory |
| OQ-M013-5: Should burn exist? | Run burn_share in {0, 0.10, 0.25, 0.30}; measure long-term system health |
| OQ-M015-1: Is 6% the right stability rate? | Sweep stability_rate in [2%, 12%]; measure solvency and adoption |
| OQ-M015-4: Anti-gaming measures | Model wash trader P&L across parameter space; find safety boundary |

---

## References

- [Phase 2.6: Economic Reboot Mechanism Specifications](../../phase-2/2.6-economic-reboot-mechanisms.md) --- M012, M013, M014, M015 formal specifications
- [Token Economics Synthesis](./token-economics-synthesis.md) --- Consolidated economic architecture
- [cadCAD Documentation](https://cadcad.org/) --- Simulation framework
- [Blockscience Token Engineering](https://block.science/) --- Carrying capacity model inspiration
- [Regen Economic Reboot Roadmap](https://forum.regen.network/t/regen-economic-reboot-roadmap/567) --- Community proposal
- [Network Coordination Architecture](https://forum.regen.network/t/regen-tokenomics-wg/19/67) --- Gregory's distribution model
- [Max Semenchuk Model Comparison](https://maxsemenchuk.github.io/regen-model-comparison/) --- Reward vs. institutional model analysis

---

*This document is part of the Regen Network Agentic Tokenomics framework.*
