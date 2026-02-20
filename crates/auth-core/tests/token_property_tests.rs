//! Property-based tests for Token Security and Lifecycle Management
//! Task 3.2: Write property test for token security and lifecycle
//!
//! This module implements comprehensive property-based tests validating:
//! **Property 3: Token Security and Lifecycle Management**
//!
//! Requirements Validated: 3.1, 3.2, 3.3, 3.4

use auth_core::models::{AccessToken, Claims};
use auth_core::services::token_service::{TokenEngine, TokenProvider};
use auth_crypto::{JwtConfig, JwtService, KeyManager};
use chrono::{Duration, Utc};
use proptest::prelude::*;
use uuid::Uuid;

/// Strategy for generating valid UUIDs
fn uuid_strategy() -> impl Strategy<Value = Uuid> {
    any::<[u8; 16]>().prop_map(|bytes| Uuid::from_bytes(bytes))
}

/// Strategy for generating valid permission strings
fn permission_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(prop::string::string_regex("[a-z]+:[a-z]+").unwrap(), 0..10)
}

/// Strategy for generating valid role strings
fn role_strategy() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(prop::string::string_regex("[a-z_]+").unwrap(), 0..5)
}

/// Strategy for generating valid Claims
fn claims_strategy() -> impl Strategy<Value = Claims> {
    (
        uuid_strategy(),
        uuid_strategy(),
        permission_strategy(),
        role_strategy(),
    )
        .prop_map(|(user_id, tenant_id, permissions, roles)| {
            let now = Utc::now();
            Claims {
                sub: user_id.to_string(),
                iss: "auth-platform".to_string(),
                aud: "auth-platform".to_string(),
                exp: (now + Duration::minutes(15)).timestamp(),
                iat: now.timestamp(),
                nbf: now.timestamp(),
                jti: Uuid::new_v4().to_string(),
                tenant_id: tenant_id.to_string(),
                permissions,
                roles,
                scope: None,
            }
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Property 1: Token Round-Trip Consistency**
    ///
    /// Feature: rust-auth-platform, Property 1
    ///
    /// For any valid claims, encoding and then decoding should yield identical claims
    /// (excluding fields that may vary like JTI if regenerated).
    ///
    /// Validates: All claim fields are preserved exactly through encode/decode cycle
    #[test]
    fn test_token_round_trip_consistency(
        claims in claims_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            // Create token engine
            let engine = TokenEngine::new().await.unwrap();

            // Issue access token
            let access_token = engine.issue_access_token(claims.clone()).await.unwrap();

            // Validate and decode token
            let decoded_claims = engine.validate_token(&access_token.token).await.unwrap();

            // Verify critical fields are preserved
            assert_eq!(decoded_claims.sub, claims.sub, "Subject (user_id) must be preserved");
            assert_eq!(decoded_claims.tenant_id, claims.tenant_id, "Tenant ID must be preserved");
            assert_eq!(decoded_claims.permissions, claims.permissions, "Permissions must be preserved");
            assert_eq!(decoded_claims.roles, claims.roles, "Roles must be preserved");
            assert_eq!(decoded_claims.iss, claims.iss, "Issuer must be preserved");
            assert_eq!(decoded_claims.aud, claims.aud, "Audience must be preserved");
        });
    }

    /// **Property 2: TTL Enforcement**
    ///
    /// Feature: rust-auth-platform, Property 2
    ///
    /// For any token, the expiry time must be ≤ issue time + 15 minutes (900 seconds).
    ///
    /// Validates: All access tokens respect the maximum TTL of 15 minutes
    #[test]
    fn test_ttl_enforcement(
        claims in claims_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let engine = TokenEngine::new().await.unwrap();
            let access_token = engine.issue_access_token(claims.clone()).await.unwrap();

            // Decode to get actual timestamps
            let decoded = engine.validate_token(&access_token.token).await.unwrap();

            // Calculate actual TTL
            let ttl = decoded.exp - decoded.iat;

            // TTL must be ≤ 15 minutes (900 seconds)
            assert!(
                ttl <= 900,
                "Token TTL ({}) must be ≤ 900 seconds (15 minutes)",
                ttl
            );

            // TTL should also be > 0
            assert!(ttl > 0, "Token TTL must be positive");
        });
    }

    /// **Property 3: Expiration Validation**
    ///
    /// Feature: rust-auth-platform, Property 3
    ///
    /// For any expired token, validation must fail with TokenExpired error.
    ///
    /// Validates: Expired tokens are never accepted as valid
    #[test]
    fn test_expiration_validation(
        user_id in uuid_strategy(),
        tenant_id in uuid_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            // Create config with very short TTL
            let mut config = JwtConfig::default();
            config.access_token_ttl = Duration::milliseconds(10); // 10ms TTL

            let engine = TokenEngine::new_with_config(config).await.unwrap();

            // Create claims and issue token
            let claims = Claims {
                sub: user_id.to_string(),
                iss: "auth-platform".to_string(),
                aud: "auth-platform".to_string(),
                exp: (Utc::now() + Duration::milliseconds(10)).timestamp(),
                iat: Utc::now().timestamp(),
                nbf: Utc::now().timestamp(),
                jti: Uuid::new_v4().to_string(),
                tenant_id: tenant_id.to_string(),
                permissions: vec![],
                roles: vec![],
                scope: None,
            };

            let access_token = engine.issue_access_token(claims).await.unwrap();

            // Wait for token to expire
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Token validation must fail
            let result = engine.validate_token(&access_token.token).await;
            assert!(
                result.is_err(),
                "Expired token must fail validation"
            );

            // Verify it's specifically an expiration error
            if let Err(e) = result {
                let error_str = format!("{:?}", e);
                assert!(
                    error_str.contains("Expired") || error_str.contains("expired"),
                    "Error must indicate token expiration, got: {}",
                    error_str
                );
            }
        });
    }

    /// **Property 4: Required Claims Presence**
    ///
    /// Feature: rust-auth-platform, Property 4
    ///
    /// For any token, all required JWT claims must be present:
    /// sub, iss, aud, exp, iat, nbf, jti, tenant_id
    ///
    /// Validates: Every token contains all required fields
    #[test]
    fn test_required_claims_presence(
        claims in claims_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let engine = TokenEngine::new().await.unwrap();
            let access_token = engine.issue_access_token(claims).await.unwrap();
            let decoded = engine.validate_token(&access_token.token).await.unwrap();

            // Verify all required fields are non-empty/valid
            assert!(!decoded.sub.is_empty(), "sub (subject) must be present");
            assert!(!decoded.iss.is_empty(), "iss (issuer) must be present");
            assert!(!decoded.aud.is_empty(), "aud (audience) must be present");
            assert!(decoded.exp > 0, "exp (expiration) must be present");
            assert!(decoded.iat > 0, "iat (issued at) must be present");
            assert!(decoded.nbf >= 0, "nbf (not before) must be present");
            assert!(!decoded.jti.is_empty(), "jti (JWT ID) must be present");
            assert!(!decoded.tenant_id.is_empty(), "tenant_id must be present");

            // Verify exp > iat (expiration is after issuance)
            assert!(
                decoded.exp > decoded.iat,
                "Expiration must be after issuance"
            );
        });
    }

    /// **Property 5: Refresh Token Rotation**
    ///
    /// Feature: rust-auth-platform, Property 5
    ///
    /// For any refresh token, after rotation the old token must be invalidated.
    ///
    /// Validates: Token rotation properly invalidates previous tokens
    #[test]
    fn test_refresh_token_rotation(
        user_id in uuid_strategy()
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let engine = TokenEngine::new().await.unwrap();

            // Issue initial refresh token
            let tenant_id = Uuid::new_v4();
            let refresh_token1 = engine.issue_refresh_token(user_id, tenant_id).await.unwrap();
            let token_hash1 = refresh_token1.token_hash.clone();

            // Use refresh token to get new pair
            let token_pair = engine.refresh_tokens(&token_hash1).await.unwrap();

            // Old refresh token should be different from new one
            assert_ne!(
                token_hash1,
                token_pair.refresh_token,
                "Rotation must create a new refresh token"
            );

            // Attempting to use old token again should fail
            let result = engine.refresh_tokens(&token_hash1).await;
            assert!(
                result.is_err(),
                "Old refresh token must be invalidated after rotation"
            );
        });
    }
}

/// Additional property tests for signature and algorithm validation

#[tokio::test]
async fn test_token_signature_tampering_detection() {
    /// **Property 6: Signature Tampering Detection**
    ///
    /// Feature: rust-auth-platform, Property 6
    ///
    /// For any token, if the payload is modified, validation must fail.
    ///
    /// Validates: RS256 signature catches any token modification
    let engine = TokenEngine::new().await.unwrap();

    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        iss: "auth-platform".to_string(),
        aud: "auth-platform".to_string(),
        exp: (Utc::now() + Duration::minutes(15)).timestamp(),
        iat: Utc::now().timestamp(),
        nbf: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
        tenant_id: Uuid::new_v4().to_string(),
        permissions: vec!["read:users".to_string()],
        roles: vec!["admin".to_string()],
        scope: None,
    };

    let access_token = engine.issue_access_token(claims).await.unwrap();
    let token = access_token.token;

    // Tamper with the token by modifying a character in the payload
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() == 3 {
        // Modify one character in the payload (middle part)
        let mut payload_bytes = parts[1].as_bytes().to_vec();
        if !payload_bytes.is_empty() {
            payload_bytes[0] = payload_bytes[0].wrapping_add(1); // Flip a bit
            let tampered_payload = String::from_utf8_lossy(&payload_bytes);
            let tampered_token = format!("{}.{}.{}", parts[0], tampered_payload, parts[2]);

            // Validation must fail
            let result = engine.validate_token(&tampered_token).await;
            assert!(result.is_err(), "Tampered token must fail validation");
        }
    }
}

#[tokio::test]
async fn test_algorithm_consistency() {
    /// **Property 7: Algorithm Enforcement**
    ///
    /// Feature: rust-auth-platform, Property 7
    ///
    /// For any configuration, only RS256 algorithm is used.
    ///
    /// Validates: Algorithm is consistently RS256
    use auth_crypto::{JwtConfig, JwtService, KeyManager};
    use jsonwebtoken::Algorithm;

    let config = JwtConfig::default();
    assert_eq!(
        config.algorithm,
        Algorithm::RS256,
        "Default algorithm must be RS256"
    );

    // Verify that the JWT service uses RS256
    let key_manager = KeyManager::new().await.unwrap();
    let jwt_service = JwtService::new(config, key_manager);

    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    let token = jwt_service
        .generate_access_token(user_id, tenant_id, vec![], vec![], None)
        .await
        .unwrap();

    // Decode header to verify algorithm
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() >= 1 {
        let header: Option<serde_json::Value> = base64::decode_config(parts[0], base64::URL_SAFE_NO_PAD)
            .ok()
            .and_then(|bytes: Vec<u8>| serde_json::from_slice::<serde_json::Value>(&bytes).ok());

        if let Some(header_json) = header {
            let alg = header_json.get("alg").and_then(|v| v.as_str());
            assert_eq!(alg, Some("RS256"), "Token algorithm must be RS256");
        }
    }
}

#[tokio::test]
async fn test_revocation_consistency() {
    /// **Property 8: Revocation Consistency**
    ///
    /// Feature: rust-auth-platform, Property 8
    ///
    /// For any token, after revocation it must never pass validation.
    ///
    /// Validates: Revoked tokens are permanently invalidated
    let engine = TokenEngine::new().await.unwrap();

    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        iss: "auth-platform".to_string(),
        aud: "auth-platform".to_string(),
        exp: (Utc::now() + Duration::minutes(15)).timestamp(),
        iat: Utc::now().timestamp(),
        nbf: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
        tenant_id: Uuid::new_v4().to_string(),
        permissions: vec![],
        roles: vec![],
        scope: None,
    };

    let token_jti = Uuid::parse_str(&claims.jti).unwrap();
    let access_token = engine.issue_access_token(claims.clone()).await.unwrap();
    let token_str = access_token.token.clone();

    // Token should be valid initially
    assert!(engine.validate_token(&token_str).await.is_ok());

    // Revoke the token
    let tenant_id = Uuid::parse_str(&claims.tenant_id).unwrap();
    let user_id = Uuid::parse_str(&claims.sub).unwrap();
    engine.revoke_token(token_jti, user_id, tenant_id).await.unwrap();

    // Token must fail validation after revocation
    let result = engine.validate_token(&token_str).await;
    assert!(result.is_err(), "Revoked token must fail validation");
}
