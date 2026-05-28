use core::num;
use std::ops::{Add, Sub, Mul, Div};
use modulo::Mod;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldElement {
    num: u64,
    prime: u64,
}

impl FieldElement {

    pub fn new(num: u64, prime: u64) -> Self {
        if prime < 2 || !is_prime(prime) {
            panic!("{} is not a prime number", prime);
        }
        if num >= prime {
            panic!("Num {} is not in field range [0, {})", num, prime);
        }

        Self { num, prime }
    }

    pub fn zero(prime: u64) -> Self {
        Self::new(0, prime)
    }

    pub fn one(prime: u64) -> Self {
        Self::new(1, prime)
    }

    pub fn pow(&self, mut exponent: u32) -> Self {
        let mut base = self.num;
        let mut result = 1u64;
        let prime = self.prime;

        while exponent > 0 {
            if exponent & 1 == 1 {
                result = (result * base) % prime;
            }
            base = (base * base) % prime;
            exponent >>= 1;
        }

        Self { num: result, prime }
    }

    pub fn inverse(&self) -> Self {
        self.pow(self.prime as u32 - 2)
    }
}


impl Add for FieldElement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.prime, rhs.prime, "Cannot add elements from different fields");
        Self {
            num: (self.num + rhs.num) % self.prime,
            prime: self.prime,
        }
    }
}

impl Sub for FieldElement {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        assert_eq!(self.prime, rhs.prime, "Cannot subtract elements from different fields");
        Self {
            num: (self.num + self.prime - rhs.num) % self.prime,
            prime: self.prime,
        }
    }
}

impl Mul for FieldElement {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.prime, rhs.prime, "Cannot multiply elements from different fields");
        Self {
            num: (self.num * rhs.num) % self.prime,
            prime: self.prime,
        }
    }
}

impl Div for FieldElement {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        assert_eq!(self.prime, rhs.prime, "Cannot divide elements from different fields");
        assert_ne!(rhs.num, 0, "Division by zero in field");

        let inv = rhs.inverse();
        self * inv
    }
}


fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    let mut i = 5u64;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_element_creation() {
        let a = FieldElement::new(7, 13);
        let b = FieldElement::new(6, 13);
        assert_ne!(a, b);
        assert_eq!(a, a);
    }

    #[test]
    fn test_add() {
        let a = FieldElement::new(7, 13);
        let b = FieldElement::new(12, 13);
        let c = FieldElement::new(6, 13); // 7+12 = 19 ≡ 6 mod 13

        assert_eq!(a + b, c);
    }

    #[test]
    fn test_sub() {
        let a = FieldElement::new(5, 13);
        let b = FieldElement::new(8, 13);
        let c = FieldElement::new(10, 13); // 5-8 = -3 ≡ 10 mod 13

        assert_eq!(a - b, c);
    }

    #[test]
    fn test_mul() {
        let a = FieldElement::new(3, 13);
        let b = FieldElement::new(12, 13);
        let c = FieldElement::new(10, 13); // 36 ≡ 10 mod 13

        assert_eq!(a * b, c);
    }

    #[test]
    fn test_pow() {
        let a = FieldElement::new(3, 13);
        assert_eq!(a.pow(3), FieldElement::new(1, 13)); // 27 ≡ 1 mod 13
    }

    #[test]
    fn test_div() {
        let p = 19;

        let a = FieldElement::new(2, p);
        let b = FieldElement::new(7, p);
        assert_eq!(a / b, FieldElement::new(3, p));

        let a = FieldElement::new(7, p);
        let b = FieldElement::new(5, p);
        assert_eq!(a / b, FieldElement::new(9, p));
    }

    #[test]
    #[should_panic(expected = "different fields")]
    fn test_different_fields() {
        let a = FieldElement::new(5, 13);
        let b = FieldElement::new(5, 17);
        let _ = a + b;
    }
}