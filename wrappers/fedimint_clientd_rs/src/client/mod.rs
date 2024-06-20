pub mod admin;
pub mod lightning;
pub mod mint;
pub mod onchain;
pub mod types;

use serde::{de::DeserializeOwned, Serialize};
use tracing::info;

pub struct FedimintClient {
    pub base_url: String,
    pub password: String,
    pub active_federation_id: String,
    pub active_gateway_id: String,
    pub built: bool,
}

impl FedimintClient {
    /// Creates a new FedimintClient
    pub fn new() -> Self {
        Self {
            base_url: "".to_string(),
            password: "".to_string(),
            active_federation_id: "".to_string(),
            active_gateway_id: "".to_string(),
            built: false,
        }
    }

    /// Sets the Base URL
    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = format!("{}/v2", base_url);
        self
    }

    /// Sets the Password
    pub fn password(mut self, password: String) -> Self {
        self.password = password;
        self
    }

    /// Sets the default active federation ID
    pub fn active_federation_id(mut self, active_federation_id: String) -> Self {
        self.active_federation_id = active_federation_id;
        self
    }

    /// Sets the default active lightning gateway ID
    pub fn active_gateway_id(mut self, active_gateway_id: String) -> Self {
        self.active_gateway_id = active_gateway_id;
        self
    }

    /// Builds the client. If `base_url`, `password`, and `active_federation_id` are set, returns
    /// Ok(FedimintClient). Errors if any are empty
    pub fn build(mut self) -> Result<Self, String> {
        if self.base_url.is_empty() || self.password.is_empty() {
            return Err("base_url and password must be set".to_string());
        }

        self.built = true;

        Ok(self)
    }

    /// Switches to use a new Federation ID. If `use_default_gateway` is specified, automatically
    /// switches to use the first available lightnig gateway.
    pub async fn switch_federation_id(
        &mut self,
        federation_id: String,
        use_default_gateway: bool,
    ) -> Result<(), String> {
        self.active_federation_id.clone_from(&federation_id);
        info!("Changed active federation id to: {}", federation_id);

        if use_default_gateway {
            match self.use_default_gateway().await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            info!("Clearing active gateway id, must be set manually on lightning calls or setDefaultGatewayId to true");
            self.active_gateway_id = "".to_string();
            Ok(())
        }
    }

    /// Uses the first available lightning gateway
    pub async fn use_default_gateway(&mut self) -> Result<(), String> {
        let gateways = self.list_gateways().await;

        match gateways {
            Ok(gws) => {
                gws[0]
                    .info
                    .gateway_id
                    .clone_into(&mut self.active_gateway_id);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Makes a GET request to the specified `endpoint`.
    /// Returns a deserialized struct of type `T`
    pub(crate) async fn get<T>(&self, endpoint: &str) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        if !self.built {
            return Err(
                "Fedimint Client not built. Call `.build()` after initializing.".to_string(),
            );
        }

        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}{}", self.base_url, endpoint))
            .header("Authorization", format!("Bearer {}", self.password))
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read body".to_string());

            return Err(format!(
                "GET request failed. Status: {}, Body: {}",
                status, body
            ));
        }
        // Deserialize the response JSON into the desired type.
        let json = response.json::<T>().await.map_err(|e| e.to_string())?;
        Ok(json)
    }

    /// Makes a POST request to the specified `endpoint` with a payload/body of type `Req`
    /// Returns a deserialized struct of type `Res`
    pub(crate) async fn post<Req, Res>(&self, endpoint: &str, payload: Req) -> Result<Res, String>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}{}", self.base_url, endpoint))
            .header("Authorization", format!("Bearer {}", self.password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read body".to_string());

            return Err(format!(
                "POST request failed. Status: {}, Body: {}",
                status, body
            ));
        }

        let json = response.json::<Res>().await.map_err(|e| e.to_string())?;
        Ok(json)
    }
}

impl Default for FedimintClient {
    fn default() -> Self {
        Self::new()
    }
}
