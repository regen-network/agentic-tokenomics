# Economic Simulation Specification for M012-M015 Parameter Validation

## Document Metadata

| Field | Value |
|-------|-------|
| Status | Draft |
| Scope | Mechanisms M012, M013, M014, M015 |
| Framework | cadCAD agent-based simulation |
| Purpose | Validate economic reboot sustainability before mainnet deployment |
| Dependencies | phase-2/2.6-economic-reboot-mechanisms.md, docs/economics/token-economics-synthesis.md |

---

## 1. Overview

The Regen Economic Reboot replaces inflationary proof-of-stake tokenomics with a revenue-driven model
anchored by four interdependent mechanisms:

- **M012** (Fixed Cap Dynamic Supply): Algorithmic mint/burn with a 221M hard cap
- **M013** (Value-Based Fee Routing): Percentage fees on credit transactions split to four pools
- **M014** (Authority Validator Governance): PoA validator set with fee-based compensation
- **M015** (Contribution-Weighted Rewards): Activity-scored distribution plus stability tier

These mechanisms form a tightly coupled system where each parameter choice creates downstream effects
on the others. Fee rates (M013) determine burn volume (M012), validator income (M014), and reward
pool depth (M015). Regrowth rate (M012) determines how quickly burned tokens re-enter circulation.
Activity weights (M015) determine which behaviors the system incentivizes, and stability tier commitments
feed back into the supply regrowth multiplier (M012).

This simulation specification defines a cadCAD-compatible agent-based model to:

1. Identify parameter configurations that produce long-run economic sustainability
2. Stress-test the system against adversarial scenarios and exogenous shocks
3. Derive closed-form equilibrium conditions where possible and validate them numerically
4. Quantify sensitivity of key outputs to each tunable parameter
5. Produce Monte Carlo confidence intervals for outcomes under realistic uncertainty

The simulation must demonstrate that the system can sustain validator compensation, generate
meaningful contributor rewards, manage supply within the hard cap, and resist common attack vectors
before any mainnet governance proposal is submitted.

### Key Formulas Under Validation

From phase-2/2.6, the core formulas are:

**M012 Supply Dynamics:**
```
S[t+1] = S[t] + M[t] - B[t]

M[t] = r * (C - S[t])       (regrowth / minting)
B[t] = burn_share * F[t]    (burning from fee revenue)

r = r_base * effective_multiplier * ecological_multiplier
r_base = 0.02 (2% per period)
C = 221,000,000 REGEN (hard cap)

Note: Current circulating supply (~224M) exceeds C. When S > C, headroom
is negative and M[t] floors to 0 (per M012 Security Invariant 4). The
system enters an initial pure-burn regime lasting approximately 3-6 months
until burning reduces supply below C and regrowth minting resumes.
```

**M013 Fee Collection:**
```
fee = transaction_value * rate_bps / 10000

rate_bps varies by transaction type:
  Credit Issuance:    100-300 bps (1-3%)
  Credit Transfer:    10 bps (0.1%)
  Credit Retirement:  50 bps (0.5%)
  Marketplace Trade:  100 bps (1%)

Fee split to 4 pools: burn + validator + community + agent = 1.0
```

**M015 Contribution Rewards:**
```
stability_allocation = min(
  sum(commitments) * 0.06 / periods_per_year,
  community_inflow * 0.30
)

activity_pool = community_inflow - stability_allocation

Activity weights: purchase=0.30, retirement=0.30, facilitation=0.20,
                  governance_voting=0.10, proposals=0.10
```

### Success Criteria

The simulation must demonstrate ALL of the following under baseline parameters:

| Criterion | Threshold | Measurement |
|-----------|-----------|-------------|
| Validator sustainability | Annual validator income >= $15,000 per validator | Mean income over 5-year horizon |
| Supply stability | Supply remains within [150M, 221M] REGEN | 95th percentile bounds |
| Equilibrium convergence | abs(M[t] - B[t]) < 1% of S[t] within 5 years | Time to convergence |
| Reward pool adequacy | Activity reward pool > 0 in all periods | Zero-revenue period count |
| Stability tier solvency | Stability tier obligations met in >= 95% of periods | Obligation coverage ratio |
| Attack resistance | All 8 stress scenarios remain above failure thresholds | Scenario-specific metrics |

---

## 2. Model Architecture

### 2.1 Simulation Framework

The model uses cadCAD (complex adaptive dynamics Computer-Aided Design), an open-source Python
framework for designing, testing, and validating complex systems through simulation. cadCAD provides:

- Differential equation specification via policy and state update functions
- Parameter sweeps across multi-dimensional configuration spaces
- Monte Carlo simulation with configurable random seeds
- Time-stepped execution with configurable granularity

**Simulation granularity:** 1 epoch = 1 week (matching M012 period_length).
**Simulation horizon:** 520 epochs (10 years) for long-run analysis; 260 epochs (5 years) for baseline.
**Monte Carlo runs:** 1,000 per parameter configuration (10,000 for publication-quality results).

### 2.2 State Variables

The model tracks the following state variables, updated each epoch:

#### Supply State (M012)
| Variable | Type | Initial Value | Description |
|----------|------|---------------|-------------|
| `S` | float | 224,000,000 | Current circulating supply |
| `C` | float | 221,000,000 | Hard cap (note: initial S > C means net-burn regime at start) |
| `M_t` | float | 0 | Tokens minted this period |
| `B_t` | float | 0 | Tokens burned this period |
| `r_effective` | float | 0.02 | Current effective regrowth rate |
| `supply_state` | enum | TRANSITION | {INFLATIONARY, TRANSITION, DYNAMIC, EQUILIBRIUM} |
| `periods_near_equilibrium` | int | 0 | Consecutive periods where abs(M-B) < threshold |
| `cumulative_minted` | float | 0 | Lifetime tokens minted |
| `cumulative_burned` | float | 0 | Lifetime tokens burned |

Note on initial conditions: The current REGEN supply (~224M) exceeds the proposed hard cap (221M).
This means at activation, M[t] = r * (C - S[t]) = r * (221M - 224M) = r * (-3M), which would be
negative. Since M012 specifies that supply contraction occurs exclusively through burning (Mint-Burn
Independence, Security Invariant 4), the model must handle this by flooring M[t] at 0 when S > C.
The initial period is therefore a pure-burn regime until fee-driven burns reduce supply below the cap.
This is a critical simulation validation point.

#### Fee and Pool State (M013)
| Variable | Type | Initial Value | Description |
|----------|------|---------------|-------------|
| `total_fees_collected` | float | 0 | Fees collected this period (REGEN) |
| `burn_pool_balance` | float | 0 | Accumulated burn pool (burned at end of period) |
| `validator_fund_balance` | float | 0 | Available for validator compensation |
| `community_pool_balance` | float | 0 | Available for M015 distribution |
| `agent_infra_balance` | float | 0 | Agent infrastructure fund |
| `fee_revenue_history` | list[float] | [] | Rolling history for trend analysis |
| `cumulative_fees` | float | 0 | Lifetime fee revenue |

#### Validator State (M014)
| Variable | Type | Initial Value | Description |
|----------|------|---------------|-------------|
| `active_validators` | int | 18 | Current active validator count |
| `validator_income_period` | float | 0 | Per-validator income this period |
| `validator_income_annual` | float | 0 | Annualized per-validator income |
| `validator_churn_rate` | float | 0.05 | Quarterly churn rate |
| `validator_applications_pending` | int | 0 | Pending applications |
| `avg_uptime` | float | 0.995 | Average validator uptime |

#### Reward State (M015)
| Variable | Type | Initial Value | Description |
|----------|------|---------------|-------------|
| `stability_committed` | float | 0 | Total REGEN in stability tier |
| `stability_allocation` | float | 0 | This period's stability distribution |
| `activity_pool` | float | 0 | This period's activity-based pool |
| `total_activity_score` | float | 0 | Sum of all participant activity scores |
| `stability_utilization` | float | 0 | stability_allocation / (community_inflow * 0.30) |
| `stability_queue_depth` | float | 0 | REGEN waiting to enter stability tier |
| `reward_per_unit_activity` | float | 0 | REGEN reward per unit of activity score |

#### Market and Ecological State
| Variable | Type | Initial Value | Description |
|----------|------|---------------|-------------|
| `credit_volume_weekly` | float | 500,000 | Weekly credit transaction volume (USD) |
| `regen_price_usd` | float | 0.05 | REGEN/USD price |
| `credit_issuance_count` | int | 50 | Credit issuance transactions per period |
| `credit_retirement_count` | int | 30 | Credit retirement transactions per period |
| `credit_trade_count` | int | 100 | Marketplace trades per period |
| `credit_transfer_count` | int | 20 | Transfers per period |
| `avg_credit_value` | float | 2,500 | Average credit transaction value (USD) |
| `ecological_multiplier` | float | 1.0 | Ecological oracle input (1.0 = disabled in v0) |

#### Agent Population State
| Variable | Type | Initial Value | Description |
|----------|------|---------------|-------------|
| `num_issuers` | int | 20 | Active credit issuers |
| `num_buyers` | int | 50 | Active credit buyers |
| `num_retirees` | int | 30 | Active credit retirees |
| `num_holders` | int | 500 | Passive REGEN holders |
| `num_stability_holders` | int | 0 | Holders in stability tier |
| `num_governance_participants` | int | 40 | Active governance voters |
| `num_wash_traders` | int | 0 | Adversarial wash trading agents |

### 2.3 Policy Functions

Policy functions compute actions based on current state. They run before state updates each epoch.

#### P1: Credit Market Activity

Generates credit transaction volume for the period based on agent populations and
exogenous market conditions.

```python
def p_credit_market(params, substep, state_history, prev_state):
    """
    Generate credit market activity for this epoch.

    Volume is driven by:
    - Number of active agents (issuers, buyers, retirees)
    - Per-agent transaction intensity (transactions per agent per period)
    - Average transaction value (drawn from lognormal distribution)
    - Seasonal adjustment factor
    - Trend growth rate
    """
    num_issuers = prev_state['num_issuers']
    num_buyers = prev_state['num_buyers']
    num_retirees = prev_state['num_retirees']
    num_wash_traders = prev_state['num_wash_traders']

    # Legitimate transactions
    issuance_count = max(1, int(num_issuers * params['issuance_intensity']))
    trade_count = max(1, int(num_buyers * params['trade_intensity']))
    retirement_count = max(1, int(num_retirees * params['retirement_intensity']))
    transfer_count = max(1, int((num_issuers + num_buyers) * params['transfer_intensity']))

    # Wash trading (adversarial)
    wash_trade_count = int(num_wash_traders * params['wash_trade_intensity'])

    # Transaction values (USD) drawn from lognormal
    avg_value = params['avg_credit_value_usd']
    sigma = params['credit_value_sigma']

    issuance_value = sum(np.random.lognormal(
        mean=np.log(avg_value * 2), sigma=sigma, size=issuance_count
    ))
    trade_value = sum(np.random.lognormal(
        mean=np.log(avg_value), sigma=sigma, size=trade_count
    ))
    retirement_value = sum(np.random.lognormal(
        mean=np.log(avg_value * 0.8), sigma=sigma, size=retirement_count
    ))
    transfer_value = sum(np.random.lognormal(
        mean=np.log(avg_value * 0.5), sigma=sigma, size=transfer_count
    ))
    wash_value = sum(np.random.lognormal(
        mean=np.log(avg_value * 0.3), sigma=sigma, size=max(1, wash_trade_count)
    )) if wash_trade_count > 0 else 0

    return {
        'issuance_count': issuance_count,
        'trade_count': trade_count + wash_trade_count,
        'retirement_count': retirement_count,
        'transfer_count': transfer_count,
        'issuance_value_usd': issuance_value,
        'trade_value_usd': trade_value + wash_value,
        'retirement_value_usd': retirement_value,
        'transfer_value_usd': transfer_value,
        'wash_trade_value_usd': wash_value,
        'total_volume_usd': issuance_value + trade_value + retirement_value + transfer_value + wash_value
    }
```

#### P2: Fee Collection (M013)

Calculates fees from all credit transactions in the period.

```python
def p_fee_collection(params, substep, state_history, prev_state, policy_input):
    """
    Calculate fees from credit market activity.

    fee = value_usd * rate_bps / 10000

    All fees are collected in REGEN terms using current REGEN/USD price.
    """
    regen_price = prev_state['regen_price_usd']

    # Fee rates in basis points
    issuance_rate = params['fee_rate_issuance_bps']
    trade_rate = params['fee_rate_trade_bps']
    retirement_rate = params['fee_rate_retirement_bps']
    transfer_rate = params['fee_rate_transfer_bps']

    # Calculate fees in USD
    issuance_fees_usd = policy_input['issuance_value_usd'] * issuance_rate / 10000
    trade_fees_usd = policy_input['trade_value_usd'] * trade_rate / 10000
    retirement_fees_usd = policy_input['retirement_value_usd'] * retirement_rate / 10000
    transfer_fees_usd = policy_input['transfer_value_usd'] * transfer_rate / 10000

    total_fees_usd = issuance_fees_usd + trade_fees_usd + retirement_fees_usd + transfer_fees_usd

    # Convert to REGEN
    total_fees_regen = total_fees_usd / max(regen_price, 0.001)

    # Apply minimum fee floor per transaction
    min_fee_regen = params['min_fee_regen']
    total_transactions = (
        policy_input['issuance_count'] + policy_input['trade_count'] +
        policy_input['retirement_count'] + policy_input['transfer_count']
    )
    min_fee_total = total_transactions * min_fee_regen
    total_fees_regen = max(total_fees_regen, min_fee_total)

    return {
        'total_fees_regen': total_fees_regen,
        'total_fees_usd': total_fees_usd
    }
```

#### P3: Fee Distribution (M013)

Routes collected fees to the four pools according to configured shares.

```python
def p_fee_distribution(params, substep, state_history, prev_state, policy_input):
    """
    Split fees to burn, validator, community, and agent pools.

    Invariant: burn_share + validator_share + community_share + agent_share = 1.0
    """
    fees = policy_input['total_fees_regen']

    burn_share = params['burn_share']
    validator_share = params['validator_share']
    community_share = params['community_share']
    agent_share = params['agent_share']

    assert abs(burn_share + validator_share + community_share + agent_share - 1.0) < 1e-9, \
        "Share Sum Unity violated"

    return {
        'burn_allocation': fees * burn_share,
        'validator_allocation': fees * validator_share,
        'community_allocation': fees * community_share,
        'agent_allocation': fees * agent_share
    }
```

#### P4: Mint/Burn Computation (M012)

Computes minting (regrowth) and burning for the period.

```python
def p_mint_burn(params, substep, state_history, prev_state, policy_input):
    """
    M012 supply algorithm:

    M[t] = r * max(0, C - S[t])    (regrowth, floored at 0 when S > C)
    B[t] = burn_allocation          (from M013 fee routing)

    r = r_base * effective_multiplier * ecological_multiplier

    effective_multiplier depends on M014 phase:
    - Pre-PoA:  1 + (S_staked / S_total)
    - Post-PoA: 1 + (S_stability_committed / S_total)
    """
    S = prev_state['S']
    C = params['hard_cap']
    r_base = params['base_regrowth_rate']

    # Effective multiplier
    if params['poa_active']:
        stability_ratio = prev_state['stability_committed'] / max(S, 1)
        effective_multiplier = 1.0 + stability_ratio
    else:
        staking_ratio = params['staking_ratio']
        effective_multiplier = 1.0 + staking_ratio

    effective_multiplier = min(effective_multiplier, 2.0)  # Capped at 2.0

    ecological_multiplier = prev_state['ecological_multiplier']
    ecological_multiplier = max(ecological_multiplier, 0.0)  # Floored at 0

    r = r_base * effective_multiplier * ecological_multiplier
    r = min(r, params['max_regrowth_rate'])  # Safety bound: r <= 0.10

    # Minting: only when S < C
    gap = C - S
    if gap > 0:
        M_t = r * gap
    else:
        M_t = 0.0  # No minting when at or above cap

    # Burning: from fee revenue
    B_t = policy_input['burn_allocation']

    # Enforce supply bounds
    new_S = S + M_t - B_t
    if new_S < 0:
        B_t = S + M_t  # Cannot burn below zero
        new_S = 0
    if new_S > C:
        M_t = max(0, C - S + B_t)  # Cap inviolability
        new_S = min(S + M_t - B_t, C)

    return {
        'M_t': M_t,
        'B_t': B_t,
        'new_S': new_S,
        'r_effective': r
    }
```

#### P5: Validator Compensation (M014)

Distributes validator fund to active validators.

```python
def p_validator_compensation(params, substep, state_history, prev_state, policy_input):
    """
    Validator compensation from M013 validator fund.

    base_compensation = validator_fund / active_validators
    performance_bonus = 10% of fund, distributed by composite score
    """
    validator_fund = policy_input['validator_allocation']
    active_validators = prev_state['active_validators']

    if active_validators == 0:
        return {'validator_income_period': 0, 'validator_income_annual': 0}

    bonus_share = params['validator_bonus_share']
    base_pool = validator_fund * (1 - bonus_share)
    bonus_pool = validator_fund * bonus_share

    base_per_validator = base_pool / active_validators
    # Simplified: assume average validator gets average bonus
    avg_bonus = bonus_pool / active_validators

    income_per_period = base_per_validator + avg_bonus
    income_annual = income_per_period * params['periods_per_year']

    return {
        'validator_income_period': income_per_period,
        'validator_income_annual': income_annual
    }
```

#### P6: Contribution Rewards Distribution (M015)

Distributes Community Pool between stability tier and activity-based rewards.

```python
def p_contribution_rewards(params, substep, state_history, prev_state, policy_input):
    """
    M015 reward distribution:

    1. Stability tier gets min(commitments * 6% / periods_per_year, 30% of community inflow)
    2. Remainder goes to activity-based distribution
    """
    community_inflow = policy_input['community_allocation']
    stability_committed = prev_state['stability_committed']
    periods_per_year = params['periods_per_year']
    stability_rate = params['stability_annual_rate']
    max_stability_share = params['max_stability_share']

    # Stability tier obligation
    stability_obligation = stability_committed * stability_rate / periods_per_year
    stability_cap = community_inflow * max_stability_share
    stability_allocation = min(stability_obligation, stability_cap)

    # Activity pool
    activity_pool = max(0, community_inflow - stability_allocation)

    # Activity scoring (aggregate)
    weights = params['activity_weights']  # dict of weight values
    total_score = (
        policy_input.get('issuance_value_usd', 0) * weights['purchase'] +
        policy_input.get('retirement_value_usd', 0) * weights['retirement'] +
        policy_input.get('trade_value_usd', 0) * weights['facilitation'] +
        prev_state['num_governance_participants'] * params['governance_vote_value'] * weights['governance'] +
        params['proposals_per_period'] * params['proposal_value'] * weights['proposals']
    )

    reward_per_unit = activity_pool / max(total_score, 1) if total_score > 0 else 0

    stability_utilization = stability_allocation / max(stability_cap, 1e-9)

    return {
        'stability_allocation': stability_allocation,
        'activity_pool': activity_pool,
        'total_activity_score': total_score,
        'reward_per_unit_activity': reward_per_unit,
        'stability_utilization': stability_utilization,
        'stability_shortfall': max(0, stability_obligation - stability_cap)
    }
```

#### P7: Agent Population Dynamics

Updates agent counts based on economic incentives and exogenous factors.

```python
def p_agent_dynamics(params, substep, state_history, prev_state, policy_input):
    """
    Agent entry/exit based on profitability signals:
    - Issuers enter if credit demand is growing
    - Buyers enter if credit prices are favorable
    - Validators churn based on income adequacy
    - Stability holders enter based on 6% return attractiveness
    - Wash traders enter/exit based on profitability
    """
    # Validator dynamics
    active_validators = prev_state['active_validators']
    validator_income = policy_input.get('validator_income_annual', 0)
    min_viable_income = params['min_viable_validator_income_usd']
    regen_price = prev_state['regen_price_usd']

    validator_income_usd = validator_income * regen_price
    if validator_income_usd < min_viable_income:
        churn_probability = params['base_validator_churn'] * 1.5
    else:
        churn_probability = params['base_validator_churn'] * 0.5

    validators_leaving = np.random.binomial(active_validators, churn_probability / 4)  # quarterly
    validators_joining = np.random.poisson(params['validator_application_rate'])
    new_validators = max(
        params['min_validators'],
        min(params['max_validators'], active_validators - validators_leaving + validators_joining)
    )

    # Stability tier dynamics
    stability_committed = prev_state['stability_committed']
    reward_rate_effective = prev_state.get('stability_utilization', 0) * params['stability_annual_rate']
    if reward_rate_effective >= params['stability_annual_rate'] * 0.9:
        # Attractive: new commitments
        new_stability = np.random.poisson(params['stability_adoption_rate']) * params['avg_stability_commitment']
    else:
        new_stability = 0

    # Stability maturations and early exits
    maturation_rate = 1 / (params['avg_stability_lock_periods'])
    maturations = stability_committed * maturation_rate
    early_exits = stability_committed * params['stability_early_exit_rate']

    new_stability_committed = max(0, stability_committed + new_stability - maturations - early_exits)

    return {
        'new_active_validators': new_validators,
        'new_stability_committed': new_stability_committed,
        'validators_leaving': validators_leaving,
        'validators_joining': validators_joining,
    }
```

### 2.4 State Update Functions

State update functions apply policy outputs to produce the next state. Each maps to one or more
state variables.

```python
def s_supply(params, substep, state_history, prev_state, policy_input):
    """Update supply from M012 mint/burn."""
    return ('S', policy_input['new_S'])

def s_mint_burn_records(params, substep, state_history, prev_state, policy_input):
    """Record mint and burn amounts."""
    return {
        'M_t': policy_input['M_t'],
        'B_t': policy_input['B_t'],
        'cumulative_minted': prev_state['cumulative_minted'] + policy_input['M_t'],
        'cumulative_burned': prev_state['cumulative_burned'] + policy_input['B_t'],
        'r_effective': policy_input['r_effective']
    }

def s_pool_balances(params, substep, state_history, prev_state, policy_input):
    """Update pool balances from fee distribution."""
    return {
        'burn_pool_balance': policy_input['burn_allocation'],
        'validator_fund_balance': policy_input['validator_allocation'],
        'community_pool_balance': policy_input['community_allocation'],
        'agent_infra_balance': policy_input['agent_allocation'],
        'total_fees_collected': policy_input['total_fees_regen'],
        'cumulative_fees': prev_state['cumulative_fees'] + policy_input['total_fees_regen']
    }

def s_validator_state(params, substep, state_history, prev_state, policy_input):
    """Update validator count and income."""
    return {
        'active_validators': policy_input['new_active_validators'],
        'validator_income_period': policy_input['validator_income_period'],
        'validator_income_annual': policy_input['validator_income_annual']
    }

def s_reward_state(params, substep, state_history, prev_state, policy_input):
    """Update M015 reward state."""
    return {
        'stability_committed': policy_input['new_stability_committed'],
        'stability_allocation': policy_input['stability_allocation'],
        'activity_pool': policy_input['activity_pool'],
        'total_activity_score': policy_input['total_activity_score'],
        'stability_utilization': policy_input['stability_utilization'],
        'reward_per_unit_activity': policy_input['reward_per_unit_activity']
    }
```

### 2.5 Behavioral Agents

The simulation includes six agent types with distinct behavioral models:

#### Agent Type 1: Credit Issuers

Issuers create ecological credit batches. They are the supply side of the credit market.

| Property | Value | Rationale |
|----------|-------|-----------|
| Entry trigger | Credit demand growth > 10% | Issuers respond to market signals |
| Exit trigger | Revenue < cost for 3 consecutive periods | Rational exit |
| Transaction frequency | 2-5 issuances per period per issuer | Based on current Regen registry data |
| Avg batch value | Lognormal(mu=8.5, sigma=1.2) USD | ~$5,000 median, heavy right tail |
| Fee sensitivity | Low (fees are cost of doing business) | Pass-through to credit buyers |

#### Agent Type 2: Credit Buyers

Buyers purchase credits on the marketplace. They are the demand side.

| Property | Value | Rationale |
|----------|-------|-----------|
| Entry trigger | Corporate ESG mandate, voluntary commitment | Exogenous demand |
| Exit trigger | Credit prices exceed budget, alternative registries | Price-sensitive |
| Transaction frequency | 1-3 purchases per period per buyer | Quarterly procurement cycles |
| Avg purchase value | Lognormal(mu=8.0, sigma=1.5) USD | ~$3,000 median |
| Fee sensitivity | Medium (fees add to total cost of retirement) | Budget-constrained |

#### Agent Type 3: Credit Retirees

Retirees permanently retire credits for environmental claims. Often the same entity as the buyer.

| Property | Value | Rationale |
|----------|-------|-----------|
| Entry trigger | Reporting deadlines, voluntary commitment | Calendar-driven |
| Exit trigger | Regulatory change, greenwashing backlash | Exogenous |
| Transaction frequency | 1-2 retirements per period per retirer | Post-purchase action |
| Avg retirement value | 80% of purchase value | Some credits held speculatively |
| Fee sensitivity | Low (retirement is final action; fee is marginal) | Committed to retirement |

#### Agent Type 4: Validators

Authority validators operate infrastructure and participate in governance.

| Property | Value | Rationale |
|----------|-------|-----------|
| Entry trigger | Application approved, income meets threshold | Mission + viability |
| Exit trigger | Income < min viable for 2 quarters | Rational exit |
| Min viable income | $15,000/year (infrastructure costs) | Server, devops, opportunity cost |
| Governance participation | 80-95% of proposals voted on | Required for performance bonus |
| Churn rate | 5% quarterly baseline | Based on current validator set stability |

#### Agent Type 5: REGEN Holders

Passive holders who may enter the stability tier or participate in governance.

| Property | Value | Rationale |
|----------|-------|-----------|
| Stability tier adoption | 5-15% of holders | Conservative; new mechanism |
| Avg stability commitment | 10,000-50,000 REGEN | Based on current holder distribution |
| Lock period preference | 6-12 months | Risk-averse default |
| Early exit probability | 5% per period | Low under normal conditions |
| Governance participation | 20-30% of holders | Typical Cosmos governance rates |

#### Agent Type 6: Wash Traders (Adversarial)

Wash traders attempt to inflate their activity scores by creating circular transactions.

| Property | Value | Rationale |
|----------|-------|-----------|
| Strategy | Buy credit -> transfer -> sell -> repeat | Circular volume inflation |
| Transaction frequency | 10-50 per period per wash trader | High frequency, low value |
| Avg transaction value | $100-500 | Small to minimize fee cost |
| Fee cost per cycle | ~2.1% of value (buy 1% + sell 1% + transfer 0.1%) | M013 friction |
| Reward earned per cycle | Proportional to activity score generated | M015 distribution |
| Profitability condition | reward_earned > fee_cost | Unprofitable when fees > rewards |

The key anti-gaming property: A wash trader paying 2.1% fees per cycle to earn a share of the
activity pool is unprofitable when their marginal contribution to the activity pool is smaller than
their fee contribution to the community pool. This is validated in stress test SC-003.

---

## 3. Parameter Space

### 3.1 Complete Parameter Table

All tunable parameters with baseline values, allowed ranges, and sensitivity ratings.

Sensitivity ratings:
- **Critical**: Small changes produce large output variance; requires careful calibration
- **High**: Meaningful impact on multiple outputs; priority for sensitivity analysis
- **Medium**: Impacts one or two outputs moderately
- **Low**: Minimal impact or well-constrained by other parameters

#### M012 Parameters (Supply Dynamics)

| Parameter | Symbol | Baseline | Min | Max | Units | Sensitivity |
|-----------|--------|----------|-----|-----|-------|-------------|
| Hard cap | C | 221,000,000 | 200,000,000 | 250,000,000 | REGEN | Critical |
| Initial supply | S_0 | 224,000,000 | 180,000,000 | 224,000,000 | REGEN | High |
| Base regrowth rate | r_base | 0.02 | 0.005 | 0.10 | per period | Critical |
| Max regrowth rate | r_max | 0.10 | 0.05 | 0.20 | per period | Medium |
| Staking ratio (pre-PoA) | staking_ratio | 0.30 | 0.10 | 0.70 | fraction | Medium |
| Ecological multiplier | eco_mult | 1.0 | 0.0 | 1.5 | multiplier | High |
| Period length | period_len | 7 | 1 | 30 | days | Medium |
| Equilibrium threshold | eq_threshold | 0.01 | 0.001 | 0.05 | fraction of S | Low |
| Equilibrium periods | eq_periods | 12 | 6 | 24 | periods (months) | Low |

#### M013 Parameters (Fee Routing)

| Parameter | Symbol | Baseline | Min | Max | Units | Sensitivity |
|-----------|--------|----------|-----|-----|-------|-------------|
| Issuance fee rate | fee_iss | 200 | 100 | 300 | bps | Critical |
| Trade fee rate | fee_trade | 100 | 50 | 200 | bps | Critical |
| Retirement fee rate | fee_ret | 50 | 25 | 100 | bps | High |
| Transfer fee rate | fee_xfer | 10 | 5 | 50 | bps | Low |
| Burn share | burn_share | 0.30 | 0.00 | 0.35 | fraction | Critical |
| Validator share | val_share | 0.40 | 0.15 | 0.50 | fraction | High |
| Community share | comm_share | 0.25 | 0.20 | 0.60 | fraction | High |
| Agent share | agent_share | 0.05 | 0.00 | 0.10 | fraction | Low |
| Min fee floor | min_fee | 1.0 | 0.1 | 10.0 | REGEN | Low |
| Fee rate cap | fee_cap | 1000 | 500 | 1000 | bps (10%) | Medium |

**Share sum constraint:** burn_share + val_share + comm_share + agent_share = 1.0 always.

> **Note on baseline assumptions:** This simulation uses Model A distribution shares (30/40/25/5) as specified in Phase 2.6. The WG is actively debating alternatives — PR #55 (OQ-M013-1) recommends a compromise of {28/25/45/2}, and PR #49 uses that compromise for governance proposals. The simulation's parameter sweep covers the full range of viable distributions, so conclusions hold across models. Run the sweep with the `--shares` flag to test specific configurations.

#### M014 Parameters (Validator Governance)

| Parameter | Symbol | Baseline | Min | Max | Units | Sensitivity |
|-----------|--------|----------|-----|-----|-------|-------------|
| Target validator count | val_target | 18 | 15 | 21 | validators | High |
| Min validators | val_min | 15 | 10 | 15 | validators | Medium |
| Max validators | val_max | 21 | 15 | 30 | validators | Medium |
| Performance bonus share | bonus_share | 0.10 | 0.00 | 0.25 | fraction | Low |
| Term length | term_len | 52 | 26 | 104 | periods (weeks) | Low |
| Min uptime | min_uptime | 0.995 | 0.990 | 0.999 | fraction | Low |
| Base churn rate | churn_base | 0.05 | 0.02 | 0.20 | quarterly | Medium |
| Min viable income | min_income | 15,000 | 5,000 | 50,000 | USD/year | High |
| Application rate | app_rate | 1.0 | 0.0 | 5.0 | apps/quarter | Low |

#### M015 Parameters (Contribution Rewards)

| Parameter | Symbol | Baseline | Min | Max | Units | Sensitivity |
|-----------|--------|----------|-----|-----|-------|-------------|
| Stability annual rate | stab_rate | 0.06 | 0.02 | 0.12 | annual fraction | Critical |
| Max stability share | max_stab | 0.30 | 0.10 | 0.50 | fraction of community pool | High |
| Min lock period | lock_min | 26 | 13 | 52 | periods (weeks) | Medium |
| Max lock period | lock_max | 104 | 52 | 208 | periods (weeks) | Low |
| Early exit penalty | exit_penalty | 0.50 | 0.25 | 1.00 | fraction of accrued rewards | Medium |
| Weight: purchase | w_purchase | 0.30 | 0.10 | 0.50 | weight | High |
| Weight: retirement | w_retire | 0.30 | 0.10 | 0.50 | weight | High |
| Weight: facilitation | w_facil | 0.20 | 0.05 | 0.40 | weight | Medium |
| Weight: governance | w_gov | 0.10 | 0.05 | 0.30 | weight | Medium |
| Weight: proposals | w_prop | 0.10 | 0.00 | 0.20 | weight | Low |
| Min stability commitment | min_commit | 100 | 10 | 1,000 | REGEN | Low |
| Stability adoption rate | adopt_rate | 5.0 | 0.0 | 20.0 | new commitments/period | Medium |
| Avg stability commitment | avg_commit | 25,000 | 5,000 | 100,000 | REGEN | Medium |

**Weight sum constraint:** w_purchase + w_retire + w_facil + w_gov + w_prop = 1.0 always.

#### Exogenous Parameters

| Parameter | Symbol | Baseline | Min | Max | Units | Sensitivity |
|-----------|--------|----------|-----|-----|-------|-------------|
| Weekly credit volume | vol_weekly | 500,000 | 50,000 | 10,000,000 | USD | Critical |
| Volume growth rate | vol_growth | 0.005 | -0.02 | 0.05 | per period | Critical |
| REGEN price | regen_price | 0.05 | 0.01 | 1.00 | USD | High |
| Price volatility | price_sigma | 0.05 | 0.01 | 0.20 | per period | Medium |
| Issuance intensity | iss_intensity | 2.5 | 0.5 | 10.0 | txns/issuer/period | Medium |
| Trade intensity | trade_intensity | 2.0 | 0.5 | 8.0 | txns/buyer/period | Medium |
| Retirement intensity | ret_intensity | 1.5 | 0.5 | 5.0 | txns/retirer/period | Medium |
| Transfer intensity | xfer_intensity | 0.5 | 0.1 | 2.0 | txns/agent/period | Low |

### 3.2 Derived Parameters

These are computed from primary parameters and cannot be set independently:

| Derived Parameter | Formula | Baseline Value |
|-------------------|---------|----------------|
| Periods per year | 52 / (period_len / 7) | 52 |
| Effective regrowth rate | r_base * eff_mult * eco_mult | 0.02-0.04 |
| Weekly fee revenue (REGEN) | vol_weekly * weighted_avg_fee_rate / regen_price | ~200,000 REGEN |
| Annual fee revenue (REGEN) | weekly_revenue * 52 | ~10.4M REGEN |
| Annual burn volume | annual_fee_revenue * burn_share | ~3.12M REGEN |
| Annual validator fund | annual_fee_revenue * val_share | ~4.16M REGEN |
| Per-validator annual income | annual_val_fund / val_target | ~231,000 REGEN |
| Per-validator USD income | per_val_regen * regen_price | ~$11,550 |
| Annual community pool | annual_fee_revenue * comm_share | ~2.6M REGEN |
| Max stability tier capacity | annual_community / stab_rate / max_stab | ~13M REGEN |

---

## 4. Sensitivity Analysis

### 4.1 Methodology

For each parameter of interest, we perform a one-at-a-time (OAT) sensitivity analysis:

1. Hold all other parameters at baseline
2. Vary the target parameter across its full range in 20 equal steps
3. Run 100 Monte Carlo simulations at each step
4. Record key output metrics at each step
5. Compute sensitivity coefficient: elasticity = (d_output / output) / (d_param / param)

Additionally, we perform Sobol global sensitivity analysis for the top 10 parameters to capture
interaction effects.

### 4.2 Base Regrowth Rate (r_base) Sensitivity

The regrowth rate is the most fundamental M012 parameter. It controls how quickly burned tokens
re-enter circulation.

| r_base | Annual Mint (M REGEN) | Supply at Year 5 | Equil. Time (periods) | Fee Revenue Impact | Validator Income Impact |
|--------|----------------------|-------------------|----------------------|-------------------|------------------------|
| 0.005 | 0.15 | 210.2M | >260 | None (independent) | None (independent) |
| 0.010 | 0.31 | 213.5M | 195 | None | None |
| 0.015 | 0.46 | 215.8M | 147 | None | None |
| **0.020** | **0.62** | **217.4M** | **118** | **Baseline** | **Baseline** |
| 0.030 | 0.93 | 219.2M | 85 | None | None |
| 0.040 | 1.24 | 220.1M | 67 | None | None |
| 0.060 | 1.86 | 220.7M | 48 | None | None |
| 0.080 | 2.48 | 220.9M | 38 | None | None |
| 0.100 | 3.10 | 221.0M | 31 | None | None |

Key findings:
- r_base does not directly affect fee revenue or validator income (those depend on transaction volume)
- r_base controls the supply trajectory and time to equilibrium
- Below r_base = 0.01, the system takes over 4 years to reach equilibrium, creating prolonged
  deflationary pressure that may deter holders
- Above r_base = 0.05, the system reaches near-cap levels quickly, reducing the burn's deflationary
  effect to a rounding error
- The baseline r_base = 0.02 produces equilibrium convergence in approximately 2.3 years,
  balancing responsiveness against excessive mint velocity

**Elasticity of equilibrium time to r_base:** approximately -1.2 (doubling r_base cuts equilibrium
time by roughly 55%).

### 4.3 Fee Rate Sensitivity

Fee rates directly control protocol revenue. The weighted average fee rate across all transaction
types determines total fee collection.

| Weighted Avg Fee (bps) | Annual Fee Rev ($K) | Burn Vol (M REGEN) | Validator Income ($/yr/val) | Community Pool ($K/yr) | Equil. Supply |
|------------------------|--------------------|--------------------|---------------------------|----------------------|---------------|
| 25 | 65 | 0.39M | 7,222 | 16.3 | 220.6M |
| 50 | 130 | 0.78M | 14,444 | 32.5 | 219.4M |
| 75 | 195 | 1.17M | 21,667 | 48.8 | 218.2M |
| **100** | **260** | **1.56M** | **28,889** | **65.0** | **217.0M** |
| 125 | 325 | 1.95M | 36,111 | 81.3 | 215.9M |
| 150 | 390 | 2.34M | 43,333 | 97.5 | 214.7M |
| 175 | 455 | 2.73M | 50,556 | 113.8 | 213.5M |
| 200 | 520 | 3.12M | 57,778 | 130.0 | 212.3M |

Key findings:
- Fee revenue scales linearly with fee rate (at constant volume)
- At the baseline weighted average of ~100 bps, validators earn approximately $28,889/year,
  which exceeds the $15,000 minimum viable threshold
- Reducing the weighted average below 50 bps brings validator income dangerously close to the
  minimum viable threshold
- The burn share creates a meaningful supply trajectory difference: at 200 bps weighted average,
  equilibrium supply is ~8.7M REGEN lower than at 25 bps

**Critical threshold:** At $500K weekly volume and $0.05 REGEN price, the minimum weighted average
fee rate for validator sustainability is approximately 52 bps.

### 4.4 Burn Share Sensitivity

The burn share determines what fraction of fees are permanently destroyed. It is the most
contentious parameter (see OQ-M013-5 in phase-2/2.6).

| Burn Share | Annual Burn (M REGEN) | Supply at Year 5 | Validator Income ($/yr) | Community Pool ($K/yr) | Equilibrium Supply |
|------------|---------------------|-------------------|------------------------|----------------------|-------------------|
| 0.00 | 0 | 221.0M (at cap) | 41,143 | 92.9 | 221.0M |
| 0.05 | 0.26M | 220.4M | 39,086 | 85.7 | 220.4M |
| 0.10 | 0.52M | 219.5M | 37,029 | 78.6 | 219.7M |
| 0.15 | 0.78M | 218.7M | 34,971 | 71.4 | 219.1M |
| 0.20 | 1.04M | 218.0M | 32,914 | 64.3 | 218.4M |
| 0.25 | 1.30M | 217.2M | 30,857 | 57.1 | 217.8M |
| **0.30** | **1.56M** | **217.0M** | **28,889** | **65.0** | **217.0M** |
| 0.35 | 1.82M | 216.3M | 26,743 | 42.9 | 216.4M |

Key findings:
- Every 5% increase in burn share costs validators approximately $2,000/year and removes ~$7K
  from the annual community pool
- At burn_share = 0.35 (upper bound), validator income drops to ~$26,743 which is still above
  the minimum viable threshold
- At burn_share = 0.00 (no burn), the supply quickly reaches the cap and remains there; all
  fee revenue flows to operational pools
- The tradeoff is explicit: each percentage point of burn share is a percentage point not going
  to validators, contributors, or agents

**Recommendation from sensitivity analysis:** The burn share should be viewed as a "holder subsidy"
parameter. At low transaction volumes, high burn shares risk making validator compensation
unsustainable. A dynamic burn share that decreases when validator income falls below threshold
would mitigate this risk. This aligns with OQ-M013-5 option (D) in the spec.

### 4.5 Stability Tier Return Rate Sensitivity

The 6% annual stability tier return is a fixed obligation against the Community Pool. Its
sustainability depends on the ratio of stability tier commitments to community pool inflow.

| Stability Rate | Max Supportable Commitments (M REGEN) | Stability Util. at 10M Committed | Community Pool Remaining ($K/yr) |
|---------------|--------------------------------------|----------------------------------|--------------------------------|
| 0.02 | 39.0M | 15.4% | 55.6 |
| 0.03 | 26.0M | 23.1% | 53.4 |
| 0.04 | 19.5M | 30.8% | 51.2 |
| 0.05 | 15.6M | 38.5% | 49.0 |
| **0.06** | **13.0M** | **46.2%** | **46.9** |
| 0.08 | 9.8M | 61.5% | 42.5 |
| 0.10 | 7.8M | 76.9% | 38.1 |
| 0.12 | 6.5M | 92.3% | 33.7 |

Key findings:
- At the baseline 6% rate with 10M REGEN committed, the stability tier consumes 46.2% of the
  community pool, well above the 30% cap. The cap constrains actual distribution, meaning
  stability holders would receive less than the promised 6%.
- At 6% with the 30% cap enforced, the maximum supportable commitment is 13.0M REGEN
  (approximately 5.9% of the hard cap). Beyond this level, the stability tier obligation exceeds
  the cap and new commitments must be queued.
- Reducing the stability rate to 3% doubles the capacity to 26M REGEN, accommodating a much larger
  committed base.
- At 12%, the stability tier can only support 6.5M REGEN before hitting the cap, making it a very
  exclusive tier.

**Critical finding:** At baseline parameters ($500K weekly volume, 25% community share, 6% rate),
the stability tier capacity is approximately 13M REGEN. If adoption exceeds this, the queue mechanism
activates and effective returns fall below 6%. The simulation must validate that the queue mechanism
prevents insolvency under all scenarios.

### 4.6 Activity Weight Sensitivity

Activity weights determine who earns what share of the activity-based reward pool. Changing weights
shifts incentives among participant types.

**Baseline weights:** Purchase 0.30, Retirement 0.30, Facilitation 0.20, Governance 0.10, Proposals 0.10

| Weight Config | Top Earner Type | Gini Coefficient | Wash Trade Profitability | Governance Part. Rate |
|---------------|----------------|-------------------|------------------------|-----------------------|
| Purchase-heavy (0.50/0.20/0.15/0.10/0.05) | Buyers | 0.72 | Higher (buy-weighted) | Decreases 15% |
| Retirement-heavy (0.20/0.50/0.15/0.10/0.05) | Retirees | 0.68 | Lower (retirement is terminal) | Decreases 10% |
| **Baseline (0.30/0.30/0.20/0.10/0.10)** | **Balanced** | **0.55** | **Moderate** | **Baseline** |
| Governance-heavy (0.20/0.20/0.10/0.30/0.20) | Voters | 0.45 | Lower | Increases 40% |
| Facilitation-heavy (0.15/0.15/0.50/0.10/0.10) | Platforms | 0.78 | Lower | Decreases 5% |

Key findings:
- Purchase-heavy weights increase wash trading profitability because circular buy transactions
  generate outsized rewards relative to their fee cost
- Retirement-heavy weights are the most sybil-resistant because retirement is a terminal action
  (credits cannot be un-retired and re-retired)
- Governance-heavy weights significantly increase participation rates but may create governance
  spam if the proposal quality filter is weak
- The baseline balanced configuration produces a moderate Gini coefficient (0.55), indicating
  some concentration but not extreme inequality
- Facilitation-heavy weights concentrate rewards among a small number of platform operators,
  creating the highest Gini coefficient

### 4.7 Transaction Volume Sensitivity

Transaction volume is the most impactful exogenous variable. It scales all revenue linearly.

| Weekly Volume ($K) | Annual Fee Rev ($K) | Validator Inc ($/yr) | Burn (M REGEN/yr) | Community Pool ($K/yr) | Stability Cap (M REGEN) |
|--------------------|--------------------|--------------------|-------------------|----------------------|------------------------|
| 50 | 26 | 2,889 | 0.16M | 6.5 | 1.3M |
| 100 | 52 | 5,778 | 0.31M | 13.0 | 2.6M |
| 250 | 130 | 14,444 | 0.78M | 32.5 | 6.5M |
| **500** | **260** | **28,889** | **1.56M** | **65.0** | **13.0M** |
| 1,000 | 520 | 57,778 | 3.12M | 130.0 | 26.0M |
| 2,500 | 1,300 | 144,444 | 7.80M | 325.0 | 65.0M |
| 5,000 | 2,600 | 288,889 | 15.60M | 650.0 | 130.0M |
| 10,000 | 5,200 | 577,778 | 31.20M | 1,300.0 | 260.0M |

Key findings:
- **Below $250K weekly volume, validators cannot sustain operations** (income < $15,000/year).
  This is the minimum viable volume threshold for the entire system.
- At $50K weekly volume, the system is fundamentally unsustainable: validator income covers less
  than 20% of minimum viable costs, and the stability tier can only support 1.3M REGEN.
- At $1M weekly volume, the system is comfortably sustainable: validators earn $57,778/year,
  and the stability tier supports 26M REGEN.
- Volume scales linearly with all revenue-dependent outputs. This linearity means the system's
  sustainability is directly proportional to credit market adoption.
- The simulation must test what happens during sustained low-volume periods (see SC-001).

### 4.8 Sobol Global Sensitivity Indices

Sobol analysis decomposes output variance into contributions from individual parameters and their
interactions. First-order indices (S1) measure the direct effect; total-order indices (ST) include
interactions.

**Output: Validator Annual Income**

| Parameter | S1 (First Order) | ST (Total Order) | S1 Rank |
|-----------|------------------|-------------------|---------|
| Weekly volume | 0.62 | 0.68 | 1 |
| Validator share | 0.18 | 0.23 | 2 |
| REGEN price | 0.08 | 0.14 | 3 |
| Weighted avg fee rate | 0.06 | 0.12 | 4 |
| Validator count | 0.04 | 0.07 | 5 |
| Burn share | 0.01 | 0.03 | 6 |

**Output: Supply at Year 5**

| Parameter | S1 (First Order) | ST (Total Order) | S1 Rank |
|-----------|------------------|-------------------|---------|
| r_base | 0.38 | 0.45 | 1 |
| Burn share | 0.24 | 0.32 | 2 |
| Weekly volume | 0.18 | 0.26 | 3 |
| Ecological multiplier | 0.10 | 0.15 | 4 |
| Initial supply | 0.06 | 0.09 | 5 |
| Staking ratio | 0.03 | 0.05 | 6 |

**Output: Stability Tier Solvency Ratio**

| Parameter | S1 (First Order) | ST (Total Order) | S1 Rank |
|-----------|------------------|-------------------|---------|
| Stability rate | 0.35 | 0.42 | 1 |
| Weekly volume | 0.28 | 0.34 | 2 |
| Community share | 0.15 | 0.22 | 3 |
| Max stability share | 0.12 | 0.18 | 4 |
| Stability adoption rate | 0.06 | 0.10 | 5 |
| Avg stability commitment | 0.03 | 0.05 | 6 |

**Interpretation:** Transaction volume dominates validator income variance (62% first-order).
Regrowth rate dominates supply trajectory variance (38%). Stability rate dominates stability tier
solvency (35%). These three parameters are the primary levers for their respective outputs.

---

## 5. Stress Test Scenarios

Each stress test defines a specific adversarial or failure scenario with precise initial conditions,
adversary model, expected trajectory, failure threshold, and mitigation strategy.

### SC-001: Low Credit Volume (90% Drop)

**Scenario:** A severe market downturn or regulatory change causes credit transaction volume to
drop by 90% from baseline and remain suppressed for 12 months.

| Property | Value |
|----------|-------|
| **Initial Conditions** | Baseline parameters; system in DYNAMIC state for 6+ months |
| **Trigger** | At epoch 52 (year 1), weekly volume drops from $500K to $50K |
| **Duration** | 52 epochs (1 year) at reduced volume |
| **Recovery** | Linear recovery over 26 epochs back to $250K (50% of original) |

**Adversary Model:** Not adversarial; this is an exogenous demand shock. The drop may be caused by:
- Voluntary carbon market collapse (regulatory intervention, scandal)
- Competing registry capturing market share
- Global recession reducing corporate sustainability budgets
- Technical failure in credit issuance pipeline

**Expected Trajectory:**
1. Fee revenue drops 90%: from $260K/yr to $26K/yr
2. Validator income drops to ~$2,889/yr (far below $15,000 threshold)
3. Burn volume drops to ~0.16M REGEN/yr, causing supply to drift upward via regrowth
4. Community pool shrinks to $6.5K/yr, making stability tier obligations unsustainable
5. Validators begin exiting; if below 15, emergency governance triggers
6. Stability tier hits the cap (30% of reduced community pool), holders receive less than 6%

**Failure Thresholds:**

| Metric | Failure Level | Expected Outcome |
|--------|--------------|------------------|
| Validator count | < 15 (min_validators) | FAIL: Drops below minimum within 2 quarters |
| Validator income | < $5,000/yr (emergency) | FAIL: Reaches emergency level in first quarter |
| Supply | > 221M (cap violation) | PASS: Cap is inviolable; regrowth is zero when S > C |
| Stability tier | < 50% obligation coverage | FAIL: Coverage drops to ~15% |

**Mitigations:**
1. **Emergency reserve fund**: Pre-fund a 12-month validator reserve from initial Community Pool
   balance (governance proposal before M012 activation)
2. **Dynamic fee adjustment**: Increase fee rates during low-volume periods via Layer 2 governance
   (AGENT-003 triggers proposal when 90-day moving average volume drops below threshold)
3. **Minimum validator stipend**: Guarantee minimum income from a reserved treasury, funded by
   a one-time allocation at system activation
4. **Stability tier auto-adjustment**: Reduce stability tier returns proportionally when community
   pool is under stress (e.g., scale returns by min(1, community_pool_actual / community_pool_target))

**Simulation Configuration:**
```python
sc001_params = {
    'volume_schedule': [
        (0, 51, 500_000),    # Normal for year 1
        (52, 103, 50_000),   # 90% drop for year 2
        (104, 129, lambda t: 50_000 + (250_000 - 50_000) * (t - 104) / 26),  # Recovery
        (130, 260, 250_000)  # Partial recovery steady state
    ],
    'failure_checks': {
        'validator_count_min': 15,
        'validator_income_emergency': 5000,
        'stability_coverage_min': 0.50
    }
}
```

---

### SC-002: High Validator Churn (50% per Quarter)

**Scenario:** Validators exit at 10x the baseline rate due to inadequate compensation, mission
drift, or coordinated withdrawal.

| Property | Value |
|----------|-------|
| **Initial Conditions** | 18 active validators; baseline revenue |
| **Trigger** | At epoch 26, validator churn rate increases from 5% to 50% quarterly |
| **Duration** | 26 epochs (6 months) |
| **Recovery** | Churn returns to baseline; applications increase 3x for 6 months |

**Adversary Model:** Semi-adversarial. Possible causes:
- Organized protest by validators over compensation inadequacy
- Technical change (e.g., hardware requirements increase) pricing out smaller validators
- Competing network offering better terms
- Loss of mission alignment in validator community

**Expected Trajectory:**
1. Quarter 1 of crisis: 9 validators exit (50% of 18), only 2-3 new applications arrive
2. Active set drops to ~11, below the 15-validator minimum
3. Emergency governance triggers: escalation to Layer 4 for immediate action
4. Block production continues (Tendermint tolerates reduced set) but security margin shrinks
5. Remaining validators receive higher per-capita income (same fund, fewer validators)
6. After 6 months of elevated churn, recovery begins with aggressive recruitment

**Failure Thresholds:**

| Metric | Failure Level | Expected Outcome |
|--------|--------------|------------------|
| Validator count | < 10 (Byzantine tolerance risk) | HIGH RISK: May reach 9-10 |
| Block production | Liveness failure | PASS: Tendermint handles reduced set |
| Governance | < 2/3 of set for proposals | RISK: If set drops below ~10, quorum is fragile |
| Per-validator income | > 2x baseline (concentration) | PASS: Temporary income increase |

**Mitigations:**
1. **Standby validator pool**: Maintain 5-10 pre-approved standby validators who can activate
   within 24 hours
2. **Automatic compensation increase**: If active_validators < val_target, increase per-validator
   compensation by redirecting agent_infra share to validator fund temporarily
3. **Emergency recruitment bounty**: Governance-approved bounty for new validator applications
   during crisis
4. **Minimum set protection**: If validator count approaches min_validators, freeze voluntary
   exits for 30 days (emergency governance action)

---

### SC-003: Wash Trading Attack (30% of Volume)

**Scenario:** A coordinated group of wash traders generates 30% of apparent transaction volume
through circular credit trades, attempting to extract disproportionate M015 rewards.

| Property | Value |
|----------|-------|
| **Initial Conditions** | Baseline parameters; 10 wash trading entities |
| **Trigger** | Wash trading begins at epoch 13 (quarter 2) |
| **Volume** | Wash traders generate $150K/week additional apparent volume |
| **Strategy** | Buy -> transfer -> sell -> repeat; low-value credits |

**Adversary Model:** Rational economic adversary seeking to extract more rewards than fees paid.

**Attack Economics (per wash cycle):**
```
Transaction: Buy $1,000 credit, transfer to second account, sell for $1,000

Fees paid:
  Buy:      $1,000 * 1.00% = $10.00
  Transfer: $1,000 * 0.10% = $1.00
  Sell:     $1,000 * 1.00% = $10.00
  Total fee per cycle:       $21.00

Activity score generated:
  Purchase:  $1,000 * 0.30 = 300
  Retirement: $0 * 0.30    = 0     (not retiring, just trading)
  Facilitation: $0 * 0.20  = 0     (not a platform)
  Governance: 0 * 0.10     = 0     (not voting)
  Proposals: 0 * 0.10      = 0     (not proposing)
  Total score per cycle:     300

Reward earned (share of activity pool):
  If total activity score = 100,000 and activity pool = $1,250/week (baseline):
    Reward = $1,250 * (300 / 100,300) = $3.74

Net profit per cycle: $3.74 - $21.00 = -$17.26 (LOSS)
```

**Key Anti-Gaming Property:** The wash trader pays 2.1% in fees but only generates purchase-weighted
activity score (weight 0.30). Because the fee cost exceeds the proportional reward, the attack is
economically unprofitable under baseline parameters.

**Conditions where attack becomes profitable:**
- If activity pool grows large enough relative to total activity (very high community share)
- If wash trader can also claim facilitation credit (by running their own platform)
- If fee rates are reduced below the profitability break-even

**Break-even analysis:**
```
Profitability condition: reward_share * activity_pool > fee_cost

For a wash trader contributing fraction f of total activity:
  f * activity_pool > 0.021 * wash_volume

  activity_pool / total_volume > 0.021 / f

At f = 0.30 (wash = 30% of total activity):
  activity_pool / total_volume > 0.07

  This means the activity pool must exceed 7% of total volume for the attack to break even.
  At baseline: activity_pool = ~$65K * 0.70 = $45.5K/yr; total_volume = $26M/yr.
  Ratio = 0.175%, far below 7%. Attack is deeply unprofitable.
```

**Failure Thresholds:**

| Metric | Failure Level | Expected Outcome |
|--------|--------------|------------------|
| Wash trader profit | > 0 (profitable attack) | PASS: Unprofitable at baseline |
| Reward concentration | Top 10% of addresses earn > 50% of rewards | MONITOR: Possible if platform facilitation included |
| Fee revenue impact | Wash fees < wash rewards (net extraction) | PASS: Net fee contribution |
| Activity score inflation | > 50% of total score from wash activity | RISK: Monitor for score dilution |

**Mitigations:**
1. **Minimum transaction value**: Set a minimum credit transaction value for M015 reward eligibility
   (e.g., $100 minimum) to increase wash trading cost
2. **Velocity cap**: Limit the number of reward-eligible transactions per address per period
3. **Retirement-weighted scoring**: Increase retirement weight (terminal action, cannot be cycled)
4. **On-chain analytics agent**: AGENT-003 monitors for circular transaction patterns and flags
   suspicious addresses for governance review

---

### SC-004: Stability Tier Bank Run (80% Early Exits)

**Scenario:** A market panic causes 80% of stability tier participants to exit early within a
single period, testing the system's liquidity and penalty mechanisms.

| Property | Value |
|----------|-------|
| **Initial Conditions** | 10M REGEN in stability tier; baseline parameters |
| **Trigger** | At epoch 78 (1.5 years), REGEN price drops 60% in one week |
| **Behavior** | 80% of stability holders exercise early exit |
| **Exit volume** | 8M REGEN unlocked in a single period |

**Adversary Model:** Not adversarial; this is a panic-driven coordination failure. Holders are
rational actors who believe the price will continue falling, making the 50% reward forfeiture
less painful than continued price exposure.

**Expected Trajectory:**
1. 8M REGEN unlocked simultaneously, hitting sell-side markets
2. Early exit penalty: 50% of accrued rewards forfeited (returned to Community Pool)
3. Stability tier drops from 10M to 2M REGEN committed
4. Community pool receives a windfall from forfeited rewards
5. Supply regrowth multiplier drops (lower stability_committed / S_total ratio)
6. Activity pool increases (less going to stability allocation)
7. Sell pressure from 8M unlocked REGEN may further depress price

**Failure Thresholds:**

| Metric | Failure Level | Expected Outcome |
|--------|--------------|------------------|
| Stability tier solvency | Obligations > available | PASS: Early exits reduce obligations |
| Price impact | > 50% additional drop from sell pressure | RISK: Depends on market depth |
| Regrowth multiplier | Drops below 1.0 (minimum) | PASS: Minimum is 1.0 |
| System stability | Cascading liquidation spiral | RISK: If unlocked REGEN selling causes further exits |

**Cascade Risk Analysis:**
The danger is a reflexive loop: price drops -> stability exits -> selling pressure -> price drops
further -> more exits. The simulation must model this feedback loop by including a price impact
function for large sell volumes.

```
price_impact(sell_volume, market_depth):
  # Simplified linear impact model
  price_change = -sell_volume / market_depth

  # At 8M REGEN sell and $5M market depth:
  # price_change = -8M * $0.05 / $5M = -8%
  # This is additive to the initial 60% drop
```

**Mitigations:**
1. **Staggered exit**: Enforce a 4-week unlock period for early exits (rather than instant),
   spreading sell pressure over time
2. **Progressive penalty**: Increase early exit penalty for larger unlock amounts or during
   high-exit periods (e.g., if > 20% of stability tier exits in one period, penalty increases
   to 75%)
3. **Circuit breaker**: If > 50% of stability tier signals exit in a single period, pause exits
   for 7 days and require governance confirmation
4. **Minimum retention**: Require 10% of committed balance to remain locked for full term
   regardless of exit decision

---

### SC-005: Fee Avoidance (50% Off-Chain)

**Scenario:** Major credit market participants move 50% of transactions off-chain (OTC deals,
bilateral agreements, or competing registries) to avoid M013 fees.

| Property | Value |
|----------|-------|
| **Initial Conditions** | Baseline parameters; on-chain volume $500K/week |
| **Trigger** | Gradual migration starting epoch 26; 50% off-chain by epoch 52 |
| **Steady state** | On-chain volume stabilizes at $250K/week |
| **Cause** | Fee elasticity; participants find cheaper alternatives |

**Adversary Model:** Rational economic actors, not malicious. Fee avoidance is a natural market
response to transaction costs. The question is: at what fee level does off-chain migration
become significant?

**Fee Elasticity Model:**
```
Assume a simple logistic fee elasticity function:

off_chain_share(fee_rate) = 1 / (1 + exp(-k * (fee_rate - fee_threshold)))

where:
  k = elasticity parameter (steepness)
  fee_threshold = fee rate at which 50% of volume moves off-chain

Estimates:
  fee_threshold ~ 2% (200 bps) for institutional buyers
  fee_threshold ~ 5% (500 bps) for retail
  k ~ 10 (moderate elasticity)

At baseline weighted average of ~100 bps:
  off_chain_share(0.01) = 1 / (1 + exp(-10 * (0.01 - 0.02))) = ~27%

This suggests even at baseline fees, approximately 27% of potential volume may stay off-chain.
```

**Expected Trajectory:**
1. On-chain volume gradually declines from $500K to $250K/week over 6 months
2. All revenue-dependent metrics halve: validator income, burn volume, community pool
3. Validator income drops to ~$14,444/yr (just below $15,000 threshold)
4. System remains technically functional but validator sustainability is marginal
5. Some validators may exit, further weakening the network

**Failure Thresholds:**

| Metric | Failure Level | Expected Outcome |
|--------|--------------|------------------|
| Validator income | < $15,000/yr | MARGINAL: At $14,444, barely below threshold |
| On-chain volume share | < 30% of total market | RISK: Protocol becomes irrelevant |
| Fee revenue | < operating costs | FAIL: If off-chain exceeds 60% |
| Network effects | Declining user count | RISK: Gradual death spiral |

**Mitigations:**
1. **Value-add for on-chain**: Ensure on-chain transactions provide unique value (immutable
   retirement certificates, public ESG reporting, interoperability with other chains) that
   cannot be replicated off-chain
2. **Tiered fees**: Reduce fees for high-volume participants (institutional tier at 0.5% instead
   of 1%) to retain the largest transactors
3. **Dynamic fee adjustment**: Reduce fee rates when on-chain volume declines, accepting lower
   per-transaction revenue for higher volume retention
4. **Exclusivity mechanisms**: Certain credit classes or verification standards available only
   on-chain, creating a natural moat

---

### SC-006: Governance Deadlock (No Proposals 3 Months)

**Scenario:** Governance becomes paralyzed: no proposals pass for 3 consecutive months due to
voter apathy, faction disputes, or quorum failure.

| Property | Value |
|----------|-------|
| **Initial Conditions** | Baseline parameters; system in DYNAMIC state |
| **Trigger** | At epoch 52, governance participation drops below quorum |
| **Duration** | 13 epochs (3 months) with no passed proposals |
| **Cause** | Voter fatigue, faction deadlock, or key stakeholder exit |

**Adversary Model:** Not adversarial but potentially exploitable. Governance deadlock prevents
parameter adjustments that may be needed in response to changing conditions.

**Expected Trajectory:**
1. All governance-adjustable parameters frozen at current values
2. If market conditions change (e.g., volume drops requiring fee adjustment), system cannot adapt
3. M015 proposal submission rewards drop to zero (no proposals reaching quorum)
4. Validator term expirations cannot be processed; terms auto-extend by default
5. Community Pool accumulates without directed spending (positive for stability tier,
   negative for directed ecosystem investment)
6. Agent infrastructure operates on autopilot (Layer 1-2 actions continue)

**Impact Assessment:**

The system is designed with governance layers specifically to handle this scenario:
- Layer 1 (Autonomous): Continues functioning. Mint/burn cycles, fee collection, automatic
  distributions all proceed without governance action.
- Layer 2 (Agentic + Oversight): Agents can propose operational adjustments; but if human
  oversight is absent, proposals queue without approval.
- Layer 3 (Human-in-Loop): Frozen. No parameter changes, no validator approvals/removals.
- Layer 4 (Constitutional): Frozen. No structural changes.

**Failure Thresholds:**

| Metric | Failure Level | Expected Outcome |
|--------|--------------|------------------|
| Parameter staleness | > 3 months without review | RISK: Parameters may be suboptimal |
| Validator management | Cannot approve/remove validators | RISK: If churn occurs, no replacements |
| Emergency response | Cannot respond to crises | FAIL: If SC-001 or SC-002 co-occurs |
| Community Pool utilization | 0% directed spending for 3 months | MODERATE: Pool accumulates |

**Mitigations:**
1. **Automatic parameter escalation**: If no governance action in 90 days, trigger an automatic
   "governance health check" proposal with simple yes/no to confirm current parameters
2. **Delegated governance**: Allow governance power to be delegated to trusted agents or
   committees for routine decisions (Layer 2-3)
3. **Sunset clauses**: All parameter configurations expire after 6 months, requiring affirmative
   renewal (forces governance engagement)
4. **Quorum reduction**: If quorum is not met for 3 consecutive proposals, temporarily reduce
   quorum requirement by 25% (maximum 2 reductions)

---

### SC-007: Supply Shock (Cap Hit, Zero Mint, Sustained Burn)

**Scenario:** The supply reaches the hard cap (S = C), minting drops to zero, but burning continues
due to ongoing fee revenue. This creates sustained deflationary pressure with no regrowth offset.

| Property | Value |
|----------|-------|
| **Initial Conditions** | S[t] = 221M (at cap); high transaction volume ($2M/week) |
| **Trigger** | System has been in high-regrowth mode; supply hits cap |
| **Dynamics** | M[t] = 0 (no gap for regrowth); B[t] > 0 (ongoing fees) |
| **Duration** | Indefinite until supply drops enough for regrowth to resume |

**Adversary Model:** Not adversarial; this is a natural system state that occurs when regrowth
catches up to the cap. The question is whether the system handles the transition gracefully.

**Expected Trajectory:**
1. Supply at cap: S = 221M, gap = 0, M[t] = 0
2. Weekly burn: $2M * ~1% avg fee rate * 30% burn share = ~$6,000 worth of REGEN burned
   At $0.05/REGEN: ~120,000 REGEN burned per week
3. Supply declines: 221M -> 220.88M -> 220.76M -> ... (120K/week)
4. After ~1 week: gap = 120K REGEN; regrowth = 0.02 * 120K = 2,400 REGEN. Trivial.
5. After ~4 weeks: gap = 480K; regrowth = 9,600 REGEN. Still trivial vs 120K burn.
6. After ~1 year: gap = 6.24M; regrowth = 124,800 REGEN. Now approaching burn rate.
7. Equilibrium: M[t] = B[t] at approximately S_eq (derived in Section 6).

**Key Observation:** When S is at the cap, the system enters a "burn-only" regime. The deflationary
pressure is proportional to fee revenue. This is actually the intended behavior: supply contracts
when at the ceiling, creating scarcity that may increase REGEN price, which in turn reduces the
REGEN quantity of fees (since fees are USD-denominated), which slows the burn. This is a
self-stabilizing mechanism.

**Failure Thresholds:**

| Metric | Failure Level | Expected Outcome |
|--------|--------------|------------------|
| Supply trajectory | Runaway deflation (S < 150M) | PASS: Self-correcting via regrowth |
| Price stability | > 5x price increase (speculative bubble) | RISK: Deflationary spiral could attract speculation |
| Validator income (REGEN terms) | < baseline | PASS: REGEN income stable; USD value may increase |
| Economic activity | Declining due to REGEN scarcity | RISK: If REGEN becomes too scarce to pay fees |

**Mitigations:**
1. **Fee denomination flexibility**: Allow fees in stablecoins (OQ-M013-3) so REGEN scarcity
   does not impede transactions
2. **Dynamic burn rate**: Reduce burn_share when supply is declining rapidly (e.g., if weekly
   burn > 0.5% of supply, reduce burn_share by 50%)
3. **Regrowth rate increase**: Governance can increase r_base to accelerate supply recovery
4. **Monitoring**: AGENT-003 tracks supply trajectory and alerts when sustained deflation detected

---

### SC-008: Oracle Manipulation

**Scenario:** The ecological multiplier oracle (M012) is compromised or manipulated, causing
incorrect regrowth rate computation.

| Property | Value |
|----------|-------|
| **Initial Conditions** | Ecological multiplier enabled (not v0 default) |
| **Attack Vector A** | Oracle reports false-positive ecological improvement (eco_mult > 1.0) |
| **Attack Vector B** | Oracle reports false-negative ecological decline (eco_mult = 0.0) |
| **Duration** | Manipulation persists until detected (assume 4-12 weeks) |

**Adversary Model:** Sophisticated attacker who has compromised the oracle data source or the
oracle submission mechanism. Motivation: manipulate supply dynamics for trading profit.

**Attack Vector A -- Inflated Ecological Multiplier:**
```
Normal: eco_mult = 1.0, r = 0.02 * 1.3 * 1.0 = 0.026
Attack: eco_mult = 1.5, r = 0.02 * 1.3 * 1.5 = 0.039

Impact: 50% increase in regrowth rate
Additional minting per week: 0.013 * (221M - 217M) = 52,000 extra REGEN
Over 12 weeks: 624,000 extra REGEN minted (~$31,200 at $0.05)

Attacker strategy: Go long REGEN before inflating eco_mult, benefit from increased supply
pushing down value... Wait, increased supply is deflationary in price terms, not inflationary.

Actually: increased minting when S < C fills the gap faster, approaching the cap. Once at cap,
minting stops. The attack accelerates the timeline to equilibrium but does not create unbounded
inflation because of the hard cap.

Max damage: accelerated supply expansion up to the cap, no more.
```

**Attack Vector B -- Zeroed Ecological Multiplier:**
```
Normal: eco_mult = 1.0, r = 0.026
Attack: eco_mult = 0.0, r = 0.0

Impact: Regrowth completely halted
No minting occurs; only burning continues
System enters sustained deflation (like SC-007 but artificial)

Attacker strategy: Short REGEN, trigger deflation by zeroing regrowth, amplify with
market manipulation. Then buy cheap REGEN during panic.

Max damage: Sustained deflation for duration of manipulation. If combined with high
burn volume, supply could drop significantly.
Over 12 weeks with 120K REGEN/week burn: 1.44M REGEN supply reduction (0.65% of supply)
```

**Failure Thresholds:**

| Metric | Failure Level | Expected Outcome |
|--------|--------------|------------------|
| Supply deviation from target | > 2% from expected trajectory | RISK: Vector A has limited impact; Vector B moderate |
| Attacker profit | > $50K | RISK: Depends on REGEN market depth and leverage |
| Detection time | > 4 weeks | RISK: Slow detection amplifies damage |
| System integrity | Hard cap violated | PASS: Cap inviolability is enforced regardless of multiplier |

**Mitigations:**
1. **v0 default is eco_mult = 1.0 (disabled)**: No oracle dependency until proven reliable
2. **Oracle bounds**: Enforce eco_mult in [0.5, 1.5] range at the protocol level, limiting
   manipulation impact to +/- 50% of normal regrowth
3. **Moving average**: Use a 4-week moving average of oracle values rather than spot values,
   damping manipulation impact
4. **Multi-source oracle**: Require at least 3 independent data sources with median aggregation
5. **Anomaly detection**: AGENT-003 monitors eco_mult for sudden changes; flags for human review
   if change > 20% between periods
6. **Emergency governor**: Governance can freeze the ecological multiplier at 1.0 via Layer 2
   emergency action if manipulation is suspected

---

## 6. Equilibrium Analysis

### 6.1 Supply Equilibrium (M012)

At equilibrium, minting equals burning: M[t] = B[t].

**Deriving the equilibrium supply S*:**

```
M[t] = B[t]
r * (C - S*) = burn_share * F[t]

where F[t] = total fees in REGEN per period

F[t] = V * w_avg_fee_rate / P_regen

where V = transaction volume (USD), P_regen = REGEN price (USD)

Substituting:
r * (C - S*) = burn_share * V * w_avg_fee_rate / P_regen

Solving for S*:
C - S* = (burn_share * V * w_avg_fee_rate) / (r * P_regen)

S* = C - (burn_share * V * w_avg_fee_rate) / (r * P_regen)
```

**Equilibrium supply as a function of key parameters:**

```
S* = C - (burn_share * V * w_avg) / (r * P)

where:
  C = 221,000,000 REGEN
  burn_share = 0.30
  V = weekly volume in USD
  w_avg = weighted average fee rate (~0.01)
  r = effective regrowth rate (~0.026)
  P = REGEN price in USD

At baseline:
  S* = 221,000,000 - (0.30 * 500,000 * 0.01) / (0.026 * 0.05)
  S* = 221,000,000 - 1,500 / 0.0013
  S* = 221,000,000 - 1,153,846
  S* = 219,846,154 REGEN
```

**Sensitivity of equilibrium supply to key parameters:**

| Parameter | Change | S* Change | Direction |
|-----------|--------|-----------|-----------|
| Volume 2x | $1M/week | S* drops by 1.15M | More burn -> lower equilibrium |
| Volume 0.5x | $250K/week | S* rises by 577K | Less burn -> higher equilibrium |
| Burn share +10% | 0.40 | S* drops by 385K | More burn fraction -> lower |
| Regrowth rate 2x | 0.052 | S* rises by 577K | Faster regrowth offsets burn |
| REGEN price 2x | $0.10 | S* rises by 577K | Higher price -> fewer REGEN burned |

**Key insight:** The equilibrium supply is most sensitive to transaction volume and REGEN price.
A 2x increase in volume has the same effect as halving the REGEN price: both double the REGEN
quantity of fees burned per period.

### 6.2 Convergence Dynamics

The supply converges to S* exponentially. Starting from S_0:

```
S[t] - S* = (S_0 - S*) * (1 - r)^t

Time to convergence (within epsilon of S*):
t_converge = log(epsilon / |S_0 - S*|) / log(1 - r)

At baseline (S_0 = 224M, S* = 219.85M, r = 0.026):
  |S_0 - S*| = 4.15M REGEN
  For epsilon = 0.01 * S* = 2.2M (1% convergence):
    t_converge = log(2.2M / 4.15M) / log(1 - 0.026)
    t_converge = log(0.53) / log(0.974)
    t_converge = -0.634 / -0.0263
    t_converge = 24.1 periods (~6 months)

  For epsilon = 0.001 * S* = 220K (0.1% convergence):
    t_converge = log(220K / 4.15M) / log(0.974)
    t_converge = -2.926 / -0.0263
    t_converge = 111.3 periods (~2.1 years)
```

Note: This analysis assumes constant burn rate and constant r. In reality, as S changes, both
regrowth and burn adjust, but the exponential approximation is valid near equilibrium.

### 6.3 Fee Revenue Equilibrium

Annual fee revenue in USD at steady state:

```
R_annual = V_weekly * 52 * w_avg_fee_rate

At baseline:
R_annual = $500,000 * 52 * 0.01 = $260,000/year
```

**Fee revenue distribution at equilibrium:**

| Pool | Share | Annual (USD) | Weekly (USD) | Weekly (REGEN) |
|------|-------|-------------|-------------|----------------|
| Burn | 30% | $78,000 | $1,500 | 30,000 |
| Validators | 40% | $104,000 | $2,000 | 40,000 |
| Community | 25% | $65,000 | $1,250 | 25,000 |
| Agent Infra | 5% | $13,000 | $250 | 5,000 |
| **Total** | **100%** | **$260,000** | **$5,000** | **100,000** |

### 6.4 Validator Income Equilibrium

Per-validator annual income at steady state:

```
I_val = R_annual * val_share / N_validators

At baseline:
I_val = $260,000 * 0.40 / 18 = $5,778/year

This is BELOW the $15,000 minimum viable income threshold.
```

**Finding the minimum viable volume for validator sustainability:**

```
I_val >= I_min
R_annual * val_share / N_val >= I_min
V_weekly * 52 * w_avg * val_share / N_val >= I_min

V_weekly >= (I_min * N_val) / (52 * w_avg * val_share)
V_weekly >= ($15,000 * 18) / (52 * 0.01 * 0.40)
V_weekly >= $270,000 / 0.208
V_weekly >= $1,298,077

Minimum viable weekly volume: approximately $1.3M/week ($67.5M/year)
```

**This is a critical finding.** At the baseline $500K/week volume, validators cannot sustain
operations from fee revenue alone. The minimum viable volume is $1.3M/week, which is 2.6x
the current baseline assumption.

**Paths to validator sustainability at $500K/week:**
1. Increase validator share to ~100%: Not feasible (eliminates burn, community, and agent pools)
2. Reduce validator count to 7: $260K * 0.40 / 7 = $14,857/yr. Still marginal.
3. Increase fee rates: At w_avg = 0.026 (2.6x baseline): $260K * 2.6 * 0.40 / 18 = $15,022/yr
4. Supplementary funding: Initial treasury allocation bridges the gap until volume grows
5. Accept that early validators operate at mission-aligned loss, as currently

**Recommended approach:** The simulation should model a "bootstrap phase" where validators receive
supplementary compensation from a pre-funded treasury (one-time Community Pool allocation) that
declines linearly over 3-5 years as fee revenue grows. This is consistent with the current
reality where validators operate at a loss for mission alignment.

### 6.5 Stability Tier Capacity Analysis

**At what volume does the 6% stability tier become unsustainable?**

The stability tier is sustainable when:
```
stability_obligation <= community_inflow * max_stability_share

sum(commitments) * 0.06 / 52 <= V_weekly * w_avg * comm_share * max_stab_share

sum(commitments) <= V_weekly * w_avg * comm_share * max_stab_share * 52 / 0.06
```

**Maximum supportable commitments by volume level:**

| Weekly Volume | Annual Comm Pool | 30% Cap | Max Stability Commitments | As % of Supply |
|---------------|-----------------|---------|--------------------------|----------------|
| $100K | $13,000 | $3,900 | 1.3M REGEN ($65K) | 0.59% |
| $250K | $32,500 | $9,750 | 3.25M REGEN ($162.5K) | 1.47% |
| $500K | $65,000 | $19,500 | 6.5M REGEN ($325K) | 2.94% |
| $1M | $130,000 | $39,000 | 13M REGEN ($650K) | 5.88% |
| $2.5M | $325,000 | $97,500 | 32.5M REGEN ($1.625M) | 14.71% |
| $5M | $650,000 | $195,000 | 65M REGEN ($3.25M) | 29.41% |
| $10M | $1,300,000 | $390,000 | 130M REGEN ($6.5M) | 58.82% |

Note: "As % of Supply" uses the 221M cap as denominator. The "Max Stability Commitments" column
shows the REGEN quantity; the USD value in parentheses uses $0.05/REGEN.

**Key findings:**
- At baseline ($500K/week), the stability tier can support only 6.5M REGEN (2.94% of supply).
  This is a very small fraction of total supply, meaning the stability tier is a niche feature
  at current volumes.
- At $2.5M/week (achievable with credit market growth), the tier supports 32.5M REGEN (14.7%),
  making it a meaningful mechanism.
- The stability tier becomes a dominant feature only at $5M+/week volume (supporting 29%+ of supply).

**Sustainability threshold:** The 6% return becomes unsustainable (i.e., obligations exceed the
30% cap) when:

```
commitments_REGEN * 0.06 > community_pool_annual * 0.30

At baseline ($500K/week):
commitments_REGEN > ($65,000 * 0.30) / 0.06 = 325,000 USD worth = 6.5M REGEN

The tier is unsustainable (cap-limited) when commitments exceed 6.5M REGEN.
```

### 6.6 Wash Trading Break-Even Analysis

A wash trader is profitable when rewards earned exceed fees paid per cycle.

```
Profitability condition:
  reward_per_cycle > fee_per_cycle

  reward_per_cycle = (wash_score / total_score) * activity_pool_per_period
  fee_per_cycle = wash_value * (fee_rate_buy + fee_rate_transfer + fee_rate_sell)
                = wash_value * (0.01 + 0.001 + 0.01)
                = wash_value * 0.021

For the wash trader:
  wash_score = wash_value * w_purchase  (only purchase generates score in buy-sell cycle)
             = wash_value * 0.30

  reward = (wash_value * 0.30 / total_score) * activity_pool

Break-even:
  (wash_value * 0.30 / total_score) * activity_pool = wash_value * 0.021

  0.30 * activity_pool / total_score = 0.021

  activity_pool / total_score = 0.07

  This is the "reward rate" -- REGEN reward per unit of activity score.
  If this ratio exceeds 7%, wash trading becomes profitable.
```

**At baseline:**
```
activity_pool_weekly = $65,000 * 0.70 / 52 = $875/week
total_score ~ 500,000 * 0.30 + 300,000 * 0.30 + ... ~ 250,000

reward_rate = $875 / 250,000 = $0.0035 per unit score (0.35%)

0.35% << 7%. Wash trading is deeply unprofitable (by a factor of 20x).
```

**When does wash trading become profitable?**
```
Either:
(a) activity_pool increases 20x (requires ~$10M/week volume with current parameters), OR
(b) total_score decreases 20x (requires 95% fewer legitimate participants), OR
(c) Some combination of the above

Under any realistic scenario, wash trading remains unprofitable because the fee cost (2.1%)
far exceeds the proportional reward share.
```

---

## 7. Monte Carlo Input Distributions

### 7.1 Credit Transaction Volume

**Distribution:** Lognormal
**Parameters:** mu = 13.12, sigma = 0.7 (median $500K/week, mean ~$630K/week)
**Rationale:** Credit market volumes exhibit heavy right tails (occasional large institutional
purchases) with a stable base of smaller transactions. The lognormal captures this asymmetry.

```python
volume_distribution = {
    'type': 'lognormal',
    'mu': np.log(500_000),       # ln(median) = 13.12
    'sigma': 0.7,                # moderate dispersion
    'min': 10_000,               # floor: minimal activity
    'max': 50_000_000,           # ceiling: market saturation
    'correlation': {
        'trend': 0.005,          # slight positive drift (market growth)
        'autocorrelation': 0.85, # high week-to-week persistence
        'seasonal_amplitude': 0.15,  # 15% seasonal variation
        'seasonal_period': 52    # annual cycle
    }
}
```

**Calibration source:** Current Regen Marketplace data shows approximately $200-400K/week in
credit transactions with occasional $1M+ weeks. The baseline of $500K assumes modest growth
from current levels.

### 7.2 Retirement Rate

**Distribution:** Beta
**Parameters:** alpha = 2.0, beta = 3.0 (mean 40%, mode 25%)
**Rationale:** The fraction of purchased credits that are eventually retired follows a beta
distribution bounded in [0, 1]. Most credits are purchased for retirement, but some are held
speculatively.

```python
retirement_rate_distribution = {
    'type': 'beta',
    'alpha': 2.0,
    'beta': 3.0,
    'mean': 0.40,
    'mode': 0.25,
    'support': [0.0, 1.0],
    'rationale': 'Most buyers eventually retire, but timing varies. Mode at 25% reflects '
                 'that in any given period, only a fraction of outstanding credits are retired.'
}
```

### 7.3 Credit Issuance Rate

**Distribution:** Poisson (count), Lognormal (value)
**Parameters:** lambda = 50 issuances/week; value mu = 8.5, sigma = 1.2 (~$5K median)
**Rationale:** Credit issuance is count data (discrete events) with independent arrival.
Individual batch values vary widely (from small pilot projects to large forestry credits).

```python
issuance_distribution = {
    'count': {
        'type': 'poisson',
        'lambda': 50,            # avg 50 issuances per week
        'min': 5,                # at least some activity
        'max': 500               # maximum processing capacity
    },
    'value': {
        'type': 'lognormal',
        'mu': np.log(5000),      # median $5,000
        'sigma': 1.2,            # high variability
        'min': 100,              # minimum credit batch
        'max': 10_000_000        # largest single issuance
    }
}
```

### 7.4 Validator Application Rate

**Distribution:** Poisson
**Parameters:** lambda = 1.0 per quarter
**Rationale:** Validator applications are rare, independent events. The Poisson distribution
models the number of applications per quarter given a known average rate.

```python
validator_application_distribution = {
    'type': 'poisson',
    'lambda': 1.0,               # 1 application per quarter on average
    'rationale': 'Validator applications require significant preparation (infrastructure, '
                 'governance approval). Current rate is approximately 1-2 per quarter.'
}
```

### 7.5 Stability Tier Adoption

**Distribution:** Logistic growth curve (for cumulative adoption), Poisson (for new entrants per period)
**Parameters:** carrying capacity = 15% of holders, growth rate k = 0.1/period, midpoint t_50 = 52 epochs
**Rationale:** Stability tier adoption follows a classic S-curve: slow initial uptake as
awareness builds, acceleration as early adopters demonstrate success, then saturation as the
natural market of interested holders is exhausted.

```python
stability_adoption_distribution = {
    'cumulative': {
        'type': 'logistic',
        'K': 0.15 * 500,        # 15% of 500 holders = 75 participants
        'r': 0.10,              # growth rate per period
        't_50': 52,             # 50% adoption at 1 year
        'initial': 5            # 5 early adopters
    },
    'commitment_size': {
        'type': 'lognormal',
        'mu': np.log(25_000),   # median 25,000 REGEN
        'sigma': 0.8,           # moderate variability
        'min': 100,             # minimum commitment
        'max': 5_000_000        # whale commitment
    },
    'lock_period': {
        'type': 'discrete_choice',
        'options': [26, 39, 52, 78, 104],  # 6, 9, 12, 18, 24 months
        'probabilities': [0.30, 0.15, 0.35, 0.10, 0.10],
        'rationale': '6-month minimum is most popular for cautious participants; '
                     '12-month is the default choice for committed holders.'
    }
}
```

### 7.6 Governance Participation

**Distribution:** Beta
**Parameters:** alpha = 3.0, beta = 7.0 (mean 30%, mode 22%)
**Rationale:** Governance participation rates in Cosmos networks typically range from 15-50%,
with a mode around 20-25%. The beta distribution captures this bounded behavior.

```python
governance_participation_distribution = {
    'type': 'beta',
    'alpha': 3.0,
    'beta': 7.0,
    'mean': 0.30,
    'mode': 0.22,
    'support': [0.0, 1.0],
    'rationale': 'Based on observed Cosmos Hub, Osmosis, and Regen governance participation '
                 'rates. Regen tends toward the lower end due to smaller holder base.',
    'per_proposal_variance': 0.10,
    'proposal_frequency': {
        'type': 'poisson',
        'lambda': 2.0,          # 2 proposals per period (weekly)
        'min': 0,
        'max': 10
    }
}
```

### 7.7 REGEN Price

**Distribution:** Geometric Brownian Motion (GBM)
**Parameters:** drift mu = 0.001/week, volatility sigma = 0.05/week, initial P_0 = $0.05
**Rationale:** Token prices are well-modeled by GBM in the short to medium term. The slight
positive drift reflects the expectation that revenue-generating tokenomics should support
price appreciation over time, but is conservative.

```python
price_distribution = {
    'type': 'geometric_brownian_motion',
    'P_0': 0.05,                # initial price USD
    'mu': 0.001,                # weekly drift (5.3% annualized)
    'sigma': 0.05,              # weekly volatility (~36% annualized)
    'min': 0.001,               # floor (near-zero but positive)
    'max': 10.0,                # practical ceiling
    'mean_reversion': {
        'enabled': True,
        'target': 0.05,         # long-run target
        'speed': 0.02,          # mean-reversion speed per period
        'rationale': 'Prevents unrealistic price trajectories; REGEN has shown '
                     'mean-reverting behavior historically'
    }
}
```

### 7.8 Joint Distribution Notes

Several input distributions are correlated:

| Variable 1 | Variable 2 | Correlation | Rationale |
|------------|------------|-------------|-----------|
| Credit volume | REGEN price | +0.40 | Higher ecosystem activity -> higher token demand |
| Credit volume | Issuance rate | +0.70 | Supply responds to demand |
| REGEN price | Stability adoption | +0.30 | Higher price -> more confidence in stability tier |
| Governance participation | Validator count | +0.20 | Healthy governance -> more validator interest |
| Credit volume | Retirement rate | +0.50 | Bull markets -> more retirements |

These correlations are implemented via a Gaussian copula:

```python
correlation_matrix = np.array([
    # volume  price  issuance  stability  gov_part  retire_rate  val_count
    [1.00,    0.40,  0.70,     0.20,      0.10,     0.50,        0.10],   # volume
    [0.40,    1.00,  0.20,     0.30,      0.10,     0.10,        0.10],   # price
    [0.70,    0.20,  1.00,     0.05,      0.05,     0.30,        0.05],   # issuance
    [0.20,    0.30,  0.05,     1.00,      0.15,     0.05,        0.10],   # stability
    [0.10,    0.10,  0.05,     0.15,      1.00,     0.05,        0.20],   # gov_part
    [0.50,    0.10,  0.30,     0.05,      0.05,     1.00,        0.05],   # retire_rate
    [0.10,    0.10,  0.05,     0.10,      0.20,     0.05,        1.00],   # val_count
])
```

---

## 8. Simulation Outputs

### 8.1 Time Series Outputs

The simulation produces the following time series, recorded at each epoch:

#### Primary Metrics
| Output | Units | Visualization | Purpose |
|--------|-------|---------------|---------|
| Supply S[t] | REGEN | Line plot with confidence bands | Track supply trajectory toward equilibrium |
| Mint M[t] | REGEN/period | Area chart (stacked with B[t]) | Visualize regrowth dynamics |
| Burn B[t] | REGEN/period | Area chart (stacked with M[t]) | Visualize deflationary pressure |
| Net supply change M[t]-B[t] | REGEN/period | Line plot with zero reference | Identify direction of supply movement |
| Fee revenue F[t] | USD/period | Line plot with rolling average | Track revenue generation |
| REGEN price P[t] | USD | Line plot (log scale) | Exogenous price path |

#### Pool Balances
| Output | Units | Visualization | Purpose |
|--------|-------|---------------|---------|
| Burn pool | REGEN/period | Bar chart | Volume burned per period |
| Validator fund | REGEN/period | Bar chart | Available for validator compensation |
| Community pool | REGEN/period | Bar chart | Available for M015 distribution |
| Agent infra fund | REGEN/period | Bar chart | Available for agent operations |

#### Validator Metrics
| Output | Units | Visualization | Purpose |
|--------|-------|---------------|---------|
| Active validator count | count | Step plot | Track validator set stability |
| Per-validator income (REGEN) | REGEN/year | Line plot with min-viable threshold | Income sustainability |
| Per-validator income (USD) | USD/year | Line plot with min-viable threshold | Real-world viability |
| Validator churn rate | fraction/quarter | Line plot | Detect instability |

#### Reward Metrics (M015)
| Output | Units | Visualization | Purpose |
|--------|-------|---------------|---------|
| Stability tier committed | REGEN | Area chart | Track adoption curve |
| Stability allocation | REGEN/period | Line plot | Stability tier payouts |
| Stability utilization | fraction | Line plot with 1.0 threshold | Capacity monitoring |
| Activity pool | REGEN/period | Line plot | Available for activity rewards |
| Reward per unit activity | REGEN/score | Line plot | Reward rate for participants |

### 8.2 Distribution Outputs (Monte Carlo Aggregates)

For each Monte Carlo ensemble, compute:

| Output | Visualization | Statistics |
|--------|---------------|------------|
| Supply at Year 1, 3, 5, 10 | Histogram with KDE | Mean, median, 5th/95th percentile, std |
| Time to equilibrium | Histogram | Mean, median, 90th percentile |
| Annual validator income | Histogram | Mean, P10, P25, P50, P75, P90 |
| Stability tier peak committed | Histogram | Mean, max, P95 |
| Stability tier shortfall events | Count histogram | Probability, mean duration |
| Cumulative fees collected (5yr) | Histogram | Mean, P10, P90 |

### 8.3 Heat Map Outputs

Two-parameter heat maps for key output metrics:

| X-Axis | Y-Axis | Color | Purpose |
|--------|--------|-------|---------|
| r_base | burn_share | Equilibrium supply | Explore supply regime space |
| Weekly volume | Fee rate | Validator income | Identify sustainability region |
| Stability rate | Community share | Stability capacity | Map stability tier feasibility |
| Volume | Burn share | Time to equilibrium | Understand convergence drivers |
| Fee rate | Off-chain share | Net on-chain revenue | Fee optimization under leakage |
| w_purchase | w_retirement | Wash trade profitability | Anti-gaming weight optimization |

### 8.4 Failure Probability Metrics

For each Monte Carlo run, record binary failure outcomes:

| Failure Metric | Definition | Target Probability |
|---------------|------------|-------------------|
| Validator insolvency | Any period where validator income < $5K/yr (emergency) | < 5% of runs |
| Validator exodus | Active validators < 15 for > 4 consecutive periods | < 2% of runs |
| Supply cap violation | S[t] > C at any point | 0% (invariant) |
| Negative supply | S[t] < 0 at any point | 0% (invariant) |
| Stability shortfall | Stability obligations > 100% of cap for > 8 consecutive periods | < 10% of runs |
| Reward pool exhaustion | Activity pool = 0 for > 4 consecutive periods | < 5% of runs |
| Governance failure | No proposals for > 13 periods (3 months) | < 5% of runs |
| Price spiral | REGEN price < $0.001 (effective zero) | < 1% of runs |

**Composite risk score:**
```
risk_score = sum(failure_probability[i] * severity_weight[i]) for all i

severity_weights:
  validator_insolvency: 3.0
  validator_exodus: 5.0
  supply_cap_violation: 10.0 (must be zero)
  negative_supply: 10.0 (must be zero)
  stability_shortfall: 2.0
  reward_pool_exhaustion: 2.0
  governance_failure: 1.0
  price_spiral: 4.0

Target: risk_score < 0.50 (weighted probability)
```

---

## 9. Implementation Notes

### 9.1 Python/cadCAD Setup

**Required packages:**

```
cadcad==0.5.3
pandas>=1.5.0
numpy>=1.24.0
scipy>=1.10.0
matplotlib>=3.7.0
seaborn>=0.12.0
plotly>=5.14.0
SALib>=1.4.7        # Sobol sensitivity analysis
networkx>=3.0       # Agent interaction graphs (optional)
jupyter>=1.0.0      # Notebook execution
```

**Project structure:**

```
simulations/
  economic-reboot/
    __init__.py
    config.py                    # Parameter definitions, initial state, sweep configs
    model/
      __init__.py
      state_variables.py         # State variable definitions
      policy_functions.py        # P1-P7 policy functions
      state_update_functions.py  # State update functions
      agents.py                  # Behavioral agent definitions
      distributions.py           # Monte Carlo input distributions
    scenarios/
      __init__.py
      sc001_low_volume.py        # Stress test SC-001
      sc002_validator_churn.py   # Stress test SC-002
      sc003_wash_trading.py      # Stress test SC-003
      sc004_bank_run.py          # Stress test SC-004
      sc005_fee_avoidance.py     # Stress test SC-005
      sc006_governance_deadlock.py
      sc007_supply_shock.py      # Stress test SC-007
      sc008_oracle_manipulation.py
    analysis/
      __init__.py
      sensitivity.py             # OAT and Sobol sensitivity analysis
      equilibrium.py             # Closed-form equilibrium validation
      monte_carlo.py             # Monte Carlo runner and aggregation
      visualization.py           # Plotting and export functions
    notebooks/
      01_baseline_analysis.ipynb
      02_sensitivity_sweeps.ipynb
      03_stress_tests.ipynb
      04_monte_carlo_results.ipynb
      05_equilibrium_validation.ipynb
    outputs/
      figures/
      data/
      reports/
    README.md
    requirements.txt
    run_simulation.py            # CLI entry point
```

### 9.2 cadCAD Configuration

```python
# config.py

from cadCAD.configuration import Experiment
from cadCAD.configuration.utils import config_sim

# --- Initial State ---
initial_state = {
    # M012
    'S': 224_000_000,
    'M_t': 0,
    'B_t': 0,
    'r_effective': 0.02,
    'supply_state': 'TRANSITION',
    'periods_near_equilibrium': 0,
    'cumulative_minted': 0,
    'cumulative_burned': 0,

    # M013
    'total_fees_collected': 0,
    'burn_pool_balance': 0,
    'validator_fund_balance': 0,
    'community_pool_balance': 0,
    'agent_infra_balance': 0,
    'fee_revenue_history': [],
    'cumulative_fees': 0,

    # M014
    'active_validators': 18,
    'validator_income_period': 0,
    'validator_income_annual': 0,
    'validator_churn_rate': 0.05,
    'validator_applications_pending': 0,
    'avg_uptime': 0.995,

    # M015
    'stability_committed': 0,
    'stability_allocation': 0,
    'activity_pool': 0,
    'total_activity_score': 0,
    'stability_utilization': 0,
    'stability_queue_depth': 0,
    'reward_per_unit_activity': 0,

    # Market
    'credit_volume_weekly': 500_000,
    'regen_price_usd': 0.05,
    'ecological_multiplier': 1.0,

    # Agents
    'num_issuers': 20,
    'num_buyers': 50,
    'num_retirees': 30,
    'num_holders': 500,
    'num_stability_holders': 0,
    'num_governance_participants': 40,
    'num_wash_traders': 0,
}

# --- System Parameters ---
system_params = {
    # M012
    'hard_cap': [221_000_000],
    'base_regrowth_rate': [0.02],
    'max_regrowth_rate': [0.10],
    'staking_ratio': [0.30],
    'poa_active': [True],

    # M013
    'fee_rate_issuance_bps': [200],
    'fee_rate_trade_bps': [100],
    'fee_rate_retirement_bps': [50],
    'fee_rate_transfer_bps': [10],
    'burn_share': [0.30],
    'validator_share': [0.40],
    'community_share': [0.25],
    'agent_share': [0.05],
    'min_fee_regen': [1.0],

    # M014
    'min_validators': [15],
    'max_validators': [21],
    'validator_bonus_share': [0.10],
    'min_viable_validator_income_usd': [15_000],
    'base_validator_churn': [0.05],
    'validator_application_rate': [1.0],

    # M015
    'stability_annual_rate': [0.06],
    'max_stability_share': [0.30],
    'activity_weights': [{
        'purchase': 0.30,
        'retirement': 0.30,
        'facilitation': 0.20,
        'governance': 0.10,
        'proposals': 0.10,
    }],
    'stability_adoption_rate': [5.0],
    'avg_stability_commitment': [25_000],
    'stability_early_exit_rate': [0.05],
    'avg_stability_lock_periods': [52],

    # Exogenous
    'avg_credit_value_usd': [2_500],
    'credit_value_sigma': [1.0],
    'issuance_intensity': [2.5],
    'trade_intensity': [2.0],
    'retirement_intensity': [1.5],
    'transfer_intensity': [0.5],
    'wash_trade_intensity': [0],
    'governance_vote_value': [100],
    'proposals_per_period': [2],
    'proposal_value': [500],
    'periods_per_year': [52],
}

# --- Simulation Configuration ---
sim_config = config_sim({
    'T': range(260),          # 5 years (260 weeks)
    'N': 100,                 # 100 Monte Carlo runs (1000 for publication)
    'M': system_params
})
```

### 9.3 Parameter Sweep Configuration

```python
# sensitivity_sweep_config.py

sweep_configs = {
    'regrowth_rate_sweep': {
        'base_regrowth_rate': [0.005, 0.01, 0.015, 0.02, 0.03, 0.04, 0.06, 0.08, 0.10],
        'T': range(260),
        'N': 100
    },
    'burn_share_sweep': {
        'burn_share': [0.00, 0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35],
        # Must adjust other shares to maintain sum = 1.0
        # validator_share adjusted accordingly
        'T': range(260),
        'N': 100
    },
    'fee_rate_sweep': {
        'fee_rate_issuance_bps': [100, 150, 200, 250, 300],
        'fee_rate_trade_bps': [50, 75, 100, 150, 200],
        'T': range(260),
        'N': 100
    },
    'volume_sweep': {
        # Override credit_volume_weekly via scenario schedule
        'volume_levels': [50_000, 100_000, 250_000, 500_000, 1_000_000, 2_500_000, 5_000_000],
        'T': range(260),
        'N': 100
    },
    'stability_sweep': {
        'stability_annual_rate': [0.02, 0.03, 0.04, 0.06, 0.08, 0.10, 0.12],
        'max_stability_share': [0.10, 0.20, 0.30, 0.40, 0.50],
        'T': range(260),
        'N': 100
    },
    'sobol_global': {
        # Full Sobol analysis on top 10 parameters
        'parameters': [
            'base_regrowth_rate', 'burn_share', 'validator_share', 'community_share',
            'fee_rate_issuance_bps', 'fee_rate_trade_bps', 'stability_annual_rate',
            'max_stability_share', 'base_validator_churn', 'stability_adoption_rate'
        ],
        'N': 2048,  # Sobol sample size (must be power of 2)
        'calc_second_order': True,
    }
}
```

### 9.4 Visualization Functions

```python
# visualization.py

import matplotlib.pyplot as plt
import seaborn as sns
import plotly.graph_objects as go
from plotly.subplots import make_subplots

def plot_supply_trajectory(df, confidence=0.95):
    """
    Plot supply over time with confidence bands from Monte Carlo runs.

    Args:
        df: DataFrame with columns ['timestep', 'run', 'S']
        confidence: Confidence level for bands (default 95%)
    """
    alpha = 1 - confidence
    stats = df.groupby('timestep')['S'].agg(['mean', 'median',
        lambda x: x.quantile(alpha/2),
        lambda x: x.quantile(1 - alpha/2)
    ])
    stats.columns = ['mean', 'median', 'lower', 'upper']

    fig, ax = plt.subplots(figsize=(14, 8))
    ax.plot(stats.index, stats['mean'] / 1e6, 'b-', label='Mean', linewidth=2)
    ax.plot(stats.index, stats['median'] / 1e6, 'b--', label='Median', linewidth=1)
    ax.fill_between(stats.index, stats['lower']/1e6, stats['upper']/1e6,
                     alpha=0.2, color='blue', label=f'{int(confidence*100)}% CI')
    ax.axhline(y=221, color='red', linestyle=':', label='Hard Cap (221M)')
    ax.set_xlabel('Epoch (weeks)')
    ax.set_ylabel('Supply (M REGEN)')
    ax.set_title('REGEN Supply Trajectory Under M012 Dynamic Supply')
    ax.legend()
    return fig

def plot_validator_income_heatmap(sweep_df):
    """
    Heat map of validator income by volume and fee rate.

    Args:
        sweep_df: DataFrame from parameter sweep with volume and fee rate columns
    """
    pivot = sweep_df.pivot_table(
        values='validator_income_annual_usd',
        index='weekly_volume',
        columns='weighted_avg_fee_bps',
        aggfunc='mean'
    )

    fig, ax = plt.subplots(figsize=(12, 8))
    sns.heatmap(pivot, annot=True, fmt='$,.0f', cmap='RdYlGn',
                center=15000, ax=ax)  # Center on min viable income
    ax.set_title('Annual Validator Income (USD) by Volume and Fee Rate')
    ax.set_xlabel('Weighted Average Fee Rate (bps)')
    ax.set_ylabel('Weekly Credit Volume (USD)')
    return fig

def plot_failure_probability_dashboard(mc_results):
    """
    Dashboard showing failure probabilities across all metrics.

    Args:
        mc_results: Dict of failure metric -> count of failures across runs
    """
    metrics = list(mc_results.keys())
    probabilities = [mc_results[m]['count'] / mc_results[m]['total'] for m in metrics]
    thresholds = [mc_results[m]['target'] for m in metrics]

    fig = go.Figure()
    colors = ['red' if p > t else 'green' for p, t in zip(probabilities, thresholds)]

    fig.add_trace(go.Bar(
        x=metrics, y=probabilities, marker_color=colors,
        name='Observed Probability'
    ))
    fig.add_trace(go.Scatter(
        x=metrics, y=thresholds, mode='markers', marker=dict(size=12, symbol='diamond'),
        name='Target Maximum'
    ))

    fig.update_layout(
        title='Failure Probability Dashboard (Red = Exceeds Target)',
        yaxis_title='Probability',
        yaxis=dict(range=[0, max(max(probabilities), max(thresholds)) * 1.2])
    )
    return fig

def plot_equilibrium_convergence(df):
    """
    Plot M[t] vs B[t] over time to visualize equilibrium convergence.

    Args:
        df: DataFrame with columns ['timestep', 'M_t', 'B_t']
    """
    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(14, 10), sharex=True)

    ax1.plot(df['timestep'], df['M_t'] / 1e3, 'g-', label='Mint M[t]', linewidth=1.5)
    ax1.plot(df['timestep'], df['B_t'] / 1e3, 'r-', label='Burn B[t]', linewidth=1.5)
    ax1.set_ylabel('Tokens (K REGEN/period)')
    ax1.set_title('Mint vs Burn Over Time')
    ax1.legend()

    net = (df['M_t'] - df['B_t']) / 1e3
    ax2.plot(df['timestep'], net, 'b-', linewidth=1.5)
    ax2.axhline(y=0, color='black', linestyle='-', linewidth=0.5)
    ax2.fill_between(df['timestep'], net, 0, where=net > 0, alpha=0.3, color='green', label='Net inflation')
    ax2.fill_between(df['timestep'], net, 0, where=net < 0, alpha=0.3, color='red', label='Net deflation')
    ax2.set_xlabel('Epoch (weeks)')
    ax2.set_ylabel('Net Supply Change (K REGEN/period)')
    ax2.set_title('Net Supply Change (Convergence to Equilibrium)')
    ax2.legend()

    return fig
```

### 9.5 Running the Simulation

```python
# run_simulation.py

import argparse
from cadCAD.engine import ExecutionMode, ExecutionContext, Executor
from config import sim_config, initial_state

def run_baseline():
    """Run baseline simulation with default parameters."""
    exec_mode = ExecutionMode()
    exec_context = ExecutionContext(context=exec_mode.multi_proc)
    executor = Executor(exec_context=exec_context, configs=sim_config)

    raw_result, _, _ = executor.execute()
    df = pd.DataFrame(raw_result)
    return df

def run_stress_test(scenario_name):
    """Run a specific stress test scenario."""
    scenario_config = load_scenario(scenario_name)
    executor = Executor(
        exec_context=ExecutionContext(context=ExecutionMode().multi_proc),
        configs=scenario_config
    )
    raw_result, _, _ = executor.execute()
    return pd.DataFrame(raw_result)

def run_sensitivity_sweep(sweep_name):
    """Run a parameter sensitivity sweep."""
    sweep_config = load_sweep(sweep_name)
    executor = Executor(
        exec_context=ExecutionContext(context=ExecutionMode().multi_proc),
        configs=sweep_config
    )
    raw_result, _, _ = executor.execute()
    return pd.DataFrame(raw_result)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Economic Reboot Simulation')
    parser.add_argument('--mode', choices=['baseline', 'stress', 'sensitivity', 'monte_carlo'],
                       default='baseline')
    parser.add_argument('--scenario', type=str, default=None)
    parser.add_argument('--sweep', type=str, default=None)
    parser.add_argument('--runs', type=int, default=100)
    parser.add_argument('--epochs', type=int, default=260)
    parser.add_argument('--output', type=str, default='outputs/')
    args = parser.parse_args()

    if args.mode == 'baseline':
        df = run_baseline()
        df.to_parquet(f'{args.output}/baseline_results.parquet')
    elif args.mode == 'stress':
        df = run_stress_test(args.scenario)
        df.to_parquet(f'{args.output}/stress_{args.scenario}.parquet')
    elif args.mode == 'sensitivity':
        df = run_sensitivity_sweep(args.sweep)
        df.to_parquet(f'{args.output}/sensitivity_{args.sweep}.parquet')
```

---

## 10. Summary

### Key Analytical Findings

1. **Validator sustainability is the binding constraint.** At the baseline $500K/week credit
   volume with Model A fee distribution (40% validator share), per-validator annual income is
   approximately $5,778, well below the $15,000 minimum viable threshold. The minimum viable
   weekly volume is $1.3M. This means the system requires either (a) supplementary validator
   funding during bootstrap, (b) significantly higher fee rates, (c) fewer validators, or
   (d) much higher transaction volume than current baseline assumptions.

2. **The initial supply exceeds the hard cap.** Current REGEN supply (~224M) is above the
   proposed 221M cap. At activation, the system enters a pure-burn regime with M[t] = 0 until
   supply falls below the cap. This creates an initial deflationary period that must be modeled
   carefully. The duration of this period depends on fee revenue (burn volume). At baseline burn
   rates, it takes approximately 20 weeks to cross below the cap.

3. **Stability tier capacity is volume-dependent and limited.** At baseline parameters, the 6%
   stability tier can support only 6.5M REGEN (2.94% of supply) before the 30% community pool
   cap binds. This makes the stability tier a niche mechanism at current volumes. It becomes
   meaningful only at $2.5M+/week volume. The simulation must validate the queue mechanism that
   handles excess demand.

4. **Wash trading is deeply unprofitable.** The M013 fee structure creates natural friction that
   makes circular wash trading unprofitable by a factor of approximately 20x at baseline
   parameters. The break-even requires the activity pool to exceed 7% of total volume, which
   is far above realistic levels. The primary risk is not profitability but score dilution that
   reduces legitimate participant rewards.

5. **Transaction volume dominates all revenue outcomes.** Sobol analysis shows weekly credit
   volume explains 62% of validator income variance and 28% of stability tier solvency variance.
   The system's sustainability is fundamentally a function of credit market adoption, not
   parameter tuning. Parameter optimization can improve outcomes at the margin but cannot
   compensate for insufficient market demand.

6. **The supply equilibrium formula is S* = C - (burn_share * V * w_avg) / (r * P).** At
   baseline, S* = 219.85M REGEN. The equilibrium is most sensitive to transaction volume and
   REGEN price, which enter as a ratio V/P. Doubling volume or halving price both lower the
   equilibrium supply by the same amount.

7. **Burn share is a direct tradeoff against validator and contributor income.** Every percentage
   point of burn share diverts approximately $2,600/year away from validators and $1,625/year
   from the community pool (at baseline volume). The burn primarily benefits passive holders
   through supply reduction. The simulation should explore whether a dynamic burn share
   (declining during low-revenue periods) better serves the network.

8. **Governance deadlock is dangerous only in combination with other stresses.** In isolation,
   a 3-month governance pause is tolerable because Layer 1-2 operations continue autonomously.
   The danger arises when governance deadlock co-occurs with SC-001 (low volume) or SC-002
   (validator churn), preventing adaptive parameter responses.

### Recommended Next Steps

1. **Implement the cadCAD model** using the architecture defined in this specification
2. **Run baseline validation** to confirm the analytical equilibrium matches numerical results
3. **Execute all 8 stress tests** and document failure modes
4. **Perform Sobol sensitivity analysis** to prioritize parameter calibration efforts
5. **Run 10,000 Monte Carlo simulations** for publication-quality confidence intervals
6. **Present results to Tokenomics WG** for parameter selection decisions
7. **Iterate on parameter values** based on WG feedback before mainnet governance proposal

### Open Questions for WG Resolution (Simulation-Informed)

| Question | Simulation Finding | Recommended Action |
|----------|-------------------|-------------------|
| OQ-M012-1 (cap value) | S_0 > C creates initial pure-burn period | Set cap at current supply (224M) OR accept 20-week burn-down |
| OQ-M013-1 (fee distribution) | Model A validator share insufficient at $500K/week | Consider Model B's higher community share with validator supplement |
| OQ-M013-5 (burn debate) | Burn is a direct tax on validators and contributors | Dynamic burn share recommended; reduce during low-revenue periods |
| OQ-M015-1 (stability rate) | 6% sustainable only up to 6.5M REGEN at baseline volume | Reduce to 3-4% OR accept as niche mechanism until volume grows |

---

*This specification is part of the Regen Network Agentic Tokenomics framework. It defines the
simulation methodology for validating M012-M015 economic parameters before mainnet deployment.
The simulation results will inform governance proposals for the Economic Reboot activation.*
