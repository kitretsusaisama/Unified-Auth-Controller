use auth_core::models::{User, UserStatus};
use auth_core::services::risk_assessment::{LoginHistory, RiskContext, RiskEngine};
use auth_core::services::session_service::SessionService;
use auth_db::repositories::session_repository::SessionRepository;
use chrono::Utc;
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    match run_test().await {
        Ok(_) => println!("Test Passed"),
        Err(e) => println!("TEST FAILED: {:?}", e),
    }
}

async fn run_test() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("Starting Risk & Session Integration Test...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to MySQL...");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    println!("Connected!");

    // Run Migrations (to ensure sessions table exists)
    println!("Running Migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("Migrations Complete!");

    // Initialize Services
    let session_repo = Arc::new(SessionRepository::new(pool.clone()));
    let risk_engine = Arc::new(RiskEngine::new());
    let session_service = SessionService::new(session_repo.clone(), risk_engine);

    // Setup Test Data
    let org_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    println!("Setting up test organization, tenant, and user...");
    sqlx::query!(
        "INSERT INTO organizations (id, name, status) VALUES (?, ?, 'active')",
        org_id.to_string(),
        "Risk Test Org"
    )
    .execute(&pool)
    .await?;

    sqlx::query!("INSERT INTO tenants (id, organization_id, name, slug, status) VALUES (?, ?, ?, ?, 'active')",
        tenant_id.to_string(), org_id.to_string(), "Risk Test Tenant", "risk-tenant")
        .execute(&pool)
        .await?;

    sqlx::query!(
        "INSERT INTO users (id, email, status) VALUES (?, ?, 'active')",
        user_id.to_string(),
        "risk_user@example.com"
    )
    .execute(&pool)
    .await?;

    // Create a mock User object for the service
    let user = User {
        id: user_id,
        email: Some("risk_user@example.com".to_string()),
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
        profile_data: serde_json::Value::Null,
        preferences: serde_json::Value::Null,
        status: UserStatus::Active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        email_verified_at: Some(Utc::now()),
        identifier_type: auth_core::models::user::IdentifierType::Email,
        phone_verified_at: None,
        tenant_id,
    };

    // Test 1: Low Risk Login
    println!("1. Testing Low Risk Login (Known Context)...");
    let context_low = RiskContext {
        user_id,
        tenant_id,
        ip_address: Some("192.168.1.1".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        device_fingerprint: Some("device_123".to_string()),
        geolocation: None,
        previous_logins: vec![LoginHistory {
            timestamp: Utc::now(),
            ip_address: "192.168.1.1".to_string(),
            success: true,
        }],
    };

    let session_low = session_service
        .create_session(user.clone(), context_low)
        .await?;
    println!(
        "   > Session Created: Token={}, Risk={}",
        session_low.session_token, session_low.risk_score
    );
    assert!(session_low.risk_score < 0.3, "Expected low risk score");

    // Test 2: High Risk Login (New IP)
    println!("2. Testing High Risk Login (New IP)...");
    let context_high = RiskContext {
        user_id,
        tenant_id,
        ip_address: Some("10.0.0.99".to_string()), // New IP not in history
        user_agent: Some("Mozilla/5.0".to_string()),
        device_fingerprint: Some("device_123".to_string()),
        geolocation: None,
        previous_logins: vec![LoginHistory {
            timestamp: Utc::now(),
            ip_address: "192.168.1.1".to_string(),
            success: true,
        }],
    };

    let session_high = session_service
        .create_session(user.clone(), context_high)
        .await?;
    println!(
        "   > Session Created: Token={}, Risk={}",
        session_high.session_token, session_high.risk_score
    );
    assert!(
        session_high.risk_score >= 0.3,
        "Expected elevated risk score due to new IP"
    );

    // Test 3: Session Validation
    println!("3. Validating Session...");
    let valid_session = session_service
        .validate_session(&session_low.session_token)
        .await?;
    assert_eq!(valid_session.id, session_low.id);
    println!("   > Session Validated");

    // Test 4: Revocation
    println!("4. Revoking Session...");
    session_service
        .revoke_session(&session_low.session_token)
        .await?;
    let result = session_service
        .validate_session(&session_low.session_token)
        .await;
    assert!(
        result.is_err(),
        "Session should be invalid after revocation"
    );
    println!("   > Session Revoked Successfully");

    // Test 5: Revoke All
    println!("5. Revoking All User Sessions...");
    session_service.revoke_user_sessions(user_id).await?;
    let result_high = session_service
        .validate_session(&session_high.session_token)
        .await;
    assert!(result_high.is_err(), "All sessions should be revoked");
    println!("   > All Sessions Revoked");

    println!("Risk & Session Integration Test Complete!");
    Ok(())
}
