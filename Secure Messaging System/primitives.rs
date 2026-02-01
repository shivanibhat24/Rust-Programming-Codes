// src/crypto/primitives.rs
//! Military-grade cryptographic primitives

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand_core::{CryptoRng, RngCore};
use sha2::Sha256;
use std::fmt;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Key size for symmetric encryption (256 bits)
pub const KEY_SIZE: usize = 32;
/// Nonce size for ChaCha20-Poly1305
pub const NONCE_SIZE: usize = 12;
/// Authentication tag size
pub const TAG_SIZE: usize = 16;

/// Secure key material that auto-zeroes on drop
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecureKey([u8; KEY_SIZE]);

impl SecureKey {
    pub fn new(bytes: [u8; KEY_SIZE]) -> Self {
        Self(bytes)
    }

    pub fn random<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let mut bytes = [0u8; KEY_SIZE];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
        if slice.len() != KEY_SIZE {
            return Err(CryptoError::InvalidKeySize);
        }
        let mut bytes = [0u8; KEY_SIZE];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }
}

impl fmt::Debug for SecureKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecureKey([REDACTED])")
    }
}

/// Message keys derived from chain keys
#[derive(ZeroizeOnDrop)]
pub struct MessageKeys {
    pub encryption_key: SecureKey,
    pub auth_key: SecureKey,
    pub iv: [u8; NONCE_SIZE],
}

/// Errors that can occur during cryptographic operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid key size")]
    InvalidKeySize,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Key derivation failed")]
    KeyDerivationFailed,
    #[error("Authentication failed")]
    AuthenticationFailed,
}

/// Perform HKDF key derivation
pub fn kdf_hkdf(
    input_key_material: &[u8],
    salt: &[u8],
    info: &[u8],
) -> Result<SecureKey, CryptoError> {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), input_key_material);
    let mut okm = [0u8; KEY_SIZE];
    hkdf.expand(info, &mut okm)
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    Ok(SecureKey::new(okm))
}

/// Derive message keys from chain key
pub fn kdf_message_keys(chain_key: &SecureKey) -> Result<(MessageKeys, SecureKey), CryptoError> {
    // Derive new chain key
    let mut mac = Hmac::<Sha256>::new_from_slice(chain_key.as_bytes())
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    mac.update(b"\x02");
    let chain_key_bytes = mac.finalize().into_bytes();
    let new_chain_key = SecureKey::from_slice(&chain_key_bytes)?;

    // Derive message keys
    let mut mac = Hmac::<Sha256>::new_from_slice(chain_key.as_bytes())
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    mac.update(b"\x01");
    let message_key_material = mac.finalize().into_bytes();

    // Split into encryption key, auth key, and IV
    let encryption_key = SecureKey::from_slice(&message_key_material[..32])?;
    
    // Derive auth key and IV from message key material
    let auth_key = kdf_hkdf(&message_key_material, b"auth", b"message-auth")?;
    
    let mut iv = [0u8; NONCE_SIZE];
    iv.copy_from_slice(&message_key_material[20..32]);

    let message_keys = MessageKeys {
        encryption_key,
        auth_key,
        iv,
    };

    Ok((message_keys, new_chain_key))
}

/// Perform Diffie-Hellman exchange
pub fn dh(secret: &StaticSecret, public: &PublicKey) -> SecureKey {
    let shared = secret.diffie_hellman(public);
    SecureKey::new(*shared.as_bytes())
}

/// Authenticated encryption with ChaCha20-Poly1305
pub fn encrypt(
    key: &SecureKey,
    nonce: &[u8; NONCE_SIZE],
    plaintext: &[u8],
    associated_data: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(key.as_bytes().into());
    let nonce = Nonce::from_slice(nonce);
    
    let payload = Payload {
        msg: plaintext,
        aad: associated_data,
    };

    cipher
        .encrypt(nonce, payload)
        .map_err(|_| CryptoError::EncryptionFailed)
}

/// Authenticated decryption with ChaCha20-Poly1305
pub fn decrypt(
    key: &SecureKey,
    nonce: &[u8; NONCE_SIZE],
    ciphertext: &[u8],
    associated_data: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(key.as_bytes().into());
    let nonce = Nonce::from_slice(nonce);
    
    let payload = Payload {
        msg: ciphertext,
        aad: associated_data,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Generate random bytes
pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = SecureKey::random(&mut rand::thread_rng());
        let nonce = random_bytes::<NONCE_SIZE>();
        let plaintext = b"Top secret military message";
        let aad = b"metadata";

        let ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();
        let decrypted = decrypt(&key, &nonce, &ciphertext, aad).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_kdf_message_keys() {
        let chain_key = SecureKey::random(&mut rand::thread_rng());
        let (msg_keys1, new_chain1) = kdf_message_keys(&chain_key).unwrap();
        let (msg_keys2, _) = kdf_message_keys(&new_chain1).unwrap();

        // Message keys should be different
        assert_ne!(
            msg_keys1.encryption_key.as_bytes(),
            msg_keys2.encryption_key.as_bytes()
        );
    }
}
