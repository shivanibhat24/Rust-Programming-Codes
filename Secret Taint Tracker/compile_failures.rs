//! This file contains examples of operations that WON'T compile.
//! 
//! These are intentionally commented out to show the compile-time
//! guarantees provided by the taint tracking system.
//! 
//! Uncomment any of these to see the compiler errors!

#![allow(dead_code)]
#![allow(unused_imports)]

use secret_taint::{Tainted, Secret, Public, Logger, NetworkSink};

/// ❌ FORBIDDEN: Logging secret data
fn forbidden_log_secret() {
    let password = Tainted::secret("my_password".to_string());
    let logger = Logger::new("TEST");
    
    // ❌ Won't compile: Logger::log requires Tainted<T, Public>
    // logger.log(&password);
    //
    // Error: expected `&Tainted<String, Public>`, found `&Tainted<String, Secret>`
}

/// ❌ FORBIDDEN: Sending secrets over network
fn forbidden_network_secret() {
    let api_key = Tainted::secret("sk_live_12345".to_string());
    let network = NetworkSink::new("external.api.com");
    
    // ❌ Won't compile: NetworkSink::send requires Tainted<T, Public>
    // network.send(&api_key);
    //
    // Error: expected `&Tainted<String, Public>`, found `&Tainted<String, Secret>`
}

/// ❌ FORBIDDEN: Displaying secret data
fn forbidden_display_secret() {
    let secret = Tainted::secret("secret_value".to_string());
    
    // ❌ Won't compile: Tainted<T, Secret> doesn't implement Display
    // println!("Secret: {}", secret);
    //
    // Error: `Tainted<String, Secret>` doesn't implement `std::fmt::Display`
}

/// ❌ FORBIDDEN: Converting secret to public without declassification
fn forbidden_implicit_declassification() {
    let secret = Tainted::secret("data".to_string());
    
    // ❌ Won't compile: No automatic conversion from Secret to Public
    // let public: Tainted<String, Public> = secret;
    //
    // Error: mismatched types
    
    // ✅ Correct way: Explicit declassification
    let _public = secret.declassify();
}

/// ❌ FORBIDDEN: Accessing secret value directly
fn forbidden_direct_access() {
    let secret = Tainted::secret("password".to_string());
    
    // ❌ Won't compile: value field is private
    // let raw = secret.value;
    //
    // Error: field `value` of struct `Tainted` is private
    
    // ✅ Correct way: Use expose_secret or into_secret
    secret.expose_secret(|s| {
        // Use s here in controlled context
        println!("Length: {}", s.len());
    });
}

/// ❌ FORBIDDEN: Cloning into different taint level
fn forbidden_taint_transmutation() {
    let secret = Tainted::secret(42);
    
    // ❌ Won't compile: Can't change taint level through clone
    // let public: Tainted<i32, Public> = secret.clone();
    //
    // Error: mismatched types
}

/// ❌ FORBIDDEN: Adding response header with secret
fn forbidden_secret_in_headers() {
    use std::collections::HashMap;
    
    let session_token = Tainted::secret("secret_session_token".to_string());
    
    // This function signature enforces public-only headers
    fn set_headers(_headers: &HashMap<String, Tainted<String, Public>>) {}
    
    let mut headers = HashMap::new();
    
    // ❌ Won't compile: Can't insert Secret into Public-only HashMap
    // headers.insert("Authorization".to_string(), session_token);
    // set_headers(&headers);
    //
    // Error: expected `Tainted<String, Public>`, found `Tainted<String, Secret>`
}

/// ❌ FORBIDDEN: Serializing secrets
fn forbidden_serialize_secret() {
    let secret = Tainted::secret("secret_data".to_string());
    
    // If we had serde support, this would be forbidden:
    // let json = serde_json::to_string(&secret).unwrap();
    //
    // Secret types should never implement Serialize to prevent accidental leaks
}

/// ❌ FORBIDDEN: Downgrading from Public to Secret (type system prevents this)
fn forbidden_public_to_secret_without_explicit_upgrade() {
    let public = Tainted::public("data".to_string());
    
    // ❌ Won't compile: No implicit conversion Public -> Secret
    // let secret: Tainted<String, Secret> = public;
    //
    // Error: mismatched types
    
    // ✅ Correct way: Explicit upgrade
    let _secret = public.taint_secret();
}

/// ❌ FORBIDDEN: Mixing taint levels in operations
fn forbidden_mixed_taint_operations() {
    let secret = Tainted::secret(10);
    let public = Tainted::public(5);
    
    // ❌ Won't compile: Can't add Secret and Public directly
    // let sum = secret + public;
    //
    // Error: no implementation for `Tainted<i32, Secret> + Tainted<i32, Public>`
    
    // ✅ Correct way: Upgrade public to secret first
    use secret_taint::combine_with_secret;
    let _sum = combine_with_secret(secret, public, |s, p| s + p);
}

/// ❌ FORBIDDEN: Returning secret from public function
fn forbidden_secret_leak_in_return() -> Tainted<String, Public> {
    let secret = Tainted::secret("leaked".to_string());
    
    // ❌ Won't compile: Can't return Secret when Public is expected
    // secret
    //
    // Error: mismatched types
    
    // Must declassify first
    secret.declassify()
}

/// ❌ FORBIDDEN: Storing secret in public struct
struct PublicData {
    // ❌ This struct can only hold Public data
    value: Tainted<String, Public>,
}

fn forbidden_secret_in_public_struct() {
    let secret = Tainted::secret("secret".to_string());
    
    // ❌ Won't compile: Can't assign Secret to Public field
    // let data = PublicData { value: secret };
    //
    // Error: mismatched types
}

/// ✅ CORRECT: Separate structs for different sensitivity levels
struct SecretData {
    value: Tainted<String, Secret>,
}

struct SafePublicData {
    value: Tainted<String, Public>,
}

/// ❌ FORBIDDEN: Extracting public from secret without declassification
fn forbidden_as_public_on_secret() {
    let secret = Tainted::secret("data".to_string());
    
    // ❌ Won't compile: as_public() only exists on Tainted<T, Public>
    // let value = secret.as_public();
    //
    // Error: no method named `as_public` found for struct `Tainted<String, Secret>`
}

fn main() {
    println!("This file demonstrates compile-time safety guarantees.");
    println!("All forbidden operations are commented out.");
    println!("Uncomment any to see the compiler prevent secret leaks!");
}
