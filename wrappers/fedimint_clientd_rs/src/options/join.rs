pub struct JoinOptions {
    pub invite_code: String,
    pub use_default_gateway: bool,
    pub set_active_federation_id: bool,
    pub use_manual_secret: bool,
}

impl JoinOptions {
    pub fn new(invite_code: String) -> Self {
        JoinOptions {
            invite_code,
            set_active_federation_id: false,
            use_default_gateway: false,
            use_manual_secret: false,
        }
    }

    pub fn set_active_federation_id(mut self) -> Self {
        self.set_active_federation_id = true;
        self
    }

    pub fn use_default_gateway(mut self) -> Self {
        self.use_default_gateway = true;
        self
    }

    pub fn use_manual_secret(mut self) -> Self {
        self.use_manual_secret = true;
        self
    }
}
