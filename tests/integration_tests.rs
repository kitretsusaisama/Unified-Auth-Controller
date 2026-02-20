//! Integration Tests for SSO Platform
//!
//! This file contains integration tests that test the complete workflow
//! of the SSO platform with mocked external dependencies where possible.

use async_trait::async_trait;
use auth_api::{app, AppState};
use auth_cache::MultiLevelCache;
use auth_core::audit::TracingAuditLogger;
use auth_core::error::AuthError;
use auth_core::models::token::{AccessToken, Claims, RefreshToken, TokenPair};
use auth_core::models::user::{CreateUserRequest, UpdateUserRequest, User, UserStatus};
use auth_core::services::{
    authorization::AuthorizationService, identity::IdentityService,
    lazy_registration::LazyRegistrationService, otp_delivery::OtpDeliveryService,
    otp_service::OtpService, rate_limiter::RateLimiter, session_service::SessionService,
    subscription_service::SubscriptionService,
};
use auth_core::services::{
    identity::UserStore,
    otp_delivery::{DeliveryError, EmailProvider, OtpProvider},
    tenant_service::TenantStore,
    token_service::{TokenIntrospectionResponse, TokenProvider},
};
use auth_core::models::tenant::Tenant;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tower::util::ServiceExt;
use uuid::Uuid;

// Comprehensive Mock Services
struct MockServices {
    token_service: Arc<dyn TokenProvider>,
    user_store: Arc<dyn UserStore>,
    sms_provider: Arc<dyn OtpProvider>,
    email_provider: Arc<dyn EmailProvider>,
}

impl MockServices {
    fn new() -> Self {
        Self {
            token_service: Arc::new(MockTokenService),
            user_store: Arc::new(MockUserStore),
            sms_provider: Arc::new(MockSmsProvider),
            email_provider: Arc::new(MockEmailProvider),
        }
    }
}

// Mock Token Service
struct MockTokenService;

#[async_trait]
impl TokenProvider for MockTokenService {
    async fn issue_access_token(&self, claims: Claims) -> Result<AccessToken, AuthError> {
        Ok(AccessToken {
            token: format!("access_token_{}", claims.sub),
            token_type: "Bearer".to_string(),
            expires_in: 1800,
            scope: None,
        })
    }

    async fn issue_refresh_token(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<RefreshToken, AuthError> {
        Ok(RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            tenant_id,
            token_family: Uuid::new_v4(),
            token_hash: format!("hash_{}", user_id),
            device_fingerprint: None,
            user_agent: None,
            ip_address: None,
            expires_at: Utc::now() + Duration::days(30),
            revoked_at: None,
            revoked_reason: None,
            created_at: Utc::now(),
        })
    }

    async fn refresh_tokens(&self, _refresh_token: &str) -> Result<TokenPair, AuthError> {
        Ok(TokenPair {
            access_token: AccessToken {
                token: format!("new_access_token_{}", Uuid::new_v4()),
                token_type: "Bearer".to_string(),
                expires_in: 1800,
                scope: None,
            },
            refresh_token: format!("new_refresh_token_{}", Uuid::new_v4()),
        })
    }

    async fn validate_token(&self, _token: &str) -> Result<Claims, AuthError> {
        Ok(Claims {
            sub: Uuid::new_v4().to_string(),
            exp: 9999999999,
            iat: 1234567890,
            nbf: 1234567890,
            iss: "test".to_string(),
            aud: "test".to_string(),
            jti: Uuid::new_v4().to_string(),
            tenant_id: Uuid::new_v4().to_string(),
            roles: vec![],
            permissions: vec![],
            scope: None,
        })
    }

    async fn revoke_token(
        &self,
        _token_jti: Uuid,
        _user_id: Uuid,
        _tenant_id: Uuid,
    ) -> Result<(), AuthError> {
        Ok(())
    }

    async fn introspect_token(
        &self,
        _token: &str,
    ) -> Result<TokenIntrospectionResponse, AuthError> {
        Ok(TokenIntrospectionResponse {
            active: true,
            token_type: Some("Bearer".to_string()),
            exp: Some(9999999999),
            iat: Some(1234567890),
            sub: Some(Uuid::new_v4().to_string()),
            username: None,
            client_id: None,
            aud: None,
            iss: None,
            jti: None,
            nbf: None,
            scope: None,
        })
    }

    async fn get_jwks(&self) -> serde_json::Value {
        json!({ "keys": [] })
    }
}

// Mock User Store
struct MockUserStore;

#[async_trait]
impl UserStore for MockUserStore {
    async fn find_by_email(
        &self,
        email: &str,
        _tenant_id: Uuid,
    ) -> Result<Option<User>, AuthError> {
        if email == "existing@example.com" {
            Ok(Some(mock_user()))
        } else {
            Ok(None)
        }
    }

    async fn find_by_phone(
        &self,
        _phone: &str,
        _tenant_id: Uuid,
    ) -> Result<Option<User>, AuthError> {
        Ok(None)
    }
    async fn find_by_identifier(
        &self,
        _identifier: &str,
        _tenant_id: Uuid,
    ) -> Result<Option<User>, AuthError> {
        Ok(None)
    }
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, AuthError> {
        Ok(Some(mock_user()))
    }

    async fn create(
        &self,
        request: CreateUserRequest,
        _hash: String,
        _tenant_id: Uuid,
    ) -> Result<User, AuthError> {
        let mut user = mock_user();
        user.email = request.email;
        // user.tenant_id = tenant_id; // User struct doesn't have tenant_id
        Ok(user)
    }

    async fn update_status(&self, _id: Uuid, _status: UserStatus) -> Result<(), AuthError> {
        Ok(())
    }
    async fn increment_failed_attempts(&self, _id: Uuid) -> Result<u32, AuthError> {
        Ok(1)
    }
    async fn reset_failed_attempts(&self, _id: Uuid) -> Result<(), AuthError> {
        Ok(())
    }
    async fn record_login(&self, _id: Uuid, _ip: Option<String>) -> Result<(), AuthError> {
        Ok(())
    }
    async fn update(&self, _user: UpdateUserRequest) -> Result<User, AuthError> {
        Ok(mock_user())
    }
    async fn update_password_hash(&self, _id: Uuid, _hash: String) -> Result<(), AuthError> {
        Ok(())
    }
    async fn set_email_verified(&self, _id: Uuid, _verified: bool) -> Result<(), AuthError> {
        Ok(())
    }
    async fn set_phone_verified(&self, _id: Uuid, _verified: bool) -> Result<(), AuthError> {
        Ok(())
    }
}

fn mock_user() -> User {
    User {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        identifier_type: auth_core::models::user::IdentifierType::Email,
        primary_identifier: auth_core::models::user::PrimaryIdentifier::Email,
        email: Some("test@example.com".to_string()),
        email_verified: true,
        email_verified_at: Some(Utc::now()),
        phone: None,
        phone_verified: false,
        phone_verified_at: None,
        password_hash: Some("$argon2id$v=19$m=19456,t=2,p=1$IjRAZWRuZXNzLmpzbg$JhD+KrWxA+vZ5sZ/oOUmg8WFH5VG2XwZF6RpcXYXKKc".to_string()),
        password_changed_at: None,
        failed_login_attempts: 0,
        locked_until: None,
        last_login_at: None,
        last_login_ip: None,
        mfa_enabled: false,
        mfa_secret: None,
        backup_codes: None,
        risk_score: 0.0,
        profile_data: serde_json::Value::Null,
        preferences: serde_json::Value::Null,
        status: UserStatus::Active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        // tenant_id: Uuid::new_v4(), // Removed
    }
}

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

fn create_test_app_state() -> AppState {
    let mock_services = MockServices::new();
    let audit_logger: Arc<dyn auth_core::audit::AuditLogger> = Arc::new(TracingAuditLogger);

    let identity_service = Arc::new(IdentityService::new(
        mock_services.user_store,
        mock_services.token_service,
        audit_logger.clone(),
    ));

    // Create a dummy MySQL pool for the test - this won't actually connect in unit tests usually, but State needs it
    // In a real integration test, we would need a real DB or a mock DB.
    // For this "integration" test which mostly mocks repositories, we just need a valid Pool struct.
    // However, Repository::new(pool) might fail if it tries to prepare statements immediately (it usually doesn't).
    // sqlx::MySqlPool::connect_lazy allows creating a pool without immediate connection.
    let pool: Pool<MySql> = sqlx::pool::PoolOptions::new()
        .max_connections(1)
        .connect_lazy("mysql://dummy:dummy@127.0.0.1:3306/dummy")
        .expect("Could not create dummy pool");

    // Use REAL repositories with dummy pool
    let role_repo = Arc::new(auth_db::repositories::RoleRepository::new(pool.clone()));
    let session_repo =
        Arc::new(auth_db::repositories::session_repository::SessionRepository::new(pool.clone()));
    let subscription_repo = Arc::new(
        auth_db::repositories::subscription_repository::SubscriptionRepository::new(pool.clone()),
    );
    let otp_repo = Arc::new(auth_db::repositories::otp_repository::OtpRepository::new(
        pool.clone(),
    ));

    let session_service = Arc::new(SessionService::new(
        session_repo,
        Arc::new(auth_core::services::risk_assessment::RiskEngine::new()),
    ));
    // Updated to AuthorizationService
    let role_service = Arc::new(AuthorizationService::new(role_repo));
    let subscription_service = Arc::new(SubscriptionService::new(subscription_repo));
    let otp_service = Arc::new(OtpService::new());
    let otp_delivery_service = Arc::new(OtpDeliveryService::new(
        mock_services.sms_provider,
        mock_services.email_provider,
    ));
    let lazy_registration_service =
        Arc::new(LazyRegistrationService::new(identity_service.clone(), Arc::new(MockTenantStore)));
    let rate_limiter = Arc::new(RateLimiter::new());
    let cache = Arc::new(MultiLevelCache::new(None).unwrap());

    AppState {
        db: pool,
        identity_service,
        session_service,
        role_service,
        subscription_service,
        otp_service,
        otp_delivery_service,
        lazy_registration_service,
        rate_limiter,
        otp_repository: otp_repo,
        audit_logger,
        cache,
    }
}

// Helper for one-shot requests
// ... (Already in tests)

#[tokio::test]
async fn test_registration_endpoint() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    let register_request = json!({
        "identifier_type": "email",
        "email": "newuser@example.com",
        "password": "SecurePass123!",
        "tenant_id": Uuid::new_v4(),
        "require_verification": false
    });

    let response = app
        .clone()
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

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_login_endpoint() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    let login_request = json!({
        "email": "existing@example.com",
        "password": "password",
        "tenant_id": Uuid::new_v4()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_health_check() {
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
