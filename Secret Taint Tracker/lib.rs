//! # Secret Taint Tracker
//! 
//! A zero-cost abstraction library for tracking secret data propagation at runtime.
//! Prevents accidental leakage of sensitive information through type-safe taint tracking.

use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, Deref};

// ============================================================================
// Core Taint Markers (Zero-sized types for compile-time guarantees)
// ============================================================================

/// Marker trait for taint levels
pub trait TaintLevel: sealed::Sealed {}

/// Secret data - must never reach unsafe sinks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Secret;

/// Public data - safe to log, transmit, display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Public;

impl TaintLevel for Secret {}
impl TaintLevel for Public {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Secret {}
    impl Sealed for super::Public {}
}

// ============================================================================
// Tainted<T, L> - Core wrapper type with phantom taint level
// ============================================================================

/// A value tagged with a taint level.
/// 
/// The taint level `L` is a zero-sized type that exists only at compile-time,
/// providing type-level guarantees about data sensitivity without runtime overhead.
pub struct Tainted<T, L: TaintLevel> {
    value: T,
    _marker: PhantomData<L>,
}

impl<T, L: TaintLevel> Tainted<T, L> {
    /// Internal constructor - kept private to control taint propagation
    fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Maps the inner value while preserving taint level
    pub fn map<U, F>(self, f: F) -> Tainted<U, L>
    where
        F: FnOnce(T) -> U,
    {
        Tainted::new(f(self.value))
    }

    /// Combines two tainted values - result has the more restrictive taint
    pub fn combine_with<U, R, F>(self, other: Tainted<U, L>, f: F) -> Tainted<R, L>
    where
        F: FnOnce(T, U) -> R,
    {
        Tainted::new(f(self.value, other.value))
    }
}

// ============================================================================
// Constructors and Conversions
// ============================================================================

impl<T> Tainted<T, Secret> {
    /// Mark data as secret - entry point for taint tracking
    pub fn secret(value: T) -> Self {
        Self::new(value)
    }

    /// Explicitly declassify secret data (AUDIT POINT)
    /// 
    /// This is the ONLY way to convert Secret -> Public.
    /// Every call should be audited.
    pub fn declassify(self) -> Tainted<T, Public> {
        // This is an intentional declassification point
        // In production, you might want to log/audit this
        Tainted::new(self.value)
    }

    /// Access secret data with explicit awareness
    /// Should only be used in controlled contexts
    pub fn expose_secret<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.value)
    }

    /// Consume and extract the secret value (AUDIT POINT)
    pub fn into_secret(self) -> T {
        self.value
    }
}

impl<T> Tainted<T, Public> {
    /// Mark data as public (safe to expose)
    pub fn public(value: T) -> Self {
        Self::new(value)
    }

    /// Extract public value - safe operation
    pub fn into_public(self) -> T {
        self.value
    }

    /// Access public value - safe operation
    pub fn as_public(&self) -> &T {
        &self.value
    }
}

// ============================================================================
// Taint Propagation Rules
// ============================================================================

/// Secret + Secret = Secret
impl<T: Add> Add for Tainted<T, Secret> {
    type Output = Tainted<T::Output, Secret>;

    fn add(self, rhs: Self) -> Self::Output {
        Tainted::new(self.value + rhs.value)
    }
}

/// Public + Public = Public
impl<T: Add> Add for Tainted<T, Public> {
    type Output = Tainted<T::Output, Public>;

    fn add(self, rhs: Self) -> Self::Output {
        Tainted::new(self.value + rhs.value)
    }
}

/// Upgrade public to secret (when combining with secret data)
impl<T> Tainted<T, Public> {
    pub fn taint_secret(self) -> Tainted<T, Secret> {
        Tainted::new(self.value)
    }
}

// ============================================================================
// Safe Sinks - Only accept Public data
// ============================================================================

/// Trait for safe output sinks that only accept public data
pub trait SafeSink {
    fn write_public<T: fmt::Display>(&mut self, value: &Tainted<T, Public>);
}

/// Logger that enforces taint safety
pub struct Logger {
    prefix: String,
}

impl Logger {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }

    /// Only public data can be logged
    pub fn log<T: fmt::Display>(&self, value: &Tainted<T, Public>) {
        println!("[{}] {}", self.prefix, value.as_public());
    }

    /// Attempting to log secret data won't compile!
    // This won't compile:
    // pub fn log_secret<T: fmt::Display>(&self, value: &Tainted<T, Secret>) {
    //     println!("[{}] {}", self.prefix, value.value);
    // }
}

/// Network sink that only accepts public data
pub struct NetworkSink {
    destination: String,
}

impl NetworkSink {
    pub fn new(destination: impl Into<String>) -> Self {
        Self {
            destination: destination.into(),
        }
    }

    /// Only public data can be transmitted
    pub fn send<T: fmt::Display>(&self, value: &Tainted<T, Public>) {
        println!("[NET->{}] Sending: {}", self.destination, value.as_public());
    }
}

// ============================================================================
// Display Implementation (Only for Public)
// ============================================================================

impl<T: fmt::Display> fmt::Display for Tainted<T, Public> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T: fmt::Debug> fmt::Debug for Tainted<T, Public> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Public({:?})", self.value)
    }
}

// Secret values don't implement Display - prevents accidental printing
impl<T> fmt::Debug for Tainted<T, Secret> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}

// ============================================================================
// Clone implementation
// ============================================================================

impl<T: Clone, L: TaintLevel> Clone for Tainted<T, L> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

// ============================================================================
// Higher-level utilities
// ============================================================================

/// Sanitizer that can convert secret data to public by transformation
pub struct Sanitizer;

impl Sanitizer {
    /// Hash a secret value, producing public data
    pub fn hash_secret<T: std::hash::Hash>(secret: &Tainted<T, Secret>) -> Tainted<String, Public> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let hash = secret.expose_secret(|val| {
            let mut hasher = DefaultHasher::new();
            val.hash(&mut hasher);
            hasher.finish()
        });

        Tainted::public(format!("hash:{:x}", hash))
    }

    /// Redact a secret string, keeping only length information
    pub fn redact_length(secret: &Tainted<String, Secret>) -> Tainted<String, Public> {
        let len = secret.expose_secret(|s| s.len());
        Tainted::public(format!("[REDACTED {} chars]", len))
    }

    /// Mask a secret, showing only first/last N chars
    pub fn mask(secret: &Tainted<String, Secret>, show: usize) -> Tainted<String, Public> {
        secret.expose_secret(|s| {
            if s.len() <= show * 2 {
                Tainted::public("***".to_string())
            } else {
                let start = &s[..show];
                let end = &s[s.len() - show..];
                Tainted::public(format!("{}...{}", start, end))
            }
        })
    }
}

// ============================================================================
// Type-safe operations that preserve or upgrade taint
// ============================================================================

/// Combine a secret with public data - result is secret
pub fn combine_with_secret<T, U, R, F>(
    secret: Tainted<T, Secret>,
    public: Tainted<U, Public>,
    f: F,
) -> Tainted<R, Secret>
where
    F: FnOnce(T, U) -> R,
{
    Tainted::secret(f(secret.into_secret(), public.into_public()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_cannot_be_logged() {
        let password = Tainted::secret("super_secret_password".to_string());
        let logger = Logger::new("TEST");

        // This won't compile:
        // logger.log(&password);

        // Must declassify first
        let safe_password = Sanitizer::redact_length(&password);
        logger.log(&safe_password);
    }

    #[test]
    fn test_taint_propagation() {
        let secret1 = Tainted::secret(42);
        let secret2 = Tainted::secret(8);

        let sum = secret1 + secret2;

        // sum is still Secret, can't be logged
        // This demonstrates taint propagation

        let declassified = sum.declassify();
        assert_eq!(*declassified.as_public(), 50);
    }

    #[test]
    fn test_public_data_flows() {
        let public1 = Tainted::public(100);
        let public2 = Tainted::public(200);

        let sum = public1 + public2;

        let logger = Logger::new("PUBLIC");
        logger.log(&sum);

        assert_eq!(*sum.as_public(), 300);
    }

    #[test]
    fn test_sanitization() {
        let api_key = Tainted::secret("sk_live_1234567890abcdef".to_string());

        let masked = Sanitizer::mask(&api_key, 3);
        
        let logger = Logger::new("SANITIZED");
        logger.log(&masked);

        assert!(masked.as_public().contains("sk_"));
        assert!(masked.as_public().contains("..."));
    }

    #[test]
    fn test_combine_secret_and_public() {
        let user_id = Tainted::public(12345);
        let password = Tainted::secret("hunter2".to_string());

        let credential = combine_with_secret(password, user_id, |pwd, uid| {
            format!("{}:{}", uid, pwd)
        });

        // credential is Secret, cannot be logged directly
        // This won't compile:
        // logger.log(&credential);

        // Can hash it though
        let cred_hash = Sanitizer::hash_secret(&credential);
        let logger = Logger::new("CRED");
        logger.log(&cred_hash);
    }
}
