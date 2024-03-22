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

type Gateway struct {
	Node_pub_key string `json:"node_pub_key"`
	Active       bool   `json:"active"`
}

// string::> FederationId
type ListGatewaysResponse map[string][]Gateway

type SwitchGatewayRequest struct {
	Gateway_id string `json:"gateway_id"`
}
