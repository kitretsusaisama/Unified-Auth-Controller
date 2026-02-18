use axum::Router;
use sqlx::MySqlPool;
use std::sync::Arc;
use auth_core::services::{
    role_service::RoleService,
    session_service::SessionService,
    subscription_service::SubscriptionService,
    otp_service::OtpService,
    otp_delivery::OtpDeliveryService,
    lazy_registration::LazyRegistrationService,
    rate_limiter::RateLimiter,
};
use auth_db::repositories::otp_repository::OtpRepository;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod router;
pub mod handlers;
pub mod error;
pub mod validation;
pub mod middleware;

use auth_cache::Cache;

// Admin UI (feature-gated)
#[cfg(feature = "admin-ui")]
pub mod admin;

/// OpenAPI documentation for the Enterprise SSO Platform
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::login,
        handlers::auth::register,
        handlers::users::ban_user,
        handlers::users::activate_user,
        handlers::health::health_check,
    ),
    components(
        schemas(
            auth_core::services::identity::AuthRequest,
            auth_core::services::identity::AuthResponse,
            auth_core::models::user::User,
            auth_core::models::user::CreateUserRequest,
            auth_core::models::user::UserStatus,
            crate::error::ErrorResponse,
            crate::error::FieldError,
        )
    ),
    tags(
        (name = "Authentication", description = "User authentication and registration endpoints"),
        (name = "User Management", description = "User administration endpoints"),
        (name = "Health", description = "Service health check endpoints")
    ),
    info(
        title = "Enterprise SSO Platform API",
        version = "0.1.0",
        description = "Production-ready SSO and Identity Platform supporting OIDC, SAML, OAuth 2.1, and SCIM 2.0",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    )
)]
pub struct ApiDoc;

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub role_service: Arc<RoleService>,
    pub session_service: Arc<SessionService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub identity_service: Arc<auth_core::services::identity::IdentityService>,
    pub otp_service: Arc<OtpService>,
    pub otp_delivery_service: Arc<OtpDeliveryService>,
    pub lazy_registration_service: Arc<LazyRegistrationService>,
    pub rate_limiter: Arc<RateLimiter>,
    pub otp_repository: Arc<OtpRepository>,
    pub audit_logger: Arc<dyn auth_core::audit::AuditLogger>,
    pub cache: Arc<dyn Cache>,
}

pub fn app(state: AppState) -> Router {
    // Build base router with swagger  
    let router = router::api_router()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
    
    // Add admin UI routes if feature is enabled
    #[cfg(feature = "admin-ui")]
    let router = {
        use axum::routing::get;
        use axum::middleware;
        
        router
            // Public auth pages (no middleware)
            .route("/admin/login", get(admin::handlers::login_page))
            .route("/admin/register", get(admin::handlers::register_page))
            
            // Protected dashboard pages (with JWT auth middleware)
            .route("/admin/dashboard", 
                get(admin::handlers::dashboard_page)
                    .route_layer(middleware::from_fn_with_state(state.clone(), crate::middleware::auth::jwt_auth)))
            .route("/admin/users", 
                get(admin::handlers::users_page)
                    .route_layer(middleware::from_fn_with_state(state.clone(), crate::middleware::auth::jwt_auth)))
            .route("/admin/roles", 
                get(admin::handlers::roles_page)
                    .route_layer(middleware::from_fn_with_state(state.clone(), crate::middleware::auth::jwt_auth)))
            .route("/admin/settings", 
                get(admin::handlers::settings_page)
                    .route_layer(middleware::from_fn_with_state(state.clone(), crate::middleware::auth::jwt_auth)))
            .route("/admin/logout", get(admin::handlers::logout))
    };
    
    router.with_state(state)
}

// Make services extractable from AppState via State<Arc<Service>>
impl axum::extract::FromRef<AppState> for Arc<auth_core::services::identity::IdentityService> {
    fn from_ref(state: &AppState) -> Self {
        state.identity_service.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<OtpService> {
    fn from_ref(state: &AppState) -> Self {
        state.otp_service.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<OtpDeliveryService> {
    fn from_ref(state: &AppState) -> Self {
        state.otp_delivery_service.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<LazyRegistrationService> {
    fn from_ref(state: &AppState) -> Self {
        state.lazy_registration_service.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<RateLimiter> {
    fn from_ref(state: &AppState) -> Self {
        state.rate_limiter.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<OtpRepository> {
    fn from_ref(state: &AppState) -> Self {
        state.otp_repository.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<dyn auth_core::audit::AuditLogger> {
    fn from_ref(state: &AppState) -> Self {
        state.audit_logger.clone()
    }
}

impl axum::extract::FromRef<AppState> for Arc<dyn Cache> {
    fn from_ref(state: &AppState) -> Self {
        state.cache.clone()
    }
}