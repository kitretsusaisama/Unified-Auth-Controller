use crate::error::ApiError;
use crate::AppState;
use auth_core::error::AuthError;
use axum::{
    extract::{Form, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequestState {
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: String,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub user_id: Option<Uuid>,
}

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

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>, // Basic Auth or Post body
    pub code_verifier: Option<String>, // PKCE
    pub refresh_token: Option<String>,
}

// ============================================================================
// Authorize Endpoint (GET /auth/authorize)
// ============================================================================

pub async fn authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<AuthorizeParams>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Validate Client ID & Redirect URI
    // In production, verify client_id exists in DB and redirect_uri is allowed.
    if params.client_id.is_empty() {
        return Err(ApiError::new(AuthError::ValidationError {
            message: "client_id required".to_string(),
        }));
    }

    // 2. Validate Response Type
    if params.response_type != "code" {
        return Err(ApiError::new(AuthError::ValidationError {
            message: "unsupported response_type".to_string(),
        }));
    }

    // 3. Check for Session Cookie
    let mut user_id: Option<Uuid> = None;

    // Extract "token" cookie
    // Cookie format: token=...;
    if let Some(cookie_header) = headers.get("cookie").and_then(|h| h.to_str().ok()) {
        for cookie in cookie_header.split(';') {
            let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
            if parts.len() == 2 && parts[0] == "token" {
                let token = parts[1];
                // Validate Session
                if let Ok(session) = state.session_service.validate_session(token).await {
                    user_id = Some(session.user_id);
                }
            }
        }
    }

    // If not authenticated, redirect to login
    if user_id.is_none() {
        // Construct return_to URL
        // Simple manual construction for now
        let return_to = format!(
            "/auth/authorize?response_type={}&client_id={}&redirect_uri={}&state={}",
            params.response_type, params.client_id, params.redirect_uri, params.state
        );

        // Assuming we have a frontend login page at /auth/login (or API that serves it)
        // For API-only, we might return 401, but OIDC flows usually redirect.
        // We'll redirect to a login handler that eventually sets the cookie and redirects back.
        return Ok(Redirect::to(&format!(
            "/auth/login?return_to={}",
            urlencoding::encode(&return_to)
        )));
    }

    // 4. User is authenticated. Generate Authorization Code.
    let code = Uuid::new_v4().to_string(); // In prod use crypto secure random string

    // 5. Store Request State in Cache
    let auth_req = AuthRequestState {
        client_id: params.client_id,
        redirect_uri: params.redirect_uri.clone(),
        scope: params.scope,
        state: params.state.clone(),
        nonce: params.nonce,
        code_challenge: params.code_challenge,
        code_challenge_method: params.code_challenge_method,
        user_id,
    };

    let val_str =
        serde_json::to_string(&auth_req).map_err(|_| ApiError::new(AuthError::InternalError))?;
    let cache_key = format!("auth_code:{}", code);
    state
        .cache
        .set(&cache_key, &val_str, Duration::from_secs(600))
        .await // 10 mins TTL
        .map_err(|_e| ApiError::new(AuthError::InternalError))?; // Log error in real app

    // 6. Redirect to Client
    let target = format!(
        "{}?code={}&state={}",
        params.redirect_uri, code, params.state
    );
    Ok(Redirect::to(&target))
}

// ============================================================================
// Token Endpoint (POST /auth/token)
// ============================================================================

pub async fn token(
    State(state): State<AppState>,
    Form(payload): Form<TokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match payload.grant_type.as_str() {
        "authorization_code" => {
            let code = payload
                .code
                .ok_or(ApiError::new(AuthError::ValidationError {
                    message: "code required".to_string(),
                }))?;

            // 1. Retrieve from Cache
            let cache_key = format!("auth_code:{}", code);
            let val_opt = state
                .cache
                .get(&cache_key)
                .await
                .map_err(|_| ApiError::new(AuthError::InternalError))?;

            let val_str = val_opt.ok_or(ApiError::new(AuthError::InvalidCredentials))?; // Invalid or expired code
            let auth_req: AuthRequestState = serde_json::from_str(&val_str)
                .map_err(|_| ApiError::new(AuthError::InternalError))?;

            // 2. Validate Client
            if auth_req.client_id != payload.client_id {
                return Err(ApiError::new(AuthError::ValidationError {
                    message: "client_id mismatch".to_string(),
                }));
            }

            // 3. Validate Redirect URI
            if let Some(uri) = payload.redirect_uri {
                if uri != auth_req.redirect_uri {
                    return Err(ApiError::new(AuthError::ValidationError {
                        message: "redirect_uri mismatch".to_string(),
                    }));
                }
            }

            // 4. PKCE Validation
            if let Some(challenge) = auth_req.code_challenge {
                let verifier =
                    payload
                        .code_verifier
                        .ok_or(ApiError::new(AuthError::ValidationError {
                            message: "code_verifier required".to_string(),
                        }))?;

                // Only S256 supported for MNC grade (plain is deprecated/insecure)
                if auth_req.code_challenge_method.as_deref() == Some("S256") {
                    let mut hasher = Sha256::new();
                    hasher.update(verifier.as_bytes());
                    let result = hasher.finalize();
                    let computed_challenge = URL_SAFE_NO_PAD.encode(result);

                    if computed_challenge != challenge {
                        return Err(ApiError::new(AuthError::ValidationError {
                            message: "PKCE verification failed".to_string(),
                        }));
                    }
                } else {
                    // Reject plain or other methods
                    return Err(ApiError::new(AuthError::ValidationError {
                        message: "Only S256 PKCE supported".to_string(),
                    }));
                }
            }

            // 5. Issue Tokens
            let user_id = auth_req
                .user_id
                .ok_or(ApiError::new(AuthError::InternalError))?;

            // Fetch user details to pass to issue_tokens
            let user = state
                .identity_service
                .get_user(user_id)
                .await
                .map_err(ApiError::from)?;
            let tenant_id = Uuid::new_v4(); // Should come from user context

            let token_response = state
                .identity_service
                .issue_tokens_for_user(&user, tenant_id, Some(payload.client_id), auth_req.scope)
                .await
                .map_err(ApiError::from)?;

            // 6. Delete Code (Replay Protection)
            let _ = state.cache.delete(&cache_key).await;

            // 7. Generate ID Token (Simulated for now as IdentityService doesn't generate it yet, but Access Token is real)
            // In a full implementation, IdentityService should generate ID Token too.
            // We'll mock the ID token content with the same claims but formatted as JWT.
            // For now, we return the access token and refresh token.

            Ok(Json(serde_json::json!({
                "access_token": token_response.access_token,
                "token_type": "Bearer",
                "expires_in": 900, // 15 mins
                "refresh_token": token_response.refresh_token,
                "id_token": "mock_id_token_jwt" // Placeholder: Requires RSA signing which is complex to add here without auth-crypto helper
            })))
        }
        "client_credentials" => {
            // 1. Verify Client ID & Secret
            // In a real implementation, look up client in DB and verify secret (bcrypt/argon2)
            if payload.client_id.is_empty() || payload.client_secret.is_none() {
                return Err(ApiError::new(AuthError::ValidationError {
                    message: "client_id and client_secret required".to_string(),
                }));
            }

            // Mock verify: assume client_123 / secret_123 is valid
            if payload.client_id != "client_123"
                || payload.client_secret.as_deref() != Some("secret_123")
            {
                return Err(ApiError::new(AuthError::InvalidCredentials));
            }

            // 2. Issue Tokens
            let user_id = Uuid::new_v4(); // Service Account ID
            let tenant_id = Uuid::new_v4();

            // Create "Service User" struct on the fly or fetch
            let user = auth_core::models::User {
                id: user_id,
                identifier_type: auth_core::models::user::IdentifierType::Email,
                primary_identifier: auth_core::models::user::PrimaryIdentifier::Email,
                email: Some(format!("service-account@{}", payload.client_id)),
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
                profile_data: serde_json::json!({"type": "service_account"}),
                preferences: serde_json::json!({}),
                status: auth_core::models::user::UserStatus::Active,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                deleted_at: None,
                email_verified_at: None,
                phone_verified_at: None,
            };

            let token_response = state
                .identity_service
                .issue_tokens_for_user(&user, tenant_id, Some(payload.client_id), None)
                .await
                .map_err(ApiError::from)?;

            Ok(Json(serde_json::json!({
                "access_token": token_response.access_token,
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": token_response.refresh_token
            })))
        }
        "refresh_token" => {
            // TODO: Implement refresh logic using IdentityService
            Err(ApiError::new(AuthError::ValidationError {
                message: "grant_type not implemented".to_string(),
            }))
        }
        _ => Err(ApiError::new(AuthError::ValidationError {
            message: "unsupported grant_type".to_string(),
        })),
    }
}

// ============================================================================
// UserInfo Endpoint (GET /auth/userinfo)
// ============================================================================

pub async fn userinfo(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Extract Bearer Token
    let token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(ApiError::new(AuthError::Unauthorized {
            message: "Missing token".to_string(),
        }))?;

    // 2. Validate Token using IdentityService
    let claims = state
        .identity_service
        .validate_token(token)
        .await
        .map_err(ApiError::from)?;

    // 3. Extract User ID
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        ApiError::new(AuthError::TokenError {
            kind: auth_core::error::TokenErrorKind::Invalid,
        })
    })?;

    // 4. Fetch User Profile
    let user = state
        .identity_service
        .get_user(user_id)
        .await
        .map_err(ApiError::from)?;

    // 5. Construct Response
    // OIDC standard claims
    Ok(Json(serde_json::json!({
        "sub": user.id.to_string(),
        "name": user.profile_data.get("name").unwrap_or(&serde_json::json!("Unknown")),
        "email": user.email,
        "email_verified": user.email_verified,
        "phone_number": user.phone,
        "phone_number_verified": user.phone_verified,
        "updated_at": user.updated_at.timestamp(),
    })))
}
