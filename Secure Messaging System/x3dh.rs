// src/crypto/x3dh.rs
//! Extended Triple Diffie-Hellman (X3DH) Key Agreement
//! Used for initial session establishment with forward secrecy

use super::primitives::*;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::ZeroizeOnDrop;

/// X3DH prekey bundle published by a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreKeyBundle {
    pub identity_key: Vec<u8>,        // Long-term identity public key (Ed25519)
    pub signed_prekey: Vec<u8>,        // Medium-term signed prekey (X25519)
    pub prekey_signature: Vec<u8>,     // Signature of signed_prekey
    pub one_time_prekey: Option<Vec<u8>>, // Ephemeral one-time prekey (X25519)
}

/// X3DH session initialization result
#[derive(ZeroizeOnDrop)]
pub struct X3DHResult {
    pub shared_secret: SecureKey,
    pub associated_data: Vec<u8>,
    pub ephemeral_public: PublicKey,
}

/// X3DH initiator state
pub struct X3DHInitiator {
    identity_key: SigningKey,
    ephemeral_key: StaticSecret,
}

/// X3DH receiver state
pub struct X3DHReceiver {
    identity_key: SigningKey,
    signed_prekey: StaticSecret,
    signed_prekey_public: PublicKey,
    prekey_signature: Signature,
    one_time_prekey: Option<StaticSecret>,
}

impl X3DHInitiator {
    /// Create new X3DH initiator
    pub fn new(identity_key: SigningKey) -> Self {
        let ephemeral_key = StaticSecret::random_from_rng(&mut rand::thread_rng());
        Self {
            identity_key,
            ephemeral_key,
        }
    }

    /// Perform X3DH as initiator (Alice)
    pub fn initiate(
        &self,
        bundle: &PreKeyBundle,
    ) -> Result<X3DHResult, CryptoError> {
        // Verify the prekey bundle signature
        let remote_identity = VerifyingKey::from_bytes(
            &bundle.identity_key.as_slice().try_into()
                .map_err(|_| CryptoError::InvalidSignature)?,
        )
        .map_err(|_| CryptoError::InvalidSignature)?;

        let signature = Signature::from_slice(&bundle.prekey_signature)
            .map_err(|_| CryptoError::InvalidSignature)?;

        remote_identity
            .verify(&bundle.signed_prekey, &signature)
            .map_err(|_| CryptoError::InvalidSignature)?;

        // Parse remote keys
        let signed_prekey = PublicKey::from(
            <[u8; 32]>::try_from(bundle.signed_prekey.as_slice())
                .map_err(|_| CryptoError::InvalidKeySize)?,
        );

        let one_time_prekey = bundle.one_time_prekey.as_ref().map(|otpk| {
            PublicKey::from(
                <[u8; 32]>::try_from(otpk.as_slice())
                    .expect("Invalid one-time prekey size"),
            )
        });

        // Convert identity key to X25519 for DH
        let identity_private = StaticSecret::from(self.identity_key.to_bytes());
        let remote_identity_x25519 = self.ed25519_to_x25519_public(&remote_identity)?;

        // Perform DH operations
        let dh1 = dh(&identity_private, &signed_prekey);
        let dh2 = dh(&self.ephemeral_key, &remote_identity_x25519);
        let dh3 = dh(&self.ephemeral_key, &signed_prekey);

        // Concatenate DH outputs
        let mut dh_concat = Vec::with_capacity(32 * 4);
        dh_concat.extend_from_slice(dh1.as_bytes());
        dh_concat.extend_from_slice(dh2.as_bytes());
        dh_concat.extend_from_slice(dh3.as_bytes());

        // Include one-time prekey if available
        if let Some(otpk) = one_time_prekey {
            let dh4 = dh(&self.ephemeral_key, &otpk);
            dh_concat.extend_from_slice(dh4.as_bytes());
        }

        // Derive shared secret
        let shared_secret = kdf_hkdf(&dh_concat, b"", b"X3DH-SharedSecret")?;

        // Create associated data for authentication
        let mut ad = Vec::new();
        ad.extend_from_slice(&self.identity_key.verifying_key().to_bytes());
        ad.extend_from_slice(&bundle.identity_key);

        let ephemeral_public = PublicKey::from(&self.ephemeral_key);

        Ok(X3DHResult {
            shared_secret,
            associated_data: ad,
            ephemeral_public,
        })
    }

    /// Convert Ed25519 public key to X25519 (curve25519)
    fn ed25519_to_x25519_public(&self, ed_key: &VerifyingKey) -> Result<PublicKey, CryptoError> {
        // This is a simplified conversion - in production, use proper conversion
        // For now, we hash the Ed25519 key to get X25519-compatible bytes
        let hash = sha2::Sha256::digest(ed_key.as_bytes());
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash[..32]);
        Ok(PublicKey::from(bytes))
    }

    pub fn ephemeral_public(&self) -> PublicKey {
        PublicKey::from(&self.ephemeral_key)
    }
}

impl X3DHReceiver {
    /// Create new X3DH receiver with prekey bundle
    pub fn new(identity_key: SigningKey) -> Self {
        let signed_prekey = StaticSecret::random_from_rng(&mut rand::thread_rng());
        let signed_prekey_public = PublicKey::from(&signed_prekey);

        // Sign the prekey
        let prekey_signature = identity_key.sign(signed_prekey_public.as_bytes());

        Self {
            identity_key,
            signed_prekey,
            signed_prekey_public,
            prekey_signature,
            one_time_prekey: None,
        }
    }

    /// Add a one-time prekey
    pub fn add_one_time_prekey(&mut self) {
        self.one_time_prekey = Some(StaticSecret::random_from_rng(&mut rand::thread_rng()));
    }

    /// Generate prekey bundle for publication
    pub fn generate_bundle(&self) -> PreKeyBundle {
        PreKeyBundle {
            identity_key: self.identity_key.verifying_key().to_bytes().to_vec(),
            signed_prekey: self.signed_prekey_public.as_bytes().to_vec(),
            prekey_signature: self.prekey_signature.to_bytes().to_vec(),
            one_time_prekey: self
                .one_time_prekey
                .as_ref()
                .map(|otpk| PublicKey::from(otpk).as_bytes().to_vec()),
        }
    }

    /// Perform X3DH as receiver (Bob)
    pub fn receive(
        &mut self,
        initiator_identity: &VerifyingKey,
        ephemeral_public: &PublicKey,
    ) -> Result<X3DHResult, CryptoError> {
        // Convert identity keys to X25519
        let identity_private = StaticSecret::from(self.identity_key.to_bytes());
        let remote_identity_x25519 = self.ed25519_to_x25519_public(initiator_identity)?;

        // Perform DH operations (same as initiator but reversed)
        let dh1 = dh(&self.signed_prekey, &remote_identity_x25519);
        let dh2 = dh(&identity_private, ephemeral_public);
        let dh3 = dh(&self.signed_prekey, ephemeral_public);

        let mut dh_concat = Vec::with_capacity(32 * 4);
        dh_concat.extend_from_slice(dh1.as_bytes());
        dh_concat.extend_from_slice(dh2.as_bytes());
        dh_concat.extend_from_slice(dh3.as_bytes());

        // Include one-time prekey if used
        if let Some(otpk) = &self.one_time_prekey {
            let dh4 = dh(otpk, ephemeral_public);
            dh_concat.extend_from_slice(dh4.as_bytes());
            // Consume the one-time prekey
            self.one_time_prekey = None;
        }

        // Derive shared secret
        let shared_secret = kdf_hkdf(&dh_concat, b"", b"X3DH-SharedSecret")?;

        // Create associated data
        let mut ad = Vec::new();
        ad.extend_from_slice(initiator_identity.as_bytes());
        ad.extend_from_slice(&self.identity_key.verifying_key().to_bytes());

        Ok(X3DHResult {
            shared_secret,
            associated_data: ad,
            ephemeral_public: *ephemeral_public,
        })
    }

    fn ed25519_to_x25519_public(&self, ed_key: &VerifyingKey) -> Result<PublicKey, CryptoError> {
        let hash = sha2::Sha256::digest(ed_key.as_bytes());
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash[..32]);
        Ok(PublicKey::from(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x3dh_key_agreement() {
        // Alice and Bob generate identity keys
        let alice_identity = SigningKey::generate(&mut rand::thread_rng());
        let bob_identity = SigningKey::generate(&mut rand::thread_rng());

        // Bob creates prekey bundle
        let mut bob_receiver = X3DHReceiver::new(bob_identity);
        bob_receiver.add_one_time_prekey();
        let bob_bundle = bob_receiver.generate_bundle();

        // Alice initiates X3DH
        let alice_initiator = X3DHInitiator::new(alice_identity);
        let alice_result = alice_initiator.initiate(&bob_bundle).unwrap();

        // Bob receives X3DH
        let bob_result = bob_receiver
            .receive(
                &alice_identity.verifying_key(),
                &alice_result.ephemeral_public,
            )
            .unwrap();

        // Shared secrets should match
        assert_eq!(
            alice_result.shared_secret.as_bytes(),
            bob_result.shared_secret.as_bytes()
        );

        // Associated data should match
        assert_eq!(alice_result.associated_data, bob_result.associated_data);
    }
}
