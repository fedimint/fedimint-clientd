pub mod cashu;
pub mod fedimint;

use std::fs::read_to_string;

use axum::Json;
use serde_json::{json, Value};

#[axum_macros::debug_handler]
pub async fn handle_readme() -> String {
    read_to_string("README.md").expect("Could not read README.md")
}

#[axum_macros::debug_handler]
pub async fn handle_status() -> Json<Value> {
    Json(json!({"status": "ok"}))
}
