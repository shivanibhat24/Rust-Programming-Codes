//! Comprehensive demonstration of the Secret Taint Tracker
//! 
//! This example shows:
//! - Secret data creation and propagation
//! - Compile-time prevention of leaks
//! - Safe declassification patterns
//! - Real-world usage scenarios

use secret_taint::{
    Tainted, Secret, Public, Logger, NetworkSink, Sanitizer, combine_with_secret,
};

/// Simulated user with mixed public/secret data
struct User {
    id: Tainted<u64, Public>,
    username: Tainted<String, Public>,
    email: Tainted<String, Public>,
    password_hash: Tainted<String, Secret>,
    api_key: Tainted<String, Secret>,
}

impl User {
    fn new(id: u64, username: String, email: String, password: String, api_key: String) -> Self {
        Self {
            id: Tainted::public(id),
            username: Tainted::public(username),
            email: Tainted::public(email),
            password_hash: Tainted::secret(password),
            api_key: Tainted::secret(api_key),
        }
    }

    /// Safe logging - only public fields
    fn log_public_info(&self, logger: &Logger) {
        logger.log(&self.id.clone().map(|id| format!("User ID: {}", id)));
        logger.log(&self.username.clone().map(|name| format!("Username: {}", name)));
        logger.log(&self.email.clone().map(|email| format!("Email: {}", email)));
        
        // This would NOT compile - trying to log secret data:
        // logger.log(&self.password_hash);
        // logger.log(&self.api_key);
    }

    /// Authenticate - works with secret data internally
    fn authenticate(&self, input_password: &str) -> Tainted<bool, Public> {
        // Compare secrets internally
        let is_valid = self.password_hash.expose_secret(|stored| {
            // In real code, use constant-time comparison
            stored == input_password
        });

        // Result is public (boolean auth status)
        Tainted::public(is_valid)
    }

    /// Generate safe API key representation for logging
    fn get_masked_api_key(&self) -> Tainted<String, Public> {
        Sanitizer::mask(&self.api_key, 4)
    }
}

/// Simulated HTTP request with tainted data
struct HttpRequest {
    path: Tainted<String, Public>,
    auth_token: Tainted<String, Secret>,
    body: Tainted<String, Public>,
}

impl HttpRequest {
    fn log_request(&self, logger: &Logger) {
        logger.log(&self.path.clone().map(|p| format!("Path: {}", p)));
        logger.log(&self.body.clone().map(|b| format!("Body: {}", b)));
        
        // Log masked token instead of raw secret
        let masked_token = Sanitizer::mask(&self.auth_token, 4);
        logger.log(&masked_token.map(|t| format!("Token: {}", t)));
        
        // This would NOT compile:
        // logger.log(&self.auth_token);
    }

    fn send_to_analytics(&self, sink: &NetworkSink) {
        // Only send public data to analytics
        sink.send(&self.path);
        sink.send(&self.body);
        
        // Cannot send secret token - won't compile:
        // sink.send(&self.auth_token);
    }
}

/// Database connection with secret credentials
struct Database {
    connection_string: Tainted<String, Secret>,
}

impl Database {
    fn new(host: &str, username: &str, password: &str) -> Self {
        let conn_str = format!("postgres://{}:{}@{}/db", username, password, host);
        Self {
            connection_string: Tainted::secret(conn_str),
        }
    }

    fn get_safe_connection_info(&self) -> Tainted<String, Public> {
        // Extract only non-sensitive parts
        self.connection_string.expose_secret(|s| {
            // Parse and redact credentials
            if let Some(at_pos) = s.rfind('@') {
                let host_part = &s[at_pos + 1..];
                Tainted::public(format!("Host: {}", host_part))
            } else {
                Tainted::public("[Invalid connection string]".to_string())
            }
        })
    }
}

/// Payment processor - handles sensitive financial data
struct Payment {
    amount: Tainted<f64, Public>,
    card_number: Tainted<String, Secret>,
    cvv: Tainted<String, Secret>,
}

impl Payment {
    fn new(amount: f64, card_number: String, cvv: String) -> Self {
        Self {
            amount: Tainted::public(amount),
            card_number: Tainted::secret(card_number),
            cvv: Tainted::secret(cvv),
        }
    }

    fn process(&self) -> Tainted<String, Public> {
        // Process payment internally with secret data
        let transaction_id = self.card_number.expose_secret(|card| {
            // Simulate processing
            format!("txn_{}", &card[card.len()-4..])
        });

        Tainted::public(transaction_id)
    }

    fn get_receipt(&self) -> Tainted<String, Public> {
        // Create receipt with masked card number
        let masked_card = Sanitizer::mask(&self.card_number, 4);
        
        self.amount.clone().combine_with(masked_card, |amt, card| {
            format!("Amount: ${:.2}\nCard: {}", amt, card)
        })
    }
}

/// Demonstrates taint propagation through computation
fn demonstrate_taint_propagation() {
    println!("\n=== Taint Propagation Demo ===");
    
    let secret_value = Tainted::secret(42);
    let public_value = Tainted::public(10);
    
    // Secret + Secret = Secret
    let secret_sum = secret_value.clone() + Tainted::secret(8);
    println!("Secret + Secret = {:?}", secret_sum);
    
    // Public + Public = Public
    let public_sum = public_value.clone() + Tainted::public(5);
    println!("Public + Public = {:?}", public_sum);
    
    // Combining secret and public = secret
    let mixed = combine_with_secret(
        Tainted::secret("secret_data"),
        Tainted::public("_public_suffix"),
        |s, p| format!("{}{}", s, p)
    );
    println!("Secret + Public = {:?}", mixed);
    
    // Only way to make secret public: explicit declassification
    let declassified = secret_sum.declassify();
    let logger = Logger::new("DECLASSIFIED");
    logger.log(&declassified);
}

/// Demonstrates different sanitization techniques
fn demonstrate_sanitization() {
    println!("\n=== Sanitization Demo ===");
    
    let logger = Logger::new("SANITIZE");
    
    // Original secret
    let api_key = Tainted::secret("sk_live_abc123def456ghi789jkl".to_string());
    
    // Method 1: Masking
    let masked = Sanitizer::mask(&api_key, 4);
    logger.log(&masked);
    
    // Method 2: Length only
    let length_info = Sanitizer::redact_length(&api_key);
    logger.log(&length_info);
    
    // Method 3: Hashing
    let hashed = Sanitizer::hash_secret(&api_key);
    logger.log(&hashed);
}

/// Demonstrates a realistic web application scenario
fn demonstrate_web_app_scenario() {
    println!("\n=== Web Application Scenario ===");
    
    let logger = Logger::new("WEBAPP");
    let analytics = NetworkSink::new("analytics.example.com");
    
    // Create user
    let user = User::new(
        12345,
        "alice".to_string(),
        "alice@example.com".to_string(),
        "hashed_password_xyz".to_string(),
        "sk_live_secret_key_12345".to_string(),
    );
    
    // Log public user info (safe)
    user.log_public_info(&logger);
    
    // Log masked API key for debugging
    let masked_key = user.get_masked_api_key();
    logger.log(&masked_key.map(|k| format!("API Key: {}", k)));
    
    // Authenticate user
    let auth_result = user.authenticate("hashed_password_xyz");
    logger.log(&auth_result.map(|valid| format!("Auth result: {}", valid)));
    
    // Handle HTTP request
    let request = HttpRequest {
        path: Tainted::public("/api/users/12345".to_string()),
        auth_token: Tainted::secret("Bearer eyJhbGc...".to_string()),
        body: Tainted::public(r#"{"action": "get_profile"}"#.to_string()),
    };
    
    request.log_request(&logger);
    request.send_to_analytics(&analytics);
}

/// Demonstrates database connection handling
fn demonstrate_database_scenario() {
    println!("\n=== Database Connection Scenario ===");
    
    let logger = Logger::new("DATABASE");
    
    let db = Database::new("localhost:5432", "admin", "super_secret_password");
    
    // Can log safe connection info
    let safe_info = db.get_safe_connection_info();
    logger.log(&safe_info);
    
    // Cannot log the actual connection string - won't compile:
    // logger.log(&db.connection_string);
}

/// Demonstrates payment processing
fn demonstrate_payment_scenario() {
    println!("\n=== Payment Processing Scenario ===");
    
    let logger = Logger::new("PAYMENT");
    
    let payment = Payment::new(
        99.99,
        "4532123456789012".to_string(),
        "123".to_string(),
    );
    
    // Process payment
    let transaction_id = payment.process();
    logger.log(&transaction_id.map(|id| format!("Transaction: {}", id)));
    
    // Generate receipt with masked data
    let receipt = payment.get_receipt();
    logger.log(&receipt.map(|r| format!("Receipt:\n{}", r)));
    
    // Cannot log raw card data - won't compile:
    // logger.log(&payment.card_number);
    // logger.log(&payment.cvv);
}

fn main() {
    println!("🔒 Secret Taint Tracker - Comprehensive Demo\n");
    println!("This demonstrates compile-time prevention of secret leakage.");
    println!("Try uncommenting the forbidden lines to see compile errors!\n");
    
    demonstrate_taint_propagation();
    demonstrate_sanitization();
    demonstrate_web_app_scenario();
    demonstrate_database_scenario();
    demonstrate_payment_scenario();
    
    println!("\n✅ All operations completed safely!");
    println!("No secrets were leaked to logs or network.");
}
