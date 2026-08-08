#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

pub use errors::Error;
pub use types::{Subscription, SubscriptionKey};

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env};

use errors::Error as ContractError;
use storage::Storage;
use types::{DataKey, SubscriptionKey as SubKey, MAX_AMOUNT, MAX_INTERVAL};

/// OrbitPay Recurring Payments Contract
///
/// Non-custodial pull-payment subscriptions on Stellar Soroban.
/// A subscriber authorises the contract once; the merchant (or a keeper bot)
/// calls `execute_pay` after each billing interval without any further
/// subscriber involvement.
#[contract]
pub struct RecurringPaymentContract;

#[contractimpl]
impl RecurringPaymentContract {
    // ─────────────────────────────────────────────────────────────────────────
    // Admin / lifecycle
    // ─────────────────────────────────────────────────────────────────────────

    /// One-time initialisation.  Must be called before any other function.
    ///
    /// # Errors
    /// * [`Error::AlreadyInitialized`] if called more than once.
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if Storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        Storage::set_admin(&env, &admin);
        Storage::set_paused(&env, false);
        events::emit_initialized(&env, &admin);
        Ok(())
    }

    /// Pause all payment execution.  Admin only.
    pub fn pause(env: Env) -> Result<(), ContractError> {
        let admin = Self::require_admin(&env)?;
        Storage::set_paused(&env, true);
        events::emit_paused(&env, &admin);
        Ok(())
    }

    /// Resume payment execution.  Admin only.
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        let admin = Self::require_admin(&env)?;
        Storage::set_paused(&env, false);
        events::emit_unpaused(&env, &admin);
        Ok(())
    }

    /// Upgrade the contract WASM in place.  Admin only.
    ///
    /// The new WASM must already be uploaded to the network.
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        Self::require_admin(&env)?;
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    /// Transfer admin rights to a new address.  Admin only.
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), ContractError> {
        Self::require_admin(&env)?;
        new_admin.require_auth();
        Storage::set_admin(&env, &new_admin);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Subscription management
    // ─────────────────────────────────────────────────────────────────────────

    /// Create or update a subscription.
    ///
    /// When overwriting an existing subscription the `last_payment_time` is
    /// preserved so a merchant cannot trigger an early charge by convincing the
    /// subscriber to re-call this function.
    ///
    /// # Arguments
    /// * `subscriber`       – wallet that will be charged; must sign this tx
    /// * `merchant`         – wallet that will receive each payment
    /// * `token`            – SEP-41 token contract address
    /// * `amount`           – base-unit amount per interval (1 .. MAX_AMOUNT)
    /// * `interval_seconds` – seconds between charges (1 .. MAX_INTERVAL)
    ///
    /// # Errors
    /// * [`Error::InvalidAmount`]   – amount out of range
    /// * [`Error::InvalidInterval`] – interval out of range
    pub fn create_sub(
        env: Env,
        subscriber: Address,
        merchant: Address,
        token: Address,
        amount: i128,
        interval_seconds: u64,
    ) -> Result<(), ContractError> {
        subscriber.require_auth();

        if amount <= 0 || amount > MAX_AMOUNT {
            return Err(ContractError::InvalidAmount);
        }
        if interval_seconds == 0 || interval_seconds > MAX_INTERVAL {
            return Err(ContractError::InvalidInterval);
        }

        let key = DataKey::Subscription(SubKey {
            subscriber: subscriber.clone(),
            merchant: merchant.clone(),
        });

        // Preserve last_payment_time if subscription already exists so that
        // re-creating with new params cannot be used to reset the billing clock.
        let last_payment_time = Storage::get_subscription(&env, &key)
            .map(|s| s.last_payment_time)
            .unwrap_or(0);

        let sub = Subscription {
            token: token.clone(),
            amount,
            interval_seconds,
            last_payment_time,
        };

        Storage::set_subscription(&env, &key, &sub);
        events::emit_subscription_created(
            &env,
            &subscriber,
            &merchant,
            &token,
            amount,
            interval_seconds,
        );

        Ok(())
    }

    /// Execute a due payment.
    ///
    /// Callable by anyone (merchant or keeper bot). No subscriber signature
    /// required at execution time — the allowance granted during `create_sub`
    /// is sufficient.
    ///
    /// The next due timestamp is anchored to `last_payment_time + interval`
    /// rather than `now`, so a late execution does not push the schedule
    /// forward.
    ///
    /// # Errors
    /// * [`Error::ContractPaused`]       – contract is paused
    /// * [`Error::SubscriptionNotFound`] – no subscription for this pair
    /// * [`Error::PaymentNotDue`]        – interval has not elapsed yet
    pub fn execute_pay(
        env: Env,
        subscriber: Address,
        merchant: Address,
    ) -> Result<(), ContractError> {
        if Storage::is_paused(&env) {
            return Err(ContractError::ContractPaused);
        }

        let key = DataKey::Subscription(SubKey {
            subscriber: subscriber.clone(),
            merchant: merchant.clone(),
        });

        let mut sub: Subscription = Storage::get_subscription(&env, &key)
            .ok_or(ContractError::SubscriptionNotFound)?;

        let current_time = env.ledger().timestamp();

        if sub.last_payment_time != 0 {
            let next_due = sub
                .last_payment_time
                .checked_add(sub.interval_seconds)
                .ok_or(ContractError::ArithmeticOverflow)?;

            if current_time < next_due {
                return Err(ContractError::PaymentNotDue);
            }
        }

        // Transfer tokens via the pre-approved allowance.
        let client = token::Client::new(&env, &sub.token);
        client.transfer_from(
            &env.current_contract_address(),
            &subscriber,
            &merchant,
            &sub.amount,
        );

        // Anchor the next due time to the billing schedule, not to now.
        // This prevents schedule drift when a keeper executes late.
        sub.last_payment_time = if sub.last_payment_time == 0 {
            current_time
        } else {
            sub.last_payment_time
                .checked_add(sub.interval_seconds)
                .ok_or(ContractError::ArithmeticOverflow)?
        };

        Storage::set_subscription(&env, &key, &sub);
        events::emit_payment_executed(&env, &subscriber, &merchant, sub.amount, current_time);

        Ok(())
    }

    /// Cancel a subscription. Only the subscriber may call this.
    ///
    /// # Errors
    /// * [`Error::SubscriptionNotFound`] – nothing to cancel
    pub fn cancel_sub(
        env: Env,
        subscriber: Address,
        merchant: Address,
    ) -> Result<(), ContractError> {
        subscriber.require_auth();

        let key = DataKey::Subscription(SubKey {
            subscriber: subscriber.clone(),
            merchant: merchant.clone(),
        });

        if !Storage::has_subscription(&env, &key) {
            return Err(ContractError::SubscriptionNotFound);
        }

        Storage::remove_subscription(&env, &key);
        events::emit_subscription_cancelled(&env, &subscriber, &merchant);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Read-only views
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns subscription details if it exists.
    pub fn get_sub(
        env: Env,
        subscriber: Address,
        merchant: Address,
    ) -> Option<Subscription> {
        let key = DataKey::Subscription(SubKey { subscriber, merchant });
        Storage::get_subscription(&env, &key)
    }

    /// Returns `true` if the subscription exists and a payment is currently due.
    pub fn is_payment_due(env: Env, subscriber: Address, merchant: Address) -> bool {
        let key = DataKey::Subscription(SubKey { subscriber, merchant });
        match Storage::get_subscription(&env, &key) {
            None => false,
            Some(sub) => {
                if sub.last_payment_time == 0 {
                    return true;
                }
                let current_time = env.ledger().timestamp();
                sub.last_payment_time
                    .checked_add(sub.interval_seconds)
                    .map(|next_due| current_time >= next_due)
                    .unwrap_or(false)
            }
        }
    }

    /// Returns the current admin address.
    pub fn get_admin(env: Env) -> Option<Address> {
        Storage::get_admin(&env)
    }

    /// Returns whether the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        Storage::is_paused(&env)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────────────

    /// Verifies the caller is the stored admin and requires their auth.
    fn require_admin(env: &Env) -> Result<Address, ContractError> {
        let admin = Storage::get_admin(env).ok_or(ContractError::Unauthorized)?;
        admin.require_auth();
        Ok(admin)
    }
}
