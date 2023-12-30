use axum::{extract::State, Json};
use fedimint_core::core::OperationId;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::UNIX_EPOCH;
use time::{format_description::well_known::iso8601, OffsetDateTime};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListOperationsRequest {
    pub limit: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OperationOutput {
    pub id: OperationId,
    pub creation_time: String,
    pub operation_kind: String,
    pub operation_meta: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<serde_json::Value>,
}

async fn _list_operations(state: AppState, req: ListOperationsRequest) -> Result<Value, AppError> {
    const ISO8601_CONFIG: iso8601::EncodedConfig = iso8601::Config::DEFAULT
        .set_formatted_components(iso8601::FormattedComponents::DateTime)
        .encode();
    let operations = state
        .fm
        .operation_log()
        .list_operations(req.limit, None)
        .await
        .into_iter()
        .map(|(k, v)| {
            let creation_time = OffsetDateTime::from_unix_timestamp(
                k.creation_time
                    .duration_since(UNIX_EPOCH)
                    .expect("Couldn't convert time from SystemTime to timestamp")
                    .as_secs() as i64,
            )
            .expect("Couldn't convert time from SystemTime to OffsetDateTime")
            .format(&iso8601::Iso8601::<ISO8601_CONFIG>)
            .expect("Couldn't format OffsetDateTime as ISO8601");

            OperationOutput {
                id: k.operation_id,
                creation_time,
                operation_kind: v.operation_module_kind().to_owned(),
                operation_meta: v.meta(),
                outcome: v.outcome(),
            }
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "operations": operations,
    }))
}

pub async fn handle_ws(v: Value, state: AppState) -> Result<Value, AppError> {
    let v = serde_json::from_value(v).unwrap();
    let operations = _list_operations(state, v).await?;
    let operations_json = json!(operations);
    Ok(operations_json)
}

#[axum_macros::debug_handler]
pub async fn handle_rest(
    State(state): State<AppState>,
    Json(req): Json<ListOperationsRequest>,
) -> Result<Json<Value>, AppError> {
    let operations = _list_operations(state, req).await?;
    Ok(Json(operations))
}
