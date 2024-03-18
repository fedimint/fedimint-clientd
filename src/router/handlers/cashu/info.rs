use std::collections::BTreeMap;

use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct Contact {
    pub method: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct NutMethod {
    pub method: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Nut {
    pub methods: Option<Vec<NutMethod>>,
    pub disabled: Option<bool>,
    pub supported: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CashuNUT06InfoResponse {
    pub name: String,
    pub pubkey: String,
    pub version: String,
    pub description: String,
    pub description_long: String,
    pub contact: Vec<Contact>,
    pub motd: String,
    pub nuts: BTreeMap<String, Nut>,
}

#[axum_macros::debug_handler]
pub async fn handle_info(
    State(state): State<AppState>,
) -> Result<Json<CashuNUT06InfoResponse>, AppError> {
    let client = state.get_cashu_client().await?;

    let config = client.get_config();
    let mut nuts = BTreeMap::new();

    nuts.insert(
        "4".to_string(),
        Nut {
            methods: Some(vec![NutMethod {
                method: "bolt11".to_string(),
                value: "sat".to_string(),
            }]),
            disabled: Some(false),
            supported: None,
        },
    );

    nuts.insert(
        "5".to_string(),
        Nut {
            methods: Some(vec![NutMethod {
                method: "bolt11".to_string(),
                value: "sat".to_string(),
            }]),
            disabled: None,
            supported: None,
        },
    );

    for &i in &[7, 8, 9, 10, 12] {
        nuts.insert(
            i.to_string(),
            Nut {
                methods: None,
                disabled: None,
                supported: Some(true),
            },
        );
    }

    let response = CashuNUT06InfoResponse {
        name: config.global.federation_name().unwrap().to_string(),
        pubkey: config.global.federation_id().to_string(),
        version: format!("{:?}", config.global.consensus_version),
        description: "Cashu <-> Fedimint Soon (tm)".to_string(),
        description_long: "Cashu <-> Fedimint Soon (tm)".to_string(),
        contact: vec![Contact {
            method: "xmpp".to_string(),
            value: "local@localhost".to_string(),
        }],
        motd: "Cashu <-> Fedimint Soon (tm)".to_string(),
        nuts,
    };

    Ok(Json(response))
}
