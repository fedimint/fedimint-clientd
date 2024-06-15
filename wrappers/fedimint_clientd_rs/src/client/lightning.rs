use crate::{
    options::{InvoiceOptions, PayOptions, TweakedInvoiceOptions},
    types::{
        AwaitInvoiceRequest, AwaitInvoiceResponse, ClaimPubkeyTweakRequest, CreateInvoiceRequest,
        CreateInvoiceResponse, CreateTweakedInvoiceRequest, Gateway, LightningPayRequest,
        LightningPayResponse, LightningPaymentResponse, ListGatewaysRequest,
    },
    FedimintClient,
};

impl FedimintClient {
    pub async fn create_invoice(
        &self,
        opts: InvoiceOptions,
    ) -> Result<CreateInvoiceResponse, String> {
        self.post::<CreateInvoiceRequest, CreateInvoiceResponse>(
            "/ln/invoice",
            CreateInvoiceRequest {
                federationId: self.active_federation_id.to_owned(),
                gatewayId: self.active_gateway_id.to_owned(),
                amountMsat: opts.amount_msat,
                description: opts.description,
                expiryTime: opts.expiry_time,
            },
        )
        .await
    }

    pub async fn create_invoice_for_pubkey_tweak(
        &self,
        opts: TweakedInvoiceOptions,
    ) -> Result<CreateInvoiceResponse, String> {
        self.post::<CreateTweakedInvoiceRequest, CreateInvoiceResponse>(
            "/ln/invoice-external-pubkey-tweaked",
            CreateTweakedInvoiceRequest {
                federationId: self.active_federation_id.to_owned(),
                gatewayId: self.active_gateway_id.to_owned(),
                amountMsat: opts.amount_msat,
                description: opts.description,
                externalPubkey: opts.external_pubkey,
                tweak: opts.tweak,
                expiryTime: opts.expiry_time,
            },
        )
        .await
    }

    pub async fn claim_pubkey_tweak_receives(
        &self,
        private_key: String,
        tweaks: Vec<usize>,
    ) -> Result<LightningPaymentResponse, String> {
        self.post::<ClaimPubkeyTweakRequest, LightningPaymentResponse>(
            "/ln/claim-external-receive-tweaked",
            ClaimPubkeyTweakRequest {
                federationId: self.active_federation_id.to_owned(),
                privateKey: private_key,
                tweaks,
            },
        )
        .await
    }

    pub async fn await_invoice(
        &self,
        operation_id: String,
    ) -> Result<AwaitInvoiceResponse, String> {
        self.post::<AwaitInvoiceRequest, AwaitInvoiceResponse>(
            "/ln/await-invoice",
            AwaitInvoiceRequest {
                federationId: self.active_federation_id.to_owned(),
                operationId: operation_id,
            },
        )
        .await
    }

    pub async fn pay(&self, args: PayOptions) -> Result<LightningPayResponse, String> {
        self.post::<LightningPayRequest, LightningPayResponse>(
            "/ln/pay",
            LightningPayRequest {
                federationId: self.active_federation_id.to_owned(),
                gatewayId: self.active_gateway_id.to_owned(),
                paymentInfo: args.payment_info,
                amountMsat: args.amount_msat,
                LightningurlComment: args.lightningurl_comment,
            },
        )
        .await
    }

    pub async fn list_gateways(&self) -> Result<Vec<Gateway>, String> {
        self.post::<ListGatewaysRequest, Vec<Gateway>>(
            "/ln/list-gateways",
            ListGatewaysRequest {
                federationId: self.active_federation_id.to_owned(),
            },
        )
        .await
    }
}
