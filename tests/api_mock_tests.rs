//! API Tests for SSO Platform
//!
//! This file contains tests for the API endpoints focusing on the core functionality.

use std::sync::Arc;
use auth_api::{AppState, app};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tokio;
use tower::ServiceExt;

// Create a helper function to build a minimal test app state
fn create_test_app_state() -> AppState {
    // For testing purposes, we'll create a dummy pool
    let pool = sqlx::pool::PoolOptions::<sqlx::MySql>::new()
        .max_connections(1)
        .connect_lazy("mysql://root:password@127.0.0.1:3306/mysql")  // Use system database for tests
        .expect("Could not create dummy pool for tests");

    // Create a minimal app state with essential services using the actual implementations
    AppState {
        db: pool.clone(),
        identity_service: Arc::new(auth_core::services::identity::IdentityService::new(
            Arc::new(auth_db::repositories::UserRepository::new(pool.clone())),
            Arc::new(auth_core::services::token_service::TokenEngine::new_with_defaults())
        )),
        session_service: Arc::new(auth_core::services::session_service::SessionService::new(
            Arc::new(auth_db::repositories::SessionRepository::new(pool.clone())),
            Arc::new(auth_core::services::risk_assessment::RiskEngine::new())
        )),
        role_service: Arc::new(auth_core::services::role_service::RoleService::new(
            Arc::new(auth_db::repositories::RoleRepository::new(pool.clone()))
        )),
        subscription_service: Arc::new(auth_core::services::subscription_service::SubscriptionService::new(
            Arc::new(auth_db::repositories::SubscriptionRepository::new(pool.clone()))
        )),
        otp_service: Arc::new(auth_core::services::otp_service::OtpService::new()),
        otp_delivery_service: Arc::new(auth_core::services::otp_delivery::OtpDeliveryService::new(
            Arc::new(auth_core::services::otp_delivery::MockSmsProvider {}),
            Arc::new(auth_core::services::otp_delivery::MockEmailProvider {})
        )),
        lazy_registration_service: Arc::new(auth_core::services::lazy_registration::LazyRegistrationService::new(
            Arc::new(auth_core::services::identity::IdentityService::new(
                Arc::new(auth_db::repositories::UserRepository::new(pool.clone())),
                Arc::new(auth_core::services::token_service::TokenEngine::new_with_defaults())
            ))
        )),
        rate_limiter: Arc::new(auth_core::services::rate_limiter::RateLimiter::new()),
        otp_repository: Arc::new(auth_db::repositories::OtpRepository::new(pool.clone())),
        audit_logger: Arc::new(auth_core::audit::TracingAuditLogger),
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_routes_exist() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    // Test that registration route exists (even if it returns an error due to validation)
    let register_request = json!({
        "email": "test@example.com",
        "password": "SecurePass123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return either OK or BAD_REQUEST (not 404)
    assert!(response.status() != StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_ready_endpoint() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}