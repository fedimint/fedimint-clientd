use crate::{
    error::AppError,
    types::fedimint::{CombineRequest, CombineResponse},
};
use anyhow::anyhow;
use axum::{http::StatusCode, Json};
use fedimint_mint_client::OOBNotes;
use itertools::Itertools;

#[axum_macros::debug_handler]
pub async fn handle_combine(req: Json<CombineRequest>) -> Result<Json<CombineResponse>, AppError> {
    let federation_id_prefix = match req
        .notes
        .iter()
        .map(|notes| notes.federation_id_prefix())
        .all_equal_value()
    {
        Ok(id) => id,
        Err(None) => Err(AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow!("E-cash notes strings from different federations"),
        ))?,
        Err(Some((a, b))) => Err(AppError::new(
            StatusCode::BAD_REQUEST,
            anyhow!(
                "E-cash notes strings from different federations: {:?} and {:?}",
                a,
                b
            ),
        ))?,
    };

    let combined_notes = req
        .notes
        .iter()
        .flat_map(|notes| notes.notes().iter_items().map(|(amt, note)| (amt, *note)))
        .collect();

    let combined_oob_notes = OOBNotes::new(federation_id_prefix, combined_notes);

    Ok(Json(CombineResponse {
        notes: combined_oob_notes,
    }))
}
