package main

import (
	"encoding/hex"
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
	data, err = fc.DiscoverVersion(1) // TS-mirror dont use any parameters
	if err != nil {
		fmt.Println("Error calling VERSION: ", err)
		return
	}
	logInputAndOutput(1, data)

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
	infoData, err := fc.Info()
	if err != nil {
		fmt.Println("Error calling INFO: ", err)
		return
	}
	logInputAndOutput(nil, infoData)

	// `/v2/admin/join`
	inviteCode := os.Getenv("FEDIMINT_CLIENTD_BASE_URL")
	if inviteCode == "" {
		inviteCode = "fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75"
	}
	logMethod("/v2/admin/join")
	joinData, err := fc.Join(inviteCode, true, true, nil)
	if err != nil {
		fmt.Println("Error calling JOIN: ", err)
		return
	}
	logInputAndOutput(inviteCode, joinData)

	// `/v2/admin/list-operations`
	logMethod("/v2/admin/list-operations")
	listOperationsData, err := fc.ListOperations(10, nil)
	if err != nil {
		fmt.Println("Error calling JOIN: ", err)
		return
	}
	logInputAndOutput([]interface{}{10}, listOperationsData)

	///////////////////////
	// LIGHTNING METHODS //
	///////////////////////

	// `/v2/ln/list-gateways`
	logMethod("/v2/ln/list-gateways")
	gatewayList, err := fc.Ln.ListGateways()
	if err != nil {
		fmt.Println("Error calling LIST_GATEWAYS: ", err)
		return
	}
	logInputAndOutput(nil, gatewayList)

	// `/v2/ln/invoice`
	logMethod("/v2/ln/invoice")
	invoiceData, err := fc.Ln.CreateInvoice(10000, "test_INVOICE", nil, nil, nil)
	if err != nil {
		fmt.Println("Error calling INVOICE: ", err)
		return
	}
	logInputAndOutput([]interface{}{10000, "test_Invoice"}, invoiceData)

	// `/v2/ln/pay`
	logMethod("/v2/ln/pay")
	payData, err := fc.Ln.Pay(invoiceData.Invoice, nil, nil, nil, nil)
	if err != nil {
		fmt.Println("Error calling PAY: ", err)
		return
	}
	logInputAndOutput(invoiceData.Invoice, payData)

	// /v2/ln/await-invoice
	logMethod("/v2/ln/await-invoice")
	awaitInvoiceData, err := fc.Ln.AwaitInvoice(invoiceData.OperationId, nil)
	if err != nil {
		fmt.Println("Error calling AWAIT_INVOICE: ", err)
		return
	}
	logInputAndOutput(invoiceData.OperationId, awaitInvoiceData)

	// `/v1/ln/invoice-external-pubkey-tweaked`
	logMethod("/v1/ln/invoice-external-pubkey-tweaked")
	tweakInvoice, err := fc.Ln.CreateInvoiceForPubkeyTweak(keyPair.PublicKey, 1, 10000, "test", nil, nil, nil)
	if err != nil {
		fmt.Println("Error calling CREATE_INVOICE_FOR_PUBKEY_TWEAK: ", err)
		return
	}
	logInputAndOutput([]interface{}{keyPair.PublicKey, 1, 10000, "test"}, tweakInvoice)
	// pay the invoice
	_, _ = fc.Ln.Pay(tweakInvoice.Invoice, nil, nil, nil, nil)

	// `/v1/ln/claim-external-pubkey-tweaked`
	logMethod("/v1/ln/claim-external-pubkey-tweaked")
	activeFederationID := fc.GetActiveFederationId()
	claimInvoice, err := fc.Ln.ClaimPubkeyReceiveTweaked(keyPair.PrivateKey, []uint64{1}, &activeFederationID)
	if err != nil {
		fmt.Println("Error calling CLAIM_PUBKEY_RECEIVE_TWEAKED: ", err)
		return
	}
	logInputAndOutput([]interface{}{keyPair.PrivateKey, []uint64{1}}, claimInvoice)

	// // `/v1/ln/invoice-external-pubkey`
	// logMethod("/v1/ln/invoice-external-pubkey")
	// invoiceInfo, err := fc.Ln.CreateInvoiceForPubkey(keyPair.PublicKey, 10000, "test", nil, nil, nil)
	// if err != nil {
	// 	fmt.Println("Error calling CREATE_INVOICE_FOR_PUBKEY: ", err)
	// 	return
	// }
	// logInputAndOutput([]interface{}{keyPair.PublicKey, 10000, "test"}, invoiceInfo)

	// // `/v1/ln/claim-external-pubkey-tweaked`
	// logMethod("/v1/ln/claim-external-pubkey-tweaked")
	// claimInvoice, err = fc.Ln.ClaimPubkeyReceive(keyPair.PrivateKey, nil)
	// if err != nil {
	// 	fmt.Println("Error calling CLAIM_PUBKEY_RECEIVE_TWEAKED: ", err)
	// 	return
	// }
	// logInputAndOutput([]interface{}{keyPair.PrivateKey}, claimInvoice)

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
	logInputAndOutput([]interface{}{3000, true, 1000}, mintData)

	// `/v2/mint/decode-notes`
	logMethod("/v2/mint/decode-notes")
	decodedData, err := fc.Mint.DecodeNotes(mintData.Notes, nil)
	if err != nil {
		fmt.Println("Error calling DECODE_NOTES: ", err)
		return
	}
	logInputAndOutput(mintData.Notes, decodedData)

	// `/v2/mint/encode-notes`
	logMethod("/v2/mint/encode-notes")
	encodedData, err := fc.Mint.EncodeNotes(decodedData.NotesJson, nil)
	if err != nil {
		fmt.Println("Error calling DECODE_NOTES: ", err)
		return
	}
	logInputAndOutput(decodedData.NotesJson, encodedData)

	// `/v2/mint/validate`
	logMethod("/v2/mint/validate")
	validateData, err := fc.Mint.Validate(mintData.Notes, nil)
	if err != nil {
		fmt.Println("Error calling VALIDATE: ", err)
		return
	}
	logInputAndOutput(mintData.Notes, validateData)

	// `/v2/mint/reissue`
	logMethod("/v2/mint/reissue")
	reissueData, err := fc.Mint.Reissue(mintData.Notes, nil)
	if err != nil {
		fmt.Println("Error calling REISSUE: ", err)
		return
	}
	logInputAndOutput(mintData.Notes, reissueData)

	// `/v2/mint/split`
	logMethod("/v2/mint/split")
	splitData, err := fc.Mint.Split(mintData.Notes)
	if err != nil {
		fmt.Println("Error calling SPLIT: ", err)
		return
	}
	logInputAndOutput(mintData.Notes, splitData)

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
	logInputAndOutput(splitData.Notes, combineData)

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
	logInputAndOutput(1000, addr)

	// `/v2/onchain/withdraw`
	logMethod("/v2/onchain/withdraw")
	withdrawData, err := fc.Onchain.Withdraw(addr.Address, 1000, nil)
	if err != nil {
		fmt.Println("Error calling WITHDRAW: ", err)
		return
	}
	logInputAndOutput([]interface{}{addr.Address, 1000}, withdrawData)

	// `/v2/onchain/await-deposit`
	logMethod("/v2/onchain/await-deposit")
	// awaitDepositData, err := fc.Onchain.AwaitDeposit(addr.OperationId, nil)
	// if err != nil {
	// 	fmt.Println("Error calling AWAIT_DEPOSIT: ", err)
	// 	return
	// }
	// logInputAndOutput(addr.Address, awaitDepositData)

}
