use axum::{extract::State, Json};
use fedimint_client::backup::Metadata;

use crate::{error::AppError, state::AppState, types::fedimint::BackupRequest};

#[axum_macros::debug_handler]
pub async fn handle_backup(
    State(state): State<AppState>,
    Json(req): Json<BackupRequest>,
) -> Result<Json<()>, AppError> {
    state
        .fm
        .backup_to_federation(Metadata::from_json_serialized(req.metadata))
        .await?;
    Ok(Json(()))
}
