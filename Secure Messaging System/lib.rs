// src/lib.rs
//! Secure Messenger Library
//! 
//! Military-grade cryptographic messaging implementation

pub mod api;
pub mod crypto;
pub mod db;

// Re-export commonly used items
pub use crypto::{
    double_ratchet::{RatchetMessage, RatchetState},
    primitives::{CryptoError, SecureKey},
    x3dh::{PreKeyBundle, X3DHInitiator, X3DHReceiver, X3DHResult},
};
