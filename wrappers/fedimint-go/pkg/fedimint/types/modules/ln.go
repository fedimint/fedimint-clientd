package modules

type LnInvoiceRequest struct {
	AmountMsat  uint64 `json:"amountMsat"`
	Description string `json:"description"`
	ExpiryTime  *int   `json:"expiryTime"`
}

type LnInvoiceExternalPubkeyRequest struct {
	AmountMsat     uint64 `json:"amountMsat"`
	Description    string `json:"description"`
	ExpiryTime     *int   `json:"expiryTime"`
	ExternalPubkey string `json:"externalPubkey"`
}

type LnInvoiceExternalPubkeyTweakedRequest struct {
	AmountMsat     uint64 `json:"amountMsat"`
	Description    string `json:"description"`
	ExpiryTime     *int   `json:"expiryTime"`
	ExternalPubkey string `json:"externalPubkey"`
	Tweak          uint64 `json:"tweak"`
}

type LnInvoiceResponse struct {
	OperationId string `json:"operationId"`
	Invoice     string `json:"invoice"`
}

type LnClaimPubkeyReceiveRequest struct {
	PrivateKey string `json:"privateKey"`
}

type LnClaimPubkeyTweakedRequest struct {
	PrivateKey string   `json:"privateKey"`
	Tweaks     []uint64 `json:"tweaks"`
}

type LnAwaitInvoiceRequest struct {
	OperationId string `json:"operationId"`
}

type LnPayRequest struct {
	PaymentInfo  string  `json:"paymentInfo"`
	AmountMsat   *uint64 `json:"amountMsat"`
	LnurlComment *string `json:"lnurlComment"`
}

type PaymentTypeInfo struct {
	Internal  *string `json:"internal,omitempty"`
	Lightning *string `json:"lightning,omitempty"`
}

type LnPayResponse struct {
	OperationId string          `json:"operationId"`
	PaymentType PaymentTypeInfo `json:"paymentType"`
	ContractId  string          `json:"contractId"`
	Fee         int             `json:"fee"`
}

type AwaitLnPayRequest struct {
	OperationId string `json:"operationId"`
}

type GatewayInfo struct {
	API                     string        `json:"api"`
	Fees                    GatewayFees   `json:"fees"`
	GatewayID               string        `json:"gateway_id"`
	GatewayRedeemKey        string        `json:"gateway_redeem_key"`
	LightningAlias          string        `json:"lightning_alias"`
	MintChannelID           int           `json:"mint_channel_id"`
	NodePubKey              string        `json:"node_pub_key"`
	RouteHints              []interface{} `json:"route_hints"` // Consider defining a more specific type if possible
	SupportsPrivatePayments bool          `json:"supports_private_payments"`
}

type GatewayFees struct {
	BaseMsat               int `json:"baseMsat"`
	ProportionalMillionths int `json:"proportionalMillionths"`
}

type GatewayTTL struct {
	Nanos int `json:"nanos"`
	Secs  int `json:"secs"`
}

type Gateway struct {
	FederationID string      `json:"federation_id"`
	Info         GatewayInfo `json:"info"`
	TTL          GatewayTTL  `json:"ttl"`
	Vetted       bool        `json:"vetted"`
}

type ListGatewaysResponse map[string][]Gateway
