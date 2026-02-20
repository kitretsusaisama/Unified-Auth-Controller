use auth_audit::AuditService;
use serde_json::json;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let audit_service = AuditService::new(pool.clone());

    let actor_id = Uuid::new_v4();
    let log = audit_service
        .log(
            "EXPORT_TEST",
            actor_id,
            "export_resource",
            Some(json!({"test": "value"})),
        )
        .await?;

    let cef_string = audit_service.export_cef(&log);
    println!("Generated CEF: {}", cef_string);

    assert!(cef_string.starts_with("CEF:0|AuthPlatform|SSO|1.0|EXPORT_TEST"));
    assert!(cef_string.contains(&format!("act={}", actor_id)));

    println!("Audit Export Test Passed!");
    Ok(())
}
