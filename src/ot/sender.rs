use std::marker::PhantomData;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::ristretto::RistrettoPoint;
use rand::rngs::OsRng;
use sha2::{Sha512, Digest};

pub struct Sender<State> {
    scalar: Scalar,
    public: RistrettoPoint,
    _state: PhantomData<State>,
}

pub struct Uninitialized;
pub struct Ready {
    receiver_point: RistrettoPoint,  // ADD: Define this field
}

impl Sender<Uninitialized> {
    pub fn new(generator: &RistrettoPoint) -> Self {
        let scalar = Scalar::random(&mut OsRng);
        let public = scalar * generator;

        Self {
            scalar,
            public,
            _state: PhantomData,
        }
    }

    // FIX: Add return type
    pub fn send(&self) -> RistrettoPoint {
        self.public
    }

    // FIX: Return correct type, don't wrap in Some()
    pub fn receive(self, receiver_point: RistrettoPoint) -> Sender<Ready> {
        Sender {
            scalar: self.scalar,
            public: self.public,
            receiver_point,  // FIX: Direct assignment, not Some()
            _state: PhantomData,
        }
    }
}

// FIX: Add generic parameter to properly define Ready struct
impl Sender<Ready> {
    pub fn encrypt(&self, m0: u64, m1: u64) -> (u64, u64) {
        let k0 = self.derive_key(self.receiver_point);
        let k1 = self.derive_key(self.receiver_point - self.public);

        let c0 = m0 ^ self.hash_to_u64(&k0);
        let c1 = m1 ^ self.hash_to_u64(&k1);

        (c0, c1)
    }

    fn derive_key(&self, point: RistrettoPoint) -> RistrettoPoint {
        point * self.scalar
    }

    fn hash_to_u64(&self, point: &RistrettoPoint) -> u64 {
        let mut hasher = Sha512::new();
        hasher.update(point.compress().as_bytes());
        let hash = hasher.finalize();
        u64::from_le_bytes(hash[0..8].try_into().unwrap())
    }

    pub fn public_key(&self) -> RistrettoPoint {
        self.public
    }
}
