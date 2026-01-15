//! Main application entry point for the SSO Platform

use anyhow::Result;
use auth_config::{ConfigLoader, ConfigManager};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use sqlx::mysql::MySqlPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;

// Repositories
use auth_db::repositories::{
    role_repository::RoleRepository,
    session_repository::SessionRepository,
    subscription_repository::SubscriptionRepository,
};

// Services
use auth_core::services::{
    role_service::RoleService,
    session_service::SessionService,
    subscription_service::SubscriptionService,
    risk_assessment::RiskEngine,
};

use auth_api::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_platform=debug,auth_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting SSO Platform");

    // Load configuration
    let environment = std::env::var("AUTH_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let config_loader = ConfigLoader::new("config", &environment);
    let config_manager = ConfigManager::new(config_loader)?;
    
    let config = config_manager.get_config();
    info!("Configuration loaded for environment: {}", environment);

    // Initialize Database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    info!("Database connection established");

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    info!("Migrations applied successfully");

    // Initialize Repositories
    let role_repo = Arc::new(RoleRepository::new(pool.clone()));
    let session_repo = Arc::new(SessionRepository::new(pool.clone()));
    let subscription_repo = Arc::new(SubscriptionRepository::new(pool.clone()));
    let user_repo = Arc::new(auth_db::repositories::user_repository::UserRepository::new(pool.clone()));

    // Initialize Services
    let role_service = Arc::new(RoleService::new(role_repo));
    
    let risk_engine = Arc::new(RiskEngine::new()); // Config thresholds could be passed here
    let session_service = Arc::new(SessionService::new(session_repo, risk_engine));
    
    let subscription_service = Arc::new(SubscriptionService::new(subscription_repo));

    // Initialize Token Engine (with in-memory stores for now)
    let token_service: Arc<dyn auth_core::services::token_service::TokenProvider> = 
        Arc::new(auth_core::services::token_service::TokenEngine::new().await.expect("Failed to initialize TokenEngine"));

    // Initialize Identity Service
    let identity_service = Arc::new(auth_core::services::identity::IdentityService::new(
        user_repo as Arc<dyn auth_core::services::identity::UserStore>,
        token_service,
    ));

    let app_state = AppState {
        db: pool,
        role_service,
        session_service,
        subscription_service,
        identity_service,
    };

    // Initialize Router
    let app = auth_api::app(app_state);

    // Start Server
    // Force port 8081 for now due to config issue
    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}