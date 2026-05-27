# Early Exit Penalty

Penalty charged when users withdraw before the lock-up period ends. Penalty is configurable and transferred to the protocol treasury.

## Overview

The early exit penalty system ensures that users who exit their bond before the lock-up period ends pay a proportional penalty to the treasury. This maintains protocol economics and prevents users from bypassing lock-up commitments.

**CRITICAL SECURITY:** The `withdraw()` function enforces lock-up expiry and will panic with "lock-up not expired; use withdraw_early" if called before lock-up ends. Users attempting early exit MUST use `withdraw_early()`, which applies the penalty. This prevents penalty bypass attacks.

## Configuration

- **treasury**: Address that receives penalty amounts.
- **early_exit_penalty_bps**: Rate in basis points (e.g. 500 = 5%). Must be ≤ 10000.

Set via `set_early_exit_config(admin, treasury, penalty_bps)`. Admin-only.

## Penalty Formula

`penalty = (amount * penalty_bps / 10000) * (remaining_time / total_duration)`

- **remaining_time**: Time left until lock-up end (`end - now`).
- **total_duration**: Bond duration at creation.

So penalty is proportional to how much of the lock period remains.

### Example

- Bond: 1000 USDC, 365 days duration
- Penalty rate: 10% (1000 bps)
- Withdraw 500 USDC after 182 days (halfway through)
- Remaining time: 183 days
- Penalty: (500 * 1000 / 10000) * (183 / 365) = 50 * 0.5 = 25 USDC
- User receives: 475 USDC
- Treasury receives: 25 USDC

## Functions

### withdraw_early(amount)

Withdraws `amount` before lock-up end. Applies penalty; penalty is attributed to treasury (in a full implementation, token transfer would send `amount - penalty` to user and `penalty` to treasury). Emits `early_exit_penalty` event with (identity, withdraw_amount, penalty_amount, treasury).

**Valid time window:** Only before lock-up expiry (`now < bond_start + bond_duration`)

**Errors:**
- `LockupNotExpired` (204) - if called at or after lock-up expiry; use `withdraw()` instead

### withdraw(amount)

Use after lock-up or after notice period for rolling bonds. No penalty.

**Valid time window:** Only at or after lock-up expiry (`now >= bond_start + bond_duration`)

**Errors:**
- `LockupStillActive` (217) - if called before lock-up expiry; use `withdraw_early()` instead

## Mutual Exclusivity

The two withdrawal functions have non-overlapping valid time windows:

| Time | withdraw() | withdraw_early() |
|------|-----------|------------------|
| Before lock-up end | ❌ Panics: "lock-up not expired" | ✅ Succeeds with penalty |
| At lock-up end | ✅ Succeeds, no penalty | ❌ Reverts with `LockupNotExpired` |
| After lock-up end | ✅ Succeeds, no penalty | ❌ Reverts with `LockupNotExpired` |

This design ensures:
1. Early exits always pay the penalty
2. Post-lock-up withdrawals never pay a penalty
3. No way to bypass the penalty system

## Events

- **early_exit_penalty**: (identity, withdraw_amount, penalty_amount, treasury)

## Security

- Penalty capped by amount and rate; no overflow in calculation.
- Config can only be set by admin.
- **Lock-up gate:** `withdraw()` enforces `now >= bond_start + bond_duration` before allowing withdrawal.
- **Early exit gate:** `withdraw_early()` enforces `now < bond_start + bond_duration` before applying penalty.
- Withdrawing after lock-up must use `withdraw`, not `withdraw_early`.
- Withdrawing before lock-up must use `withdraw_early`, not `withdraw`.

## Attack Prevention

### Penalty Bypass Attack (PREVENTED)

**Attack scenario:**
1. Attacker creates bond with 365-day lock-up
2. On day 364, attacker calls `withdraw()` to avoid penalty
3. Attacker receives full amount without paying penalty to treasury

**Prevention:**
The `withdraw()` function computes `end = bond_start + bond_duration` and requires `now >= end`. If called before lock-up expiry, it panics with "lock-up not expired; use withdraw_early", forcing the attacker to use `withdraw_early()` which applies the penalty.

```rust
// In withdraw():
let end = bond.bond_start.checked_add(bond.bond_duration).expect("overflow");
if now < end {
    panic!("lock-up not expired; use withdraw_early");
}
```

This ensures the treasury receives penalties from all early exits, maintaining protocol economics.
