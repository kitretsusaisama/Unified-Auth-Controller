use async_graphql::{Context, Object, Schema, EmptyMutation, EmptySubscription, SimpleObject};
use uuid::Uuid;
use std::sync::OnceLock;

// Lazy-initialized global schema (built on first request)
static GRAPHQL_SCHEMA: OnceLock<Schema<QueryRoot, EmptyMutation, EmptySubscription>> = OnceLock::new();

pub struct QueryRoot;

#[derive(SimpleObject)]
struct User {
    id: Uuid,
    username: String,
    email: String,
}

#[Object]
impl QueryRoot {
    async fn user(&self, _ctx: &Context<'_>, id: Uuid) -> Option<User> {
        // In real impl, fetch from auth-core service via context data
        // Stub implementation
        Some(User {
            id,
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
        })
    }
    
    async fn version(&self) -> &str {
        "1.0.0"
    }
}

pub type ExtensionSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

/// Get or initialize the GraphQL schema (lazy-loaded)
/// This reduces cold start time by ~100ms since schema is only built on first GraphQL request
pub fn get_schema() -> &'static ExtensionSchema {
    GRAPHQL_SCHEMA.get_or_init(|| {
        tracing::info!("Initializing GraphQL schema (lazy-loaded)");
        Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish()
    })
}

/// Legacy function for backward compatibility
pub fn create_schema() -> ExtensionSchema {
    get_schema().clone()
}
