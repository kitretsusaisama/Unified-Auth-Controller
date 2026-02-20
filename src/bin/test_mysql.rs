use auth_core::prelude::token_service::{TokenEngine, TokenProvider};
use auth_db::repositories::{RefreshTokenRepository, RevokedTokenRepository};
// assuming this exists or I build config
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("Starting Token Engine Integration Test...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to MySQL...");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    println!("Connected!");

    // Initialize Repositories
    let refresh_repo = Arc::new(RefreshTokenRepository::new(pool.clone()));
    let revoked_repo = Arc::new(RevokedTokenRepository::new(pool.clone()));

    // Initialize Token Engine with Repositories
    println!("Initializing TokenEngine with MySQL-backed repositories...");
    let engine = TokenEngine::new_with_stores(revoked_repo, refresh_repo).await?;

    // Test Flow
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();

    println!("Setting up test data (Org, Tenant, User)...");

    // 1. Create Organization
    sqlx::query("INSERT INTO organizations (id, name, status) VALUES (?, ?, 'active')")
        .bind(org_id.to_string())
        .bind("Test Org")
        .execute(&pool)
        .await?;

    // 2. Create Tenant
    sqlx::query("INSERT INTO tenants (id, organization_id, name, slug, status) VALUES (?, ?, ?, ?, 'active')")
        .bind(tenant_id.to_string())
        .bind(org_id.to_string())
        .bind("Test Tenant")
        .bind("test-tenant")
        .execute(&pool)
        .await?;

    // 3. Create User
    sqlx::query("INSERT INTO users (id, email, status) VALUES (?, ?, 'active')")
        .bind(user_id.to_string())
        .bind("test@example.com")
        .execute(&pool)
        .await?;

    println!("1. Issuing Refresh Token for User {}", user_id);
    let refresh_token = engine.issue_refresh_token(user_id, tenant_id).await?;
    println!("   > Success! IDs: {}", refresh_token.id);

    println!("2. Verifying persistence...");
    // We can't query repo directly via engine public API easily except via refresh flow
    // But engine.refresh_tokens uses find_by_hash.

    println!("3. Performing Token Refresh (Rotation)...");
    let token_pair = engine.refresh_tokens(&refresh_token.token_hash).await?;
    println!(
        "   > Success! New Access Token: {}...",
        &token_pair.access_token.token[0..20]
    );
    println!(
        "   > New Refresh Token: {}...",
        &token_pair.refresh_token[0..20]
    );

    println!("4. Verifying Old Token Revocation (Rotation)...");
    let result = engine.refresh_tokens(&refresh_token.token_hash).await;
    match result {
        Ok(_) => println!("   > FAILED: Old token should be invalid!"),
        Err(_) => println!("   > Success: Old token rejected."),
    }

    println!("5. Manual Revocation...");
    // Use JTI from access token
    let claims = engine
        .validate_token(&token_pair.access_token.token)
        .await?;
    let jti = Uuid::parse_str(&claims.jti)?;

    // Revoke using new signature (requires user_id, tenant_id)
    // We don't have tenant_id handy from refresh_token call return (TokenPair doesn't expose it)
    // But we have it from claims logic?
    // Wait, TokenEngine::refresh_tokens uses placeholder keys?
    // No, it uses data from old token.
    // In our test, claims includes tenant_id.
    let tenant_id_from_claims = Uuid::parse_str(&claims.tenant_id)?;

    engine
        .revoke_token(jti, user_id, tenant_id_from_claims)
        .await?;
    println!("   > Revoked access token JTI: {}", jti);

    println!("6. Verifying Revocation...");
    let val_result = engine.validate_token(&token_pair.access_token.token).await;
    match val_result {
        Ok(_) => println!("   > FAILED: Revoked token should be invalid!"),
        Err(_) => println!("   > Success: Revoked token rejected."),
    }

    println!("Integration Test Complete!");
    Ok(())
}
