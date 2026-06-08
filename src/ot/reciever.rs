use std::marker::PhantomData;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::ristretto::RistrettoPoint;
use rand::rngs::OsRng;
use sha2::{Sha512, Digest};

pub struct Receiver<State> {
    scalar: Scalar,
    public_key: RistrettoPoint,
    chosen_bit: u8,  // ADD: Store chosen bit for later use
    received_spk: RistrettoPoint,  // ADD: Store sender's public key
    _state: PhantomData<State>,
}

pub struct FirstPhase;
pub struct SecondPhase {
    encrypted_tuple: RistrettoPoint,
}

impl Receiver<FirstPhase> {
    pub fn new(
        chosen_bit: u8,  // CHANGE: Remove &, u8 is Copy
        generator: &RistrettoPoint,
        received_spk: &RistrettoPoint,
    ) -> Self {
        // FIX: Use lowercase 'random', assign result
        let scalar = Scalar::random(&mut OsRng);
        
        // FIX: Move public_key outside if/else block
        let public_key = if chosen_bit == 0 {
            scalar * generator
        } else if chosen_bit == 1 {
            received_spk + (scalar * generator)
        } else {
            panic!("Chosen bit must be 0 or 1")
        };

        Self {
            scalar,
            public_key,
            chosen_bit,  // ADD: Store for later decryption
            received_spk: *received_spk,  // ADD: Store for later use
            _state: PhantomData,
        }
    }

    // FIX: Add return type
    pub fn send(&self) -> RistrettoPoint {
        self.public_key
    }

    // FIX: Return type should be Receiver<SecondPhase>
    pub fn receive(self, encrypted_tuple: RistrettoPoint) -> Receiver<SecondPhase> {
        Receiver {
            scalar: self.scalar,
            public_key: self.public_key,
            chosen_bit: self.chosen_bit,  // ADD: Carry forward
            received_spk: self.received_spk,  // ADD: Carry forward
            _state: PhantomData,
        }
    }
}

impl Receiver<SecondPhase> {
    fn hash_to_u64(&self, point: &RistrettoPoint) -> u64 {
        let mut hasher = Sha512::new();
        hasher.update(point.compress().as_bytes());
        let hash = hasher.finalize();
        u64::from_le_bytes(hash[0..8].try_into().unwrap())
    }

    // FIX: Add parameter type, return type, and fix logic
    pub fn decrypt(&self, encrypted_tuple: &(u64, u64)) -> u64 {
        // FIX: Use proper variable names and types
        let k_point = self.scalar * self.received_spk;
        let hashed_k = self.hash_to_u64(&k_point);
        
        // FIX: Access tuple elements correctly
        if self.chosen_bit == 0 {
            hashed_k ^ encrypted_tuple.0
        } else if self.chosen_bit == 1 {
            hashed_k ^ encrypted_tuple.1
        } else {
            panic!("Invalid chosen bit")
        }
    }
}
