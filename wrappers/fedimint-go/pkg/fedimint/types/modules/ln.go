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

type LnPayResponse struct {
	PperationId string `json:"operationId"`
	PaymentType string `json:"paymentType"`
	ContractId  string `json:"contractId"`
	Fee         int    `json:"fee"`
}

type AwaitLnPayRequest struct {
	OperationId string `json:"operationId"`
}

type GatewayInfo struct {
	API                     string        `json:"api"`
	Fees                    GatewayFees   `json:"fees"`
	GatewayID               string        `json:"gatewayId"`
	GatewayRedeemKey        string        `json:"gatewayRedeemKey"`
	LightningAlias          string        `json:"lightningAlias"`
	MintChannelID           int           `json:"mintChannelId"`
	NodePubKey              string        `json:"nodePubKey"`
	RouteHints              []interface{} `json:"routeHints"` // Adjust the type according to the actual structure of route hints
	SupportsPrivatePayments bool          `json:"supportsPrivatePayments"`
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
	FederationID string      `json:"federationId"`
	Info         GatewayInfo `json:"info"`
	TTL          GatewayTTL  `json:"ttl"`
	Vetted       bool        `json:"vetted"`
}

// string::> FederationId
type ListGatewaysResponse map[string][]Gateway

type SwitchGatewayRequest struct {
	GatewayId string `json:"gatewayId"`
}
