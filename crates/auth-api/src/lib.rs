use axum::Router;
use sqlx::MySqlPool;
use std::sync::Arc;
use auth_core::services::{
    role_service::RoleService,
    session_service::SessionService,
    subscription_service::SubscriptionService,
};
use auth_db::repositories::{
    role_repository::RoleRepository,
    session_repository::SessionRepository,
    subscription_repository::SubscriptionRepository,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod router;
pub mod handlers;
pub mod error;
pub mod validation;
pub mod middleware;

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
    // Add other services here
}

pub fn app(state: AppState) -> Router {
    router::api_router()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}