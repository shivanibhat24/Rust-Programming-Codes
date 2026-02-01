// src/crypto/double_ratchet.rs
//! Double Ratchet Algorithm implementation (Signal Protocol)
//! Provides forward secrecy and post-compromise security

use super::primitives::*;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::ZeroizeOnDrop;

const MAX_SKIP: usize = 1000; // Maximum skipped messages

/// Double Ratchet state for a session
#[derive(ZeroizeOnDrop)]
pub struct RatchetState {
    // DH Ratchet
    dh_self: StaticSecret,
    dh_remote: Option<PublicKey>,

    // Root chain
    root_key: SecureKey,

    // Sending chain
    chain_key_send: Option<SecureKey>,
    send_counter: u32,

    // Receiving chain
    chain_key_recv: Option<SecureKey>,
    recv_counter: u32,
    prev_chain_length: u32,

    // Skipped message keys for out-of-order delivery
    skipped_keys: HashMap<(PublicKey, u32), MessageKeys>,

    // Identity for authentication
    identity_key: SigningKey,
    remote_identity: Option<VerifyingKey>,
}

/// Header for ratchet messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub dh_public: Vec<u8>,      // Current DH public key
    pub prev_chain_length: u32,  // Length of previous sending chain
    pub message_number: u32,     // Message number in current chain
}

/// Encrypted message with header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetMessage {
    pub header: MessageHeader,
    pub ciphertext: Vec<u8>,
    pub signature: Vec<u8>, // Ed25519 signature for authentication
}

impl RatchetState {
    /// Initialize as Alice (sender)
    pub fn init_alice(
        shared_secret: SecureKey,
        remote_public_key: PublicKey,
        identity_key: SigningKey,
        remote_identity: VerifyingKey,
    ) -> Self {
        let dh_self = StaticSecret::random_from_rng(&mut rand::thread_rng());
        let dh_remote = Some(remote_public_key);

        // Initial DH ratchet
        let dh_output = dh(&dh_self, &remote_public_key);
        let (root_key, chain_key_send) = Self::kdf_rk(&shared_secret, &dh_output);

        Self {
            dh_self,
            dh_remote,
            root_key,
            chain_key_send: Some(chain_key_send),
            send_counter: 0,
            chain_key_recv: None,
            recv_counter: 0,
            prev_chain_length: 0,
            skipped_keys: HashMap::new(),
            identity_key,
            remote_identity: Some(remote_identity),
        }
    }

    /// Initialize as Bob (receiver)
    pub fn init_bob(
        shared_secret: SecureKey,
        dh_keypair: StaticSecret,
        identity_key: SigningKey,
    ) -> Self {
        Self {
            dh_self: dh_keypair,
            dh_remote: None,
            root_key: shared_secret,
            chain_key_send: None,
            send_counter: 0,
            chain_key_recv: None,
            recv_counter: 0,
            prev_chain_length: 0,
            skipped_keys: HashMap::new(),
            identity_key,
            remote_identity: None,
        }
    }

    /// Root key derivation function
    fn kdf_rk(root_key: &SecureKey, dh_output: &SecureKey) -> (SecureKey, SecureKey) {
        let new_root = kdf_hkdf(
            dh_output.as_bytes(),
            root_key.as_bytes(),
            b"root-chain",
        )
        .expect("Root KDF failed");

        let chain_key = kdf_hkdf(
            dh_output.as_bytes(),
            root_key.as_bytes(),
            b"chain-key",
        )
        .expect("Chain KDF failed");

        (new_root, chain_key)
    }

    /// Perform DH ratchet step
    fn dh_ratchet(&mut self, remote_public: PublicKey) {
        // Update previous chain length
        self.prev_chain_length = self.send_counter;
        self.send_counter = 0;
        self.recv_counter = 0;

        // Store remote public key
        self.dh_remote = Some(remote_public);

        // Derive receiving chain
        let dh_output = dh(&self.dh_self, &remote_public);
        let (new_root, recv_chain) = Self::kdf_rk(&self.root_key, &dh_output);
        self.root_key = new_root;
        self.chain_key_recv = Some(recv_chain);

        // Generate new DH keypair
        self.dh_self = StaticSecret::random_from_rng(&mut rand::thread_rng());

        // Derive sending chain
        let dh_output = dh(&self.dh_self, &remote_public);
        let (new_root, send_chain) = Self::kdf_rk(&self.root_key, &dh_output);
        self.root_key = new_root;
        self.chain_key_send = Some(send_chain);
    }

    /// Skip message keys for out-of-order messages
    fn skip_message_keys(&mut self, until: u32) -> Result<(), CryptoError> {
        if self.recv_counter + MAX_SKIP as u32 < until {
            return Err(CryptoError::DecryptionFailed);
        }

        if let (Some(chain_key), Some(dh_remote)) = (&self.chain_key_recv, &self.dh_remote) {
            let mut current_chain = chain_key.clone();

            while self.recv_counter < until {
                let (msg_keys, new_chain) = kdf_message_keys(&current_chain)?;
                self.skipped_keys
                    .insert((*dh_remote, self.recv_counter), msg_keys);
                current_chain = new_chain;
                self.recv_counter += 1;
            }

            self.chain_key_recv = Some(current_chain);
        }

        Ok(())
    }

    /// Encrypt a message
    pub fn encrypt(&mut self, plaintext: &[u8], associated_data: &[u8]) -> Result<RatchetMessage, CryptoError> {
        let chain_key = self
            .chain_key_send
            .as_ref()
            .ok_or(CryptoError::EncryptionFailed)?;

        // Derive message keys
        let (msg_keys, new_chain) = kdf_message_keys(chain_key)?;
        self.chain_key_send = Some(new_chain);

        // Create header
        let header = MessageHeader {
            dh_public: PublicKey::from(&self.dh_self).as_bytes().to_vec(),
            prev_chain_length: self.prev_chain_length,
            message_number: self.send_counter,
        };

        // Encrypt message
        let header_bytes = bincode::serialize(&header).map_err(|_| CryptoError::EncryptionFailed)?;
        let mut ad = header_bytes.clone();
        ad.extend_from_slice(associated_data);

        let ciphertext = encrypt(
            &msg_keys.encryption_key,
            &msg_keys.iv,
            plaintext,
            &ad,
        )?;

        self.send_counter += 1;

        // Sign header + ciphertext
        let mut to_sign = header_bytes;
        to_sign.extend_from_slice(&ciphertext);
        let signature = self.identity_key.sign(&to_sign);

        Ok(RatchetMessage {
            header,
            ciphertext,
            signature: signature.to_bytes().to_vec(),
        })
    }

    /// Decrypt a message
    pub fn decrypt(
        &mut self,
        message: &RatchetMessage,
        associated_data: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        // Verify signature if we have remote identity
        if let Some(remote_identity) = &self.remote_identity {
            let header_bytes = bincode::serialize(&message.header)
                .map_err(|_| CryptoError::DecryptionFailed)?;
            let mut to_verify = header_bytes;
            to_verify.extend_from_slice(&message.ciphertext);

            let signature = Signature::from_slice(&message.signature)
                .map_err(|_| CryptoError::InvalidSignature)?;

            remote_identity
                .verify(&to_verify, &signature)
                .map_err(|_| CryptoError::InvalidSignature)?;
        }

        let remote_public = PublicKey::from(
            <[u8; 32]>::try_from(message.header.dh_public.as_slice())
                .map_err(|_| CryptoError::DecryptionFailed)?,
        );

        // Check if we have a skipped message key
        if let Some(msg_keys) = self.skipped_keys.remove(&(remote_public, message.header.message_number)) {
            let header_bytes = bincode::serialize(&message.header)
                .map_err(|_| CryptoError::DecryptionFailed)?;
            let mut ad = header_bytes;
            ad.extend_from_slice(associated_data);

            return decrypt(&msg_keys.encryption_key, &msg_keys.iv, &message.ciphertext, &ad);
        }

        // Check if we need to perform DH ratchet
        if Some(remote_public) != self.dh_remote {
            self.skip_message_keys(message.header.prev_chain_length)?;
            self.dh_ratchet(remote_public);
        }

        // Skip messages if needed
        self.skip_message_keys(message.header.message_number)?;

        // Decrypt message
        let chain_key = self
            .chain_key_recv
            .as_ref()
            .ok_or(CryptoError::DecryptionFailed)?;

        let (msg_keys, new_chain) = kdf_message_keys(chain_key)?;
        self.chain_key_recv = Some(new_chain);

        let header_bytes = bincode::serialize(&message.header)
            .map_err(|_| CryptoError::DecryptionFailed)?;
        let mut ad = header_bytes;
        ad.extend_from_slice(associated_data);

        let plaintext = decrypt(&msg_keys.encryption_key, &msg_keys.iv, &message.ciphertext, &ad)?;

        self.recv_counter += 1;

        Ok(plaintext)
    }

    /// Set remote identity key (for Bob after receiving first message)
    pub fn set_remote_identity(&mut self, remote_identity: VerifyingKey) {
        self.remote_identity = Some(remote_identity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_ratchet() {
        // Setup identities
        let alice_identity = SigningKey::generate(&mut rand::thread_rng());
        let bob_identity = SigningKey::generate(&mut rand::thread_rng());
        let alice_public_identity = alice_identity.verifying_key();
        let bob_public_identity = bob_identity.verifying_key();

        // Initial shared secret (from X3DH)
        let shared_secret = SecureKey::random(&mut rand::thread_rng());

        // Bob's initial DH keypair
        let bob_dh = StaticSecret::random_from_rng(&mut rand::thread_rng());
        let bob_dh_public = PublicKey::from(&bob_dh);

        // Initialize states
        let mut alice = RatchetState::init_alice(
            shared_secret.clone(),
            bob_dh_public,
            alice_identity,
            bob_public_identity,
        );
        let mut bob = RatchetState::init_bob(shared_secret, bob_dh, bob_identity);

        // Alice sends first message
        let msg1 = alice.encrypt(b"Hello Bob!", b"").unwrap();
        bob.set_remote_identity(alice_public_identity);
        let plain1 = bob.decrypt(&msg1, b"").unwrap();
        assert_eq!(plain1, b"Hello Bob!");

        // Bob replies
        let msg2 = bob.encrypt(b"Hello Alice!", b"").unwrap();
        let plain2 = alice.decrypt(&msg2, b"").unwrap();
        assert_eq!(plain2, b"Hello Alice!");

        // Multiple messages
        for i in 0..10 {
            let msg = alice.encrypt(format!("Message {}", i).as_bytes(), b"").unwrap();
            let plain = bob.decrypt(&msg, b"").unwrap();
            assert_eq!(plain, format!("Message {}", i).as_bytes());
        }
    }
}
