// src/db/mod.rs
//! Database models and operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub identity_key: Vec<u8>,
    pub signed_prekey: Vec<u8>,
    pub prekey_signature: Vec<u8>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OneTimePreKey {
    pub id: String,
    pub user_id: String,
    pub public_key: Vec<u8>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub user_a: String,
    pub user_b: String,
    pub state_a: Option<Vec<u8>>, // Serialized RatchetState for user A
    pub state_b: Option<Vec<u8>>, // Serialized RatchetState for user B
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub encrypted_content: Vec<u8>,
    pub header: Vec<u8>,
    pub signature: Vec<u8>,
    pub delivered: bool,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub ip_address: String,
    pub user_agent: String,
    pub success: bool,
    pub created_at: DateTime<Utc>,
}

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create new database connection
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                identity_key BLOB NOT NULL,
                signed_prekey BLOB NOT NULL,
                prekey_signature BLOB NOT NULL,
                password_hash TEXT NOT NULL,
                created_at DATETIME NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS one_time_prekeys (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                public_key BLOB NOT NULL,
                used BOOLEAN NOT NULL DEFAULT 0,
                created_at DATETIME NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_a TEXT NOT NULL,
                user_b TEXT NOT NULL,
                state_a BLOB,
                state_b BLOB,
                created_at DATETIME NOT NULL,
                last_message_at DATETIME,
                FOREIGN KEY (user_a) REFERENCES users(id),
                FOREIGN KEY (user_b) REFERENCES users(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                recipient_id TEXT NOT NULL,
                encrypted_content BLOB NOT NULL,
                header BLOB NOT NULL,
                signature BLOB NOT NULL,
                delivered BOOLEAN NOT NULL DEFAULT 0,
                created_at DATETIME NOT NULL,
                delivered_at DATETIME,
                FOREIGN KEY (session_id) REFERENCES sessions(id),
                FOREIGN KEY (sender_id) REFERENCES users(id),
                FOREIGN KEY (recipient_id) REFERENCES users(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                action TEXT NOT NULL,
                ip_address TEXT NOT NULL,
                user_agent TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                created_at DATETIME NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indices for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient_id, delivered)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(user_id, created_at)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        username: &str,
        identity_key: &[u8],
        signed_prekey: &[u8],
        prekey_signature: &[u8],
        password_hash: &str,
    ) -> Result<User, sqlx::Error> {
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            identity_key: identity_key.to_vec(),
            signed_prekey: signed_prekey.to_vec(),
            prekey_signature: prekey_signature.to_vec(),
            password_hash: password_hash.to_string(),
            created_at: Utc::now(),
        };

        sqlx::query(
            "INSERT INTO users (id, username, identity_key, signed_prekey, prekey_signature, password_hash, created_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.identity_key)
        .bind(&user.signed_prekey)
        .bind(&user.prekey_signature)
        .bind(&user.password_hash)
        .bind(&user.created_at)
        .execute(&self.pool)
        .await?;

        Ok(user)
    }

    /// Get user by username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
    }

    /// Add one-time prekey
    pub async fn add_one_time_prekey(
        &self,
        user_id: &str,
        public_key: &[u8],
    ) -> Result<(), sqlx::Error> {
        let prekey = OneTimePreKey {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            public_key: public_key.to_vec(),
            used: false,
            created_at: Utc::now(),
        };

        sqlx::query(
            "INSERT INTO one_time_prekeys (id, user_id, public_key, used, created_at) 
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&prekey.id)
        .bind(&prekey.user_id)
        .bind(&prekey.public_key)
        .bind(prekey.used)
        .bind(&prekey.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get and mark one-time prekey as used
    pub async fn consume_one_time_prekey(
        &self,
        user_id: &str,
    ) -> Result<Option<Vec<u8>>, sqlx::Error> {
        let prekey = sqlx::query_as::<_, OneTimePreKey>(
            "SELECT * FROM one_time_prekeys WHERE user_id = ? AND used = 0 LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(prekey) = prekey {
            sqlx::query("UPDATE one_time_prekeys SET used = 1 WHERE id = ?")
                .bind(&prekey.id)
                .execute(&self.pool)
                .await?;

            Ok(Some(prekey.public_key))
        } else {
            Ok(None)
        }
    }

    /// Create or get session between two users
    pub async fn get_or_create_session(
        &self,
        user_a: &str,
        user_b: &str,
    ) -> Result<Session, sqlx::Error> {
        // Check if session exists
        let existing = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE (user_a = ? AND user_b = ?) OR (user_a = ? AND user_b = ?)"
        )
        .bind(user_a)
        .bind(user_b)
        .bind(user_b)
        .bind(user_a)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(session) = existing {
            return Ok(session);
        }

        // Create new session
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_a: user_a.to_string(),
            user_b: user_b.to_string(),
            state_a: None,
            state_b: None,
            created_at: Utc::now(),
            last_message_at: None,
        };

        sqlx::query(
            "INSERT INTO sessions (id, user_a, user_b, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&session.id)
        .bind(&session.user_a)
        .bind(&session.user_b)
        .bind(&session.created_at)
        .execute(&self.pool)
        .await?;

        Ok(session)
    }

    /// Store a message
    pub async fn store_message(
        &self,
        session_id: &str,
        sender_id: &str,
        recipient_id: &str,
        encrypted_content: &[u8],
        header: &[u8],
        signature: &[u8],
    ) -> Result<Message, sqlx::Error> {
        let message = Message {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            recipient_id: recipient_id.to_string(),
            encrypted_content: encrypted_content.to_vec(),
            header: header.to_vec(),
            signature: signature.to_vec(),
            delivered: false,
            created_at: Utc::now(),
            delivered_at: None,
        };

        sqlx::query(
            "INSERT INTO messages (id, session_id, sender_id, recipient_id, encrypted_content, header, signature, delivered, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&message.id)
        .bind(&message.session_id)
        .bind(&message.sender_id)
        .bind(&message.recipient_id)
        .bind(&message.encrypted_content)
        .bind(&message.header)
        .bind(&message.signature)
        .bind(message.delivered)
        .bind(&message.created_at)
        .execute(&self.pool)
        .await?;

        // Update session last message time
        sqlx::query("UPDATE sessions SET last_message_at = ? WHERE id = ?")
            .bind(&message.created_at)
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(message)
    }

    /// Get undelivered messages for a user
    pub async fn get_undelivered_messages(
        &self,
        user_id: &str,
    ) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE recipient_id = ? AND delivered = 0 ORDER BY created_at"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Mark message as delivered
    pub async fn mark_delivered(&self, message_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE messages SET delivered = 1, delivered_at = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(message_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Log audit event
    pub async fn log_audit(
        &self,
        user_id: &str,
        action: &str,
        ip_address: &str,
        user_agent: &str,
        success: bool,
    ) -> Result<(), sqlx::Error> {
        let log = AuditLog {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            action: action.to_string(),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
            success,
            created_at: Utc::now(),
        };

        sqlx::query(
            "INSERT INTO audit_logs (id, user_id, action, ip_address, user_agent, success, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&log.id)
        .bind(&log.user_id)
        .bind(&log.action)
        .bind(&log.ip_address)
        .bind(&log.user_agent)
        .bind(log.success)
        .bind(&log.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
