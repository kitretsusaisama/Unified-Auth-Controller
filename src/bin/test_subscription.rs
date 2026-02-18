use auth_core::services::subscription_service::SubscriptionService;
use auth_db::repositories::subscription_repository::SubscriptionRepository;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::sync::Arc;
use uuid::Uuid;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    match run_test().await {
        Ok(_) => println!("Test Passed"),
        Err(e) => println!("TEST FAILED: {:?}", e),
    }
}

async fn run_test() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("Starting Subscription Integration Test...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Running Migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    let sub_repo = Arc::new(SubscriptionRepository::new(pool.clone()));
    let sub_service = SubscriptionService::new(sub_repo);

    let org_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    
    // Create Tenant (Org & Tenant needed for FK)
    println!("Setting up tenant...");
    sqlx::query("INSERT INTO organizations (id, name, status) VALUES (?, ?, 'active')")
        .bind(org_id.to_string())
        .bind("Sub Test Org")
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO tenants (id, organization_id, name, slug, status) VALUES (?, ?, ?, ?, 'active')")
        .bind(tenant_id.to_string())
        .bind(org_id.to_string())
        .bind("Sub Test Tenant")
        .bind("sub-tenant")
        .execute(&pool)
        .await?;

    // 1. Assign Free Plan
    println!("1. Assigning Free Plan...");
    let sub = sub_service.assign_plan(tenant_id, "free").await?;
    println!("   > Plan assigned: {}", sub.plan_id);

    // 2. Check Feature Access
    println!("2. Checking Feature Access...");
    let basic_access = sub_service.check_feature_access(tenant_id, "basic_access").await?;
    assert!(basic_access, "Free plan should have basic_access");
    
    let advanced_access = sub_service.check_feature_access(tenant_id, "advanced_reporting").await?;
    assert!(!advanced_access, "Free plan should NOT have advanced_reporting");
    println!("   > Feature checks passed");

    // 3. Check Quota & Usage
    println!("3. Checking Usage Quotas...");
    let can_add_user = sub_service.check_quota(tenant_id, "users", 1).await?;
    assert!(can_add_user, "Should be able to add user (usage 0 < limit 5)");

    sub_service.record_usage(tenant_id, "users", 3).await?;
    println!("   > Recorded usage: 3 users");
    
    // Usage is now 3. Limit is 5.
    let can_add_user_more = sub_service.check_quota(tenant_id, "users", 2).await?;
    assert!(can_add_user_more, "Should be able to add 2 more users (usage 3+2 <= 5)");
    
    sub_service.record_usage(tenant_id, "users", 2).await?;
    // Usage is now 5.
    
    let can_add_over_limit = sub_service.check_quota(tenant_id, "users", 1).await?;
    assert!(!can_add_over_limit, "Should NOT be able to add user (usage 5 >= limit 5)");
    println!("   > Quota enforcement passed");

    // 4. Upgrade Plan
    println!("4. Upgrading to Pro Plan...");
    sub_service.assign_plan(tenant_id, "pro").await?;
    let upgraded_access = sub_service.check_feature_access(tenant_id, "advanced_reporting").await?;
    assert!(upgraded_access, "Pro plan should have advanced_reporting");
    println!("   > Upgrade verify passed");

    println!("Subscription Integration Test Complete!");
    Ok(())
}
