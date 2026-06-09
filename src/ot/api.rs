// api.rs
use crate::ot::sender::Sender;
use crate::ot::receiver::Receiver;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

/// Message passed from receiver to sender
pub struct ReceiverMessage {
    pub receiver_public_key: RistrettoPoint,
}

/// Message passed from sender to receiver
pub struct SenderMessage {
    pub sender_public_key: RistrettoPoint,
    pub encrypted_messages: (u64, u64),
}

/// Oblivious Transfer Protocol Orchestrator
pub struct ObliviousTransferProtocol;

impl ObliviousTransferProtocol {
    /// Step 1: Initialize sender
    pub fn initialize_sender() -> Sender<crate::ot::sender::Uninitialized> {
        Sender::new(&RISTRETTO_BASEPOINT_POINT)
    }

    /// Step 2: Sender sends public key to receiver
    pub fn sender_send_pubkey(sender: &Sender<crate::ot::sender::Uninitialized>) -> ReceiverMessage {
        ReceiverMessage {
            receiver_public_key: sender.send(),
        }
    }

    /// Step 3: Initialize receiver with chosen bit
    pub fn initialize_receiver(
        chosen_bit: u8,
        sender_pubkey: RistrettoPoint,
    ) -> Receiver<crate::ot::receiver::FirstPhase> {
        Receiver::new(chosen_bit, &RISTRETTO_BASEPOINT_POINT, &sender_pubkey)
    }

    /// Step 4: Receiver sends public key back to sender
    pub fn receiver_send_pubkey(
        receiver: &Receiver<crate::ot::receiver::FirstPhase>,
    ) -> ReceiverMessage {
        ReceiverMessage {
            receiver_public_key: receiver.send(),
        }
    }

    /// Step 5: Sender receives receiver's public key and encrypts messages
    pub fn sender_encrypt(
        sender: Sender<crate::ot::sender::Uninitialized>,
        receiver_pubkey: RistrettoPoint,
        m0: u64,
        m1: u64,
    ) -> (Sender<crate::ot::sender::Ready>, SenderMessage) {
        let sender_ready = sender.receive(receiver_pubkey);
        let encrypted = sender_ready.encrypt(m0, m1);

        let message = SenderMessage {
            sender_public_key: sender_ready.public_key(),
            encrypted_messages: encrypted,
        };

        (sender_ready, message)
    }

    /// Step 6: Receiver transitions to second phase and receives encrypted messages
    pub fn receiver_receive(
        receiver: Receiver<crate::ot::receiver::FirstPhase>,
        encrypted_tuple: RistrettoPoint,
    ) -> Receiver<crate::ot::receiver::SecondPhase> {
        receiver.receive(encrypted_tuple)
    }

    /// Step 7: Receiver decrypts to get the message corresponding to chosen bit
    pub fn receiver_decrypt(
        receiver: &Receiver<crate::ot::receiver::SecondPhase>,
        encrypted_tuple: &(u64, u64),
    ) -> u64 {
        receiver.decrypt(encrypted_tuple)
    }
}

/// High-level convenience API
pub struct OTSession {
    pub sender: Option<Sender<crate::ot::sender::Ready>>,
    pub receiver: Option<Receiver<crate::ot::receiver::SecondPhase>>,
}

impl OTSession {
    pub fn new() -> Self {
        Self {
            sender: None,
            receiver: None,
        }
    }

    /// Run full OT protocol
    pub fn run(
        chosen_bit: u8,
        m0: u64,
        m1: u64,
    ) -> (u64, u64) {
        // Step 1: Sender initialization
        let sender = ObliviousTransferProtocol::initialize_sender();
        let sender_pubkey = sender.send();

        // Step 2: Receiver initialization
        let receiver = ObliviousTransferProtocol::initialize_receiver(chosen_bit, sender_pubkey);
        let receiver_pubkey = receiver.send();

        // Step 3: Sender encrypts
        let (sender_ready, sender_msg) = ObliviousTransferProtocol::sender_encrypt(
            sender,
            receiver_pubkey,
            m0,
            m1,
        );

        // Step 4: Receiver decrypts
        let receiver_phase2 = receiver.receive(sender_msg.sender_public_key);
        let decrypted = receiver_phase2.decrypt(&sender_msg.encrypted_messages);

        // Return both encrypted tuple and decrypted result for verification
        (sender_msg.encrypted_messages.0, decrypted)
    }
}
