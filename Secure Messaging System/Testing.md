# Testing Guide

## Quick Start

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suites
```bash
# Cryptographic tests
cargo test crypto::

# Database tests  
cargo test db::

# API tests
cargo test api::

# Run with output
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test
```

### Run the Demo
```bash
cargo run --example client_demo
```

### API Integration Tests
```bash
# Start the server
cargo run &

# Run API tests
./test_api.sh

# Stop the server
pkill secure-messenger
```

## Unit Tests

### Cryptographic Primitives
Test vectors for encryption/decryption:
```bash
cargo test test_encrypt_decrypt
cargo test test_kdf_message_keys
```

### Double Ratchet
Test the ratchet algorithm:
```bash
cargo test test_double_ratchet
```

### X3DH
Test key agreement:
```bash
cargo test test_x3dh_key_agreement
```

## Integration Tests

### End-to-End Flow
The `client_demo.rs` example demonstrates:
1. Identity generation
2. X3DH key agreement
3. Double Ratchet initialization
4. Message encryption/decryption
5. Forward secrecy
6. Out-of-order delivery

Run it:
```bash
cargo run --example client_demo
```

Expected output:
```
🔐 Military-Grade Secure Messaging Demo
═══════════════════════════════════════════════════════════════

📋 SETUP PHASE
─────────────
✅ Alice generated long-term identity key
✅ Bob generated long-term identity key

🤝 X3DH KEY AGREEMENT PHASE
───────────────────────────
✅ Bob published prekey bundle
✅ Alice performed X3DH key agreement
✅ Bob completed X3DH key agreement
✅ Shared secret established (verified)

...
```

### API Testing

#### Manual Testing
```bash
# 1. Start server
cargo run

# 2. Register user
curl -X POST http://localhost:3000/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"pass123"}'

# 3. Login
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"pass123"}'

# 4. Get prekey bundle
curl -X POST http://localhost:3000/prekey-bundle \
  -H "Content-Type: application/json" \
  -d '{"username":"bob"}'
```

#### Automated Testing
```bash
./test_api.sh
```

## Performance Testing

### Benchmarks
```bash
# Run benchmarks (if implemented)
cargo bench

# Profile a specific test
cargo test test_double_ratchet --release -- --nocapture
```

### Load Testing
Use tools like `wrk` or `k6`:

```bash
# Install wrk
# Ubuntu: sudo apt-get install wrk
# macOS: brew install wrk

# Run load test
wrk -t4 -c100 -d30s http://localhost:3000/health
```

## Security Testing

### Static Analysis
```bash
# Run clippy for code quality
cargo clippy -- -D warnings

# Check for known vulnerabilities
cargo audit

# Check dependencies
cargo tree
```

### Fuzzing
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Create fuzz target (if not exists)
cargo fuzz init

# Run fuzzer
cargo fuzz run fuzz_target_1
```

### Memory Safety
```bash
# Run with address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo test

# Run with leak sanitizer  
RUSTFLAGS="-Z sanitizer=leak" cargo test

# Run with memory sanitizer
RUSTFLAGS="-Z sanitizer=memory" cargo test
```

## Test Coverage

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html
```

## Docker Testing

### Build Docker Image
```bash
docker build -t secure-messenger .
```

### Run in Docker
```bash
docker-compose up -d

# Run tests
./test_api.sh

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

## Continuous Integration

### GitHub Actions Example
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo audit
```

## Known Test Issues

### Rate Limiting Tests
Rate limiting tests may fail if tests run too quickly. Add delays:
```bash
sleep 1 && cargo test test_rate_limit
```

### Database Tests
Database tests create `messenger.db` in the project root. Clean up:
```bash
rm -f messenger.db messenger.db-*
```

## Test Data

### Test Vectors
Cryptographic test vectors are in the unit tests:
- `src/crypto/primitives.rs::tests`
- `src/crypto/double_ratchet.rs::tests`
- `src/crypto/x3dh.rs::tests`

### Mock Data
For integration tests, use:
```rust
let test_user = User {
    id: Uuid::new_v4().to_string(),
    username: "test_user".to_string(),
    // ...
};
```

## Debugging Tests

### Enable Debug Output
```bash
# Specific test with output
cargo test test_name -- --nocapture

# All tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Run Single Test
```bash
cargo test specific_test_name -- --exact
```

### Run Tests Sequentially
```bash
cargo test -- --test-threads=1
```

## Test Checklist

Before committing:
- [ ] All unit tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy`
- [ ] No security vulnerabilities: `cargo audit`
- [ ] Code formatted: `cargo fmt`
- [ ] Demo runs successfully: `cargo run --example client_demo`
- [ ] API tests pass: `./test_api.sh`
- [ ] Documentation builds: `cargo doc --no-deps`

## Troubleshooting

### "Address already in use"
Another process is using port 3000:
```bash
# Find process
lsof -i :3000

# Kill it
kill -9 <PID>
```

### Database locked
SQLite database locked by another process:
```bash
# Remove database files
rm -f messenger.db messenger.db-*

# Or wait for other process to finish
```

### Out of memory (fuzzing)
Reduce fuzzing corpus:
```bash
cargo fuzz run fuzz_target_1 -- -max_len=1024
```

## Additional Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo test documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [SQLx Testing Guide](https://github.com/launchbadge/sqlx#testing)
- [Axum Testing Examples](https://github.com/tokio-rs/axum/tree/main/examples)
