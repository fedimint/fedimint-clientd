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
	BaseURL            string
	Password           string
	ActiveFederationId string
	ActiveGatewayId    string
	Ln                 LnModule
	Onchain            OnchainModule
	Mint               MintModule
}

type LnModule struct {
	Client *FedimintClient
}

type MintModule struct {
	Client *FedimintClient
}

type OnchainModule struct {
	Client *FedimintClient
}

func NewFedimintClient(baseURL, password string, activeFederationId string) *FedimintClient {
	fc := &FedimintClient{
		BaseURL:            baseURL + "/fedimint/v2",
		Password:           password,
		ActiveFederationId: activeFederationId,
	}
	fc.Ln.Client = fc
	fc.Onchain.Client = fc
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
	return fc.ActiveFederationId
}

func (fc *FedimintClient) setActiveFederationId(federationId string, useDefaultGateway bool) {
	fc.ActiveFederationId = federationId
	if useDefaultGateway {
		fc.useDefaultGateway()
	}
}

func (fc *FedimintClient) getActiveGatewayId() string {
	return fc.ActiveGatewayId
}

func (fc *FedimintClient) setActiveGatewayId(gatewayId string) {
	fc.ActiveGatewayId = gatewayId
}

func (fc *FedimintClient) useDefaultGateway() error {
	// hits list_gateways and sets activeGatewayId to the first gateway
	gateways, err := fc.Ln.ListGateways()
	if err != nil {
		return fmt.Errorf("error getting gateways: %w", err)
	}
	if len(gateways) == 0 {
		return fmt.Errorf("no gateways available")
	}
	fc.ActiveGatewayId = gateways[0].Info.GatewayID

	return nil
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

func (fc *FedimintClient) postWithFederationId(endpoint string, body interface{}, federationId string) ([]byte, error) {
	effectiveFederationId := federationId
	if effectiveFederationId == "" {
		effectiveFederationId = fc.ActiveFederationId
	}

	return fc.post(endpoint, map[string]interface{}{
		"body":         body,
		"federationId": effectiveFederationId,
	})
}

func (fc *FedimintClient) postWithGatewayIdAndFederationId(endpoint string, body interface{}, gatewayId string, federationId string) ([]byte, error) {
	effectiveFederationId := federationId
	if effectiveFederationId == "" {
		effectiveFederationId = fc.ActiveFederationId
	}
	effectiveGatewayId := gatewayId
	if effectiveGatewayId == "" {
		effectiveGatewayId = fc.ActiveGatewayId
	}

	return fc.post(endpoint, map[string]interface{}{
		"body":         body,
		"gatewayId":    effectiveGatewayId,
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

func (fc *FedimintClient) Backup(metadata *types.BackupRequest, federationId string) error {
	_, err := fc.postWithFederationId("/admin/backup", metadata, federationId)
	return err
}

func (fc *FedimintClient) DiscoverVersion(threshold *int) (*types.FedimintResponse, error) {
	resp, err := fc.post("/admin/discover-version", threshold)
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

func (fc *FedimintClient) ListOperations(limit int, federationId *string) (*types.OperationOutput, error) {
	request := types.ListOperationsRequest{Limit: limit}
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

func (fc *FedimintClient) Join(inviteCode string, setActiveFederationId bool, useDefaultGateway bool, useManualSecret bool) (types.JoinResponse, error) {
	request := types.JoinRequest{InviteCode: inviteCode, UseManualSecret: useManualSecret}

	var response types.JoinResponse
	responseBody, err := fc.post("/admin/join", request)

	if setActiveFederationId {
		fc.setActiveFederationId(response.ThisFederationId, useDefaultGateway)
	}

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
// Onchain //
////////////

func (onchain *OnchainModule) createDepositAddress(timeout int, federationId *string) (*modules.DepositAddressResponse, error) {
	request := modules.DepositAddressRequest{Timeout: timeout}
	resp, err := onchain.Client.postWithFederationId("/onchain/deposit-address", request, *federationId)
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

func (onchain *OnchainModule) awaitDeposit(operationId string, federationId *string) (*modules.AwaitDepositResponse, error) {
	request := modules.AwaitDepositRequest{OperationId: operationId}
	resp, err := onchain.Client.postWithFederationId("/onchain/await-deposit", request, *federationId)
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

func (onchain *OnchainModule) withdraw(address string, amountSat int, federationId *string) (*modules.WithdrawResponse, error) {
	request := modules.WithdrawRequest{Address: address, AmountSat: amountSat}
	resp, err := onchain.Client.postWithFederationId("/onchain/withdraw", request, *federationId)
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
	resp, err := mint.Client.postWithFederationId("/mint/reissue", request, *federationId)
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
	resp, err := mint.Client.postWithFederationId("/mint/spend", request, *federationId)
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
	resp, err := mint.Client.postWithFederationId("/mint/validate", request, *federationId)
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
	resp, err := ln.Client.postWithFederationId("/ln/invoice", request, *federationId)
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
	resp, err := ln.Client.postWithFederationId("/ln/await-invoice", request, *federationId)
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
	resp, err := ln.Client.postWithFederationId("/ln/pay", request, *federationId)
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
	resp, err := ln.Client.postWithFederationId("/ln/await-pay", request, *federationId)
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
	resp, err := ln.Client.postWithFederationId("/ln/switch-gateway", request, *federationId)
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
