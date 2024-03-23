package modules

type OnchainDepositAddressRequest struct {
	Timeout int `json:"timeout"`
}

type OnchainDepositAddressResponse struct {
	OperationId string `json:"operationId"`
	Address     string `json:"address"`
}

type OnchainAwaitDepositRequest struct {
	OperationId string `json:"operationId"`
}

type OnchainAwaitDepositResponse struct {
	Status string `json:"status"`
}

type OnchainWithdrawRequest struct {
	Address   string `json:"address"`
	AmountSat int    `json:"amountSat"`
}

type OnchainWithdrawResponse struct {
	Txid    string `json:"txid"`
	FeesSat int    `json:"feesSat"`
}
