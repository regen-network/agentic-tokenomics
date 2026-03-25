# M013 Fee Routing — CosmWasm Contract Architecture

## Overview

This document specifies the CosmWasm contract architecture for M013 Value-Based Fee Routing. The contract handles fee calculation on ecocredit transactions, pool distribution, and immediate burn execution. It is the first mechanism in the implementation sequence and can be deployed independently of M014 (PoA) and M012 (Dynamic Supply).

## Design Principles

1. **All parameters are governance-controlled** — fee rates, pool splits, and burn percentage are updatable via governance proposal without software upgrade
2. **Burn is immediate** — the burn portion of each fee is destroyed in the same transaction, emitting standard Cosmos burn events
3. **M012 integration via event consumption** — the supply algorithm reads aggregate burn events; it does not need to know about M013 specifically
4. **Multiple burn sources coexist** — M013's burn is one of several independent burn mechanisms feeding M012's `B[t]`

---

## Contract: `regen-fee-router`

### State

```rust
/// Fee schedule: rate per credit transaction type
/// Stored as basis points (1 bp = 0.01%)
/// Example: 200 = 2.00%
pub struct FeeSchedule {
    pub issuance_rate_bps: u16,      // 100-300 (1-3%)
    pub transfer_rate_bps: u16,      // 10 (0.1%)
    pub retirement_rate_bps: u16,    // 50 (0.5%)
    pub trade_rate_bps: u16,         // 100 (1%)
    pub min_fee_uregen: Uint128,     // Floor fee in uregen
    pub max_rate_bps: u16,           // Constitutional max (1000 = 10%)
}

/// Pool distribution: how collected fees are split
/// Stored as basis points, MUST sum to 10000
pub struct PoolDistribution {
    pub burn_bps: u16,               // e.g., 2000 (20%) — includes the 2% universal burn
    pub validator_fund_bps: u16,     // e.g., 4000 (40%)
    pub community_pool_bps: u16,     // e.g., 3500 (35%)
    pub agent_infra_bps: u16,        // e.g., 500 (5%)
}

/// Contract configuration
pub struct Config {
    pub admin: Addr,                 // x/gov module address
    pub fee_schedule: FeeSchedule,
    pub pool_distribution: PoolDistribution,
    pub enabled: bool,               // Circuit breaker
    pub transition_end_height: Option<u64>, // Dual-fee period end
}
```

### Messages

#### Execute

```rust
pub enum ExecuteMsg {
    /// Called by x/ecocredit hooks on credit transactions
    /// Calculates fee, splits to pools, executes burn
    ProcessFee {
        tx_type: CreditTxType,       // Issuance | Transfer | Retirement | Trade
        credit_value_uregen: Uint128, // Estimated value of credits transacted
        sender: String,               // Fee payer
    },

    /// Governance-only: update fee schedule
    UpdateFeeSchedule {
        new_schedule: FeeSchedule,
    },

    /// Governance-only: update pool distribution
    UpdatePoolDistribution {
        new_distribution: PoolDistribution,
    },

    /// Governance-only: enable/disable (circuit breaker)
    SetEnabled {
        enabled: bool,
    },
}
```

#### Query

```rust
pub enum QueryMsg {
    /// Get current configuration
    Config {},

    /// Simulate fee for a given transaction
    SimulateFee {
        tx_type: CreditTxType,
        credit_value_uregen: Uint128,
    },

    /// Get accumulated stats (total fees, total burned, per-pool totals)
    Stats {},

    /// Get fee events for a block range (for M012 consumption)
    FeeEvents {
        start_height: u64,
        end_height: u64,
    },
}
```

### Fee Calculation Logic

```
fee_amount = max(credit_value × rate_bps / 10000, min_fee_uregen)

if fee_amount / credit_value > max_rate_bps / 10000:
    fee_amount = credit_value × max_rate_bps / 10000
```

All arithmetic uses `Uint128` (128-bit unsigned integers) — no floating point. Division uses integer division with explicit remainder handling.

### Pool Distribution Logic

```
burn_amount     = fee_amount × burn_bps / 10000
community_amount = fee_amount × community_pool_bps / 10000
agent_amount    = fee_amount × agent_infra_bps / 10000
validator_amount = fee_amount - burn_amount - community_amount - agent_amount
```

Validator fund receives the remainder to avoid rounding dust accumulation.

### Burn Execution

```rust
// Within ProcessFee handler:
if burn_amount > Uint128::zero() {
    let burn_msg = BankMsg::Burn {
        amount: vec![Coin {
            denom: "uregen".to_string(),
            amount: burn_amount,
        }],
    };
    // Emits standard cosmos.bank.v1beta1.EventBurn
    // M012 reads these events to compute B[t]
    msgs.push(SubMsg::new(burn_msg));
}
```

The `BankMsg::Burn` emits a standard Cosmos SDK burn event. M012's supply algorithm aggregates all burn events on-chain — it doesn't need to know they came from M013.

### Pool Routing

```rust
// Send to validator fund module account
if validator_amount > Uint128::zero() {
    msgs.push(SubMsg::new(BankMsg::Send {
        to_address: VALIDATOR_FUND_ADDR,
        amount: vec![coin(validator_amount, "uregen")],
    }));
}

// Send to community pool via x/distribution FundCommunityPool
if community_amount > Uint128::zero() {
    msgs.push(SubMsg::new(CosmosMsg::Distribution(
        DistributionMsg::FundCommunityPool {
            amount: vec![coin(community_amount, "uregen")],
        },
    )));
}

// Send to agent infrastructure account
if agent_amount > Uint128::zero() {
    msgs.push(SubMsg::new(BankMsg::Send {
        to_address: AGENT_INFRA_ADDR,
        amount: vec![coin(agent_amount, "uregen")],
    }));
}
```

---

## Integration with x/ecocredit

### Hook Pattern

The fee router needs to intercept credit transactions. Two approaches:

**Option A: Native Module Hook (Preferred)**
Add a `PostHandler` or `BeginBlocker` hook in x/ecocredit that calls the fee router contract after each credit transaction. This is cleaner but requires a native module change (part of the SDK upgrade).

**Option B: Wrapper Contract**
Deploy a wrapper contract that users interact with instead of x/ecocredit directly. The wrapper calls x/ecocredit and then M013. This is deployable without native module changes but adds UX complexity.

**Recommendation:** Option A for mainnet (aligns with the SDK v0.54 upgrade timeline). Option B for testnet/proof-of-concept.

### Credit Value Estimation

The `credit_value_uregen` parameter requires a value estimation for the credits being transacted:

| Transaction Type | Value Source |
|-----------------|-------------|
| Trade (MsgBuySellOrder) | Explicit price in the order |
| Transfer (MsgSend) | Last marketplace trade price for the credit class, or governance-set default |
| Retirement (MsgRetire) | Same as Transfer |
| Issuance (MsgCreateBatch) | Governance-set default per credit class (cold-start) |

**Cold-start problem:** New credit classes have no trade history. Use a governance-set default price per credit class. If no default exists, fall back to `min_fee_uregen`.

---

## Relationship to Existing and Future Burn Sources

M013's burn is one input to M012's aggregate `B[t]`:

```
B[t] = sum of all burns in period t
  ├── M013 fee routing burn        ← this contract
  ├── Credit class creation burn   ← existing (1,000 credits, native module)
  ├── Retirement-triggered burns   ← under development
  ├── Voluntary MsgBurn            ← existing
  └── Future sources               ← new contracts, etc.
```

M012 reads `cosmos.bank.v1beta1.EventBurn` events across the chain. It does not need to distinguish sources. Each burn mechanism is independently governed and deployed.

**The 2% marketplace burn currently scheduled** for the next upgrade should be implemented as M013's `trade_rate_bps` burn portion rather than as a separate standalone mechanism. This consolidates all fee-based burns into a single governed contract.

---

## Governance Parameters

| Parameter | Initial Value | Governance Layer | Notes |
|-----------|--------------|-----------------|-------|
| `issuance_rate_bps` | 200 (2%) | Layer 2 | Adjustable by standard proposal |
| `transfer_rate_bps` | 10 (0.1%) | Layer 2 | |
| `retirement_rate_bps` | 50 (0.5%) | Layer 2 | |
| `trade_rate_bps` | 100 (1%) | Layer 2 | |
| `min_fee_uregen` | 1,000,000 (1 REGEN) | Layer 2 | Review if micro-tx friction is too high |
| `max_rate_bps` | 1000 (10%) | Layer 4 | Constitutional max, 67% supermajority |
| `burn_bps` | 2000 (20%) | Layer 3 | Includes the universal 2% burn |
| `validator_fund_bps` | 4000 (40%) | Layer 3 | Pending OQ-M013-4 (Model A vs B) |
| `community_pool_bps` | 3500 (35%) | Layer 3 | |
| `agent_infra_bps` | 500 (5%) | Layer 3 | |
| `enabled` | true | Layer 2 | Circuit breaker |

**Invariant:** `burn_bps + validator_fund_bps + community_pool_bps + agent_infra_bps = 10000`

The contract enforces this invariant on every `UpdatePoolDistribution` call.

---

## Transition Plan

### Phase 1: Testnet (Q2 2026)
- Deploy contract to regen-upgrade testnet
- Test with synthetic credit transactions
- Validate fee calculation, pool routing, burn execution
- Verify M012 can read burn events

### Phase 2: Dual-Fee Period (mainnet)
- Deploy contract via governance proposal
- Run alongside existing flat gas fees for 90 days
- Monitor fee revenue, distribution, burn rate
- Community can compare old vs new fee model

### Phase 3: Full Activation
- Governance proposal to disable legacy flat fee for credit transactions
- M013 becomes sole fee mechanism for credit operations
- Gas fees remain for non-credit transactions (DoS protection)

---

## Security Considerations

1. **Fee Conservation:** Total distributed = total collected. No tokens created or lost in routing.
2. **Share Sum Unity:** Pool percentages always sum to 10000 bps. Enforced on-chain.
3. **Rate Bound Safety:** No fee rate can exceed `max_rate_bps` (constitutional parameter).
4. **Integer Arithmetic Only:** All calculations use `Uint128`. No floating point.
5. **Remainder Handling:** Validator fund receives remainder after other pools — no dust accumulation.
6. **Circuit Breaker:** `enabled` flag can be toggled by governance to halt fee collection in emergency.
7. **Admin Restricted:** Only x/gov module address can update parameters.

---

## Open Questions Requiring Resolution Before Implementation

| OQ | Question | Impact on Contract |
|-----|---------|-------------------|
| OQ-M013-3 | Fee denomination (REGEN-only vs multi-denom) | Determines whether contract handles denom conversion |
| OQ-M013-4 | Distribution model (Model A vs B) | Sets initial `PoolDistribution` values |
| OQ-M013-5 | Burn pool existence | If no burn, `burn_bps` starts at 0 and `community_pool_bps` absorbs it |

The contract architecture supports any resolution of these questions — the params are governed and updatable.

---

## Dependencies

| Dependency | Status | Blocking? |
|-----------|--------|----------|
| CosmWasm on Regen Ledger | Live (v7.0, governance-gated uploads) | No |
| `BankMsg::Burn` support in CosmWasm | Available in wasmd v0.60+ | No |
| x/ecocredit hook for fee interception | Needs native module change (Option A) or wrapper (Option B) | Option A blocks on SDK upgrade |
| Credit value oracle / estimation | Cold-start defaults needed | Partial — trades have explicit prices |
| Pool module accounts | Need to be registered | Part of deployment |

---

## Related

- M013 mechanism spec: `phase-2/2.6-economic-reboot-mechanisms.md`
- M012 supply algorithm: `phase-2/2.6-economic-reboot-mechanisms.md`
- M014 validator compensation: `phase-2/2.6-economic-reboot-mechanisms.md`
- Smart contract specs: `phase-3/3.1-smart-contract-specs.md`
- Cosmos Labs x/poa: `docs/cosmos-poa-module.md`
- SDK migration: `docs/sdk-v054-migration.md`
