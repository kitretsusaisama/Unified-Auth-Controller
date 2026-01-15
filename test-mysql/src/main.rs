//! Standalone MySQL Connection Test

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("========================================");
    println!("MySQL Connection Test");
    println!("========================================\n");

    use sqlx::mysql::MySqlPoolOptions;
    
    let database_url = "mysql://u413456342_sias:V%26zTudOgd9v1@srv1873.hstgr.io/u413456342_sias";
    
    println!("📡 Connecting to MySQL...");
    println!("   Host: srv1873.hstgr.io");
    println!("   Database: u413456342_sias\n");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    println!("✅ Connection successful!\n");

    // Test database access
    println!("🔍 Testing database access...");
    let row: (String,) = sqlx::query_as("SELECT DATABASE()")
        .fetch_one(&pool)
        .await?;
    println!("   Current database: {}\n", row.0);

    // Run migrations
    println!("📋 Running migrations...");
    run_migrations(&pool).await?;

    // Insert sample data
    println!("\n📝 Inserting sample data...");
    insert_sample_data(&pool).await?;

    // Verify
    println!("\n🔍 Verifying data...");
    verify_data(&pool).await?;

    println!("\n========================================");
    println!("✅ All tests passed!");
    println!("========================================");

    Ok(())
}

async fn run_migrations(pool: &sqlx::MySqlPool) -> Result<(), Box<dyn Error>> {
    let sql = std::fs::read_to_string("../migrations/complete_migration.sql")?;
    
    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() && 
           !trimmed.starts_with("--") && 
           !trimmed.starts_with("/*") &&
           !trimmed.contains("SELECT 'Migrations") {
            match sqlx::query(trimmed).execute(pool).await {
                Ok(_) => {},
                Err(e) => {
                    if !e.to_string().contains("already exists") {
                        eprintln!("   Warning: {}", e);
                    }
                }
            }
        }
    }
    
    println!("   ✅ Migrations completed");
    Ok(())
}

async fn insert_sample_data(pool: &sqlx::MySqlPool) -> Result<(), Box<dyn Error>> {
    // Check if data already exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM organizations")
        .fetch_one(pool)
        .await?;
    
    if count.0 > 0 {
        println!("   Data already exists, skipping");
        return Ok(());
    }

    let org_id = format!("{:032x}", rand::random::<u128>());
    let tenant_id = format!("{:032x}", rand::random::<u128>());
    let user1_id = format!("{:032x}", rand::random::<u128>());
    let user2_id = format!("{:032x}", rand::random::<u128>());

    println!("   Creating organization...");
    sqlx::query("INSERT INTO organizations (id, name, domain, status) VALUES (?, ?, ?, 'active')")
        .bind(&org_id).bind("Acme Corp").bind("acme.com").execute(pool).await?;

    println!("   Creating tenant...");
    sqlx::query("INSERT INTO tenants (id, organization_id, name, slug, status) VALUES (?, ?, ?, ?, 'active')")
        .bind(&tenant_id).bind(&org_id).bind("Acme Production").bind("acme-prod").execute(pool).await?;

    println!("   Creating users...");
    let password_hash = "$argon2id$v=19$m=19456,t=2,p=1$test$hash";

    for (id, email) in [(&user1_id, "admin@acme.com"), (&user2_id, "user@acme.com")] {
        sqlx::query("INSERT INTO users (id, email, password_hash, status) VALUES (?, ?, ?, 'active')")
            .bind(id).bind(email).bind(password_hash).execute(pool).await?;
        
        sqlx:: query("INSERT INTO user_tenants (user_id, tenant_id, status) VALUES (?, ?, 'active')")
            .bind(id).bind(&tenant_id).execute(pool).await?;
    }

    println!("   ✅ Sample data inserted");
    Ok(())
}

async fn verify_data(pool: &sqlx::MySqlPool) -> Result<(), Box<dyn Error>> {
    let tables = ["organizations", "tenants", "users", "user_tenants", "roles", "permissions", "refresh_tokens", "revoked_tokens"];

    for table in tables {
        let query = format!("SELECT COUNT(*) FROM {}", table);
        let count: (i64,) = sqlx::query_as(&query).fetch_one(pool).await?;
        println!("   {}: {} rows", table, count.0);
    }

    let users: Vec<(String, String)> = sqlx::query_as("SELECT email, status FROM users LIMIT 3")
        .fetch_all(pool).await?;

    if !users.is_empty() {
        println!("\n   Sample Users:");
        for (email, status) in users {
            println!("   - {} ({})", email, status);
        }
    }

    Ok(())
}
