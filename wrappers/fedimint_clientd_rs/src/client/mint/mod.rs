pub mod types;

use crate::FedimintClient;
pub use types::*;

use super::types::NotesJson;

impl FedimintClient {
    pub async fn decode_notes(&self, notes: String) -> Result<DecodeNotesResponse, String> {
        self.post::<DecodeNotesRequest, DecodeNotesResponse>(
            "/mint/decode-notes",
            DecodeNotesRequest { notes },
        )
        .await
    }

    pub async fn encode_notes(&self, notes_json: NotesJson) -> Result<EncodeNotesResponse, String> {
        let json_notes = serde_json::to_string(&notes_json);

        match json_notes {
            Ok(json) => {
                self.post::<EncodeNotesRequest, EncodeNotesResponse>(
                    "/mint/encode-notes",
                    EncodeNotesRequest { notesJsonStr: json },
                )
                .await
            }
            Err(_) => Err("Failed to stringify JSON Notes".to_string()),
        }
    }

    pub async fn reissue(&self, notes: String) -> Result<ReissueResponse, String> {
        let federation_id = self.active_federation_id.clone();

        if federation_id.is_empty() {
            return Err("Federation ID Required".to_string());
        }

        self.post::<ReissueRequest, ReissueResponse>(
            "/mint/reissue",
            ReissueRequest {
                federationId: federation_id,
                notes,
            },
        )
        .await
    }

    pub async fn spend(&self, request: SpendOptions) -> Result<SpendResponse, String> {
        let federation_id = self.active_federation_id.clone();

        if federation_id.is_empty() {
            return Err("Federation ID Required".to_string());
        }

        self.post::<SpendRequest, SpendResponse>(
            "/mint/spend",
            SpendRequest {
                federationId: federation_id,
                allowOverpay: request.allow_overpay,
                amountMsat: request.amount_msat,
                includeInvite: request.include_invite,
                timeout: request.timeout,
            },
        )
        .await
    }

    pub async fn validate(&self, notes: String) -> Result<ValidateResponse, String> {
        let federation_id = self.active_federation_id.clone();

        if federation_id.is_empty() {
            return Err("Federation ID Required".to_string());
        }

        self.post::<ValidateRequest, ValidateResponse>("/mint/validate", {
            ValidateRequest {
                federationId: federation_id,
                notes,
            }
        })
        .await
    }

    pub async fn split(&self, notes: String) -> Result<SplitResponse, String> {
        self.post::<SplitRequest, SplitResponse>("/mint/split", SplitRequest { notes })
            .await
    }

    pub async fn combine(&self, notes_vec: Vec<String>) -> Result<CombineResponse, String> {
        self.post::<CombineRequest, CombineResponse>(
            "/mint/combine",
            CombineRequest {
                notesVec: notes_vec,
            },
        )
        .await
    }
}
