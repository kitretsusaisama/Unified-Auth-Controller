use async_graphql::{Context, Object, Schema, EmptyMutation, EmptySubscription, SimpleObject};
use uuid::Uuid;

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

pub fn create_schema() -> ExtensionSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish()
}
