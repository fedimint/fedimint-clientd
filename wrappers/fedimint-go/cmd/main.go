package main

import (
	"encoding/hex"
	"encoding/json"
	"fedimint-go-client/pkg/fedimint"
	"fmt"
	"os"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/joho/godotenv"
)

func logMethod(method string) {
	fmt.Println("--------------------")
	fmt.Printf("Method: %s\n", method)
}

func logInputAndOutput(input interface{}, output interface{}) {
	fmt.Println("Input: ", input)
	fmt.Println("Output: ", output)
	fmt.Println("--------------------")
}

type KeyPair struct {
	PrivateKey string
	PublicKey  string
}

func newKeyPair() KeyPair {
	privateKey, err := btcec.NewPrivateKey()
	if err != nil {
		panic("failed to generate a private key")
	}

	pubKey := privateKey.PubKey()
	return KeyPair{
		PrivateKey: hex.EncodeToString(privateKey.Serialize()),
		PublicKey:  hex.EncodeToString(pubKey.SerializeCompressed()),
	}
}

func buildTestClient() *fedimint.FedimintClient {
	err := godotenv.Load(".env")
	if err != nil {
		fmt.Println("Error loading .env file")
	}

	baseUrl := os.Getenv("FEDIMINT_CLIENTD_BASE_URL")
	if baseUrl == "" {
		baseUrl = "http://127.0.0.1:3333"
	}

	password := os.Getenv("FEDIMINT_CLIENTD_PASSWORD")
	if password == "" {
		password = "password"
	}

	federationId := os.Getenv("FEDIMINT_CLIENTD_ACTIVE_FEDERATION_ID")
	if federationId == "" {
		federationId = "15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3"
	}

	return fedimint.NewFedimintClient(baseUrl, password, federationId)
}

func main() {
	fc := buildTestClient()
	fc.UseDefaultGateway()
	keyPair := newKeyPair()
	fmt.Printf("Generated Key Pair: ")
	fmt.Printf("       Private Key: %s\n", keyPair.PrivateKey)
	fmt.Printf("        Public Key: %s\n", keyPair.PublicKey)

	///////////////////
	// ADMIN METHODS //
	///////////////////

	// `/v2/admin/config`
	logMethod("/v2/admin/config")
	data, err := fc.Config()
	if err != nil {
		fmt.Println("Error calling CONFIG: ", err)
		return
	}
	logInputAndOutput(nil, data)

	// `/v2/admin/discover-version`
	logMethod("/v2/admin/discover-version")
	discoverResponseData, err := fc.DiscoverVersion(1)
	if err != nil {
		fmt.Println("Error calling VERSION: ", err)
		return
	}

	jsonBytes, err := json.Marshal(discoverResponseData)
	if err != nil {
		fmt.Println("Error marshaling JSON(discover-version):", err)
		return
	}
	var fedimintResponseData interface{}
	err = json.Unmarshal(jsonBytes, &fedimintResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(discover-version):", err)
		return
	}

	logInputAndOutput(1, fedimintResponseData)

	// `/v2/admin/federation-ids
	logMethod("/v2/admin/federation-ids")
	federationIds, err := fc.FederationIds()
	if err != nil {
		fmt.Println("Error calling FEDERATION_IDS: ", err)
		return
	}
	logInputAndOutput(nil, federationIds)

	// `/v2/admin/info`
	logMethod("/v2/admin/info")
	infoDataResponse, err := fc.Info()
	if err != nil {
		fmt.Println("Error calling INFO: ", err)
		return
	}

	jsonBytes, err = json.Marshal(infoDataResponse)
	if err != nil {
		fmt.Println("Error marshaling JSON(discover-version):", err)
		return
	}
	var infoResponseData interface{}
	err = json.Unmarshal(jsonBytes, &infoResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(discover-version):", err)
		return
	}

	logInputAndOutput(nil, infoResponseData)

	// `/v2/admin/join`
	inviteCode := os.Getenv("FEDIMINT_CLIENTD_BASE_URL")
	if inviteCode == "" {
		inviteCode = "fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75"
	}
	logMethod("/v2/admin/join")
	joinData, err := fc.Join(inviteCode, true, true, false)
	if err != nil {
		fmt.Println("Error calling JOIN: ", err)
		return
	}

	jsonBytes, err = json.Marshal(joinData)
	if err != nil {
		fmt.Println("Error marshaling JSON(join):", err)
		return
	}
	var joinResponseData interface{}
	err = json.Unmarshal(jsonBytes, &joinResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(join):", err)
		return
	}

	logInputAndOutput(inviteCode, joinResponseData)

	// `/v2/admin/list-operations`
	logMethod("/v2/admin/list-operations")
	listOperationsData, err := fc.ListOperations(10, nil)
	if err != nil {
		fmt.Println("Error calling JOIN: ", err)
		return
	}

	jsonBytes, err = json.Marshal(listOperationsData)
	if err != nil {
		fmt.Println("Error marshaling JSON(list-operations):", err)
		return
	}
	var listOperationsResponseData interface{}
	err = json.Unmarshal(jsonBytes, &listOperationsResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(list-operations):", err)
		return
	}

	logInputAndOutput([]interface{}{10}, listOperationsResponseData)

	///////////////////////
	// LIGHTNING METHODS //
	///////////////////////

	// `/v2/ln/list-gateways`
	logMethod("/v2/ln/list-gateways")
	gatewayList, err := fc.Ln.ListGateways(nil)
	if err != nil {
		fmt.Println("Error calling LIST_GATEWAYS: ", err)
		return
	}

	jsonBytes, err = json.Marshal(gatewayList)
	if err != nil {
		fmt.Println("Error marshaling JSON(list-gateways):", err)
		return
	}
	var gatewayListResponseData interface{}
	err = json.Unmarshal(jsonBytes, &gatewayListResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(list-gateways):", err)
		return
	}

	logInputAndOutput(nil, gatewayListResponseData)

	// `/v2/ln/invoice`
	logMethod("/v2/ln/invoice")
	invoiceData, err := fc.Ln.CreateInvoice(10000, "test", nil, nil, nil)
	if err != nil {
		fmt.Println("Error calling INVOICE: ", err)
		return
	}

	jsonBytes, err = json.Marshal(invoiceData)
	if err != nil {
		fmt.Println("Error marshaling JSON(invoice):", err)
		return
	}
	var invoiceResponseData interface{}
	err = json.Unmarshal(jsonBytes, &invoiceResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(invoice):", err)
		return
	}

	logInputAndOutput([]interface{}{10000, "test_Invoice"}, invoiceResponseData)

	// `/v2/ln/pay`
	logMethod("/v2/ln/pay")
	payData, err := fc.Ln.Pay(invoiceData.Invoice, fc.GetActiveGatewayId(), nil, nil, nil)
	if err != nil {
		fmt.Println("Error calling PAY: ", err)
		return
	}

	jsonBytes, err = json.Marshal(payData)
	if err != nil {
		fmt.Println("Error marshaling JSON(pay):", err)
		return
	}
	var payResponseData interface{}
	err = json.Unmarshal(jsonBytes, &payResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(pay):", err)
		return
	}

	logInputAndOutput(invoiceData.Invoice, payResponseData)

	// /v2/ln/await-invoice
	logMethod("/v2/ln/await-invoice")
	awaitInvoiceData, err := fc.Ln.AwaitInvoice(invoiceData.OperationId, fc.GetActiveGatewayId(), nil)
	if err != nil {
		fmt.Println("Error calling AWAIT_INVOICE: ", err)
		return
	}

	jsonBytes, err = json.Marshal(awaitInvoiceData)
	if err != nil {
		fmt.Println("Error marshaling JSON(await-invoice):", err)
		return
	}
	var awaitInvoiceResponseData interface{}
	err = json.Unmarshal(jsonBytes, &awaitInvoiceResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(await-invoice):", err)
		return
	}

	logInputAndOutput(invoiceData.OperationId, awaitInvoiceResponseData)

	// `/v1/ln/invoice-external-pubkey-tweaked`
	logMethod("/v1/ln/invoice-external-pubkey-tweaked")
	tweakInvoice, err := fc.Ln.CreateInvoiceForPubkeyTweak(keyPair.PublicKey, 1, 10000, "test", fc.GetActiveGatewayId(), nil, nil)
	if err != nil {
		fmt.Println("Error calling CREATE_INVOICE_FOR_PUBKEY_TWEAK: ", err)
		return
	}

	jsonBytes, err = json.Marshal(tweakInvoice)
	if err != nil {
		fmt.Println("Error marshaling JSON(await-invoice):", err)
		return
	}
	var tweakInvoiceResponseData interface{}
	err = json.Unmarshal(jsonBytes, &tweakInvoiceResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(await-invoice):", err)
		return
	}

	logInputAndOutput([]interface{}{keyPair.PublicKey, 1, 10000, "test"}, tweakInvoiceResponseData)
	// pay the invoice
	_, _ = fc.Ln.Pay(tweakInvoice.Invoice, fc.GetActiveGatewayId(), nil, nil, nil)
	fmt.Println("Paid locked invoice!")

	// `/v1/ln/claim-external-pubkey-tweaked`
	logMethod("/v1/ln/claim-external-pubkey-tweaked")
	claimInvoice, err := fc.Ln.ClaimPubkeyTweakReceive(keyPair.PrivateKey, []uint64{1}, fc.GetActiveFederationId(), fc.GetActiveFederationId())
	if err != nil {
		fmt.Println("Error calling CLAIM_PUBKEY_RECEIVE_TWEAKED: ", err)
		return
	}

	jsonBytes, err = json.Marshal(claimInvoice)
	if err != nil {
		fmt.Println("Error marshaling JSON(claim-external-pubkey-tweaked):", err)
		return
	}
	var claimInvoiceResponseData interface{}
	err = json.Unmarshal(jsonBytes, &claimInvoiceResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(claim-external-pubkey-tweaked):", err)
		return
	}

	logInputAndOutput([]interface{}{keyPair.PrivateKey, []uint64{1}}, claimInvoiceResponseData)

	//////////////////
	// MINT METHODS //
	//////////////////

	// `/v2/mint/spend`
	logMethod("/v2/mint/spend")
	mintData, err := fc.Mint.Spend(3000, true, 1000, false, nil)
	if err != nil {
		fmt.Println("Error calling SPEND: ", err)
		return
	}

	jsonBytes, err = json.Marshal(mintData)
	if err != nil {
		fmt.Println("Error marshaling JSON(spend):", err)
		return
	}
	var mintResponseData interface{}
	err = json.Unmarshal(jsonBytes, &mintResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(spend):", err)
		return
	}

	logInputAndOutput([]interface{}{3000, true, 1000}, mintResponseData)

	// `/v2/mint/decode-notes`
	logMethod("/v2/mint/decode-notes")
	decodedData, err := fc.Mint.DecodeNotes(mintData.Notes)
	if err != nil {
		fmt.Println("Error calling DECODE_NOTES: ", err)
		return
	}

	jsonBytes, err = json.Marshal(decodedData)
	if err != nil {
		fmt.Println("Error marshaling JSON(decode-notes):", err)
		return
	}
	var decodedResponseData interface{}
	err = json.Unmarshal(jsonBytes, &decodedResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(decode-notes):", err)
		return
	}

	logInputAndOutput(mintData.Notes, decodedResponseData)

	// `/v2/mint/encode-notes`
	logMethod("/v2/mint/encode-notes")
	encodedData, err := fc.Mint.EncodeNotes(decodedData.NotesJson)
	if err != nil {
		fmt.Println("Error calling DECODE_NOTES: ", err)
		return
	}

	jsonBytes, err = json.Marshal(encodedData)
	if err != nil {
		fmt.Println("Error marshaling JSON(encode-notes):", err)
		return
	}
	var encodedResponseData interface{}
	err = json.Unmarshal(jsonBytes, &encodedResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(encode-notes):", err)
		return
	}

	logInputAndOutput(decodedData.NotesJson, encodedResponseData)

	// `/v2/mint/validate`
	logMethod("/v2/mint/validate")
	validateData, err := fc.Mint.Validate(mintData.Notes, nil)
	if err != nil {
		fmt.Println("Error calling VALIDATE: ", err)
		return
	}

	jsonBytes, err = json.Marshal(validateData)
	if err != nil {
		fmt.Println("Error marshaling JSON(validate):", err)
		return
	}
	var validateResponseData interface{}
	err = json.Unmarshal(jsonBytes, &validateResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(validate):", err)
		return
	}

	logInputAndOutput(mintData.Notes, validateResponseData)

	// `/v2/mint/reissue`
	logMethod("/v2/mint/reissue")
	reissueData, err := fc.Mint.Reissue(mintData.Notes, nil)
	if err != nil {
		fmt.Println("Error calling REISSUE: ", err)
		return
	}

	jsonBytes, err = json.Marshal(reissueData)
	if err != nil {
		fmt.Println("Error marshaling JSON(reissue):", err)
		return
	}
	var reissueResponseData interface{}
	err = json.Unmarshal(jsonBytes, &reissueResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(reissue):", err)
		return
	}

	logInputAndOutput(mintData.Notes, reissueResponseData)

	// `/v2/mint/split`
	logMethod("/v2/mint/split")
	splitData, err := fc.Mint.Split(mintData.Notes)
	if err != nil {
		fmt.Println("Error calling SPLIT: ", err)
		return
	}

	jsonBytes, err = json.Marshal(splitData)
	if err != nil {
		fmt.Println("Error marshaling JSON(split):", err)
		return
	}
	var splitResponseData interface{}
	err = json.Unmarshal(jsonBytes, &splitResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(split):", err)
		return
	}

	logInputAndOutput(mintData.Notes, splitResponseData)

	// `/v2/mint/combine`
	logMethod("/v2/mint/combine")
	notesVec := func() []string {
		result := make([]string, 0, len(splitData.Notes))
		for _, value := range splitData.Notes {
			result = append(result, value)
		}
		return result
	}()
	combineData, err := fc.Mint.Combine(notesVec)
	if err != nil {
		fmt.Println("Error calling COMBINE: ", err)
		return
	}

	jsonBytes, err = json.Marshal(combineData)
	if err != nil {
		fmt.Println("Error marshaling JSON(split):", err)
		return
	}
	var combineResponseData interface{}
	err = json.Unmarshal(jsonBytes, &combineResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(split):", err)
		return
	}

	logInputAndOutput(splitData.Notes, combineResponseData)

	/////////////////////
	// ONCHAIN METHODS //
	/////////////////////

	// `/v2/onchain/deposit-address`
	logMethod("/v2/onchain/deposit-address")
	addr, err := fc.Onchain.CreateDepositAddress(1000, nil)
	if err != nil {
		fmt.Println("Error calling CREATE_DEPOSIT_ADDRESS: ", err)
		return
	}

	jsonBytes, err = json.Marshal(addr)
	if err != nil {
		fmt.Println("Error marshaling JSON(deposit-address):", err)
		return
	}
	var addrResponseData interface{}
	err = json.Unmarshal(jsonBytes, &addrResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(deposit-address):", err)
		return
	}

	logInputAndOutput(1000, addrResponseData)

	// `/v2/onchain/withdraw`
	logMethod("/v2/onchain/withdraw")
	withdrawData, err := fc.Onchain.Withdraw(addr.Address, 1000, nil)
	if err != nil {
		fmt.Println("Error calling WITHDRAW: ", err)
		return
	}

	jsonBytes, err = json.Marshal(withdrawData)
	if err != nil {
		fmt.Println("Error marshaling JSON(withdraw):", err)
		return
	}
	var withdrawResponseData interface{}
	err = json.Unmarshal(jsonBytes, &withdrawResponseData)
	if err != nil {
		fmt.Println("Error unmarshaling JSON(withdraw):", err)
		return
	}

	logInputAndOutput([]interface{}{addr.Address, 1000}, withdrawResponseData)

	fmt.Println("============================================")
	fmt.Println("|| Done: All methods tested successfully! ||")
	fmt.Println("============================================")
}
