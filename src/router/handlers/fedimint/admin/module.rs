use fedimint_core::core::{ModuleInstanceId, ModuleKind};
use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Clone, Deserialize)]
pub enum ModuleSelector {
    Id(ModuleInstanceId),
    Kind(ModuleKind),
}

#[derive(Debug, Deserialize)]
pub struct ModuleRequest {
    pub module: ModuleSelector,
    pub args: Vec<String>,
}

#[axum_macros::debug_handler]
pub async fn handle_module() -> Result<(), AppError> {
    // TODO: Figure out how to impl this
    Ok(())
}
