use thiserror::Error;


#[derive(Debug, Error)]
pub enum FieldError{

    #[error("Invalid prime: {0} is not a prime number")]
    NotPrime(u64),

    #[error("Value {value} is outside field range [0, {prime})")]
    OutOfRange{value: u64, prime: u64},

    #[error("Cannot perform operation on elements from different fields (prime {left} vs {right})")]
    FieldMismatch{left: u64, right: u64},

     #[error("Division by zero in field")]
    DivisionByZero,

}
//new

#[derive(Debug, Error)]
pub enum ShamirError {
    #[error("Threshold {threshold} cannot exceed total shares {total_shares}")]
    InvalidThreshold {
        threshold: u64,
        total_shares: u64,
    },

    #[error("Threshold must be at least 1, got {0}")]
    ThresholdTooSmall(u64),

    #[error("At least {0} shares required for reconstruction, got {1}")]
    InsufficientShares(u64, usize),

    #[error("Cannot reconstruct: shares from different fields (prime {left} vs {right})")]
    ShareFieldMismatch { left: u64, right: u64 },

    #[error(transparent)]
    FieldError(#[from] FieldError),
}