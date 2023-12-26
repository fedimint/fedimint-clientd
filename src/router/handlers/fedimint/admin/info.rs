use axum::{extract::State, Json};
use fedimint_mint_client::MintClientModule;
use fedimint_wallet_client::WalletClientModule;

use crate::{error::AppError, state::AppState, types::fedimint::InfoResponse};

#[axum_macros::debug_handler]
pub async fn handle_info(State(state): State<AppState>) -> Result<Json<InfoResponse>, AppError> {
    let mint_client = state.fm.get_first_module::<MintClientModule>();
    let wallet_client = state.fm.get_first_module::<WalletClientModule>();
    let summary = mint_client
        .get_wallet_summary(
            &mut state
                .fm
                .db()
                .begin_transaction_nc()
                .await
                .to_ref_with_prefix_module_id(1),
        )
        .await;
    Ok(Json(InfoResponse {
        federation_id: state.fm.federation_id(),
        network: wallet_client.get_network().to_string(),
        meta: state.fm.get_config().global.meta.clone(),
        total_amount_msat: summary.total_amount(),
        total_num_notes: summary.count_items(),
        denominations_msat: summary,
    }))
}
