pub struct SpendOptions {
    pub amount_msat: u64,
    pub allow_overpay: bool,
    pub timeout: u64,
    pub include_invite: bool,
}

impl SpendOptions {
    pub fn new() -> Self {
        SpendOptions {
            amount_msat: 0,
            allow_overpay: false,
            timeout: 0,
            include_invite: false,
        }
    }

    pub fn msats(mut self, msats: u64) -> Self {
        self.amount_msat = msats;
        self
    }

    pub fn sats(mut self, sats: u64) -> Self {
        self.amount_msat = sats * 1000;
        self
    }

    pub fn allow_overpay(mut self, allow_overpay: bool) -> Self {
        self.allow_overpay = allow_overpay;
        self
    }

    pub fn include_invite(mut self, include_invite: bool) -> Self {
        self.include_invite = include_invite;
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

impl Default for SpendOptions {
    fn default() -> Self {
        Self::new()
    }
}
