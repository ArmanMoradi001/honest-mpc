use crate::error::{FieldError, ShamirError};
use crate::field::FieldElement;
use rand::rngs::OsRng;
use rand::Rng;

#[derive(Debug)]
pub struct Shamir {
    secret: FieldElement,
    total_shares: u64,
    threshold: u64,
}

impl Shamir {
    /// Construct a new Shamir secret sharing scheme.
    ///
    /// # Errors
    /// - `ShamirError::InvalidThreshold` if `threshold > total_shares`
    /// - `ShamirError::ThresholdTooSmall` if `threshold < 1`
    /// - `ShamirError::FieldError` if `prime` is invalid or `secret` out of range
    pub fn new(
        secret: u64,
        total_shares: u64,
        threshold: u64,
        prime: u64,
    ) -> Result<Self, ShamirError> {
        if threshold < 1 {
            return Err(ShamirError::ThresholdTooSmall(threshold));
        }
        if threshold > total_shares {
            return Err(ShamirError::InvalidThreshold { threshold, total_shares });
        }
        let field_secret = FieldElement::new(secret, prime)?;
        Ok(Self { secret: field_secret, total_shares, threshold })
    }

    /// Split the secret into `total_shares` shares.
    /// Any `threshold` of them can reconstruct the secret.
    pub fn split(&self) -> Vec<(FieldElement, FieldElement)> {
        let mut coefficients = vec![self.secret];
        let mut rng = OsRng;

        for _ in 1..self.threshold {
            let random_num = rng.gen_range(0..self.secret.prime());
            // safe: prime is already validated, random_num is in range
            let coeff = FieldElement::new(random_num, self.secret.prime()).unwrap();
            coefficients.push(coeff);
        }

        let mut shares = Vec::new();

        for x in 1..=self.total_shares {
            // safe: x < prime because total_shares must be < prime for correctness
            let x_fe = FieldElement::new(x, self.secret.prime()).unwrap();
            let mut y = FieldElement::zero(self.secret.prime()).unwrap();

            for (i, coeff) in coefficients.iter().enumerate() {
                let term = *coeff * x_fe.pow(i as u64);
                y = y + term;
            }

            shares.push((x_fe, y));
        }

        shares
    }

    /// Reconstruct the secret from a slice of shares using Lagrange interpolation.
    ///
    /// # Errors
    /// - `ShamirError::InsufficientShares` if `shares` is empty
    /// - `ShamirError::ShareFieldMismatch` if shares come from different fields
    pub fn reconstruct(
        shares: &[(FieldElement, FieldElement)],
        threshold: u64,
    ) -> Result<FieldElement, ShamirError> {
        if shares.is_empty() {
            return Err(ShamirError::InsufficientShares(threshold, 0));
        }
        if (shares.len() as u64) < threshold {
            return Err(ShamirError::InsufficientShares(threshold, shares.len()));
        }

        let prime = shares[0].0.prime();

        // Verify all shares are from the same field
        for (x, _) in shares.iter() {
            if x.prime() != prime {
                return Err(ShamirError::ShareFieldMismatch {
                    left: prime,
                    right: x.prime(),
                });
            }
        }

        let mut result = FieldElement::zero(prime).unwrap();

        for (i, (xi, yi)) in shares.iter().enumerate() {
            let mut numerator = FieldElement::one(prime).unwrap();
            let mut denominator = FieldElement::one(prime).unwrap();

            for (j, (xj, _)) in shares.iter().enumerate() {
                if i == j { continue; }
                numerator = numerator * (FieldElement::zero(prime).unwrap() - *xj);
                denominator = denominator * (*xi - *xj);
            }

            let lagrange_basis = numerator / denominator;
            result = result + (*yi * lagrange_basis);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIME: u64 = 97;
    const SECRET: u64 = 42;

    #[test]
    fn test_reconstruct_with_exact_threshold() {
        let shamir = Shamir::new(SECRET, 5, 3, PRIME).unwrap();
        let shares = shamir.split();
        let recovered = Shamir::reconstruct(&shares[0..3], 3).unwrap();
        assert_eq!(recovered.value(), SECRET);
    }

    #[test]
    fn test_reconstruct_with_more_than_threshold() {
        let shamir = Shamir::new(SECRET, 5, 3, PRIME).unwrap();
        let shares = shamir.split();
        let recovered = Shamir::reconstruct(&shares[0..5], 3).unwrap();
        assert_eq!(recovered.value(), SECRET);
    }

    #[test]
    fn test_different_share_combinations_give_same_secret() {
        let shamir = Shamir::new(SECRET, 6, 4, PRIME).unwrap();
        let shares = shamir.split();
        assert_eq!(Shamir::reconstruct(&shares[0..4], 4).unwrap().value(), SECRET);
        assert_eq!(Shamir::reconstruct(&shares[1..5], 4).unwrap().value(), SECRET);
        assert_eq!(Shamir::reconstruct(&shares[2..6], 4).unwrap().value(), SECRET);
    }

    #[test]
    fn test_threshold_too_small_returns_err() {
        assert!(matches!(
            Shamir::new(SECRET, 5, 0, PRIME),
            Err(ShamirError::ThresholdTooSmall(0))
        ));
    }

    #[test]
    fn test_threshold_exceeds_total_returns_err() {
        assert!(matches!(
            Shamir::new(SECRET, 5, 6, PRIME),
            Err(ShamirError::InvalidThreshold { threshold: 6, total_shares: 5 })
        ));
    }

    #[test]
    fn test_empty_shares_returns_err() {
        assert!(matches!(
            Shamir::reconstruct(&[], 3),
            Err(ShamirError::InsufficientShares(3, 0))
        ));
    }

    #[test]
    fn test_insufficient_shares_returns_err() {
        let shamir = Shamir::new(SECRET, 5, 3, PRIME).unwrap();
        let shares = shamir.split();
        assert!(matches!(
            Shamir::reconstruct(&shares[0..2], 3),
            Err(ShamirError::InsufficientShares(3, 2))
        ));
    }

    #[test]
    fn test_invalid_prime_returns_err() {
        assert!(matches!(
            Shamir::new(SECRET, 5, 3, 10),
            Err(ShamirError::FieldError(FieldError::NotPrime(10)))
        ));
    }
}