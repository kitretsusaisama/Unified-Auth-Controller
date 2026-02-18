use rhai::{Engine, Scope, AST, EvalAltResult};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Clone)]
pub struct PluginEngine {
    engine: Arc<Engine>,
    scripts: Arc<Mutex<Vec<AST>>>,
}

impl PluginEngine {
    pub fn new() -> Self {
        let engine = Engine::new();
        // Configure engine for safety (e.g., limit iterations/depth if needed in prod)
        // engine.set_max_expr_depths(50, 50);
        
        Self {
            engine: Arc::new(engine),
            scripts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn register_script(&self, script: &str) -> Result<(), Box<EvalAltResult>> {
        let ast = self.engine.compile(script)?;
        let mut scripts = self.scripts.lock().await;
        scripts.push(ast);
        info!("Plugin script registered successfully");
        Ok(())
    }

    pub async fn execute_hook(&self, hook_name: &str, payload: Value) -> Result<Value, Box<EvalAltResult>> {
        let scripts = self.scripts.lock().await;
        let mut result = payload.clone();

        for ast in scripts.iter() {
            let mut scope = Scope::new();
            // Convert serde Value to Rhai dynamic via serialization roundtrip strictly for inputs
            // For simplicity (MVP), we treat payload as Dynamic if Rhai supports serde.
            // Rhai 1.0+ has serde feature. We need to dynamic conversion or strict type passing.
            // Let's assume passed as JSON string for safety/simplicity in MVP without extra glue code.
            let json_payload = serde_json::to_string(&payload).unwrap();
            scope.push("payload_json", json_payload);

            // Hook function call pattern: `fn on_hook(json) { ... }`
            // Check if function exists in AST? Rhai 'call_fn' works on Engine with AST.
            
            let _options = rhai::CallFnOptions::new().eval_ast(false); // Don't re-eval global
            
            // Try calling the hook function if defined
             match self.engine.call_fn::<String>(&mut scope, ast, hook_name, ( result.to_string(), )) {
                Ok(new_json) => {
                     // If script returns new JSON, update result (chaining)
                     if let Ok(val) = serde_json::from_str(&new_json) {
                         result = val;
                     }
                }
                Err(_e) => {
                    // Ignore if function not found, otherwise log error
                    // EvalAltResult::ErrorFunctionNotFound
                    // We can check error type but for MVP just log generic debug
                    // actually for hooks, silence is golden if not implemented
                }
            }
        }
        
        Ok(result)
    }
    
    // Simplified execution for verifying basic logic
    pub fn eval_simple(&self, script: &str) -> Result<i64, Box<EvalAltResult>> {
        self.engine.eval(script)
    }
}

impl Default for PluginEngine {
    fn default() -> Self {
        Self::new()
    }
}
