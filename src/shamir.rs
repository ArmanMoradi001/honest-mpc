use rand::rng;
use crate::FieldElement;

#[derive(Debug)]
pub struct Shamir {
    secret: FieldElement,
    total_shares: u64,
    threshold: u64,
}
impl Shamir {
    pub fn new(
        secret: u64,
        total_shares: u64,
        threshold: u64,
        prime: u64,
    ) -> Self {
        if threshold > total_shares {
            panic!(
                "Threshold {} could not be greater than total shares {}",
                threshold,
                total_shares
            );
        }

        if threshold < 1 {
            panic!("Threshold must be at least 1");
        }

        let field_secret = FieldElement::new(secret, prime);

        Self {
            secret: field_secret,
            total_shares,
            threshold,
        }
    }

    pub fn split(&self) -> Vec<(FieldElement, FieldElement)> {
        let mut coefficients = vec![self.secret.clone()];

        let mut rng = rand::rng();

        for _ in 1..self.threshold {
            let random_num = rng.random_range(0..self.secret.prime);

            let random_coeff =
                FieldElement::new(random_num, self.secret.prime);

            coefficients.push(random_coeff);
        }

        let mut shares = Vec::new();

        for x in 1..=self.total_shares {
            let x_fe = FieldElement::new(x, self.secret.prime);

            let mut y = FieldElement::new(0, self.secret.prime);

            for (i, coeff) in coefficients.iter().enumerate() {
                let term = coeff.clone() * x_fe.pow(i as u32);
                y = y + term;
            }

            shares.push((x_fe, y));
        }

        shares
    }

    pub fn reconstruct(
        shares: &[(FieldElement, FieldElement)],
    ) -> FieldElement {
        if shares.is_empty() {
            panic!("At least one share is required");
        }

        let prime = shares[0].0.prime;

        let mut result = FieldElement::new(0, prime);

        for (i, (xi, yi)) in shares.iter().enumerate() {
            let mut numerator = FieldElement::new(1, prime);

            let mut denominator = FieldElement::new(1, prime);

            for (j, (xj, _)) in shares.iter().enumerate() {
                if i == j {
                    continue;
                }

                numerator =
                    numerator * (FieldElement::new(0, prime) - xj.clone());

                denominator =
                    denominator * (xi.clone() - xj.clone());
            }

            let lagrange_basis = numerator / denominator;

            result = result + (yi.clone() * lagrange_basis);
        }

        result
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::FieldElement;

    #[test]
    fn test_shamir_threshold_correctness() {
        let prime = 41; // Small prime for testing
        let secret = 25u64;
        let total = 5;
        let threshold = 3;

        let shamir = Shamir::new(secret, total, threshold, prime);
        let shares = shamir.split();

        // Reconstruct with exactly threshold shares
        let reconstructed = Shamir::reconstruct(&shares[0..threshold as usize]);
        assert_eq!(reconstructed.value(), secret);

        // Reconstruct with more than threshold shares
        let reconstructed2 = Shamir::reconstruct(&shares[0..4]);
        assert_eq!(reconstructed2.value(), secret);
    }

    #[test]
    #[should_panic]
    fn test_invalid_threshold() {
        let _ = Shamir::new(42, 5, 6, 17);
    }

    #[test]
    fn test_different_share_combinations() {
        let prime = 97;
        let secret = 58u64;
        let shamir = Shamir::new(secret, 6, 4, prime);
        let shares = shamir.split();

        // Try different combinations of 4 shares
        assert_eq!(Shamir::reconstruct(&shares[1..5]).num, secret);
        assert_eq!(Shamir::reconstruct(&shares[2..6]).num, secret);
    }
}


