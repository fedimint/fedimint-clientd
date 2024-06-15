use crate::FedimintClient;
use serde::{de::DeserializeOwned, Serialize};

impl FedimintClient {
    /// Makes a GET request to the specified `endpoint`.
    /// Returns a deserialized struct of type `T`
    pub(crate) async fn get<T>(&self, endpoint: &str) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        if !self.built {
            panic!("Fedimint Client not built. Call `.build()` after initializing.")
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
