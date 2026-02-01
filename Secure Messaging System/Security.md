# Security Architecture Documentation

## Overview
This document describes the security architecture of the secure messaging backend.

## Threat Model

### Assets
1. **Message Content** - The plaintext messages
2. **User Identities** - Long-term identity keys
3. **Session Keys** - Ephemeral encryption keys
4. **Metadata** - Communication patterns, timestamps

### Threats
1. **Passive Adversary** - Eavesdropping on network traffic
2. **Active Adversary** - MITM attacks, message tampering
3. **Compromised Server** - Server breach exposing stored data
4. **Compromised Client** - Client device compromise
5. **Insider Threat** - Malicious system administrators

### Security Goals
1. **Confidentiality** - Messages unreadable by unauthorized parties
2. **Integrity** - Messages cannot be tampered with
3. **Authentication** - Verify sender identity
4. **Forward Secrecy** - Past messages secure if current keys compromised
5. **Post-Compromise Security** - Future messages secure after recovery
6. **Deniability** - Messages cannot be proven to third parties

## Cryptographic Protocols

### X3DH (Extended Triple Diffie-Hellman)
Used for initial key agreement between parties who haven't communicated before.

**Security Properties:**
- Mutual authentication
- Forward secrecy
- Key compromise impersonation (KCI) resistance
- Unknown key-share (UKS) resistance

**Protocol Flow:**
1. Bob publishes prekey bundle (IKb, SPKb, Sig(SPKb), OPKb)
2. Alice fetches bundle and performs DH operations
3. Alice derives shared secret SK
4. Bob receives Alice's ephemeral key and derives same SK

**Threat Mitigation:**
- MITM: Prevented by signature verification
- Replay: One-time prekeys consumed after use
- Impersonation: Identity keys verified

### Double Ratchet Algorithm
Provides forward secrecy and post-compromise security for ongoing communication.

**Components:**
1. **DH Ratchet** - Generates new DH key pairs for each reply
2. **Symmetric Ratchet** - Derives message keys from chain keys
3. **Root Key** - Derives new chain keys after DH ratchet

**Security Properties:**
- Forward secrecy: Old keys deleted, can't decrypt past messages
- Post-compromise security: DH ratchet establishes new shared secret
- Out-of-order resilience: Skipped message keys stored temporarily

**Key Derivation:**
```
Root KDF:
  RK, CK = KDF(RK, DH_output)

Chain KDF:
  MK = KDF(CK, 0x01)
  CK' = KDF(CK, 0x02)
```

### ChaCha20-Poly1305 AEAD
Authenticated encryption with associated data.

**Parameters:**
- Key: 256 bits
- Nonce: 96 bits (unique per message)
- Tag: 128 bits (authenticity guarantee)

**Security Properties:**
- IND-CPA: Indistinguishable under chosen plaintext attack
- INT-CTXT: Integrity of ciphertexts
- Nonce-misuse resistance: Different nonces derived for each message

**Usage:**
```
Ciphertext || Tag = ChaCha20-Poly1305.Encrypt(
    key=message_key,
    nonce=derived_iv,
    plaintext=message,
    aad=header
)
```

## Implementation Security

### Memory Safety
- **Zeroize**: All sensitive key material zeroed on drop
- **Constant-time**: Cryptographic operations resistant to timing attacks
- **No copies**: Key material passed by reference when possible

### Randomness
- **CSPRNG**: All random values from cryptographically secure RNG
- **OS entropy**: Relies on OS random number generator
- **Nonce generation**: Unique nonces derived from KDF

### Key Management
- **Key Lifecycle**:
  1. Generation: From CSPRNG
  2. Storage: Encrypted in memory, zeroed on drop
  3. Usage: Passed securely to crypto functions
  4. Deletion: Explicit zeroization

- **Key Rotation**:
  - Message keys: Every message
  - Chain keys: Every message
  - DH keys: Every reply (DH ratchet)
  - Identity keys: Long-term (user-managed)

### Database Security
- **Encryption at Rest**: Should be enabled in production
- **Access Control**: Database access restricted
- **Audit Logging**: All database operations logged
- **SQL Injection**: Prevented by parameterized queries (SQLx)

### Network Security
- **TLS 1.3**: Required for all connections in production
- **Certificate Pinning**: Recommended for mobile clients
- **HSTS**: HTTP Strict Transport Security enabled
- **Security Headers**: CSP, X-Frame-Options, etc.

## Attack Scenarios & Mitigations

### Scenario 1: Network Eavesdropping
**Attack**: Adversary captures all network traffic
**Mitigation**: 
- End-to-end encryption (E2EE)
- TLS 1.3 for transport security
- Forward secrecy ensures past messages secure

### Scenario 2: Server Compromise
**Attack**: Attacker gains access to server database
**Impact**: 
- Cannot read message content (E2EE)
- Can see metadata (users, timestamps)
- Can see encrypted ciphertexts (useless without keys)
**Mitigation**:
- Client-side encryption
- Ephemeral keys stored client-side only
- Database encryption at rest

### Scenario 3: Active MITM
**Attack**: Adversary intercepts and modifies messages
**Mitigation**:
- Message authentication (Poly1305 MAC)
- Digital signatures (Ed25519)
- TLS certificate verification

### Scenario 4: Key Compromise
**Attack**: Attacker steals current encryption keys
**Impact**:
- Can decrypt current/future messages until DH ratchet
- Cannot decrypt past messages (forward secrecy)
**Mitigation**:
- Forward secrecy prevents past decryption
- Post-compromise security recovers via DH ratchet
- Regular key rotation limits exposure window

### Scenario 5: Replay Attack
**Attack**: Adversary replays old messages
**Mitigation**:
- Message numbering in header
- Nonces prevent replay
- Signatures include message number

### Scenario 6: Side-Channel Attacks
**Attack**: Timing analysis, power analysis
**Mitigation**:
- Constant-time cryptographic operations
- Subtle crate for constant-time comparisons
- Memory-hard password hashing (Argon2)

## Compliance & Standards

### Standards Implemented
- **NIST SP 800-56A**: Key establishment using DH
- **NIST SP 800-108**: Key derivation functions
- **RFC 7539**: ChaCha20-Poly1305 AEAD
- **RFC 8032**: Edwards-curve signatures (Ed25519)
- **Signal Protocol**: Double Ratchet specification

### Best Practices
- OWASP Top 10 mitigations
- Principle of least privilege
- Defense in depth
- Secure by default configuration
- Explicit security boundaries

## Security Monitoring

### Audit Logging
All security-relevant events logged:
- User registration/login
- Failed authentication attempts
- Message sending/receiving
- Key exchanges
- Administrative actions

### Rate Limiting
- 100 requests/minute per endpoint
- Prevents brute force attacks
- DoS mitigation

### Intrusion Detection
Monitor for:
- Unusual login patterns
- Excessive failed authentication
- Anomalous messaging volumes
- Database access patterns

## Security Testing

### Recommended Tests
1. **Penetration Testing**
   - Network attacks
   - API fuzzing
   - SQL injection attempts

2. **Cryptographic Testing**
   - Known-answer tests (KAT)
   - Randomness testing (NIST suite)
   - Side-channel analysis

3. **Code Audit**
   - Static analysis (clippy, cargo-audit)
   - Dependency scanning
   - Manual code review

4. **Integration Testing**
   - End-to-end message flow
   - Key rotation scenarios
   - Error handling

## Incident Response

### Compromise Procedure
1. **Detection**: Monitor audit logs, alerts
2. **Containment**: Isolate affected systems
3. **Investigation**: Determine scope of breach
4. **Recovery**: Rotate all keys, revoke sessions
5. **Post-mortem**: Document and improve

### Key Rotation
In case of suspected compromise:
1. Generate new identity keys
2. Publish new prekey bundles
3. Invalidate old sessions
4. Notify affected users
5. Establish new sessions with fresh keys

## Future Enhancements

### Recommended Additions
1. **Message Padding** - Hide message length
2. **Sealed Sender** - Enhanced metadata protection  
3. **Multi-device Support** - Sesame algorithm
4. **Group Messaging** - Sender Keys protocol
5. **Disappearing Messages** - Automatic deletion
6. **Screenshot Detection** - Client-side protection
7. **Backup Encryption** - Secure message history

### Research Areas
- Post-quantum cryptography integration
- Zero-knowledge proofs for authentication
- Verifiable deletion guarantees
- Traffic analysis resistance

## References

1. Signal Protocol Specification
2. "The Double Ratchet Algorithm" - Perrin & Marlinspike
3. "The X3DH Key Agreement Protocol" - Perrin
4. RFC 7539 - ChaCha20 and Poly1305
5. RFC 8032 - Edwards-Curve Digital Signature Algorithm
6. NIST Special Publications on Cryptography

---

**Document Version**: 1.0  
**Last Updated**: 2025-02-01  
**Classification**: Internal Use
