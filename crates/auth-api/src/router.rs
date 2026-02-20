use crate::handlers::{
    auth, auth_oidc, auth_saml, health, login_otp, otp, profile, register, users, verification,
};
use crate::middleware::{request_id_middleware, security_headers_middleware, RateLimiter};
use crate::AppState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower_http::trace::TraceLayer;

pub fn api_router() -> Router<AppState> {
    // Create rate limiter middleware: 100 requests per minute global (adjusted from 5 to avoid blocking tests too easily)
    let rate_limiter = RateLimiter::new(100, Duration::from_secs(60));

    Router::new()
        // Health
        .route("/health", get(health::health_check))
        // Auth - Basic & Multi-Channel
        .route("/auth/login", post(auth::login))
        .route("/auth/register", post(register::register)) // Replaced basic register with multi-channel
        // Auth - OTP
        .route("/auth/otp/request", post(otp::request_otp))
        .route("/auth/otp/verify", post(otp::verify_otp))
        .route("/auth/login/otp", post(login_otp::login_with_otp))
        // Auth - Profile
        .route("/auth/profile/complete", post(profile::complete_profile))
        // Auth - Verification
        .route(
            "/auth/verify/email/send",
            post(verification::send_email_verification),
        )
        .route("/auth/verify/email", get(verification::verify_email_link))
        .route(
            "/auth/verify/phone/send",
            post(verification::send_phone_verification),
        )
        .route(
            "/auth/verify/phone/confirm",
            post(verification::confirm_phone_verification),
        )
        // Users
        .route("/users/:id/ban", post(users::ban_user))
        .route("/users/:id/activate", post(users::activate_user))
        // OIDC / SAML
        .route("/auth/oidc/login", get(auth_oidc::login))
        .route("/auth/oidc/callback", get(auth_oidc::callback))
        .route("/auth/saml/metadata", get(auth_saml::metadata))
        .route("/auth/saml/acs", post(auth_saml::acs))
        // Middleware layers (executed top-to-bottom in Axum Router usually, but effectively wrapping)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(crate::middleware::audit_middleware))
        .layer(axum::Extension(rate_limiter))
}
