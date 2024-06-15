use tracing::info;

use crate::FedimintClient;

impl FedimintClient {
    pub fn new() -> Self {
        Self {
            base_url: "".to_string(),
            password: "".to_string(),
            active_federation_id: "".to_string(),
            active_gateway_id: "".to_string(),
            built: false,
        }
    }

    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = format!("{}/v2", base_url);
        self
    }

    pub fn password(mut self, password: String) -> Self {
        self.password = password;
        self
    }

    pub fn active_federation_id(mut self, active_federation_id: String) -> Self {
        self.active_federation_id = active_federation_id;
        self
    }

    pub fn active_gateway_id(mut self, active_gateway_id: String) -> Self {
        self.active_federation_id = active_gateway_id;
        self
    }

    pub fn build(mut self) -> Self {
        if self.base_url.is_empty()
            || self.password.is_empty()
            || self.active_federation_id.is_empty()
        {
            panic!("base_url, password, and active_federation_id must be set");
        }

        self.built = true;

        self
    }

    pub async fn set_active_federation_id(
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
}

impl Default for FedimintClient {
    fn default() -> Self {
        FedimintClient::new()
    }
}
