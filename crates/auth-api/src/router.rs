use axum::{routing::{get, post}, Router, middleware};
use tower_http::trace::TraceLayer;
use crate::AppState;
use crate::handlers::{health, auth, users, auth_oidc, auth_saml};
use crate::middleware::{request_id_middleware, security_headers_middleware, RateLimiter};
use std::time::Duration;

pub fn api_router() -> Router<AppState> {
    // Create rate limiter: 5 requests per minute
    let rate_limiter = RateLimiter::new(5, Duration::from_secs(60));

    Router::new()
        .route("/health", get(health::health_check))
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(auth::register))
        .route("/users/:id/ban", post(users::ban_user))
        .route("/users/:id/activate", post(users::activate_user))
        .route("/auth/oidc/login", get(auth_oidc::login))
        .route("/auth/oidc/callback", get(auth_oidc::callback))
        .route("/auth/saml/metadata", get(auth_saml::metadata))
        .route("/auth/saml/acs", post(auth_saml::acs))
        // Add middleware layers (executed bottom-to-top)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(axum::Extension(rate_limiter))
}
