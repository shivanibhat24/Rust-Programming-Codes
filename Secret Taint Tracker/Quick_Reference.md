# 🔒 Secret Taint Tracker - Quick Reference

## Core Types

```rust
Tainted<T, Secret>  // Sensitive data - cannot leak
Tainted<T, Public>  // Safe data - can be logged/transmitted
```

## Creating Tainted Values

```rust
// Mark as secret
let password = Tainted::secret("hunter2".to_string());
let api_key = Tainted::secret(env::var("API_KEY")?);

// Mark as public
let user_id = Tainted::public(12345);
let username = Tainted::public("alice".to_string());
```

## Operations That Preserve Taint

```rust
// Map - preserves taint level
let doubled = secret.map(|x| x * 2);  // Still Secret

// Combine same taint level
let sum = secret1.combine_with(secret2, |a, b| a + b);  // Still Secret

// Arithmetic (Secret + Secret = Secret)
let total = secret1 + secret2;  // Still Secret
```

## Declassification (Audit Points)

```rust
// Explicit declassification - AUDIT THIS!
let public = secret.declassify();

// Access secret value in controlled context
secret.expose_secret(|value| {
    // Use value here
});

// Extract raw value - AUDIT THIS!
let raw = secret.into_secret();
```

## Sanitization

```rust
// Show only first/last N chars
let masked = Sanitizer::mask(&api_key, 4);
// "sk_l...2345"

// Show only length
let length = Sanitizer::redact_length(&password);
// "[REDACTED 16 chars]"

// Hash for comparison
let hash = Sanitizer::hash_secret(&token);
// "hash:a3f2b8c1d4e5"
```

## Safe Sinks (Type-Enforced)

```rust
// Logger - only accepts Public
let logger = Logger::new("APP");
logger.log(&public_value);   // ✅ OK
logger.log(&secret_value);   // ❌ Won't compile

// Network - only accepts Public
let network = NetworkSink::new("api.example.com");
network.send(&public_data);  // ✅ OK
network.send(&secret_data);  // ❌ Won't compile
```

## Taint Upgrading

```rust
// Public -> Secret (when needed)
let public = Tainted::public("data");
let secret = public.taint_secret();

// Secret + Public -> Secret
use secret_taint::combine_with_secret;
let result = combine_with_secret(secret, public, |s, p| {
    format!("{}{}", s, p)
});
```

## Common Patterns

### User Authentication

```rust
struct User {
    id: Tainted<u64, Public>,
    username: Tainted<String, Public>,
    password_hash: Tainted<String, Secret>,
}

impl User {
    fn authenticate(&self, input: &str) -> Tainted<bool, Public> {
        let valid = self.password_hash.expose_secret(|hash| {
            hash == input  // Use proper comparison in production!
        });
        Tainted::public(valid)
    }
}
```

### Payment Processing

```rust
struct Payment {
    amount: Tainted<f64, Public>,
    card_number: Tainted<String, Secret>,
}

impl Payment {
    fn get_receipt(&self) -> Tainted<String, Public> {
        let masked = Sanitizer::mask(&self.card_number, 4);
        self.amount.clone().combine_with(masked, |amt, card| {
            format!("${:.2} - Card: {}", amt, card)
        })
    }
}
```

### Configuration

```rust
struct Config {
    port: Tainted<u16, Public>,
    db_url: Tainted<String, Secret>,
}

impl Config {
    fn log_safe_info(&self, logger: &Logger) {
        logger.log(&self.port);  // ✅ OK
        
        // Extract non-sensitive parts
        let host = self.db_url.expose_secret(|url| {
            // Parse and extract host only
            Tainted::public(extract_host(url))
        });
        logger.log(&host);  // ✅ OK
    }
}
```

### HTTP Request Handling

```rust
struct Request {
    path: Tainted<String, Public>,
    body: Tainted<String, Public>,
    auth_token: Tainted<String, Secret>,
}

impl Request {
    fn log_request(&self, logger: &Logger) {
        logger.log(&self.path);
        logger.log(&self.body);
        
        // Log hashed token
        let token_hash = Sanitizer::hash_secret(&self.auth_token);
        logger.log(&token_hash);
    }
}
```

## What Won't Compile (Good!)

```rust
// ❌ Logging secrets
logger.log(&password);  // Error: expected Public, found Secret

// ❌ Sending secrets over network
network.send(&api_key);  // Error: expected Public, found Secret

// ❌ Displaying secrets
println!("{}", secret);  // Error: Secret doesn't implement Display

// ❌ Direct field access
let raw = secret.value;  // Error: field `value` is private

// ❌ Implicit declassification
let public: Tainted<_, Public> = secret;  // Error: mismatched types

// ❌ Mixed taint arithmetic
let sum = secret + public;  // Error: no implementation for this
```

## Zero-Cost Guarantee

```rust
// These have identical size in memory:
assert_eq!(
    std::mem::size_of::<String>(),
    std::mem::size_of::<Tainted<String, Secret>>()
);

// PhantomData<L> is zero-sized!
// All taint tracking is at compile-time only
```

## Audit Checklist

Search your codebase for these patterns:

```bash
# Find all declassifications
rg '\.declassify\(\)'

# Find all secret extractions
rg '\.into_secret\(\)'

# Find all controlled accesses
rg '\.expose_secret'

# Review each one for proper authorization and necessity
```

## Best Practices

✅ **DO:**
- Tag all sensitive data as `Secret` at entry points
- Use `Public` for non-sensitive data
- Sanitize before declassifying
- Audit all declassification points
- Use type-safe APIs throughout

❌ **DON'T:**
- Bypass the type system
- Declassify without sanitization
- Ignore compiler errors
- Use `unsafe` to extract values
- Forget to tag user input

## Type-Safe API Design

```rust
// ✅ Good - explicit taint requirements
fn process_public(data: &Tainted<String, Public>) { }
fn process_secret(data: &Tainted<String, Secret>) -> Tainted<bool, Public> { }

// ✅ Good - return type indicates safety
fn get_user_id() -> Tainted<u64, Public> { }
fn get_password_hash() -> Tainted<String, Secret> { }

// ❌ Bad - loses taint information
fn process(data: &str) { }  // Is this secret or public?
```

## Common Mistakes

### Mistake 1: Forgetting to tag at boundaries

```rust
// ❌ Bad - raw string
fn handle_password(password: String) { }

// ✅ Good - tainted at boundary
fn handle_password(password: Tainted<String, Secret>) { }
```

### Mistake 2: Declassifying too early

```rust
// ❌ Bad - loses protection
let password = Tainted::secret(input).declassify();
// Now it's Public everywhere!

// ✅ Good - keep as Secret until needed
let password = Tainted::secret(input);
// Process as Secret, only declassify specific outputs
```

### Mistake 3: Not using sanitization

```rust
// ❌ Bad - direct declassification
logger.log(&secret.declassify());

// ✅ Good - sanitize first
let safe = Sanitizer::mask(&secret, 4);
logger.log(&safe);
```

## Performance Notes

- **Zero runtime cost** - all checks at compile-time
- **No heap allocations** - PhantomData is zero-sized
- **No virtual dispatch** - static typing throughout
- **Optimizes to raw types** - compiler eliminates wrappers

## Testing

```rust
#[test]
fn test_cannot_log_secret() {
    let secret = Tainted::secret("password");
    let logger = Logger::new("TEST");
    
    // This won't compile:
    // logger.log(&secret);
    
    // Must sanitize:
    let safe = Sanitizer::redact_length(&secret);
    logger.log(&safe);
}
```

## Integration Examples

### With Actix-Web

```rust
struct AuthHeader(Tainted<String, Secret>);

impl FromRequest for AuthHeader {
    fn from_request(req: &HttpRequest) -> Self {
        let token = req.headers()
            .get("Authorization")
            .map(|h| h.to_str().unwrap())
            .unwrap_or("");
        AuthHeader(Tainted::secret(token.to_string()))
    }
}
```

### With Tokio

```rust
async fn authenticate(
    token: Tainted<String, Secret>
) -> Result<Tainted<UserId, Public>, Error> {
    let user_id = verify_token(&token).await?;
    Ok(Tainted::public(user_id))
}
```

### With Serde (careful!)

```rust
// Don't derive Serialize for Secret types!
#[derive(Serialize)]
struct UserResponse {
    id: u64,  // ✅ Raw public data OK
    username: String,  // ✅ Raw public data OK
    // Never include Tainted<_, Secret> fields here!
}
```

---

**Remember: The compiler is your friend. If it won't compile, you're probably trying to leak a secret!** 🔒
