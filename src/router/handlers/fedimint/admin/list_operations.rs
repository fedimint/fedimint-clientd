use std::time::UNIX_EPOCH;

use axum::{extract::State, Json};
use serde_json::{json, Value};
use time::{format_description::well_known::iso8601, OffsetDateTime};

use crate::{
    error::AppError,
    state::AppState,
    types::fedimint::{ListOperationsRequest, OperationOutput},
};

#[axum_macros::debug_handler]
pub async fn handle_list_operations(
    State(state): State<AppState>,
    Json(req): Json<ListOperationsRequest>,
) -> Result<Json<Value>, AppError> {
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

    Ok(Json(json!({
        "operations": operations,
    })))
}
