use std::marker::PhantomData;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::ristretto::RistrettoPoint;
use rand::rngs::OsRng;
use k256::sha2::{Sha512, Digest};

pub struct Sender<State> {
    scalar: Scalar,
    public: RistrettoPoint,
    receiver_point: Option<RistrettoPoint>,
    _state: PhantomData<State>,
}

pub struct Uninitialized;
pub struct Ready;

impl Sender<Uninitialized> {
    pub fn new(generator: &RistrettoPoint) -> Self {
        let scalar = Scalar::random(&mut OsRng);
        let public = scalar * generator;

        Self {
            scalar,
            public,
            receiver_point: None,
            _state: PhantomData,
        }
    }

    pub fn send(&self) -> RistrettoPoint {
        self.public
    }

    pub fn receive(self, receiver_point: RistrettoPoint) -> Sender<Ready> {
        Sender {
            scalar: self.scalar,
            public: self.public,
            receiver_point: Some(receiver_point),
            _state: PhantomData,
        }
    }
}

impl Sender<Ready> {
    pub fn public_key(&self) -> RistrettoPoint {
        self.public
    }

    pub fn encrypt(&self, m0: &[u8; 32], m1: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let r = self.receiver_point.unwrap();
    let k0 = self.derive_key(r);
    let k1 = self.derive_key(r - self.public);

    let c0 = self.xor_bytes(m0, &self.hash_to_bytes(&k0));
    let c1 = self.xor_bytes(m1, &self.hash_to_bytes(&k1));

    (c0, c1)
}

    fn hash_to_bytes(&self, point: &RistrettoPoint) -> [u8; 32] {
        let mut hasher = Sha512::new();
        hasher.update(point.compress().as_bytes());
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash[0..32]);
        result
    }

    fn derive_key(&self, point: RistrettoPoint) -> RistrettoPoint {
    self.scalar * point
}

    fn xor_bytes(&self, a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        let mut result = [0u8; 32];
        for i in 0..32 {
            result[i] = a[i] ^ b[i];
        }
        result
    }

}