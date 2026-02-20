use auth_core::services::identity::{IdentityService, AuthRequest};
use auth_core::services::token_service::{TokenEngine, TokenProvider}; // TokenEngine implements TokenProvider
use auth_core::models::user::{CreateUserRequest, UserStatus};
use auth_db::repositories::UserRepository;
use auth_core::services::identity::UserStore;
use sqlx::{MySqlPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup Logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Auth Flow Verification...");

    // 2. Connect to DB
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "mysql://root:password@localhost:3306/sso".to_string());
    info!("Connecting to database at {}...", db_url);

    let pool = MySqlPool::connect(&db_url).await.expect("Failed to connect to DB. Ensure MySQL is running and DATABASE_URL is set.");

    // 3. Setup Schema (Temporary Migration)
    // We create the users table if it doesn't exist matching UserRepository expectation.
    info!("Ensuring schema...");
    sqlx::query(r#"
    CREATE TABLE IF NOT EXISTS users (
        id VARCHAR(36) PRIMARY KEY,
        tenant_id VARCHAR(36) NOT NULL,
        email VARCHAR(255) NOT NULL,
        password_hash VARCHAR(255) NOT NULL,
        status VARCHAR(50) NOT NULL, -- JSON or String Enum
        email_verified BOOLEAN DEFAULT FALSE,
        phone VARCHAR(50),
        phone_verified BOOLEAN DEFAULT FALSE,
        failed_login_attempts INT DEFAULT 0,
        locked_until TIMESTAMP NULL,
        last_login_at TIMESTAMP NULL,
        last_login_ip VARCHAR(50),
        mfa_enabled BOOLEAN DEFAULT FALSE,
        mfa_secret VARCHAR(255),
        backup_codes JSON,
        risk_score FLOAT DEFAULT 0.0,
        profile_data JSON,
        preferences JSON,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
        deleted_at TIMESTAMP NULL,
        UNIQUE KEY unique_email (email, tenant_id)
    )
    "#).execute(&pool).await?;

    // 4. Initialize Services (Explicit casting to Arc<dyn Trait>)
    let user_repo: Arc<dyn UserStore> = Arc::new(UserRepository::new(pool.clone()));

    // We need TokenEngine. It needs RevokedTokenStore and RefreshTokenStore.
    // We can use the default in-memory ones via `TokenEngine::new()`.
    let token_service: Arc<dyn TokenProvider> = Arc::new(TokenEngine::new().await?);

    let identity_service = IdentityService::new(user_repo.clone(), token_service.clone());

    // 5. Run Scenario
    let tenant_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", Uuid::new_v4()); // Unique email
    let password = "StrongPassword123!";

    // A. Register
    info!("Step A: Registering user {}", email);
    let register_req = CreateUserRequest {
        identifier_type: auth_core::models::user::IdentifierType::Email,
        email: Some(email.clone()),
        phone: None,
        primary_identifier: Some(auth_core::models::user::PrimaryIdentifier::Email),
        password: Some(password.to_string()),
        profile_data: None,
        require_verification: Some(true),
    };

    let user = identity_service.register(register_req, tenant_id).await?;
    info!("User registered: ID={}", user.id);
    assert_eq!(user.email, Some(email.clone()));
    assert!(matches!(user.status, UserStatus::PendingVerification));

    // B. Activate User (Directly via Repo or Service if exposed)
    // IdentityService::activate_user calls store.update_status
    info!("Step B: Activating user...");
    identity_service.activate_user(user.id).await?;

    // Verify status in DB
    let user_fetched = user_repo.find_by_id(user.id).await?.expect("User should exist");
    assert!(matches!(user_fetched.status, UserStatus::Active));

    // C. Login (Success)
    info!("Step C: Login with correct credentials...");
    let login_req = AuthRequest {
        email: email.clone(),
        password: password.to_string(),
        tenant_id,
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("TestAgent".to_string()),
    };

    let auth_resp = identity_service.login(login_req.clone()).await?;
    info!("Login successful! Access Token: {}...", &auth_resp.access_token[..10]);

    // D. Login (Failure - Wrong Password)
    info!("Step D: Login with wrong network...");
    let mut bad_req = login_req.clone();
    bad_req.password = "WrongPass".to_string();

    let result = identity_service.login(bad_req).await;
    assert!(result.is_err());
    info!("Login correctly failed with wrong password.");

    // E. Ban User
    info!("Step E: Banning user...");
    identity_service.ban_user(user.id).await?;

    // F. Login (Failure - Banned)
    info!("Step F: Login while banned...");
    let result_banned = identity_service.login(login_req).await;
    assert!(result_banned.is_err());
    // Use format! to verify error message if possible, or just is_err
    info!("Login correctly failed for banned user.");

    info!("Integration Test PASSED!");
    Ok(())
}
