use soroban_sdk::{symbol_short, Address, Env};

pub fn emit_initialized(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("init"), admin.clone()), ());
}

pub fn emit_paused(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("paused"), admin.clone()), ());
}

pub fn emit_unpaused(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("unpaused"), admin.clone()), ());
}

pub fn emit_subscription_created(
    env: &Env,
    subscriber: &Address,
    merchant: &Address,
    token: &Address,
    amount: i128,
    interval_seconds: u64,
) {
    env.events().publish(
        (symbol_short!("sub_new"), subscriber.clone(), merchant.clone()),
        (token.clone(), amount, interval_seconds),
    );
}

pub fn emit_payment_executed(
    env: &Env,
    subscriber: &Address,
    merchant: &Address,
    amount: i128,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("pay_exec"), subscriber.clone(), merchant.clone()),
        (amount, timestamp),
    );
}

pub fn emit_subscription_cancelled(env: &Env, subscriber: &Address, merchant: &Address) {
    env.events().publish(
        (symbol_short!("sub_del"), subscriber.clone(), merchant.clone()),
        (),
    );
}
