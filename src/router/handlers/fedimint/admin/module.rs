use crate::error::AppError;

#[axum_macros::debug_handler]
pub async fn handle_module() -> Result<(), AppError> {
    // TODO: Figure out how to impl this
    Ok(())
}
