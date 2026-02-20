use async_graphql::Request;
use auth_extension::{create_schema, PluginEngine, WebhookDispatcher};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Testing Extension and Integration Framework...");

    // 1. Test Plugin Engine (Scripting)
    let plugin = PluginEngine::new();
    let script = r#"
        let x = 10;
        let y = 20;
        x + y
    "#;
    let result = plugin.eval_simple(script).expect("Script execution failed");
    println!("Rhai Script Result: {}", result);
    assert_eq!(result, 30);

    // 2. Test Webhook Dispatcher
    let dispatcher = WebhookDispatcher::new();
    dispatcher
        .dispatch("mock://webhook", "user.created", json!({"id": "123"}))
        .await
        .expect("Webhook dispatch failed");
    println!("Webhook Dispatch Verified");

    // 3. Test GraphQL API
    let schema = create_schema();
    let query = "{ version }";
    let res = schema.execute(Request::new(query)).await;
    let json = serde_json::to_value(&res).unwrap();
    println!("GraphQL Response: {}", json);
    assert_eq!(json["data"]["version"], "1.0.0");

    println!("Extension Framework Tests Passed!");
}
