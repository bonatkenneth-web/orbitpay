use soroban_sdk::{Address, Env};

use crate::types::{DataKey, Subscription};

/// ~30 days at 5-second ledger close time.
const SUBSCRIPTION_BUMP_AMOUNT: u32 = 518_400;
/// Bump when less than ~15 days of TTL remain.
const SUBSCRIPTION_LIFETIME_THRESHOLD: u32 = 259_200;

pub struct Storage;

impl Storage {
    // ── Admin ─────────────────────────────────────────────────────────────────

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&DataKey::Admin, admin);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    pub fn has_admin(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Admin)
    }

    // ── Pause flag ────────────────────────────────────────────────────────────

    pub fn set_paused(env: &Env, paused: bool) {
        env.storage().instance().set(&DataKey::Paused, &paused);
    }

    pub fn is_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    // ── Subscriptions ─────────────────────────────────────────────────────────

    /// Persist a subscription and bump its TTL.
    pub fn set_subscription(env: &Env, key: &DataKey, sub: &Subscription) {
        env.storage().persistent().set(key, sub);
        env.storage().persistent().extend_ttl(
            key,
            SUBSCRIPTION_LIFETIME_THRESHOLD,
            SUBSCRIPTION_BUMP_AMOUNT,
        );
    }

    /// Retrieve a subscription, bumping TTL on hit.
    pub fn get_subscription(env: &Env, key: &DataKey) -> Option<Subscription> {
        let maybe: Option<Subscription> = env.storage().persistent().get(key);
        if maybe.is_some() {
            env.storage().persistent().extend_ttl(
                key,
                SUBSCRIPTION_LIFETIME_THRESHOLD,
                SUBSCRIPTION_BUMP_AMOUNT,
            );
        }
        maybe
    }

    pub fn has_subscription(env: &Env, key: &DataKey) -> bool {
        env.storage().persistent().has(key)
    }

    pub fn remove_subscription(env: &Env, key: &DataKey) {
        env.storage().persistent().remove(key);
    }
}
