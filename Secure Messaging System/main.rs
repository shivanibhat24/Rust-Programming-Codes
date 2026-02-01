// src/main.rs
//! Military-grade secure messaging backend
//! 
//! Features:
//! - Double Ratchet Algorithm (Signal Protocol)
//! - X3DH Key Agreement
//! - Forward Secrecy & Post-Compromise Security
//! - Authenticated Encryption (ChaCha20-Poly1305)
//! - Rate Limiting & Intrusion Detection
//! - Comprehensive Audit Logging

mod api;
mod crypto;
mod db;

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "secure_messenger=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🔐 Starting Military-Grade Secure Messenger");

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:messenger.db".to_string());

    let db = Arc::new(db::Database::new(&database_url).await?);
    db.init_schema().await?;

    tracing::info!("✅ Database initialized");

    // Create API router
    let app = api::create_router(db);

    // Server configuration
    let addr = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 Server listening on {}", addr);
    tracing::info!("📡 API Endpoints:");
    tracing::info!("   POST /register        - Register new user");
    tracing::info!("   POST /login          - User login");
    tracing::info!("   POST /prekey-bundle  - Get user's prekey bundle");
    tracing::info!("   POST /send           - Send encrypted message");
    tracing::info!("   POST /messages       - Get undelivered messages");
    tracing::info!("   GET  /health         - Health check");

    // Start server
    axum::serve(listener, app).await?;

    Ok(())
}
