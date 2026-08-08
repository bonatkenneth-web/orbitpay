# OrbitPay — Decentralized Subscriptions on Stellar

OrbitPay is a non-custodial, automated recurring-payments protocol built on Stellar's **Soroban** smart contract platform.  It bridges traditional SaaS billing with decentralised finance, letting businesses and creators accept recurring crypto payments (USDC or any SEP-41 token) without relying on centralised payment processors or credit-card networks.

---

## How It Works

```
Subscriber                 OrbitPay Contract            Merchant
    │                            │                          │
    │── create_sub() ──────────► │                          │
    │   (signs once, sets        │                          │
    │    amount + interval)      │                          │
    │                            │                          │
    │  [30 days later]           │                          │
    │                            │◄── execute_pay() ───────-│
    │◄── transfer_from() ────────│── token ────────────────►│
    │                            │                          │
    │── cancel_sub() ──────────► │  (revoke anytime)        │
```

1. **Authorisation** – the subscriber calls `create_sub`, signing a transaction that records the merchant, token, amount, and billing interval on-chain.  At the same time they grant the contract a token *allowance* via the SEP-41 `approve` call.
2. **Time-locked execution** – anyone (the merchant or a keeper bot) calls `execute_pay`.  The contract enforces the interval and pulls funds atomically.  No subscriber signature is needed at execution time.
3. **Full user control** – the subscriber can call `cancel_sub` at any moment, revoking the merchant's ability to charge them.

---

## Project Structure

```
orbitpay/
├── Cargo.toml                        # workspace manifest
├── Makefile                          # developer commands
├── README.md
├── .gitignore
├── scripts/
│   └── deploy.sh                     # deployment automation
└── contracts/
    └── recurring-payments/
        ├── Cargo.toml
        └── src/
            ├── lib.rs                # contract entry-points
            ├── types.rs              # DataKey, Subscription structs
            ├── errors.rs             # typed error codes
            ├── events.rs             # on-chain event helpers
            ├── storage.rs            # persistent storage + TTL management
            └── test.rs               # comprehensive test suite
```

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | ≥ 1.78  | `rustup update stable` |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| stellar CLI | latest | [docs.stellar.org/tools/stellar-cli](https://developers.stellar.org/docs/tools/stellar-cli) |
| wasm-opt (optional) | — | `apt install binaryen` / `brew install binaryen` |

---

## Quick Start

```bash
# 1. Clone and enter the repo
git clone https://github.com/your-org/orbitpay.git
cd orbitpay

# 2. Run the test suite
make test

# 3. Build the release WASM
make build

# 4. (Optional) Shrink the binary
make optimize
```

---

## Contract Interface

### `create_sub`
```rust
fn create_sub(
    env: Env,
    subscriber: Address,   // must sign
    merchant:   Address,
    token:      Address,   // SEP-41 token contract
    amount:     i128,      // base units per interval
    interval_seconds: u64, // e.g. 2_592_000 for 30 days
) -> Result<(), Error>
```

### `execute_pay`
```rust
fn execute_pay(
    env:        Env,
    subscriber: Address,
    merchant:   Address,
) -> Result<(), Error>
```
Callable by anyone.  Reverts with `PaymentNotDue` if the interval has not elapsed.

### `cancel_sub`
```rust
fn cancel_sub(
    env:        Env,
    subscriber: Address,   // must sign
    merchant:   Address,
) -> Result<(), Error>
```

### `get_sub`
```rust
fn get_sub(env: Env, subscriber: Address, merchant: Address) -> Option<Subscription>
```

### `is_payment_due`
```rust
fn is_payment_due(env: Env, subscriber: Address, merchant: Address) -> bool
```

---

## Error Codes

| Code | Name | Meaning |
|------|------|---------|
| 1 | `InvalidAmount` | amount must be > 0 |
| 2 | `InvalidInterval` | interval_seconds must be > 0 |
| 3 | `SubscriptionNotFound` | no subscription for this pair |
| 4 | `PaymentNotDue` | interval has not yet elapsed |
| 5 | `ArithmeticOverflow` | timestamp overflow (should never occur) |

---

## On-Chain Events

| Topic | Data | Trigger |
|-------|------|---------|
| `(sub_new, subscriber, merchant)` | `(token, amount, interval)` | `create_sub` |
| `(pay_exec, subscriber, merchant)` | `(amount, timestamp)` | `execute_pay` |
| `(sub_del, subscriber, merchant)` | `()` | `cancel_sub` |

---

## Deployment

```bash
# Set up a funded identity (first time only)
stellar keys generate --global deployer
stellar keys fund deployer --network testnet

# Deploy to testnet
make deploy-testnet

# Deploy to mainnet (interactive confirmation required)
make deploy-mainnet
```

Deployment records are written to `deployments/<network>.json`.

---

## Ecosystem Integrations (Drips Wave)

OrbitPay is intentionally modular.  The core contract is a stable foundation for community-built extensions:

- **Keeper Bot** – a Node.js / Python service that monitors `is_payment_due` and calls `execute_pay` automatically.
- **Merchant Dashboard** – a Next.js app consuming on-chain events to display active subscribers.
- **Subscriber Portal** – a consumer UI for browsing and cancelling active subscriptions.

---

## License

MIT — see [LICENSE](LICENSE).
