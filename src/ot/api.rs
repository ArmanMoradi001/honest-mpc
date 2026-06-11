use crate::ot::sender::{Sender, Uninitialized, Ready};
use crate::ot::receiver::{Receiver, FirstPhase, SecondPhase};
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

/// The sender's public key, sent to the receiver at the start of the protocol.
pub struct SenderPublicKey {
    pub key: RistrettoPoint,
}

/// The receiver's public key, sent back to the sender.
pub struct ReceiverPublicKey {
    pub key: RistrettoPoint,
}

/// The two encrypted messages produced by the sender.
pub struct EncryptedMessages {
    pub ciphertexts: ([u8; 32], [u8; 32]),
}

/// The decrypted result obtained by the receiver.
pub struct OTResult {
    pub decrypted: [u8; 32],
}

/// Step-by-step orchestrator for the Chou-Orlandi OT protocol.
///
/// Use this when you need explicit control over each protocol step —
/// for example, when sender and receiver are on separate machines and
/// messages are passed over a network.
pub struct ObliviousTransferProtocol;

impl ObliviousTransferProtocol {
    /// Step 1: Sender generates its keypair.
    pub fn initialize_sender() -> Sender<Uninitialized> {
        Sender::new(&RISTRETTO_BASEPOINT_POINT)
    }

    /// Step 2: Extract sender's public key to send to the receiver.
    pub fn sender_public_key(sender: &Sender<Uninitialized>) -> SenderPublicKey {
        SenderPublicKey { key: sender.send() }
    }

    /// Step 3: Receiver initializes with its choice bit and the sender's public key.
    pub fn initialize_receiver(
        chosen_bit: u8,
        sender_key: &SenderPublicKey,
    ) -> Receiver<FirstPhase> {
        Receiver::new(chosen_bit, &RISTRETTO_BASEPOINT_POINT, &sender_key.key)
    }

    /// Step 4: Extract receiver's public key to send back to the sender.
    pub fn receiver_public_key(receiver: &Receiver<FirstPhase>) -> ReceiverPublicKey {
        ReceiverPublicKey { key: receiver.send() }
    }

    /// Step 5: Sender receives the receiver's public key and encrypts both messages.
    pub fn sender_encrypt(
        sender: Sender<Uninitialized>,
        receiver_key: &ReceiverPublicKey,
        m0: &[u8; 32],
        m1: &[u8; 32],
    ) -> (Sender<Ready>, EncryptedMessages) {
        let sender_ready = sender.receive(receiver_key.key);
        let ciphertexts = sender_ready.encrypt(m0, m1);
        (sender_ready, EncryptedMessages { ciphertexts })
    }

    /// Step 6: Receiver advances to the decryption phase.
    pub fn receiver_advance(receiver: Receiver<FirstPhase>) -> Receiver<SecondPhase> {
        receiver.receive()
    }

    /// Step 7: Receiver decrypts the message corresponding to its chosen bit.
    pub fn receiver_decrypt(
        receiver: &Receiver<SecondPhase>,
        encrypted: &EncryptedMessages,
    ) -> OTResult {
        OTResult {
            decrypted: receiver.decrypt(&encrypted.ciphertexts),
        }
    }
}

/// High-level convenience API — runs the full protocol in a single call.
///
/// Both sender and receiver are simulated in the same process.
/// Use [`ObliviousTransferProtocol`] directly for distributed scenarios.
pub struct OTSession;

impl OTSession {
    /// Run the full 1-of-2 OT protocol.
    ///
    /// # Arguments
    /// - `chosen_bit`: `0` or `1` — which message the receiver wants
    /// - `m0`: the sender's first message (32 bytes)
    /// - `m1`: the sender's second message (32 bytes)
    ///
    /// # Returns
    /// [`OTResult`] containing the decrypted message for `chosen_bit`.
    ///
    /// # Example
    /// ```
    /// let m0 = b"this is the first  message!!!!!!";
    /// let m1 = b"this is the second message!!!!!!";
    /// let result = OTSession::run(1, m0, m1);
    /// assert_eq!(&result.decrypted, m1);
    /// ```
    pub fn run(chosen_bit: u8, m0: &[u8; 32], m1: &[u8; 32]) -> OTResult {
        let sender = ObliviousTransferProtocol::initialize_sender();
        let sender_key = ObliviousTransferProtocol::sender_public_key(&sender);

        let receiver = ObliviousTransferProtocol::initialize_receiver(chosen_bit, &sender_key);
        let receiver_key = ObliviousTransferProtocol::receiver_public_key(&receiver);

        let (_, encrypted) = ObliviousTransferProtocol::sender_encrypt(
            sender,
            &receiver_key,
            m0,
            m1,
        );

        let receiver_ready = ObliviousTransferProtocol::receiver_advance(receiver);
        ObliviousTransferProtocol::receiver_decrypt(&receiver_ready, &encrypted)
    }
}