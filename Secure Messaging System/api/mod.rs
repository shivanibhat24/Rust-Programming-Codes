// src/api/mod.rs
//! API module with rate limiting and security middleware

pub mod handlers;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use governor::{
    clock::DefaultClock,
    state::{direct::NotKeyed, InMemoryState},
    Quota, RateLimiter,
};
use std::{num::NonZeroU32, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};

use handlers::AppState;

/// Rate limiter for API endpoints
pub type ApiRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Create the API router
pub fn create_router(db: AppState) -> Router {
    // Create rate limiter: 100 requests per minute
    let rate_limiter = Arc::new(RateLimiter::direct(
        Quota::per_minute(NonZeroU32::new(100).unwrap()),
    ));

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/prekey-bundle", post(handlers::get_prekey_bundle))
        .route("/send", post(handlers::send_message))
        .route("/messages", post(handlers::get_messages))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default()))
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn_with_state(
                    rate_limiter.clone(),
                    rate_limit_middleware,
                )),
        )
        .with_state(db)
}

/// Rate limiting middleware
async fn rate_limit_middleware(
    State(limiter): State<ApiRateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    match limiter.check() {
        Ok(_) => next.run(request).await,
        Err(_) => (
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded. Please try again later.",
        )
            .into_response(),
    }
}

/// Security headers middleware
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'".parse().unwrap(),
    );

    response
}
