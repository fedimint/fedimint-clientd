use std::collections::BTreeMap;

use axum::Json;
use fedimint_core::TieredMulti;
use fedimint_mint_client::OOBNotes;

use crate::{
    error::AppError,
    types::fedimint::{SplitRequest, SplitResponse},
};

#[axum_macros::debug_handler]
pub async fn handle_split(Json(req): Json<SplitRequest>) -> Result<Json<SplitResponse>, AppError> {
    let federation = req.notes.federation_id_prefix();
    let notes = req
        .notes
        .notes()
        .iter()
        .map(|(amount, notes)| {
            let notes = notes
                .iter()
                .map(|note| {
                    OOBNotes::new(
                        federation,
                        TieredMulti::new(vec![(*amount, vec![*note])].into_iter().collect()),
                    )
                })
                .collect::<Vec<_>>();
            (*amount, notes[0].clone()) // clone the amount and return a single OOBNotes
        })
        .collect::<BTreeMap<_, _>>();

    Ok(Json(SplitResponse { notes }))
}
