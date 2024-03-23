package modules

type LnInvoiceRequest struct {
	AmountMsat  int    `json:"amount_msat"`
	Description string `json:"description"`
	ExpiryTime  *int   `json:"expiry_time"`
}

type LnInvoiceResponse struct {
	OperationID string `json:"operation_id"`
	Invoice     string `json:"invoice"`
}

type AwaitInvoiceRequest struct {
	OperationID string `json:"operation_id"`
}

type LnPayRequest struct {
	Payment_info         string  `json:"payment_info"`
	Amount_msat          *int    `json:"amount_msat"`
	Finish_in_background bool    `json:"finish_in_background"`
	Lnurl_comment        *string `json:"lnurl_comment"`
}

type LnPayResponse struct {
	Pperation_id string `json:"operation_id"`
	Payment_type string `json:"payment_type"`
	Contract_id  string `json:"contract_id"`
	Fee          int    `json:"fee"`
}

type AwaitLnPayRequest struct {
	Operation_id string `json:"operation_id"`
}

type GatewayInfo struct {
	API                     string        `json:"api"`
	Fees                    GatewayFees   `json:"fees"`
	GatewayID               string        `json:"gateway_id"`
	GatewayRedeemKey        string        `json:"gateway_redeem_key"`
	LightningAlias          string        `json:"lightning_alias"`
	MintChannelID           int           `json:"mint_channel_id"`
	NodePubKey              string        `json:"node_pub_key"`
	RouteHints              []interface{} `json:"route_hints"` // Adjust the type according to the actual structure of route hints
	SupportsPrivatePayments bool          `json:"supports_private_payments"`
}

type GatewayFees struct {
	BaseMsat               int `json:"base_msat"`
	ProportionalMillionths int `json:"proportional_millionths"`
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

// string::> FederationId
type ListGatewaysResponse map[string][]Gateway

type SwitchGatewayRequest struct {
	Gateway_id string `json:"gateway_id"`
}
