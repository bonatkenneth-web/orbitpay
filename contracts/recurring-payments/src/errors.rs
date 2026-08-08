use soroban_sdk::contracterror;

/// All error codes returned by the OrbitPay contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// `amount` must be between 1 and MAX_AMOUNT.
    InvalidAmount = 1,
    /// `interval_seconds` must be between 1 and MAX_INTERVAL.
    InvalidInterval = 2,
    /// No subscription found for the given (subscriber, merchant) pair.
    SubscriptionNotFound = 3,
    /// The billing interval has not yet elapsed since the last payment.
    PaymentNotDue = 4,
    /// An arithmetic overflow occurred when computing the next due timestamp.
    ArithmeticOverflow = 5,
    /// Contract is paused; no payments may be executed.
    ContractPaused = 6,
    /// `initialize` has already been called.
    AlreadyInitialized = 7,
    /// Caller is not the contract admin.
    Unauthorized = 8,
}
