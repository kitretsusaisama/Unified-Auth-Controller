//! API Tests for SSO Platform
//!
//! This file contains tests for the API endpoints focusing on the core functionality.

use async_trait::async_trait;
use auth_api::{app, AppState};
use auth_cache::MultiLevelCache;
use auth_core::services::identity::IdentityService;
use auth_core::services::otp_delivery::{DeliveryError, EmailProvider, OtpProvider};
use auth_core::services::token_service::TokenEngine;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use std::sync::Arc;
use tower::util::ServiceExt;
use auth_core::error::AuthError;
use auth_core::models::tenant::Tenant;
use auth_core::services::tenant_service::TenantStore;
use chrono::Utc;
use uuid::Uuid;

// Mock OTP Provider
struct MockSmsProvider;
#[async_trait]
impl OtpProvider for MockSmsProvider {
    async fn send_otp(&self, _to: &str, _otp: &str) -> Result<String, DeliveryError> {
        Ok("sent".to_string())
    }
}

// Mock Email Provider
struct MockEmailProvider;
#[async_trait]
impl EmailProvider for MockEmailProvider {
    async fn send_email(
        &self,
        _to: &str,
        _sub: &str,
        _body: &str,
    ) -> Result<String, DeliveryError> {
        Ok("sent".to_string())
    }
}

// Mock Tenant Store
struct MockTenantStore;

#[async_trait]
impl TenantStore for MockTenantStore {
    async fn get_tenant(&self, _tenant_id: Uuid) -> Result<Option<Tenant>, AuthError> {
        Ok(Some(Tenant {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "Mock Tenant".to_string(),
            slug: "mock-tenant".to_string(),
            custom_domain: None,
            branding_config: serde_json::Value::Null,
            auth_config: json!({ "allow_lazy_registration": true }),
            compliance_config: serde_json::Value::Null,
            status: auth_core::models::tenant::TenantStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }))
    }
}

async fn create_test_app_state() -> AppState {
    // For testing purposes, we'll create a dummy pool
    let pool = sqlx::pool::PoolOptions::<sqlx::MySql>::new()
        .max_connections(1)
        .connect_lazy("mysql://dummy:dummy@127.0.0.1:3306/dummy")
        .expect("Could not create dummy pool for tests");

    let audit_logger: Arc<dyn auth_core::audit::AuditLogger> =
        Arc::new(auth_core::audit::TracingAuditLogger);

    // We use real implementations with dummy DB.
    // TokenEngine::new() creates in-memory stores.
    let token_service = Arc::new(TokenEngine::new().await.unwrap());

    let identity_service = Arc::new(IdentityService::new(
        Arc::new(auth_db::repositories::user_repository::UserRepository::new(
            pool.clone(),
        )),
        token_service,
        audit_logger.clone(),
    ));

    AppState {
        db: pool.clone(),
        identity_service: identity_service.clone(),
        session_service: Arc::new(auth_core::services::session_service::SessionService::new(
            Arc::new(
                auth_db::repositories::session_repository::SessionRepository::new(pool.clone()),
            ),
            Arc::new(auth_core::services::risk_assessment::RiskEngine::new()),
        )),
        // Updated to AuthorizationService
        role_service: Arc::new(
            auth_core::services::authorization::AuthorizationService::new(Arc::new(
                auth_db::repositories::RoleRepository::new(pool.clone()),
            )),
        ),
        subscription_service: Arc::new(
            auth_core::services::subscription_service::SubscriptionService::new(Arc::new(
                auth_db::repositories::subscription_repository::SubscriptionRepository::new(
                    pool.clone(),
                ),
            )),
        ),
        otp_service: Arc::new(auth_core::services::otp_service::OtpService::new()),
        otp_delivery_service: Arc::new(auth_core::services::otp_delivery::OtpDeliveryService::new(
            Arc::new(MockSmsProvider {}),
            Arc::new(MockEmailProvider {}),
        )),
        lazy_registration_service: Arc::new(
            auth_core::services::lazy_registration::LazyRegistrationService::new(identity_service, Arc::new(MockTenantStore)),
        ),
        rate_limiter: Arc::new(auth_core::services::rate_limiter::RateLimiter::new()),
        otp_repository: Arc::new(auth_db::repositories::otp_repository::OtpRepository::new(
            pool.clone(),
        )),
        audit_logger,
        cache: Arc::new(MultiLevelCache::new(None).unwrap()),
    }
}

#[tokio::test]
async fn test_health_endpoint() {
    let app_state = create_test_app_state().await;
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
    let app_state = create_test_app_state().await;
    let app = app(app_state);

    // Test that registration route exists (even if it returns an error due to validation or DB)
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

    // Should return BAD_REQUEST (due to missing fields like identifier_type) or 500 (DB error) or 404 (if missing)
    // We assert it is NOT 404
    assert!(response.status() != StatusCode::NOT_FOUND);
}
