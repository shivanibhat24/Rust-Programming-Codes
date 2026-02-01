//! Advanced patterns for secret taint tracking
//! 
//! Demonstrates:
//! - Integration with async/await
//! - Middleware patterns
//! - Secret rotation
//! - Audit logging
//! - Type-safe configuration

use secret_taint::{Tainted, Secret, Public, Logger, Sanitizer};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration with mixed sensitivity levels
#[derive(Clone)]
struct AppConfig {
    // Public configuration
    app_name: Tainted<String, Public>,
    port: Tainted<u16, Public>,
    debug_mode: Tainted<bool, Public>,
    
    // Secret configuration
    database_url: Tainted<String, Secret>,
    jwt_secret: Tainted<String, Secret>,
    encryption_key: Tainted<Vec<u8>, Secret>,
}

impl AppConfig {
    fn new() -> Self {
        Self {
            app_name: Tainted::public("MyApp".to_string()),
            port: Tainted::public(8080),
            debug_mode: Tainted::public(false),
            database_url: Tainted::secret("postgres://user:pass@localhost/db".to_string()),
            jwt_secret: Tainted::secret("super_secret_jwt_key".to_string()),
            encryption_key: Tainted::secret(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        }
    }

    /// Log only safe configuration values
    fn log_safe_config(&self, logger: &Logger) {
        logger.log(&self.app_name.clone().map(|n| format!("App: {}", n)));
        logger.log(&self.port.clone().map(|p| format!("Port: {}", p)));
        logger.log(&self.debug_mode.clone().map(|d| format!("Debug: {}", d)));
        
        // Log sanitized versions of secrets
        let db_info = self.database_url.expose_secret(|url| {
            if let Some(at_idx) = url.rfind('@') {
                Tainted::public(format!("DB Host: {}", &url[at_idx + 1..]))
            } else {
                Tainted::public("DB Host: [unknown]".to_string())
            }
        });
        logger.log(&db_info);
    }
}

/// Audit trail for declassification events
struct AuditLog {
    entries: Vec<String>,
}

impl AuditLog {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn record_declassification(&mut self, context: &str, reason: &str) {
        let entry = format!(
            "[AUDIT] Declassification at {}: {}",
            context, reason
        );
        self.entries.push(entry.clone());
        println!("{}", entry);
    }

    fn get_entries(&self) -> &[String] {
        &self.entries
    }
}

/// Token with automatic rotation
struct RotatingToken {
    current: Tainted<String, Secret>,
    generation: u64,
}

impl RotatingToken {
    fn new(initial: String) -> Self {
        Self {
            current: Tainted::secret(initial),
            generation: 1,
        }
    }

    fn rotate(&mut self, new_token: String) {
        self.current = Tainted::secret(new_token);
        self.generation += 1;
        println!("🔄 Token rotated to generation {}", self.generation);
    }

    fn get_generation(&self) -> Tainted<u64, Public> {
        Tainted::public(self.generation)
    }

    fn get_token_hash(&self) -> Tainted<String, Public> {
        Sanitizer::hash_secret(&self.current)
    }
}

/// Request context with taint-aware data
struct RequestContext {
    request_id: Tainted<String, Public>,
    user_id: Tainted<u64, Public>,
    session_token: Tainted<String, Secret>,
    ip_address: Tainted<String, Public>,
}

impl RequestContext {
    fn log_request(&self, logger: &Logger) {
        logger.log(&self.request_id.clone().map(|id| format!("Request ID: {}", id)));
        logger.log(&self.user_id.clone().map(|uid| format!("User ID: {}", uid)));
        logger.log(&self.ip_address.clone().map(|ip| format!("IP: {}", ip)));
        
        // Log hashed session token
        let token_hash = Sanitizer::hash_secret(&self.session_token);
        logger.log(&token_hash.map(|h| format!("Session: {}", h)));
    }
}

/// Middleware that ensures no secrets leak to response headers
struct SecurityMiddleware;

impl SecurityMiddleware {
    fn validate_response_headers(headers: &HashMap<String, Tainted<String, Public>>) -> bool {
        // All headers must be Public - enforced at compile time!
        // This function cannot even accept Tainted<String, Secret>
        println!("✅ All {} response headers are safe", headers.len());
        true
    }
}

/// Secret store with key derivation
struct SecretStore {
    master_key: Tainted<Vec<u8>, Secret>,
}

impl SecretStore {
    fn new(master_key: Vec<u8>) -> Self {
        Self {
            master_key: Tainted::secret(master_key),
        }
    }

    /// Derive a session key (still secret)
    fn derive_session_key(&self, session_id: &str) -> Tainted<Vec<u8>, Secret> {
        self.master_key.expose_secret(|key| {
            // Simple derivation (use proper KDF in production)
            let mut derived = key.clone();
            for byte in session_id.bytes() {
                derived.push(byte);
            }
            Tainted::secret(derived)
        })
    }

    /// Get key fingerprint (public)
    fn get_key_fingerprint(&self) -> Tainted<String, Public> {
        self.master_key.expose_secret(|key| {
            // Hash first 32 bytes
            let sample = if key.len() > 32 { &key[..32] } else { key };
            let fingerprint = sample.iter()
                .take(4)
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            Tainted::public(format!("fp:{}", fingerprint))
        })
    }
}

/// Demonstrates secret rotation pattern
fn demonstrate_rotation() {
    println!("\n=== Secret Rotation Pattern ===");
    
    let logger = Logger::new("ROTATION");
    let mut token = RotatingToken::new("initial_token_v1".to_string());
    
    // Log generation (public)
    logger.log(&token.get_generation().map(|g| format!("Generation: {}", g)));
    
    // Log token hash (public)
    let hash = token.get_token_hash();
    logger.log(&hash);
    
    // Rotate token
    token.rotate("rotated_token_v2".to_string());
    
    // Log new generation
    logger.log(&token.get_generation().map(|g| format!("New generation: {}", g)));
}

/// Demonstrates audit logging
fn demonstrate_audit_trail() {
    println!("\n=== Audit Trail Pattern ===");
    
    let mut audit = AuditLog::new();
    
    // Record sensitive operations
    let secret = Tainted::secret("sensitive_data".to_string());
    
    audit.record_declassification(
        "user_export",
        "User requested data export via authenticated API"
    );
    
    // Declassify with audit
    let _public = secret.declassify();
    
    audit.record_declassification(
        "system_log",
        "Debug output for system administrator"
    );
    
    println!("\n📋 Audit entries: {}", audit.get_entries().len());
}

/// Demonstrates secure configuration management
fn demonstrate_config_management() {
    println!("\n=== Configuration Management ===");
    
    let logger = Logger::new("CONFIG");
    let config = AppConfig::new();
    
    config.log_safe_config(&logger);
    
    // Public values can be used freely
    let port = config.port.clone();
    logger.log(&port.map(|p| format!("Listening on port {}", p)));
    
    // Secret values require explicit handling
    // Cannot log directly - won't compile:
    // logger.log(&config.database_url);
}

/// Demonstrates request/response middleware
fn demonstrate_middleware() {
    println!("\n=== Middleware Pattern ===");
    
    let logger = Logger::new("MIDDLEWARE");
    
    // Create request context
    let ctx = RequestContext {
        request_id: Tainted::public("req_abc123".to_string()),
        user_id: Tainted::public(42),
        session_token: Tainted::secret("eyJhbGciOi...".to_string()),
        ip_address: Tainted::public("192.168.1.100".to_string()),
    };
    
    ctx.log_request(&logger);
    
    // Prepare response headers (all must be Public)
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), Tainted::public("application/json".to_string()));
    headers.insert("X-Request-ID".to_string(), ctx.request_id.clone());
    
    // Cannot add secret to headers - won't compile:
    // headers.insert("X-Session-Token".to_string(), ctx.session_token);
    
    SecurityMiddleware::validate_response_headers(&headers);
}

/// Demonstrates secret store with key derivation
fn demonstrate_secret_store() {
    println!("\n=== Secret Store Pattern ===");
    
    let logger = Logger::new("KEYSTORE");
    
    // Master key (secret)
    let master_key = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
    let store = SecretStore::new(master_key);
    
    // Can log public fingerprint
    let fingerprint = store.get_key_fingerprint();
    logger.log(&fingerprint.map(|fp| format!("Master key: {}", fp)));
    
    // Derive session key (still secret)
    let session_key = store.derive_session_key("session_xyz");
    
    // Can hash derived key for logging
    let session_hash = Sanitizer::hash_secret(&session_key);
    logger.log(&session_hash.map(|h| format!("Session key: {}", h)));
}

/// Demonstrates type-safe API design
fn demonstrate_type_safety() {
    println!("\n=== Type Safety Guarantees ===");
    
    // Function that only accepts public data
    fn send_to_external_service(data: &Tainted<String, Public>) {
        println!("📤 Sending to external service: {}", data.as_public());
    }
    
    // Function that handles secrets internally
    fn process_secret_internally(secret: &Tainted<String, Secret>) -> Tainted<bool, Public> {
        let is_valid = secret.expose_secret(|s| s.len() > 10);
        Tainted::public(is_valid)
    }
    
    let public_data = Tainted::public("safe_data".to_string());
    let secret_data = Tainted::secret("secret_password".to_string());
    
    // This works
    send_to_external_service(&public_data);
    
    // This won't compile:
    // send_to_external_service(&secret_data);
    
    // But we can process and declassify
    let result = process_secret_internally(&secret_data);
    send_to_external_service(&result.map(|b| format!("Valid: {}", b)));
}

fn main() {
    println!("🔐 Advanced Secret Taint Tracking Patterns\n");
    
    demonstrate_rotation();
    demonstrate_audit_trail();
    demonstrate_config_management();
    demonstrate_middleware();
    demonstrate_secret_store();
    demonstrate_type_safety();
    
    println!("\n✨ All advanced patterns demonstrated successfully!");
}
