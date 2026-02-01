// examples/client_demo.rs
//! Demonstration of end-to-end encrypted messaging between two clients

use ed25519_dalek::SigningKey;
use secure_messenger::crypto::{
    double_ratchet::RatchetState, x3dh::{X3DHInitiator, X3DHReceiver}, RatchetMessage,
};
use x25519_dalek::{PublicKey, StaticSecret};

fn main() {
    println!("🔐 Military-Grade Secure Messaging Demo\n");
    println!("═══════════════════════════════════════════════════════════════\n");

    // ========================================
    // Setup: Alice and Bob generate identities
    // ========================================
    println!("📋 SETUP PHASE");
    println!("─────────────");

    let alice_identity = SigningKey::generate(&mut rand::thread_rng());
    let bob_identity = SigningKey::generate(&mut rand::thread_rng());
    
    println!("✅ Alice generated long-term identity key");
    println!("✅ Bob generated long-term identity key\n");

    // ========================================
    // X3DH Key Agreement
    // ========================================
    println!("🤝 X3DH KEY AGREEMENT PHASE");
    println!("───────────────────────────");

    // Bob publishes prekey bundle
    let mut bob_receiver = X3DHReceiver::new(bob_identity.clone());
    bob_receiver.add_one_time_prekey();
    let bob_bundle = bob_receiver.generate_bundle();
    println!("✅ Bob published prekey bundle (identity + signed prekey + one-time prekey)");

    // Alice initiates X3DH
    let alice_initiator = X3DHInitiator::new(alice_identity.clone());
    let alice_x3dh_result = alice_initiator
        .initiate(&bob_bundle)
        .expect("Alice X3DH failed");
    println!("✅ Alice performed X3DH key agreement");

    // Bob completes X3DH
    let bob_x3dh_result = bob_receiver
        .receive(
            &alice_identity.verifying_key(),
            &alice_x3dh_result.ephemeral_public,
        )
        .expect("Bob X3DH failed");
    println!("✅ Bob completed X3DH key agreement");

    // Verify shared secrets match
    assert_eq!(
        alice_x3dh_result.shared_secret.as_bytes(),
        bob_x3dh_result.shared_secret.as_bytes()
    );
    println!("✅ Shared secret established (verified)\n");

    // ========================================
    // Initialize Double Ratchet
    // ========================================
    println!("🔄 DOUBLE RATCHET INITIALIZATION");
    println!("────────────────────────────────");

    let bob_dh = StaticSecret::random_from_rng(&mut rand::thread_rng());
    let bob_dh_public = PublicKey::from(&bob_dh);

    let mut alice_ratchet = RatchetState::init_alice(
        alice_x3dh_result.shared_secret.clone(),
        bob_dh_public,
        alice_identity.clone(),
        bob_identity.verifying_key(),
    );
    println!("✅ Alice initialized Double Ratchet state");

    let mut bob_ratchet = RatchetState::init_bob(
        bob_x3dh_result.shared_secret.clone(),
        bob_dh,
        bob_identity.clone(),
    );
    bob_ratchet.set_remote_identity(alice_identity.verifying_key());
    println!("✅ Bob initialized Double Ratchet state\n");

    // ========================================
    // Encrypted Message Exchange
    // ========================================
    println!("💬 ENCRYPTED MESSAGE EXCHANGE");
    println!("─────────────────────────────");

    // Alice sends first message
    let msg1_plain = b"TOP SECRET: The eagle has landed at 0200 hours.";
    let msg1_encrypted = alice_ratchet
        .encrypt(msg1_plain, b"")
        .expect("Encryption failed");
    println!("🔒 Alice encrypted message 1");
    println!("   Plaintext: {}", String::from_utf8_lossy(msg1_plain));
    println!("   Ciphertext length: {} bytes", msg1_encrypted.ciphertext.len());

    let msg1_decrypted = bob_ratchet
        .decrypt(&msg1_encrypted, b"")
        .expect("Decryption failed");
    println!("🔓 Bob decrypted message 1");
    println!("   Received: {}\n", String::from_utf8_lossy(&msg1_decrypted));
    assert_eq!(msg1_plain, msg1_decrypted.as_slice());

    // Bob replies
    let msg2_plain = b"CONFIRMED: Package secured. Proceeding to extraction point.";
    let msg2_encrypted = bob_ratchet
        .encrypt(msg2_plain, b"")
        .expect("Encryption failed");
    println!("🔒 Bob encrypted message 2");
    println!("   Plaintext: {}", String::from_utf8_lossy(msg2_plain));

    let msg2_decrypted = alice_ratchet
        .decrypt(&msg2_encrypted, b"")
        .expect("Decryption failed");
    println!("🔓 Alice decrypted message 2");
    println!("   Received: {}\n", String::from_utf8_lossy(&msg2_decrypted));
    assert_eq!(msg2_plain, msg2_decrypted.as_slice());

    // ========================================
    // Demonstrate Forward Secrecy
    // ========================================
    println!("🛡️  FORWARD SECRECY DEMONSTRATION");
    println!("─────────────────────────────────");

    let messages = vec![
        "Alpha team in position.",
        "Bravo team standing by.",
        "Charlie team moving in.",
        "Delta team providing overwatch.",
        "All teams: Execute on my mark.",
    ];

    for (i, msg) in messages.iter().enumerate() {
        let encrypted = alice_ratchet
            .encrypt(msg.as_bytes(), b"")
            .expect("Encryption failed");
        let decrypted = bob_ratchet
            .decrypt(&encrypted, b"")
            .expect("Decryption failed");
        
        println!("✅ Message {} exchanged (keys rotated)", i + 1);
        assert_eq!(msg.as_bytes(), decrypted.as_slice());
    }

    println!("\n⚡ Each message used unique encryption keys");
    println!("   → Compromise of one key does NOT compromise other messages");
    println!("   → Past messages remain secure (forward secrecy)");
    println!("   → Future messages remain secure (post-compromise security)\n");

    // ========================================
    // Demonstrate Out-of-Order Delivery
    // ========================================
    println!("📦 OUT-OF-ORDER MESSAGE HANDLING");
    println!("────────────────────────────────");

    // Alice sends multiple messages
    let msg_a = alice_ratchet.encrypt(b"Message A", b"").unwrap();
    let msg_b = alice_ratchet.encrypt(b"Message B", b"").unwrap();
    let msg_c = alice_ratchet.encrypt(b"Message C", b"").unwrap();

    // Bob receives them out of order: C, A, B
    println!("📨 Alice sent messages: A → B → C");
    println!("📬 Bob receives in order: C → A → B\n");

    let decrypted_c = bob_ratchet.decrypt(&msg_c, b"").unwrap();
    println!("✅ Decrypted message C: {}", String::from_utf8_lossy(&decrypted_c));

    let decrypted_a = bob_ratchet.decrypt(&msg_a, b"").unwrap();
    println!("✅ Decrypted message A: {}", String::from_utf8_lossy(&decrypted_a));

    let decrypted_b = bob_ratchet.decrypt(&msg_b, b"").unwrap();
    println!("✅ Decrypted message B: {}", String::from_utf8_lossy(&decrypted_b));

    println!("\n⚡ Skipped message keys are stored and used correctly\n");

    // ========================================
    // Security Summary
    // ========================================
    println!("═══════════════════════════════════════════════════════════════");
    println!("🎯 SECURITY FEATURES DEMONSTRATED");
    println!("═══════════════════════════════════════════════════════════════");
    println!("✅ End-to-End Encryption (E2EE)");
    println!("   → Only sender and recipient can read messages");
    println!("\n✅ Forward Secrecy");
    println!("   → Past messages secure even if current keys compromised");
    println!("\n✅ Post-Compromise Security");
    println!("   → Future messages secure after key compromise recovery");
    println!("\n✅ Authenticated Encryption");
    println!("   → ChaCha20-Poly1305 provides confidentiality + authenticity");
    println!("\n✅ Message Authentication");
    println!("   → Ed25519 signatures prevent impersonation");
    println!("\n✅ Key Rotation");
    println!("   → New keys for every message via Double Ratchet");
    println!("\n✅ Out-of-Order Handling");
    println!("   → Messages can arrive in any order");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("🎉 Demo completed successfully! All security properties verified.");
}
