package modules

type DepositAddressRequest struct {
	Timeout int `json:"timeout"`
}

type DepositAddressResponse struct {
	OperationId string `json:"operationId"`
	Address     string `json:"address"`
}

type AwaitDepositRequest struct {
	OperationId string `json:"operationId"`
}

type AwaitDepositResponse struct {
	Status string `json:"status"`
}

type WithdrawRequest struct {
	Address   string `json:"address"`
	AmountSat int    `json:"amountSat"`
}

type WithdrawResponse struct {
	Txid    string `json:"txid"`
	FeesSat int    `json:"feesSat"`
}
