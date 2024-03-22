package modules

type DepositAddressRequest struct {
	Timeout int `json:"timeout"`
}

type DepositAddressResponse struct {
	OperationID string `json:"operation_id"`
	Address     string `json:"address"`
}

type AwaitDepositRequest struct {
	OperationID string `json:"operation_id"`
}

type AwaitDepositResponse struct {
	Status string `json:"status"`
}

type WithdrawRequest struct {
	Address    string `json:"address"`
	AmountMsat string `json:"amount_msat"`
}

type WithdrawResponse struct {
	Txid    string `json:"txid"`
	FeesSat int    `json:"fees_sat"`
}
