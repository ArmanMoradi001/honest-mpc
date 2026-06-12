use std::fmt;
use std::ops::{Add, Sub, Mul, Div};
use crate::error::FieldError;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldElement {
    num: u64,
    prime: u64,
}

impl FieldElement {
    /// Construct a new field element.
    /// Returns `Err` if `prime` is not prime or `num` is out of range.
    pub fn new(num: u64, prime: u64) -> Result<Self, FieldError> {
        if prime < 2 || !is_prime(prime) {
            return Err(FieldError::NotPrime(prime));
        }
        if num >= prime {
            return Err(FieldError::OutOfRange { value: num, prime });
        }
        Ok(Self { num, prime })
    }

    /// Returns the zero element of the field.
    pub fn zero(prime: u64) -> Result<Self, FieldError> {
        Self::new(0, prime)
    }

    /// Returns the multiplicative identity of the field.
    pub fn one(prime: u64) -> Result<Self, FieldError> {
        Self::new(1, prime)
    }

    /// Returns the raw value of this element.
    pub fn value(&self) -> u64 {
        self.num
    }

    /// Returns the prime modulus of this field.
    pub fn prime(&self) -> u64 {
        self.prime
    }

    /// Fast binary exponentiation: computes self^exponent mod prime.
    pub fn pow(&self, mut exponent: u64) -> Self {
        let mut base = self.num;
        let mut result = 1u64;
        let prime = self.prime;

        while exponent > 0 {
            if exponent & 1 == 1 {
                result = ((result as u128 * base as u128) % prime as u128) as u64;
            }
            base = ((base as u128 * base as u128) % prime as u128) as u64;
            exponent >>= 1;
        }

        Self { num: result, prime }
    }

    /// Modular inverse via Fermat's little theorem: self^(p-2) mod p.
    /// Returns `Err` if self is zero (no inverse exists).
    pub fn inverse(&self) -> Result<Self, FieldError> {
        if self.num == 0 {
            return Err(FieldError::DivisionByZero);
        }
        Ok(self.pow(self.prime - 2))
    }
}

impl fmt::Display for FieldElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (mod {})", self.num, self.prime)
    }
}

impl Add for FieldElement {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        assert_eq!(
            self.prime, rhs.prime,
            "Cannot add elements from different fields ({} vs {})",
            self.prime, rhs.prime
        );
        Self {
            num: (self.num + rhs.num) % self.prime,
            prime: self.prime,
        }
    }
}

impl Sub for FieldElement {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        assert_eq!(
            self.prime, rhs.prime,
            "Cannot subtract elements from different fields ({} vs {})",
            self.prime, rhs.prime
        );
        Self {
            num: (self.num + self.prime - rhs.num) % self.prime,
            prime: self.prime,
        }
    }
}

impl Mul for FieldElement {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        assert_eq!(
            self.prime, rhs.prime,
            "Cannot multiply elements from different fields ({} vs {})",
            self.prime, rhs.prime
        );
        Self {
            num: ((self.num as u128 * rhs.num as u128) % self.prime as u128) as u64,
            prime: self.prime,
        }
    }
}

/// Division panics on zero divisor — use `inverse()` directly if you need
/// a `Result`-returning path.
impl Div for FieldElement {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        assert_eq!(
            self.prime, rhs.prime,
            "Cannot divide elements from different fields ({} vs {})",
            self.prime, rhs.prime
        );
        assert_ne!(rhs.num, 0, "Division by zero in field");
        self * rhs.inverse().unwrap()
    }
}

pub(crate) fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 || n == 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut i = 5u64;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 { return false; }
        i += 6;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction_valid() {
        let a = FieldElement::new(7, 13).unwrap();
        assert_eq!(a.value(), 7);
        assert_eq!(a.prime(), 13);
    }

    #[test]
    fn test_construction_invalid_prime() {
        assert!(matches!(
            FieldElement::new(3, 4),
            Err(FieldError::NotPrime(4))
        ));
    }

    #[test]
    fn test_construction_out_of_range() {
        assert!(matches!(
            FieldElement::new(13, 13),
            Err(FieldError::OutOfRange { value: 13, prime: 13 })
        ));
    }

    #[test]
    fn test_add() {
        let a = FieldElement::new(7, 13).unwrap();
        let b = FieldElement::new(12, 13).unwrap();
        let c = FieldElement::new(6, 13).unwrap(); // 19 mod 13 = 6
        assert_eq!(a + b, c);
    }

    #[test]
    fn test_sub_with_wraparound() {
        let a = FieldElement::new(5, 13).unwrap();
        let b = FieldElement::new(8, 13).unwrap();
        let c = FieldElement::new(10, 13).unwrap(); // -3 mod 13 = 10
        assert_eq!(a - b, c);
    }

    #[test]
    fn test_mul() {
        let a = FieldElement::new(3, 13).unwrap();
        let b = FieldElement::new(12, 13).unwrap();
        let c = FieldElement::new(10, 13).unwrap(); // 36 mod 13 = 10
        assert_eq!(a * b, c);
    }

    #[test]
    fn test_pow() {
        let a = FieldElement::new(3, 13).unwrap();
        assert_eq!(a.pow(3), FieldElement::new(1, 13).unwrap()); // 27 mod 13 = 1
    }

    #[test]
    fn test_inverse_correctness() {
        let a = FieldElement::new(7, 13).unwrap();
        let one = FieldElement::new(1, 13).unwrap();
        assert_eq!(a * a.inverse().unwrap(), one);
    }

    #[test]
    fn test_inverse_of_zero_is_err() {
        let a = FieldElement::new(0, 13).unwrap();
        assert!(matches!(a.inverse(), Err(FieldError::DivisionByZero)));
    }

    #[test]
    fn test_div() {
        let a = FieldElement::new(2, 19).unwrap();
        let b = FieldElement::new(7, 19).unwrap();
        assert_eq!(a / b, FieldElement::new(3, 19).unwrap());
    }

    #[test]
    fn test_additive_identity() {
        let a = FieldElement::new(9, 13).unwrap();
        let zero = FieldElement::zero(13).unwrap();
        assert_eq!(a + zero, a);
    }

    #[test]
    fn test_multiplicative_identity() {
        let a = FieldElement::new(9, 13).unwrap();
        let one = FieldElement::one(13).unwrap();
        assert_eq!(a * one, a);
    }

    #[test]
    #[should_panic(expected = "different fields")]
    fn test_field_mismatch_panics() {
        let a = FieldElement::new(5, 13).unwrap();
        let b = FieldElement::new(5, 17).unwrap();
        let _ = a + b;
    }
}