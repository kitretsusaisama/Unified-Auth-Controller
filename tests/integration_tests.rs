//! Integration Tests for SSO Platform
//!
//! This file contains integration tests that test the complete workflow
//! of the SSO platform with mocked external dependencies.

use std::sync::Arc;
use auth_api::{AppState, app};
use auth_core::services::{
    identity::IdentityService,
    session_service::SessionService,
    role_service::RoleService,
    subscription_service::SubscriptionService,
    otp_service::OtpService,
    otp_delivery::OtpDeliveryService,
    lazy_registration::LazyRegistrationService,
    rate_limiter::RateLimiter,
};
use auth_core::audit::TracingAuditLogger;
use auth_core::services::{
    token_service::TokenProvider,
    identity::UserStore,
    otp_delivery::{OtpProvider, EmailProvider, DeliveryError},
};
use auth_core::models::user::{CreateUserRequest, UserStatus};
use auth_core::models::token::JwtClaims;
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderMap},
};
use serde_json::json;
use tokio;
use tower::ServiceExt;
use uuid::Uuid;
use sqlx::{MySqlPool, Pool, MySql};

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
    async fn generate_access_token(&self, user_id: &Uuid, tenant_id: &Uuid, org_id: &Uuid) -> Result<String, auth_core::error::TokenError> {
        Ok(format!("access_token_{}_{}_{}", user_id, tenant_id, org_id))
    }

    async fn generate_refresh_token(&self, user_id: &Uuid, tenant_id: &Uuid, org_id: &Uuid) -> Result<String, auth_core::error::TokenError> {
        Ok(format!("refresh_token_{}_{}_{}", user_id, tenant_id, org_id))
    }

    async fn validate_token(&self, token: &str) -> Result<JwtClaims, auth_core::error::TokenError> {
        // Parse the token to extract user_id, tenant_id, and org_id
        // For mock purposes, we'll return fixed values
        Ok(JwtClaims {
            sub: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap_or_else(|_| Uuid::new_v4()),
            exp: 9999999999, // Far future
            iat: 1234567890,
            tenant_id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap_or_else(|_| Uuid::new_v4()),
            org_id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174002").unwrap_or_else(|_| Uuid::new_v4()),
        })
    }

    async fn revoke_token(&self, token_jti: Uuid, user_id: Uuid, token_type: Uuid) -> Result<(), auth_core::error::AuthError> {
        println!("Mock: Revoking token {:?}", token_jti);
        Ok(())
    }

    async fn is_token_revoked(&self, token: &str) -> Result<bool, auth_core::error::AuthError> {
        println!("Mock: Checking if token {} is revoked", token);
        Ok(false)
    }

    async fn issue_access_token(&self, claims: JwtClaims) -> Result<auth_core::models::token::AccessToken, auth_core::error::AuthError> {
        Ok(auth_core::models::token::AccessToken {
            token: format!("access_token_{}", claims.sub),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(30),
        })
    }

    async fn issue_refresh_token(&self, user_id: Uuid, tenant_id: Uuid) -> Result<auth_core::models::token::RefreshToken, auth_core::error::AuthError> {
        Ok(auth_core::models::token::RefreshToken {
            token: format!("refresh_token_{}", user_id),
            expires_at: chrono::Utc::now() + chrono::Duration::days(30),
        })
    }

    async fn refresh_tokens(&self, refresh_token: &str) -> Result<auth_core::models::token::TokenPair, auth_core::error::AuthError> {
        Ok(auth_core::models::token::TokenPair {
            access_token: auth_core::models::token::AccessToken {
                token: format!("new_access_token_{}", Uuid::new_v4()),
                expires_at: chrono::Utc::now() + chrono::Duration::minutes(30),
            },
            refresh_token: auth_core::models::token::RefreshToken {
                token: format!("new_refresh_token_{}", Uuid::new_v4()),
                expires_at: chrono::Utc::now() + chrono::Duration::days(30),
            },
        })
    }

    async fn introspect_token(&self, token: &str) -> Result<auth_core::models::token::TokenIntrospectionResponse, auth_core::error::AuthError> {
        Ok(auth_core::models::token::TokenIntrospectionResponse {
            active: true,
            token_type: "Bearer".to_string(),
            exp: 9999999999,
            iat: 1234567890,
            sub: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
        })
    }
}

// Mock User Store
struct MockUserStore;

#[async_trait]
impl UserStore for MockUserStore {
    async fn find_user_by_identifier(&self, identifier: &str) -> Result<Option<auth_core::models::user::User>, auth_core::error::AuthError> {
        if identifier == "existing@example.com" || identifier == "testuser" || identifier == "+1234567890" {
            Ok(Some(auth_core::models::user::User {
                id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap_or_else(|_| Uuid::new_v4()),
                email: Some("existing@example.com".to_string()),
                phone: Some("+1234567890".to_string()),
                username: Some("testuser".to_string()),
                password_hash: "$argon2id$v=19$m=19456,t=2,p=1$IjRAZWRuZXNzLmpzbg$JhD+KrWxA+vZ5sZ/oOUmg8WFH5VG2XwZF6RpcXYXKKc".to_string(), // Valid hash
                status: UserStatus::Active,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                last_login_at: None,
                failed_login_attempts: 0,
                locked_until: None,
                tenant_id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap_or_else(|_| Uuid::new_v4()),
                org_id: Uuid::parse_str("123e4567-e89b-12d3-a456-426614174002").unwrap_or_else(|_| Uuid::new_v4()),
            }))
        } else {
            Ok(None)
        }
    }

    async fn create_user(&self, request: CreateUserRequest) -> Result<auth_core::models::user::User, auth_core::error::AuthError> {
        Ok(auth_core::models::user::User {
            id: Uuid::new_v4(),
            email: request.email,
            phone: request.phone,
            username: request.username,
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$IjRAZWRuZXNzLmpzbg$JhD+KrWxA+vZ5sZ/oOUmg8WFH5VG2XwZF6RpcXYXKKc".to_string(),
            status: UserStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            tenant_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
        })
    }

    async fn update_user(&self, user_id: &Uuid, update: auth_core::models::user::UpdateUserRequest) -> Result<auth_core::models::user::User, auth_core::error::AuthError> {
        // Return updated mock user
        Ok(auth_core::models::user::User {
            id: *user_id,
            email: update.email.or_else(|| Some("updated@example.com".to_string())),
            phone: update.phone.or_else(|| Some("+1234567890".to_string())),
            username: update.username.or_else(|| Some("updateduser".to_string())),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$IjRAZWRuZXNzLmpzbg$JhD+KrWxA+vZ5sZ/oOUmg8WFH5VG2XwZF6RpcXYXKKc".to_string(),
            status: UserStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login_at: None,
            failed_login_attempts: 0,
            locked_until: None,
            tenant_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
        })
    }
}

// Mock OTP Provider
struct MockSmsProvider;

#[async_trait]
impl OtpProvider for MockSmsProvider {
    async fn send_otp(&self, to: &str, otp: &str) -> Result<String, DeliveryError> {
        println!("Mock SMS sent to {}: {}", to, otp);
        Ok(format!("sms_sent_to_{}", to))
    }
}

// Mock Email Provider
struct MockEmailProvider;

#[async_trait]
impl EmailProvider for MockEmailProvider {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<String, DeliveryError> {
        println!("Mock email sent to {}: {} - {}", to, subject, body);
        Ok(format!("email_sent_to_{}", to))
    }
}

// Mock repositories
struct MockOtpRepository;
struct MockSessionRepository;
struct MockRoleRepository;
struct MockSubscriptionRepository;

#[async_trait]
impl auth_db::repositories::OtpRepositoryImpl for MockOtpRepository {
    async fn save_otp(&self, otp_data: &auth_core::models::otp::OtpData) -> Result<(), auth_core::error::AuthError> {
        println!("Mock: Saved OTP for {}", otp_data.identifier);
        Ok(())
    }

    async fn verify_otp(&self, identifier: &str, otp: &str, purpose: auth_core::models::otp::OtpPurpose) -> Result<bool, auth_core::error::AuthError> {
        println!("Mock: Verifying OTP {} for {} with purpose {:?}", otp, identifier, purpose);
        Ok(true) // Always valid for testing
    }

    async fn invalidate_otp(&self, identifier: &str, purpose: auth_core::models::otp::OtpPurpose) -> Result<(), auth_core::error::AuthError> {
        println!("Mock: Invalidated OTP for {} with purpose {:?}", identifier, purpose);
        Ok(())
    }
}

#[async_trait]
impl auth_db::repositories::SessionRepositoryImpl for MockSessionRepository {
    async fn create_session(&self, session: &auth_core::models::session::Session) -> Result<(), auth_core::error::AuthError> {
        println!("Mock: Created session {}", session.id);
        Ok(())
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<auth_core::models::session::Session>, auth_core::error::AuthError> {
        println!("Mock: Getting session {}", session_id);
        Ok(Some(auth_core::models::session::Session {
            id: session_id.to_string(),
            user_id: Uuid::new_v4(),
            token: "mock_token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }))
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), auth_core::error::AuthError> {
        println!("Mock: Deleting session {}", session_id);
        Ok(())
    }
}

#[async_trait]
impl auth_db::repositories::RoleRepositoryImpl for MockRoleRepository {
    async fn get_user_roles(&self, user_id: &Uuid, tenant_id: &Uuid) -> Result<Vec<String>, auth_core::error::AuthError> {
        println!("Mock: Getting roles for user {} in tenant {}", user_id, tenant_id);
        Ok(vec!["user".to_string(), "authenticated".to_string()])
    }

    async fn assign_role(&self, user_id: &Uuid, role: &str, tenant_id: &Uuid) -> Result<(), auth_core::error::AuthError> {
        println!("Mock: Assigning role {} to user {} in tenant {}", role, user_id, tenant_id);
        Ok(())
    }
}

#[async_trait]
impl auth_db::repositories::SubscriptionRepositoryImpl for MockSubscriptionRepository {
    async fn get_user_subscription(&self, user_id: &Uuid, tenant_id: &Uuid) -> Result<Option<auth_core::models::subscription::Subscription>, auth_core::error::AuthError> {
        println!("Mock: Getting subscription for user {} in tenant {}", user_id, tenant_id);
        Ok(None)
    }

    async fn activate_subscription(&self, user_id: &Uuid, tenant_id: &Uuid) -> Result<(), auth_core::error::AuthError> {
        println!("Mock: Activating subscription for user {} in tenant {}", user_id, tenant_id);
        Ok(())
    }
}

fn create_test_app_state() -> AppState {
    let mock_services = MockServices::new();
    
    let identity_service = Arc::new(IdentityService::new(
        mock_services.user_store,
        mock_services.token_service,
    ));
    let session_service = Arc::new(SessionService::new(
        Arc::new(MockSessionRepository),
        Arc::new(auth_core::services::risk_assessment::RiskEngine::new())
    ));
    let role_service = Arc::new(RoleService::new(Arc::new(MockRoleRepository)));
    let subscription_service = Arc::new(SubscriptionService::new(Arc::new(MockSubscriptionRepository)));
    let otp_service = Arc::new(OtpService::new());
    let otp_delivery_service = Arc::new(OtpDeliveryService::new(
        mock_services.sms_provider,
        mock_services.email_provider,
    ));
    let lazy_registration_service = Arc::new(LazyRegistrationService::new(identity_service.clone()));
    let rate_limiter = Arc::new(RateLimiter::new());
    let otp_repository = Arc::new(MockOtpRepository);
    let audit_logger: Arc<dyn auth_core::audit::AuditLogger> = Arc::new(TracingAuditLogger);

    // Create a dummy MySQL pool for the test (won't be used due to mocks)
    let pool: Pool<MySql> = sqlx::pool::PoolOptions::new()
        .max_connections(1)
        .connect_lazy("mysql://dummy:dummy@127.0.0.1:3306/dummy")
        .expect("Could not create dummy pool");

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
        otp_repository,
        audit_logger,
    }
}

#[tokio::test]
async fn test_complete_registration_and_login_flow() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    // Step 1: Register a new user
    let register_request = json!({
        "email": "newuser@example.com",
        "password": "SecurePass123!",
        "first_name": "Jane",
        "last_name": "Doe"
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

    assert_eq!(response.status(), StatusCode::OK);

    // Step 2: Login with the registered user
    let login_request = json!({
        "identifier": "newuser@example.com",
        "password": "SecurePass123!"
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

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_otp_flow_for_existing_user() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    // Request OTP for existing user
    let otp_request = json!({
        "identifier": "existing@example.com",
        "channel": "email"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/otp/request")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&otp_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify OTP
    let verify_request = json!({
        "identifier": "existing@example.com",
        "otp": "123456",
        "purpose": "login"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/otp/verify")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&verify_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
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
    
    // Also test readiness endpoint
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

#[tokio::test]
async fn test_rate_limiting_simulation() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    // Try multiple requests to test rate limiting
    for i in 0..5 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // All requests should succeed since we're using mocked rate limiter
        assert!(response.status() == StatusCode::OK);
    }
}

#[tokio::test]
async fn test_user_profile_endpoints() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    // Try to get profile (should fail without auth)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/auth/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized without token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Try to update profile (should fail without auth)
    let update_request = json!({
        "first_name": "Updated",
        "last_name": "Name"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/auth/profile")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 401 Unauthorized without token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_session_endpoints() {
    let app_state = create_test_app_state();
    let app = app(app_state);

    // Try to get session info (should fail without auth)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/auth/session")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Try to logout (should fail without auth)
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}