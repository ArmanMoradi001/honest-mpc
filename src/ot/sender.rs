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

    pub fn encrypt(&self, m0: u64, m1: u64) -> (u64, u64) {
        let r = self.receiver_point.unwrap();
        let k0 = self.derive_key(r);
        let k1 = self.derive_key(r - self.public);

        let c0 = m0 ^ self.hash_to_u64(&k0);
        let c1 = m1 ^ self.hash_to_u64(&k1);

        (c0, c1)
    }

    fn derive_key(&self, point: RistrettoPoint) -> RistrettoPoint {
        self.scalar * point
    }

    fn hash_to_u64(&self, point: &RistrettoPoint) -> u64 {
        let mut hasher = Sha512::new();
        hasher.update(point.compress().as_bytes());
        let hash = hasher.finalize();
        u64::from_le_bytes(hash[0..8].try_into().unwrap())
    }
}