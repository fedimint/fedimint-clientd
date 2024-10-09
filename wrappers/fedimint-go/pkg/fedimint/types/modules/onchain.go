package modules

type OnchainDepositAddressResponse struct {
	OperationId string `json:"operationId"`
	Address     string `json:"address"`
	TweakIdx    int    `json:"tweakIdx"`
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
