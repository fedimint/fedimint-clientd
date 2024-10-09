package fedimint

import (
	"bytes"
	"encoding/json"
	"fedimint-go-client/pkg/fedimint/types"
	"fedimint-go-client/pkg/fedimint/types/modules"
	"fmt"
	"io"
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
		BaseURL:            baseURL + "/v2",
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

	responseBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("failed to read response body: %w", err)
	}

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("HTTP error! status: %d, message: %s", resp.StatusCode, string(responseBody))
	}
	return responseBody, nil
}

func (fc *FedimintClient) GetActiveFederationId() string {
	return fc.ActiveFederationId
}

func (fc *FedimintClient) SetActiveFederationId(federationId string, useDefaultGateway bool) {
	fc.ActiveFederationId = federationId
	if useDefaultGateway {
		fc.UseDefaultGateway()
	}
}

func (fc *FedimintClient) GetActiveGatewayId() string {
	return fc.ActiveGatewayId
}

func (fc *FedimintClient) SetActiveGatewayId(gatewayId string) {
	fc.ActiveGatewayId = gatewayId
}

func (fc *FedimintClient) UseDefaultGateway() error {
	// hits list_gateways and sets activeGatewayId to the first gateway
	gateways, err := fc.Ln.ListGateways(nil)
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

// Post performs a POST request with JSON body, handling JSON marshalling within.
func (fc *FedimintClient) post(endpoint string, body interface{}) ([]byte, error) {
	jsonBody, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("error marshaling request body: %w", err)
	}

	return fc.fetchWithAuth(endpoint, "POST", jsonBody)
}

// postWithFederationId takes any request object, marshals it to JSON, optionally adds a federationId, and makes a POST request.
func (fc *FedimintClient) postWithFederationId(endpoint string, requestBody interface{}, federationId *string) ([]byte, error) {
	// Initialize an empty map for the request body.
	requestMap := make(map[string]interface{})

	// If the requestBody is not nil and not empty, marshal and unmarshal it into the map.
	if requestBody != nil {
		requestJSON, err := json.Marshal(requestBody)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal request body: %w", err)
		}

		// Unmarshal the JSON into the map only if it's not empty to avoid overwriting the initialized map.
		if string(requestJSON) != "{}" {
			err = json.Unmarshal(requestJSON, &requestMap)
			if err != nil {
				return nil, fmt.Errorf("failed to unmarshal request JSON into map: %w", err)
			}
		}
	}

	// Determine the effective federationId to use
	var effectiveFederationId string
	if federationId != nil {
		effectiveFederationId = *federationId
	} else {
		effectiveFederationId = fc.ActiveFederationId
	}
	// Add federationId to the map, which is now guaranteed to be initialized.
	requestMap["federationId"] = effectiveFederationId

	// Marshal the request map back to JSON to use as the request body.
	modifiedRequestJSON, err := json.Marshal(requestMap)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal modified request map: %w", err)
	}

	// Proceed to make the POST request with the modified JSON body.
	return fc.fetchWithAuth(endpoint, "POST", modifiedRequestJSON)
}

func (fc *FedimintClient) postWithGatewayIdAndFederationId(endpoint string, requestBody interface{}, gatewayId *string, federationId *string) ([]byte, error) {
	// Initialize an empty map for the request body.
	requestMap := make(map[string]interface{})

	// If the requestBody is not nil and not empty, marshal and unmarshal it into the map.
	if requestBody != nil {
		requestJSON, err := json.Marshal(requestBody)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal request body: %w", err)
		}

		// Unmarshal the JSON into the map only if it's not empty to avoid overwriting the initialized map.
		if string(requestJSON) != "{}" {
			err = json.Unmarshal(requestJSON, &requestMap)
			if err != nil {
				return nil, fmt.Errorf("failed to unmarshal request JSON into map: %w", err)
			}
		}
	}

	// Determine the effective federationId to use
	effectiveFederationId := fc.ActiveFederationId
	if federationId != nil {
		effectiveFederationId = *federationId
	}
	fmt.Printf("effectiveFederationId: %s\n", effectiveFederationId)
	requestMap["federationId"] = effectiveFederationId

	// Determine the effective gatewayId to use
	effectiveGatewayId := fc.ActiveGatewayId
	if gatewayId != nil {
		effectiveGatewayId = *gatewayId
	}
	fmt.Printf("effectiveGatewayId: %s\n", effectiveGatewayId)

	// Add gatewayId to the map, which is now guaranteed to be initialized.
	requestMap["gatewayId"] = effectiveGatewayId

	// Marshal the request map back to JSON to use as the request body.
	modifiedRequestJSON, err := json.Marshal(requestMap)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal modified request map: %w", err)
	}

	fmt.Printf("modifiedRequestJSON: %s\n", modifiedRequestJSON)

	// Proceed to make the POST request with the modified JSON body.
	return fc.fetchWithAuth(endpoint, "POST", modifiedRequestJSON)
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

func (fc *FedimintClient) Backup(metadata *types.BackupRequest, federationId *string) error {
	_, err := fc.postWithFederationId("/admin/backup", metadata, federationId)
	return err
}

func (fc *FedimintClient) DiscoverVersion(federationId *string) (*types.FedimintResponse, error) {
	resp, err := fc.postWithFederationId("/admin/discover-version", nil, federationId)
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

func (fc *FedimintClient) ListOperations(limit uint16, federationId *string) (*types.ListOperationsResponse, error) {
	request := types.ListOperationsRequest{Limit: limit}
	resp, err := fc.postWithFederationId("/admin/list-operations", request, federationId)
	if err != nil {
		return nil, err
	}
	var operationsResp types.ListOperationsResponse
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
	if err != nil {
		return response, err
	}

	err = json.Unmarshal(responseBody, &response)
	if err != nil {
		return response, err
	}

	if setActiveFederationId {
		fc.SetActiveFederationId(response.ThisFederationId, useDefaultGateway)
	}
	return response, nil
}

////////////
// Onchain //
////////////

func (onchain *OnchainModule) CreateDepositAddress(federationId *string) (*modules.OnchainDepositAddressResponse, error) {
	resp, err := onchain.Client.postWithFederationId("/onchain/deposit-address", nil, federationId)
	if err != nil {
		return nil, err
	}
	var depositAddressResp modules.OnchainDepositAddressResponse
	err = json.Unmarshal(resp, &depositAddressResp)
	if err != nil {
		return nil, err
	}
	return &depositAddressResp, nil
}

func (onchain *OnchainModule) AwaitDeposit(operationId string, federationId *string) (*modules.OnchainAwaitDepositResponse, error) {
	request := modules.OnchainAwaitDepositRequest{OperationId: operationId}
	resp, err := onchain.Client.postWithFederationId("/onchain/await-deposit", request, federationId)
	if err != nil {
		return nil, err
	}
	var depositResp modules.OnchainAwaitDepositResponse
	err = json.Unmarshal(resp, &depositResp)
	if err != nil {
		return nil, err
	}
	return &depositResp, nil
}

func (onchain *OnchainModule) Withdraw(address string, amountSat int, federationId *string) (*modules.OnchainWithdrawResponse, error) {
	request := modules.OnchainWithdrawRequest{Address: address, AmountSat: amountSat}
	resp, err := onchain.Client.postWithFederationId("/onchain/withdraw", request, federationId)
	if err != nil {
		return nil, err
	}
	var withdrawResp modules.OnchainWithdrawResponse
	err = json.Unmarshal(resp, &withdrawResp)
	if err != nil {
		return nil, err
	}
	return &withdrawResp, nil
}

//////////
// mint //
//////////

func (mint *MintModule) DecodeNotes(notes string) (*modules.DecodeNotesResponse, error) {
	request := modules.DecodeNotesRequest{Notes: notes}
	resp, err := mint.Client.post("/mint/decode-notes", request)
	if err != nil {
		return nil, err
	}
	var decodeResp modules.DecodeNotesResponse
	err = json.Unmarshal(resp, &decodeResp)
	if err != nil {
		return nil, err
	}
	return &decodeResp, nil
}

func (mint *MintModule) EncodeNotes(notesJson modules.NotesJson) (*modules.EncodeNotesResponse, error) {
	notesJsonStr, err := json.Marshal(notesJson)
	if err != nil {
		return nil, err
	}
	request := modules.EncodeNotesRequest{NotesJsonStr: string(notesJsonStr)}
	resp, err := mint.Client.post("/mint/encode-notes", request)
	if err != nil {
		return nil, err
	}
	var encodeResp modules.EncodeNotesResponse
	err = json.Unmarshal(resp, &encodeResp)
	if err != nil {
		return nil, err
	}
	return &encodeResp, nil
}

func (mint *MintModule) Reissue(notes string, federationId *string) (*modules.MintReissueResponse, error) {
	request := modules.MintReissueRequest{Notes: notes}
	resp, err := mint.Client.postWithFederationId("/mint/reissue", request, federationId)
	if err != nil {
		return nil, err
	}
	var reissueResp modules.MintReissueResponse
	err = json.Unmarshal(resp, &reissueResp)
	if err != nil {
		return nil, err
	}
	return &reissueResp, nil
}

func (mint *MintModule) Spend(amountMsat uint64, allowOverpay bool, timeout uint64, includeInvite bool, federationId *string) (*modules.MintSpendResponse, error) {
	request := modules.MintSpendRequest{
		AmountMsat:    amountMsat,
		AllowOverpay:  allowOverpay,
		Timeout:       timeout,
		IncludeInvite: includeInvite,
	}
	resp, err := mint.Client.postWithFederationId("/mint/spend", request, federationId)
	if err != nil {
		return nil, err
	}
	var spendResp modules.MintSpendResponse
	err = json.Unmarshal(resp, &spendResp)
	if err != nil {
		return nil, err
	}
	return &spendResp, nil
}

func (mint *MintModule) Validate(notes string, federationId *string) (*modules.MintValidateResponse, error) {
	request := modules.MintValidateRequest{Notes: notes}
	resp, err := mint.Client.postWithFederationId("/mint/validate", request, federationId)
	if err != nil {
		return nil, err
	}
	var validateResp modules.MintValidateResponse
	err = json.Unmarshal(resp, &validateResp)
	if err != nil {
		return nil, err
	}
	return &validateResp, nil
}

func (mint *MintModule) Split(notes string) (*modules.MintSplitResponse, error) {
	request := modules.MintSplitRequest{Notes: notes}
	resp, err := mint.Client.post("/mint/split", request)
	if err != nil {
		return nil, err
	}
	var splitResp modules.MintSplitResponse
	err = json.Unmarshal(resp, &splitResp)
	if err != nil {
		return nil, err
	}
	return &splitResp, nil
}

func (mint *MintModule) Combine(notesVec []string) (*modules.MintCombineResponse, error) {
	request := modules.MintCombineRequest{NotesVec: notesVec}
	resp, err := mint.Client.post("/mint/combine", request)
	if err != nil {
		return nil, err
	}
	var combineResp modules.MintCombineResponse
	err = json.Unmarshal(resp, &combineResp)
	if err != nil {
		return nil, err
	}
	return &combineResp, nil
}

////////
// Ln //
////////

func (ln *LnModule) CreateInvoice(amountMsat uint64, description string, expiryTime *int, gatewayId *string, federationId *string) (*modules.LnInvoiceResponse, error) {
	request := modules.LnInvoiceRequest{
		AmountMsat:  amountMsat,
		Description: description,
		ExpiryTime:  expiryTime,
	}
	resp, err := ln.Client.postWithGatewayIdAndFederationId("/ln/invoice", request, gatewayId, federationId)
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

func (ln *LnModule) CreateInvoiceForPubkey(pubkey string, amountMsat uint64, description string, gatewayId string, expiryTime *int, federationId *string) (*modules.LnInvoiceResponse, error) {
	request := modules.LnInvoiceExternalPubkeyRequest{
		AmountMsat:     amountMsat,
		Description:    description,
		ExpiryTime:     expiryTime,
		ExternalPubkey: pubkey,
	}
	fmt.Println("request: ", request)
	resp, err := ln.Client.postWithGatewayIdAndFederationId("/ln/invoice-external-pubkey", request, &gatewayId, federationId)
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

func (ln *LnModule) CreateInvoiceForPubkeyTweak(pubkey string, tweak uint64, amountMsat uint64, description string, gatewayId string, expiryTime *int, federationId *string) (*modules.LnInvoiceResponse, error) {
	request := modules.LnInvoiceExternalPubkeyTweakedRequest{
		AmountMsat:     amountMsat,
		Description:    description,
		ExpiryTime:     expiryTime,
		ExternalPubkey: pubkey,
		Tweak:          tweak,
	}
	fmt.Println("request: ", request)
	resp, err := ln.Client.postWithGatewayIdAndFederationId("/ln/invoice-external-pubkey-tweaked", request, &gatewayId, federationId)
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

func (ln *LnModule) ClaimPubkeyTweakReceive(privateKey string, tweaks []uint64, gatewayId *string, federationId *string) (*modules.LnPaymentResponse, error) {
	request := modules.LnClaimPubkeyTweakedRequest{PrivateKey: privateKey, Tweaks: tweaks}
	resp, err := ln.Client.postWithGatewayIdAndFederationId("/ln/claim-external-receive-tweaked", request, gatewayId, federationId)
	if err != nil {
		return nil, err
	}
	var paymentResp modules.LnPaymentResponse
	err = json.Unmarshal(resp, &paymentResp)
	if err != nil {
		return nil, err
	}
	return &paymentResp, nil
}

func (ln *LnModule) AwaitInvoice(operationId string, gatewayId string, federationId *string) (*modules.LnPaymentResponse, error) {
	request := modules.LnAwaitInvoiceRequest{OperationId: operationId}
	resp, err := ln.Client.postWithGatewayIdAndFederationId("/ln/await-invoice", request, &gatewayId, federationId)
	if err != nil {
		return nil, err
	}
	var paymentResp modules.LnPaymentResponse
	err = json.Unmarshal(resp, &paymentResp)
	if err != nil {
		return nil, err
	}
	return &paymentResp, nil
}

func (ln *LnModule) Pay(paymentInfo string, gatewayId *string, amountMsat *uint64, lnurlComment *string, federationId *string) (*modules.LnPayResponse, error) {
	request := modules.LnPayRequest{
		PaymentInfo:  paymentInfo,
		AmountMsat:   amountMsat,
		LnurlComment: lnurlComment,
	}
	fmt.Println("request: ", request)
	resp, err := ln.Client.postWithGatewayIdAndFederationId("/ln/pay", request, gatewayId, federationId)
	if err != nil {
		return nil, err
	}

	fmt.Printf("Raw response body: %s\n", string(resp))

	var payResp modules.LnPayResponse
	err = json.Unmarshal(resp, &payResp)
	if err != nil {
		return nil, err
	}
	return &payResp, nil
}

func (ln *LnModule) ListGateways(federationId *string) ([]modules.Gateway, error) {
	resp, err := ln.Client.postWithFederationId("/ln/list-gateways", nil, federationId)
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
