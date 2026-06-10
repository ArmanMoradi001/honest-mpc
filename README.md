# Honest-mpc

A Rust library implementing the foundational cryptographic primitives for
Multi-Party Computation (MPC) from the ground up: finite field arithmetic,
Shamir's Secret Sharing, and Oblivious Transfer.

Built as an educational yet rigorous implementation, each layer depends
directly on the one below it, mirroring how real-world MPC systems are
structured.

```
┌─────────────────────────────────────────┐
│         Oblivious Transfer (OT)         │  protocol layer
├─────────────────────────────────────────┤
│       Shamir's Secret Sharing (SSS)     │  MPC primitive
├─────────────────────────────────────────┤
│        Finite Field Arithmetic (Fp)     │  math foundation
└─────────────────────────────────────────┘
```

---

## Modules

### `field` — Finite Field Arithmetic over Fp

Implements arithmetic in a prime field **Fp = Z/pZ** — the integers modulo a
prime p. Every operation wraps around modulo p, giving a closed algebraic
structure where addition, subtraction, multiplication, and division (except by
zero) are all well-defined.

Key properties:
- Validated prime modulus at construction time
- Binary exponentiation (`O(log n)`) for fast modular powers
- Modular inverse via Fermat's little theorem: `a⁻¹ = a^(p-2) mod p`
- Overflow-safe multiplication via `u128` widening
- Full operator overloading (`+`, `-`, `*`, `/`)
- Compile-time field separation — adding elements from different fields is
  caught at runtime with a clear error

```rust
let a = FieldElement::new(7, 13);
let b = FieldElement::new(10, 13);
let c = a + b; // 17 mod 13 = 4
let inv = a.inverse(); // a * inv ≡ 1 (mod 13)
```

---

### `shamir` — Shamir's Secret Sharing

Implements **(k, n) threshold secret sharing** — a secret is split into `n`
shares such that any `k` of them reconstruct the secret exactly, while any
`k-1` shares reveal nothing.

The construction uses polynomial interpolation over Fp:

1. Choose a random degree-(k-1) polynomial `f` where `f(0) = secret`
2. Each share is a point `(i, f(i))` on the polynomial
3. Reconstruction uses **Lagrange interpolation** to recover `f(0)`

```rust
let shamir = Shamir::new(secret, total_shares: 5, threshold: 3, prime: 97);
let shares = shamir.split();

// Any 3 shares reconstruct the secret
let recovered = Shamir::reconstruct(&shares[0..3]);
assert_eq!(recovered.value(), secret);
```

Security relies on the field being large enough that random polynomial
coefficients are indistinguishable from uniform — use a cryptographically
large prime in production.

---

### `ot` — 1-of-2 Oblivious Transfer (Chou-Orlandi)

Implements the **Chou-Orlandi OT protocol** (2015) over the Ristretto255
group (Curve25519). This is the industry-standard OT construction used in
production MPC frameworks.

**Protocol guarantees:**
- Receiver learns exactly one of two messages, `m_b` where `b ∈ {0, 1}`
- Receiver learns nothing about `m_{1-b}`
- Sender learns nothing about the receiver's choice bit `b`

**Protocol sketch:**

```
Sender generates:   scalar r,  public A = r·B
Receiver (bit b):   scalar s,  sends R = s·B  if b=0
                               sends R = A + s·B  if b=1

Sender derives:     k0 = H(r·R),  k1 = H(r·(R−A))
                    sends (e0, e1) = (k0 ⊕ m0, k1 ⊕ m1)

Receiver derives:   k_b = H(s·A)
                    recovers m_b = k_b ⊕ e_b
```

The implementation uses the **typestate pattern** to enforce protocol order
at compile time — it is structurally impossible to call `encrypt` before
the sender has received the receiver's public key.

```rust
// Full protocol in one call
let (_, decrypted) = OTSession::run(
    chosen_bit: 1,
    m0: 12345u64,
    m1: 67890u64,
);
assert_eq!(decrypted, 67890);
```

Or step-by-step with explicit state transitions:

```rust
// Sender setup
let sender = ObliviousTransferProtocol::initialize_sender();
let sender_pubkey = sender.send();

// Receiver setup (choice bit = 1)
let receiver = ObliviousTransferProtocol::initialize_receiver(1, sender_pubkey);
let receiver_pubkey = receiver.send();

// Sender encrypts — can only be called after receiving receiver's key
let (sender_ready, sender_msg) = ObliviousTransferProtocol::sender_encrypt(
    sender, receiver_pubkey, m0, m1
);

// Receiver decrypts — gets only m1
let receiver_phase2 = receiver.receive(sender_msg.sender_public_key);
let result = receiver_phase2.decrypt(&sender_msg.encrypted_messages);
```

---

## Design Principles

**Type-safe protocol enforcement.** The OT sender and receiver are modeled as
state machines using Rust's typestate pattern. Calling protocol steps out of
order is a compile error, not a runtime panic.

**No unsafe code.** The entire library is implemented in safe Rust.

**Separation of concerns.** Each layer is independent and testable in
isolation. The field module has no knowledge of Shamir or OT. Shamir depends
only on the field. OT is self-contained over Ristretto255.

**Cryptographic correctness over convenience.** Operations that can fail
(division by zero, invalid prime) are explicit. The OT protocol uses
`OsRng` for all randomness — no user-supplied seeds.

---

## Dependencies

| Crate | Purpose |
|---|---|
| `curve25519-dalek` | Ristretto255 group arithmetic for OT |
| `sha2` | SHA-512 for key derivation in OT |
| `rand` | `OsRng` cryptographically secure randomness |

---

## Security Notes

This library is intended for **educational and research purposes**. Before
using in any production system:

- The `FieldElement` implementation uses `u64` arithmetic, limiting safe
  primes to roughly 32 bits before multiplication overflow risk (mitigated
  by `u128` widening, but not suitable for production field sizes)
- OT messages are currently `u64` — production OT should transfer `[u8; 32]`
  keys and compose with a symmetric cipher for arbitrary-length messages
- No side-channel mitigations (constant-time operations) are implemented
- The Shamir implementation has not been audited

For production MPC, consider audited libraries such as
[`arkworks`](https://github.com/arkworks-rs),
[`threshold_crypto`](https://github.com/poanetwork/threshold_crypto), or
[`emp-toolkit`](https://github.com/emp-toolkit).

---

## References

- Chou, T., & Orlandi, C. (2015). *The Simplest Protocol for Oblivious Transfer*. LATINCRYPT 2015.
- Shamir, A. (1979). *How to share a secret*. Communications of the ACM.
- Ristretto group: https://ristretto.group
- Programming Bitcoin, Jimmy Song — field arithmetic foundations
