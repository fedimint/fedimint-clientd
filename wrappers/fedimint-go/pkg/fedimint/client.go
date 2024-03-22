package fedimint

import (
	"bytes"
	"encoding/json"
	"fedimint-go-client/pkg/fedimint/types"
	"fedimint-go-client/pkg/fedimint/types/modules"
	"fmt"
	"io/ioutil"
	"net/http"
)

type FedimintClient struct {
	BaseURL      string
	Password     string
	FederationId string
	Ln           LnModule
	Wallet       WalletModule
	Mint         MintModule
}

type LnModule struct {
	Client *FedimintClient
}

type MintModule struct {
	Client *FedimintClient
}

type WalletModule struct {
	Client *FedimintClient
}

func NewFedimintClient(baseURL, password string, federationId string) *FedimintClient {
	fc := &FedimintClient{
		BaseURL:      baseURL + "/fedimint/v2",
		Password:     password,
		FederationId: federationId,
	}
	fc.Ln.Client = fc
	fc.Wallet.Client = fc
	fc.Mint.Client = fc

	return fc
}

func (fc *FedimintClient) fetchWithAuth(endpoint string, method string, body []byte) ([]byte, error) {
	client := &http.Client{}
	req, err := http.NewRequest(method, fc.BaseURL+endpoint, bytes.NewBuffer(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Authorization", "Bearer "+fc.Password)
	req.Header.Set("Content-Type", "application/json")
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("HTTP error! status: %d", resp.StatusCode)
	}
	return ioutil.ReadAll(resp.Body)
}

func (fc *FedimintClient) getActiveFederationId() string {
	return fc.FederationId
}

func (fc *FedimintClient) setActiveFederationId(federationId string) {
	fc.FederationId = federationId
}

func (fc *FedimintClient) get(endpoint string) ([]byte, error) {
	return fc.fetchWithAuth(endpoint, "GET", nil)
}

func (fc *FedimintClient) post(endpoint string, body interface{}) ([]byte, error) {
	jsonBody, err := json.Marshal(body)
	fmt.Println("jsonBody: ", string(jsonBody))
	if err != nil {
		return nil, err
	}
	return fc.fetchWithAuth(endpoint, "POST", jsonBody)
}

func (fc *FedimintClient) postWithId(endpoint string, body interface{}, federationId string) ([]byte, error) {
	effectiveFederationId := federationId
	if effectiveFederationId == "" {
		effectiveFederationId = fc.FederationId
	}

	return fc.post(endpoint, map[string]interface{}{
		"body":         body,
		"federationId": effectiveFederationId,
	})
}

func (fc *FedimintClient) Info() (*types.InfoResponse, error) {
	resp, err := fc.get("/admin/info")
	if err != nil {
		return nil, err
	}
	var infoResp types.InfoResponse
	err = json.Unmarshal(resp, &infoResp)
	if err != nil {
		return nil, err
	}
	return &infoResp, nil
}

func (fc *FedimintClient) Backup(metadata *types.BackupRequest, federationId string) error {
	_, err := fc.postWithId("/admin/backup", metadata, federationId)
	return err
}

func (fc *FedimintClient) DiscoverVersion() (*types.FedimintResponse, error) {
	resp, err := fc.get("/admin/discover-version")
	if err != nil {
		return nil, err
	}
	var versionResp types.FedimintResponse
	err = json.Unmarshal(resp, &versionResp)
	if err != nil {
		return nil, err
	}
	return &versionResp, nil
}

func (fc *FedimintClient) ListOperations(request *types.ListOperationsRequest, federationId *string) (*types.OperationOutput, error) {
	resp, err := fc.post("/admin/list-operations", request)
	if err != nil {
		return nil, err
	}
	var operationsResp types.OperationOutput
	err = json.Unmarshal(resp, &operationsResp)
	if err != nil {
		return nil, err
	}
	return &operationsResp, nil
}

func (fc *FedimintClient) Config() (*types.FedimintResponse, error) {
	resp, err := fc.get("/admin/config")
	if err != nil {
		return nil, err
	}
	var configResp types.FedimintResponse
	err = json.Unmarshal(resp, &configResp)
	if err != nil {
		return nil, err
	}
	return &configResp, nil
}

func (fc *FedimintClient) Join(inviteCode string, setDefault bool) (types.FederationIdsResponse, error) {
	var response types.FederationIdsResponse
	responseBody, err := fc.post("/admin/join", map[string]interface{}{
		"inviteCode": inviteCode,
		"setDefault": setDefault,
	})

	if err != nil {
		return response, err
	}

	err = json.Unmarshal(responseBody, &response)
	if err != nil {
		return response, err
	}
	return response, nil
}

func (fc *FedimintClient) FederationIds() (types.FederationIdsResponse, error) {
	var response types.FederationIdsResponse
	responseBody, err := fc.get("/admin/federation-ids")

	if err != nil {
		return response, err
	}

	err = json.Unmarshal(responseBody, &response)
	if err != nil {
		return response, err
	}
	return response, nil
}

////////////
// Wallet //
////////////

func (wallet *WalletModule) createDepositAddress(request modules.DepositAddressRequest, federationId *string) (*modules.DepositAddressResponse, error) {
	resp, err := wallet.Client.postWithId("/wallet/deposit-address", request, *federationId)
	if err != nil {
		return nil, err
	}
	var depositAddressResp modules.DepositAddressResponse
	err = json.Unmarshal(resp, &depositAddressResp)
	if err != nil {
		return nil, err
	}
	return &depositAddressResp, nil
}

func (wallet *WalletModule) awaitDeposit(request modules.AwaitDepositRequest, federationId *string) (*modules.AwaitDepositResponse, error) {
	resp, err := wallet.Client.postWithId("/wallet/await-deposit", request, *federationId)
	if err != nil {
		return nil, err
	}
	var depositResp modules.AwaitDepositResponse
	err = json.Unmarshal(resp, &depositResp)
	if err != nil {
		return nil, err
	}
	return &depositResp, nil
}

func (wallet *WalletModule) withdraw(request modules.WithdrawRequest, federationId *string) (*modules.WithdrawResponse, error) {
	resp, err := wallet.Client.postWithId("/wallet/withdraw", request, *federationId)
	if err != nil {
		return nil, err
	}
	var withdrawResp modules.WithdrawResponse
	err = json.Unmarshal(resp, &withdrawResp)
	if err != nil {
		return nil, err
	}
	return &withdrawResp, nil
}

//////////
// mint //
//////////

func (mint *MintModule) Reissue(request modules.ReissueRequest, federationId *string) (*modules.ReissueResponse, error) {
	resp, err := mint.Client.postWithId("/mint/reissue", request, *federationId)
	if err != nil {
		return nil, err
	}
	var reissueResp modules.ReissueResponse
	err = json.Unmarshal(resp, &reissueResp)
	if err != nil {
		return nil, err
	}
	return &reissueResp, nil
}

func (mint *MintModule) Spend(request modules.SpendRequest, federationId *string) (*modules.SpendResponse, error) {
	resp, err := mint.Client.postWithId("/mint/spend", request, *federationId)
	if err != nil {
		return nil, err
	}
	var spendResp modules.SpendResponse
	err = json.Unmarshal(resp, &spendResp)
	if err != nil {
		return nil, err
	}
	return &spendResp, nil
}

func (mint *MintModule) Validate(request modules.ValidateRequest, federationId *string) (*modules.ValidateResponse, error) {
	resp, err := mint.Client.postWithId("/mint/validate", request, *federationId)
	if err != nil {
		return nil, err
	}
	var validateResp modules.ValidateResponse
	err = json.Unmarshal(resp, &validateResp)
	if err != nil {
		return nil, err
	}
	return &validateResp, nil
}

func (mint *MintModule) Split(request modules.SplitRequest) (*modules.SplitResponse, error) {
	resp, err := mint.Client.post("/mint/split", request)
	if err != nil {
		return nil, err
	}
	var splitResp modules.SplitResponse
	err = json.Unmarshal(resp, &splitResp)
	if err != nil {
		return nil, err
	}
	return &splitResp, nil
}

func (mint *MintModule) Combine(request modules.CombineRequest) (*modules.CombineResponse, error) {
	resp, err := mint.Client.post("/mint/combine", request)
	if err != nil {
		return nil, err
	}
	var combineResp modules.CombineResponse
	err = json.Unmarshal(resp, &combineResp)
	if err != nil {
		return nil, err
	}
	return &combineResp, nil
}

////////
// Ln //
////////

func (ln *LnModule) CreateInvoice(request modules.LnInvoiceRequest, federationId *string) (*modules.LnInvoiceResponse, error) {
	fmt.Println("request: ", request)
	resp, err := ln.Client.postWithId("/ln/invoice", request, *federationId)
	if err != nil {
		return nil, err
	}
	var invoiceResp modules.LnInvoiceResponse
	err = json.Unmarshal(resp, &invoiceResp)
	if err != nil {
		return nil, err
	}
	return &invoiceResp, nil
}

func (ln *LnModule) AwaitInvoice(request modules.AwaitInvoiceRequest, federationId *string) (*types.InfoResponse, error) {
	resp, err := ln.Client.postWithId("/ln/await-invoice", request, *federationId)
	if err != nil {
		return nil, err
	}
	var infoResp types.InfoResponse
	err = json.Unmarshal(resp, &infoResp)
	if err != nil {
		return nil, err
	}
	return &infoResp, nil
}

func (ln *LnModule) Pay(request modules.LnPayRequest, federationId *string) (*modules.LnPayResponse, error) {
	resp, err := ln.Client.postWithId("/ln/pay", request, *federationId)
	if err != nil {
		return nil, err
	}
	var payResp modules.LnPayResponse
	err = json.Unmarshal(resp, &payResp)
	if err != nil {
		return nil, err
	}
	return &payResp, nil
}

func (ln *LnModule) AwaitPay(request modules.AwaitLnPayRequest, federationId *string) (*modules.LnPayResponse, error) {
	resp, err := ln.Client.postWithId("/ln/await-pay", request, *federationId)
	if err != nil {
		return nil, err
	}
	var payResp modules.LnPayResponse
	err = json.Unmarshal(resp, &payResp)
	if err != nil {
		return nil, err
	}
	return &payResp, nil
}

func (ln *LnModule) ListGateways() ([]modules.Gateway, error) {
	resp, err := ln.Client.get("/ln/list-gateways")
	if err != nil {
		return nil, err
	}
	var gateways []modules.Gateway
	err = json.Unmarshal(resp, &gateways)
	if err != nil {
		return nil, err
	}
	return gateways, nil
}

func (ln *LnModule) SwitchGateway(request modules.SwitchGatewayRequest, federationId *string) (*modules.Gateway, error) {
	resp, err := ln.Client.postWithId("/ln/switch-gateway", request, *federationId)
	if err != nil {
		return nil, err
	}
	var gateway modules.Gateway
	err = json.Unmarshal(resp, &gateway)
	if err != nil {
		return nil, err
	}
	return &gateway, nil
}
