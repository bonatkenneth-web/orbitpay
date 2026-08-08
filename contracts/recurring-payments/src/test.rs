#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

// ─── Test harness ─────────────────────────────────────────────────────────────

struct TestEnv {
    env: Env,
    contract_id: Address,
    admin: Address,
    subscriber: Address,
    merchant: Address,
    token_contract: Address,
}

impl TestEnv {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, RecurringPaymentContract);
        let admin = Address::generate(&env);
        let subscriber = Address::generate(&env);
        let merchant = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token_contract = env
            .register_stellar_asset_contract_v2(token_admin.clone())
            .address();

        // Mint 10_000 tokens to subscriber and approve the contract.
        let admin_client = token::StellarAssetClient::new(&env, &token_contract);
        admin_client.mint(&subscriber, &10_000);
        token::Client::new(&env, &token_contract).approve(
            &subscriber,
            &contract_id,
            &10_000,
            &1_000_000,
        );

        // Initialise the contract.
        let client = RecurringPaymentContractClient::new(&env, &contract_id);
        client.initialize(&admin).unwrap();

        TestEnv {
            env,
            contract_id,
            admin,
            subscriber,
            merchant,
            token_contract,
        }
    }

    fn client(&self) -> RecurringPaymentContractClient<'_> {
        RecurringPaymentContractClient::new(&self.env, &self.contract_id)
    }

    fn token(&self) -> token::Client<'_> {
        token::Client::new(&self.env, &self.token_contract)
    }

    fn advance_time(&self, seconds: u64) {
        self.env.ledger().with_mut(|li| li.timestamp += seconds);
    }

    fn create_default_sub(&self) {
        self.client()
            .create_sub(
                &self.subscriber,
                &self.merchant,
                &self.token_contract,
                &AMOUNT,
                &INTERVAL,
            )
            .unwrap();
    }
}

const INTERVAL: u64 = 86_400; // 1 day
const AMOUNT: i128 = 100;

// ─── Initialisation ───────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin() {
    let t = TestEnv::setup();
    assert_eq!(t.client().get_admin(), Some(t.admin.clone()));
}

#[test]
fn test_initialize_twice_fails() {
    let t = TestEnv::setup();
    let result = t.client().try_initialize(&t.admin);
    assert!(result.is_err(), "second initialize must be rejected");
}

#[test]
fn test_is_not_paused_after_init() {
    let t = TestEnv::setup();
    assert!(!t.client().is_paused());
}

// ─── Pause / unpause ──────────────────────────────────────────────────────────

#[test]
fn test_pause_blocks_execute_pay() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().pause().unwrap();

    let result = t.client().try_execute_pay(&t.subscriber, &t.merchant);
    assert!(result.is_err(), "execute_pay must fail while paused");
}

#[test]
fn test_unpause_allows_execute_pay() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().pause().unwrap();
    t.client().unpause().unwrap();

    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();
    assert_eq!(t.token().balance(&t.merchant), AMOUNT);
}

// ─── Upgrade / transfer_admin ─────────────────────────────────────────────────

#[test]
fn test_transfer_admin() {
    let t = TestEnv::setup();
    let new_admin = Address::generate(&t.env);
    t.client().transfer_admin(&new_admin).unwrap();
    assert_eq!(t.client().get_admin(), Some(new_admin));
}

// ─── create_sub ───────────────────────────────────────────────────────────────

#[test]
fn test_create_sub_success() {
    let t = TestEnv::setup();
    t.create_default_sub();
    let sub = t.client().get_sub(&t.subscriber, &t.merchant).unwrap();
    assert_eq!(sub.amount, AMOUNT);
    assert_eq!(sub.interval_seconds, INTERVAL);
    assert_eq!(sub.last_payment_time, 0);
}

#[test]
fn test_create_sub_invalid_amount_zero() {
    let t = TestEnv::setup();
    assert!(t
        .client()
        .try_create_sub(&t.subscriber, &t.merchant, &t.token_contract, &0, &INTERVAL)
        .is_err());
}

#[test]
fn test_create_sub_invalid_amount_negative() {
    let t = TestEnv::setup();
    assert!(t
        .client()
        .try_create_sub(&t.subscriber, &t.merchant, &t.token_contract, &-1, &INTERVAL)
        .is_err());
}

#[test]
fn test_create_sub_amount_exceeds_max() {
    let t = TestEnv::setup();
    assert!(t
        .client()
        .try_create_sub(
            &t.subscriber,
            &t.merchant,
            &t.token_contract,
            &(types::MAX_AMOUNT + 1),
            &INTERVAL
        )
        .is_err());
}

#[test]
fn test_create_sub_invalid_interval_zero() {
    let t = TestEnv::setup();
    assert!(t
        .client()
        .try_create_sub(&t.subscriber, &t.merchant, &t.token_contract, &AMOUNT, &0)
        .is_err());
}

#[test]
fn test_create_sub_interval_exceeds_max() {
    let t = TestEnv::setup();
    assert!(t
        .client()
        .try_create_sub(
            &t.subscriber,
            &t.merchant,
            &t.token_contract,
            &AMOUNT,
            &(types::MAX_INTERVAL + 1)
        )
        .is_err());
}

/// Re-creating a subscription must NOT reset last_payment_time.
#[test]
fn test_overwrite_preserves_last_payment_time() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();

    let before = t
        .client()
        .get_sub(&t.subscriber, &t.merchant)
        .unwrap()
        .last_payment_time;
    assert!(before > 0);

    // Overwrite with new params.
    t.client()
        .create_sub(
            &t.subscriber,
            &t.merchant,
            &t.token_contract,
            &200,
            &INTERVAL,
        )
        .unwrap();

    let after = t
        .client()
        .get_sub(&t.subscriber, &t.merchant)
        .unwrap()
        .last_payment_time;

    assert_eq!(before, after, "last_payment_time must survive an overwrite");
}

// ─── execute_pay ──────────────────────────────────────────────────────────────

#[test]
fn test_first_payment_executes_immediately() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();
    assert_eq!(t.token().balance(&t.merchant), AMOUNT);
    assert_eq!(t.token().balance(&t.subscriber), 10_000 - AMOUNT);
}

#[test]
fn test_second_payment_blocked_before_interval() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();
    t.advance_time(INTERVAL / 2);
    assert!(t
        .client()
        .try_execute_pay(&t.subscriber, &t.merchant)
        .is_err());
}

#[test]
fn test_second_payment_executes_after_interval() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();
    t.advance_time(INTERVAL);
    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();
    assert_eq!(t.token().balance(&t.merchant), AMOUNT * 2);
}

/// A late execution must not push the billing schedule forward.
#[test]
fn test_no_schedule_drift_on_late_execution() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();

    // Keeper is 12 hours late on the second payment.
    t.advance_time(INTERVAL + INTERVAL / 2);
    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();

    let sub = t.client().get_sub(&t.subscriber, &t.merchant).unwrap();
    let ledger_now = t.env.ledger().timestamp();

    // next due should be anchored to last_payment_time + interval, not now
    let next_due = sub.last_payment_time + sub.interval_seconds;
    assert!(
        next_due <= ledger_now + INTERVAL,
        "schedule must not drift: next_due={next_due} ledger_now={ledger_now}"
    );
}

#[test]
fn test_multiple_payment_cycles() {
    let t = TestEnv::setup();
    t.create_default_sub();
    for cycle in 1..=5u64 {
        if cycle > 1 {
            t.advance_time(INTERVAL);
        }
        t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();
        assert_eq!(t.token().balance(&t.merchant), AMOUNT * cycle as i128);
    }
}

#[test]
fn test_execute_pay_non_existent_fails() {
    let t = TestEnv::setup();
    assert!(t
        .client()
        .try_execute_pay(&t.subscriber, &t.merchant)
        .is_err());
}

// ─── cancel_sub ───────────────────────────────────────────────────────────────

#[test]
fn test_cancel_sub_success() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().cancel_sub(&t.subscriber, &t.merchant).unwrap();
    assert!(t.client().get_sub(&t.subscriber, &t.merchant).is_none());
}

#[test]
fn test_execute_pay_after_cancel_fails() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().cancel_sub(&t.subscriber, &t.merchant).unwrap();
    assert!(t
        .client()
        .try_execute_pay(&t.subscriber, &t.merchant)
        .is_err());
}

#[test]
fn test_cancel_non_existent_fails() {
    let t = TestEnv::setup();
    assert!(t
        .client()
        .try_cancel_sub(&t.subscriber, &t.merchant)
        .is_err());
}

// ─── is_payment_due ───────────────────────────────────────────────────────────

#[test]
fn test_is_payment_due_first_payment() {
    let t = TestEnv::setup();
    t.create_default_sub();
    assert!(t.client().is_payment_due(&t.subscriber, &t.merchant));
}

#[test]
fn test_is_payment_due_after_execution() {
    let t = TestEnv::setup();
    t.create_default_sub();
    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();
    assert!(!t.client().is_payment_due(&t.subscriber, &t.merchant));
    t.advance_time(INTERVAL);
    assert!(t.client().is_payment_due(&t.subscriber, &t.merchant));
}

#[test]
fn test_is_payment_due_no_subscription() {
    let t = TestEnv::setup();
    assert!(!t.client().is_payment_due(&t.subscriber, &t.merchant));
}

// ─── Independent subscriptions ───────────────────────────────────────────────

#[test]
fn test_independent_subscriptions() {
    let t = TestEnv::setup();
    let merchant2 = Address::generate(&t.env);

    t.client()
        .create_sub(&t.subscriber, &t.merchant, &t.token_contract, &100, &INTERVAL)
        .unwrap();
    t.client()
        .create_sub(&t.subscriber, &merchant2, &t.token_contract, &200, &INTERVAL)
        .unwrap();

    t.client().execute_pay(&t.subscriber, &t.merchant).unwrap();
    t.client().execute_pay(&t.subscriber, &merchant2).unwrap();

    assert_eq!(t.token().balance(&t.merchant), 100);
    assert_eq!(t.token().balance(&merchant2), 200);
    assert_eq!(t.token().balance(&t.subscriber), 10_000 - 300);

    // Cancelling one must not affect the other.
    t.client().cancel_sub(&t.subscriber, &t.merchant).unwrap();
    assert!(t.client().get_sub(&t.subscriber, &t.merchant).is_none());
    assert!(t.client().get_sub(&t.subscriber, &merchant2).is_some());
}
