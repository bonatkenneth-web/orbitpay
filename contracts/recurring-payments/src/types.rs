use soroban_sdk::{contracttype, Address};

// ── Bounds ────────────────────────────────────────────────────────────────────

/// Maximum single-payment amount (10 billion in base units).
pub const MAX_AMOUNT: i128 = 10_000_000_000_i128;

/// Maximum billing interval: 366 days in seconds.
pub const MAX_INTERVAL: u64 = 366 * 24 * 60 * 60;

// ── Storage keys ──────────────────────────────────────────────────────────────

/// Top-level storage namespace discriminant.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Per-subscription record, keyed by (subscriber, merchant).
    Subscription(SubscriptionKey),
    /// Singleton: the contract admin address.
    Admin,
    /// Singleton: pause flag (bool).
    Paused,
}

/// Composite key that uniquely identifies a subscription.
#[derive(Clone)]
#[contracttype]
pub struct SubscriptionKey {
    pub subscriber: Address,
    pub merchant: Address,
}

// ── Domain types ──────────────────────────────────────────────────────────────

/// On-chain subscription record.
#[derive(Clone)]
#[contracttype]
pub struct Subscription {
    /// SEP-41 / Stellar asset contract that will be transferred.
    pub token: Address,
    /// Amount (in token base units) charged each interval.
    pub amount: i128,
    /// Minimum seconds between successive charges.
    pub interval_seconds: u64,
    /// Ledger timestamp of the most recent successful charge.
    /// Zero means the subscription has never been charged.
    pub last_payment_time: u64,
}
