use axum::{extract::State, Json};
use fedimint_ln_client::LightningClientModule;
use serde_json::{json, Value};

use crate::{error::AppError, state::AppState};

#[axum_macros::debug_handler]
pub async fn handle_listgateways(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let lightning_module = state.fm.get_first_module::<LightningClientModule>();
    let gateways = lightning_module.fetch_registered_gateways().await?;
    if gateways.is_empty() {
        return Ok(Json(serde_json::to_value(Vec::<String>::new()).unwrap()));
    }

    let mut gateways_json = json!(&gateways);
    let active_gateway = lightning_module.select_active_gateway().await?;

    gateways_json
        .as_array_mut()
        .expect("gateways_json is not an array")
        .iter_mut()
        .for_each(|gateway| {
            if gateway["node_pub_key"] == json!(active_gateway.node_pub_key) {
                gateway["active"] = json!(true);
            } else {
                gateway["active"] = json!(false);
            }
        });
    Ok(Json(serde_json::to_value(gateways_json).unwrap()))
}
