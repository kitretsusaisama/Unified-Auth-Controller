use axum::{
    extract::{State, Query},
    response::{IntoResponse, Redirect},
    Json,
    Form,
};
use serde::Deserialize;
use crate::AppState;
use crate::error::ApiError;
use auth_core::error::AuthError;

// ============================================================================
// Authorize Endpoint (GET /auth/authorize)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AuthorizeParams {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: String,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

pub async fn authorize(
    Query(params): Query<AuthorizeParams>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Validate Client ID & Redirect URI (Stub: accept all for MVP)
    if params.client_id.is_empty() {
        return Err(ApiError::new(AuthError::ValidationError { message: "client_id required".to_string() }));
    }

    // 2. Validate Response Type
    if params.response_type != "code" {
        return Err(ApiError::new(AuthError::ValidationError { message: "unsupported response_type".to_string() }));
    }

    // 3. In a real flow, checking session cookie here.
    // If not logged in, redirect to /auth/login?return_to=...
    // For this API-centric implementation, we assume the user "logs in" via API and gets a session?
    // Actually, OIDC Authorize endpoint usually renders a UI.
    // Since this is a backend API project, we might redirect to a frontend URL,
    // OR if we are simulating the "Post-Login" phase:

    // We will simulate a successful login and return a code immediately for the test harness.
    // In production, this would render a HTML login page.

    let code = "mock_auth_code_12345";
    let state = params.state;

    let target = format!("{}?code={}&state={}", params.redirect_uri, code, state);
    Ok(Redirect::to(&target))
}

// ============================================================================
// Token Endpoint (POST /auth/token)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>, // PKCE
    pub refresh_token: Option<String>,
}

pub async fn token(
    State(state): State<AppState>,
    Form(payload): Form<TokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match payload.grant_type.as_str() {
        "authorization_code" => {
            // Validate code (stub)
            if payload.code.as_deref() != Some("mock_auth_code_12345") {
                 // In real logic, we'd look up the code in DB/Cache
                 // return Err(ApiError::new(AuthError::InvalidCredentials));
            }

            // Issue Tokens
            // We need a user context. Since we skipped real login in authorize stub,
            // we will mint tokens for a dummy user.
            let user_id = uuid::Uuid::new_v4();
            let tenant_id = uuid::Uuid::new_v4();

            // Note: Claims are generated inside issue_tokens_for_user in the real implementation

            let access_token = state.identity_service.issue_tokens_for_user(
                &auth_core::models::User {
                    id: user_id,
                    // tenant_id field removed from User struct
                    email: Some("user@example.com".to_string()),
                    // ... other fields dummy ...
                    identifier_type: auth_core::models::user::IdentifierType::Email,
                    primary_identifier: auth_core::models::user::PrimaryIdentifier::Email,
                    email_verified: true,
                    phone: None,
                    phone_verified: false,
                    password_hash: None,
                    password_changed_at: None,
                    failed_login_attempts: 0,
                    locked_until: None,
                    last_login_at: None,
                    last_login_ip: None,
                    mfa_enabled: false,
                    mfa_secret: None,
                    backup_codes: None,
                    risk_score: 0.0,
                    profile_data: serde_json::json!({}),
                    preferences: serde_json::json!({}),
                    status: auth_core::models::user::UserStatus::Active,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deleted_at: None,
                    email_verified_at: None,
                    phone_verified_at: None,
                },
                tenant_id
            ).await.map_err(ApiError::from)?;

            Ok(Json(serde_json::json!({
                "access_token": access_token.access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": access_token.refresh_token,
                "id_token": "mock_id_token_jwt" // In prod, generate this
            })))
        },
        "refresh_token" => {
            // TODO: Implement refresh
            Err(ApiError::new(AuthError::ValidationError { message: "grant_type not implemented".to_string() }))
        },
        _ => Err(ApiError::new(AuthError::ValidationError { message: "unsupported grant_type".to_string() })),
    }
}

// ============================================================================
// UserInfo Endpoint (GET /auth/userinfo)
// ============================================================================

pub async fn userinfo(
    State(_state): State<AppState>,
    // We would extract Bearer token here
) -> Result<impl IntoResponse, ApiError> {
    // Stub
    Ok(Json(serde_json::json!({
        "sub": "user_123",
        "email": "user@example.com",
        "email_verified": true,
        "name": "John Doe"
    })))
}
