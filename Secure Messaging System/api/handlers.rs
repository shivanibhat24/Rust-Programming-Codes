// src/api/handlers.rs
//! HTTP API handlers for authentication and messaging

use crate::crypto::{PreKeyBundle, X3DHInitiator, X3DHReceiver};
use crate::db::Database;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type AppState = Arc<Database>;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub username: String,
    pub token: String, // In production, use proper JWT
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreKeyBundleResponse {
    pub user_id: String,
    pub bundle: PreKeyBundle,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub recipient_username: String,
    pub encrypted_content: String, // Base64 encoded
    pub header: String,             // Base64 encoded
    pub signature: String,          // Base64 encoded
    pub ephemeral_public: Option<String>, // For first message (X3DH)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message_id: String,
    pub created_at: String,
}

/// Register a new user
pub async fn register(
    State(db): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate username
    if req.username.is_empty() || req.username.len() > 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username must be 1-32 characters".to_string(),
        ));
    }

    // Check if user exists
    if db
        .get_user_by_username(&req.username)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .is_some()
    {
        return Err((StatusCode::CONFLICT, "Username already exists".to_string()));
    }

    // Hash password with Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .to_string();

    // Generate identity and prekey
    let identity_key = SigningKey::generate(&mut rand::thread_rng());
    let mut x3dh_receiver = X3DHReceiver::new(identity_key.clone());
    x3dh_receiver.add_one_time_prekey();
    let bundle = x3dh_receiver.generate_bundle();

    // Create user
    let user = db
        .create_user(
            &req.username,
            &bundle.identity_key,
            &bundle.signed_prekey,
            &bundle.prekey_signature,
            &password_hash,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Store one-time prekey
    if let Some(otpk) = &bundle.one_time_prekey {
        db.add_one_time_prekey(&user.id, otpk)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // Log audit
    db.log_audit(&user.id, "register", "0.0.0.0", "API", true)
        .await
        .ok();

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id: user.id,
            username: user.username,
        }),
    ))
}

/// Login
pub async fn login(
    State(db): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Get user
    let user = db
        .get_user_by_username(&req.username)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    // Log audit
    db.log_audit(&user.id, "login", "0.0.0.0", "API", true)
        .await
        .ok();

    // In production, generate proper JWT token
    let token = format!("token_{}", user.id);

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            user_id: user.id,
            username: user.username,
            token,
        }),
    ))
}

/// Get prekey bundle for a user
pub async fn get_prekey_bundle(
    State(db): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let username = req["username"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "Missing username".to_string()))?;

    // Get user
    let user = db
        .get_user_by_username(username)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    // Get one-time prekey if available
    let one_time_prekey = db
        .consume_one_time_prekey(&user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let bundle = PreKeyBundle {
        identity_key: user.identity_key,
        signed_prekey: user.signed_prekey,
        prekey_signature: user.prekey_signature,
        one_time_prekey,
    };

    Ok((
        StatusCode::OK,
        Json(PreKeyBundleResponse {
            user_id: user.id,
            bundle,
        }),
    ))
}

/// Send a message
pub async fn send_message(
    State(db): State<AppState>,
    Json(req): Json<SendMessageRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Decode base64 content
    let encrypted_content = base64::decode(&req.encrypted_content)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let header = base64::decode(&req.header)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let signature = base64::decode(&req.signature)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // Get recipient
    let recipient = db
        .get_user_by_username(&req.recipient_username)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Recipient not found".to_string()))?;

    // For this example, we'll use a dummy sender ID
    // In production, extract from authenticated session
    let sender_id = "sender_id".to_string();

    // Get or create session
    let session = db
        .get_or_create_session(&sender_id, &recipient.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Store message
    let message = db
        .store_message(
            &session.id,
            &sender_id,
            &recipient.id,
            &encrypted_content,
            &header,
            &signature,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(MessageResponse {
            message_id: message.id,
            created_at: message.created_at.to_rfc3339(),
        }),
    ))
}

/// Get undelivered messages
pub async fn get_messages(
    State(db): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = req["user_id"]
        .as_str()
        .ok_or((StatusCode::BAD_REQUEST, "Missing user_id".to_string()))?;

    let messages = db
        .get_undelivered_messages(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Mark as delivered
    for msg in &messages {
        db.mark_delivered(&msg.id).await.ok();
    }

    Ok((StatusCode::OK, Json(messages)))
}

/// Health check
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
