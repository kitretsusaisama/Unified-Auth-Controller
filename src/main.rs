//! Main application entry point for the SSO Platform

use anyhow::Result;
use auth_config::{ConfigLoader, ConfigManager};
use secrecy::ExposeSecret;
use sqlx::mysql::MySqlPoolOptions;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Port management
use auth_platform::{shutdown_signal, PortAuthority, PortClass, PortPolicy};

// Repositories
use auth_db::repositories::{
    otp_repository::OtpRepository, session_repository::SessionRepository,
    subscription_repository::SubscriptionRepository, user_repository::UserRepository,
    RoleRepository, TenantRepository,
};

// Services
use async_trait::async_trait;
use auth_core::services::{
    authorization::AuthorizationService, lazy_registration::LazyRegistrationService,
    otp_delivery::OtpDeliveryService, otp_service::OtpService, rate_limiter::RateLimiter,
    risk_assessment::RiskEngine, session_service::SessionService,
    subscription_service::SubscriptionService,
};

use auth_audit::AuditService;
use auth_core::audit::{AuditLogger, TracingAuditLogger};
use auth_core::services::background::audit_worker::{AsyncAuditLogger, AuditWorker};

use auth_api::AppState;
use auth_cache::{Cache, MultiLevelCache};

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
    let environment =
        std::env::var("AUTH__ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
    let config_loader = ConfigLoader::new("config", &environment);
    let config_manager = ConfigManager::new(config_loader)?;

    let config = config_manager.get_config();
    info!("Configuration loaded for environment: {}", environment);

    // Initialize Database - Use MySQL from config
    let database_url = config.database.mysql_url.expose_secret();
    let pool = MySqlPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(database_url)
        .await
        .expect("Failed to connect to MySQL database");

    info!("Database connection established");

    // Run migrations - Handle dirty migrations gracefully
    if let Err(e) = sqlx::migrate!().run(&pool).await {
        match e {
            sqlx::migrate::MigrateError::Dirty(version) => {
                // Migration already applied but marked as dirty, continue
                info!(
                    "Migrations already applied (dirty: {}), continuing...",
                    version
                );
            }
            sqlx::migrate::MigrateError::VersionMissing(_) => {
                // Migration already applied, continue
                info!("Migrations already applied, continuing...");
            }
            _ => {
                eprintln!("Failed to run migrations: {:?}", e);
                panic!("Failed to run migrations: {:?}", e);
            }
        }
    } else {
        info!("Migrations applied successfully");
    }

    // Initialize Repositories
    let role_repo = Arc::new(RoleRepository::new(pool.clone()));
    let session_repo = Arc::new(SessionRepository::new(pool.clone()));
    let subscription_repo = Arc::new(SubscriptionRepository::new(pool.clone()));
    let user_repo = Arc::new(UserRepository::new(pool.clone()));
    let otp_repo = Arc::new(OtpRepository::new(pool.clone()));
    let tenant_repo = Arc::new(TenantRepository::new(pool.clone()));

    // Initialize Services
    // We use AuthorizationService for RBAC instead of legacy RoleService
    let role_service = Arc::new(AuthorizationService::new(role_repo));

    let risk_engine = Arc::new(RiskEngine::new()); // Config thresholds could be passed here
    let session_service = Arc::new(SessionService::new(session_repo, risk_engine));

    let subscription_service = Arc::new(SubscriptionService::new(subscription_repo));

    // Initialize Token Engine (with in-memory stores for now)
    let token_service: Arc<dyn auth_core::services::token_service::TokenProvider> = Arc::new(
        auth_core::services::token_service::TokenEngine::new()
            .await
            .expect("Failed to initialize TokenEngine"),
    );

    // Initialize Async Audit
    // We use TracingAuditLogger as the underlying persistent logger (or DbAuditLogger in real life)
    let persistent_logger = Arc::new(TracingAuditLogger);
    let (async_logger, audit_rx) = AsyncAuditLogger::new(1000);
    let audit_logger: Arc<dyn AuditLogger> = Arc::new(async_logger);

    // Spawn Audit Worker
    let audit_worker = AuditWorker::new(audit_rx, persistent_logger);
    tokio::spawn(audit_worker.run());

    // Initialize Identity Service
    let identity_service = Arc::new(auth_core::services::identity::IdentityService::new(
        user_repo as Arc<dyn auth_core::services::identity::UserStore>,
        token_service,
        audit_logger.clone(),
    ));

    // Initialize OTP Service
    let otp_service = Arc::new(OtpService::new());

    // Initialize OTP Delivery Service (using mock providers for now)
    // Since the test mocks aren't available publicly, create simple implementations
    use auth_core::services::otp_delivery::DeliveryError;
    use auth_core::services::otp_delivery::{EmailProvider, OtpProvider};

    struct SimpleSmsProvider;
    struct SimpleEmailProvider;

    #[async_trait]
    impl OtpProvider for SimpleSmsProvider {
        async fn send_otp(&self, to: &str, _otp: &str) -> Result<String, DeliveryError> {
            // In production, this would call a real SMS provider
            Ok(format!("sms_sent_to_{}", to))
        }
    }

    #[async_trait]
    impl EmailProvider for SimpleEmailProvider {
        async fn send_email(
            &self,
            to: &str,
            _subject: &str,
            _body: &str,
        ) -> Result<String, DeliveryError> {
            // In production, this would call a real email provider
            Ok(format!("email_sent_to_{}", to))
        }
    }

    let sms_provider = Arc::new(SimpleSmsProvider);
    let email_provider = Arc::new(SimpleEmailProvider);
    let otp_delivery_service = Arc::new(OtpDeliveryService::new(sms_provider, email_provider));

    // Initialize Lazy Registration Service
    let lazy_registration_service =
        Arc::new(LazyRegistrationService::new(identity_service.clone(), tenant_repo));

    // Initialize Rate Limiter
    let rate_limiter = Arc::new(RateLimiter::new());

    // Initialize Audit Service
    let _audit_service = Arc::new(AuditService::new(pool.clone()));

    // Initialize Cache
    let redis_url = if let Some(redis_config) = config.external_services.redis {
        Some(redis_config.url)
    } else {
        None
    };

    if redis_url.is_none() && environment == "production" {
        tracing::error!("Production environment detected but Redis is not configured! Falling back to in-memory cache.");
    }

    let cache: Arc<dyn Cache> = match MultiLevelCache::new(redis_url.clone()) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!(
                "Failed to connect to Redis: {}. Falling back to in-memory.",
                e
            );
            Arc::new(MultiLevelCache::new(None).unwrap())
        }
    };

    let app_state = AppState {
        db: pool,
        role_service,
        session_service,
        subscription_service,
        identity_service,
        otp_service,
        otp_delivery_service,
        lazy_registration_service,
        rate_limiter,
        otp_repository: otp_repo,
        audit_logger,
        cache,
    };

    // Initialize Router
    let app = auth_api::app(app_state);

    // Initialize Port Authority for production-grade port management
    let port_authority = PortAuthority::new()?;

    // Get or create port policy
    let port_policy = config.server.port_policy.clone().unwrap_or_else(|| {
        // Fallback to legacy port configuration
        PortPolicy::new(config.server.port, PortClass::Public, "http")
            .with_fallback_range((config.server.port + 1)..=(config.server.port + 9))
    });

    // Acquire port with policy enforcement
    let managed_listener = port_authority
        .acquire(&port_policy, &config.server.host)
        .await?;

    let bound_port = managed_listener.port();

    // Determine display host (localhost for 0.0.0.0 binding)
    let display_host = if config.server.host == "0.0.0.0" {
        "localhost"
    } else {
        &config.server.host
    };

    // User-facing startup message
    println!("\n🚀 SSO Platform Starting...");
    println!("📍 Server URL: http://{}:{}", display_host, bound_port);
    println!("🔧 Service: {}", managed_listener.service_name());
    println!(
        "✅ Port Management: Production-grade (PID: {})",
        std::process::id()
    );
    println!(
        "⏱  Graceful Shutdown: {}s drain timeout",
        config.server.drain_timeout_seconds
    );
    println!("📊 Health: http://{}:{}/health", display_host, bound_port);
    println!("📖 Docs: http://{}:{}/swagger-ui", display_host, bound_port);
    println!("\n✨ Ready to accept connections!\n");

    // Convert to tokio listener
    let listener = managed_listener.into_tokio_listener()?;

    // Start server with graceful shutdown

    tokio::select! {
        result = axum::serve(listener, app) => {
            result?;
        }
        _ = shutdown_signal() => {
            info!("Shutdown signal received, initiating graceful shutdown");

            // Release port lease
            if let Err(e) = port_authority.release(bound_port).await {
                tracing::warn!("Failed to release port lease: {}", e);
            }

            info!("Graceful shutdown complete");
        }
    }

    Ok(())
}
