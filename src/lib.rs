//! # mpc-primitives
//!
//! A Rust library implementing foundational cryptographic primitives for
//! Multi-Party Computation (MPC):
//!
//! - [`field`] — Finite field arithmetic over Fp
//! - [`shamir`] — Shamir's Secret Sharing (k-of-n threshold scheme)
//! - [`ot`] — 1-of-2 Oblivious Transfer (Chou-Orlandi protocol)
//!
//! ## Quick Start
//!
//! ```rust
//! use mpc_primitives::ot::api::OTSession;
//!
//! let m0 = b"this is the first  message!!!!!!";
//! let m1 = b"this is the second message!!!!!!";
//!
//! let result = OTSession::run(1, m0, m1);
//! assert_eq!(&result.decrypted, m1);
//! ```

pub mod field;
pub mod shamir;
pub mod ot;
pub mod error;
