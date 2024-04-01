// Client_tests
package fedimint

import (
	"encoding/hex"
	"fmt"
	"log"
	"os"
	"testing"

	"github.com/btcsuite/btcd/btcec/v2"
)

type KeyPair struct {
	PrivateKey string
	PublicKey  string
}

func logMethod(method string) {
	fmt.Println("--------------------")
	fmt.Println("Method:", method)
}

func logInputAndOutput(input interface{}, output interface{}) {
	fmt.Println("Input: ", input)
	fmt.Println("Output: ", output)
	fmt.Println("====================")
}

// BIG ISSUE //
// func newKeyPair() KeyPair {
// 	var privateKey *big.Int
// 	var err error

// 	for {
// 		privateKey, err = rand.Int(rand.Reader, secp256k1.S256().Params().N)
// 		if err != nil {
// 			log.Fatal(err)
// 		}
// 		if privateKey.Sign() > 0 && privateKey.Cmp(secp256k1.S256().Params().N) < 0 {
// 			break
// 		}
// 	}

// 	// publicKey := secp256k1.PublicKey{}
// 	// secp256k1.S256().SerializePublicKey(publicKey[:], privateKey.Bytes())

// 	publicKey := secp256k1.PublicKey{}
// 	publicKey1 := secp256k1.PublicKey{
// 		x: new(big.Int),
// 		y: new(big.Int),
// 	}
// //(publicKey1.x, publicKey1.y) = secp256k1.S256().ScalarBaseMult(privateKey.Bytes())
// 	secp256k1.S256().ScalarBaseMult(privateKey.Bytes())

// 		return KeyPair{
// 			PrivateKey: hex.EncodeToString(privateKey.Bytes()),
// 			PublicKey:  hex.EncodeToString(publicKey[:]),
// 		}
// 	}

func newKeyPair() KeyPair {
	privKey, err := btcec.NewPrivateKey()
	if err != nil {
		log.Fatal("Error generating private key:", err)
	}

	// Get the corresponding public key
	pubKey := privKey.PubKey()

	// Print the private and public keys
	fmt.Printf("Private key: %x\n", privKey.Serialize())
	fmt.Printf("Public key: %x\n", pubKey.SerializeCompressed())

	return KeyPair{
		PrivateKey: hex.EncodeToString(privKey.Serialize()),
		PublicKey:  hex.EncodeToString(pubKey.SerializeCompressed()),
	}
}

func CreateNewFedimintClient() *FedimintClient {
	// Define test data
	baseURL := os.Getenv("FEDIMINT_CLIENTD_BASE_URL")
	if baseURL == "" {
		baseURL = "127.0.0.1:3333"
	}
	password := os.Getenv("FEDIMINT_CLIENTD_PASSWORD")
	if password == "" {
		password = "password"
	}
	activeFederationID := os.Getenv("FEDIMINT_CLIENTD_ACTIVE_FEDERATION_ID")
	if activeFederationID == "" {
		activeFederationID = "15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3"
	}

	done := make(chan *FedimintClient)
	go func() {
		fc := NewFedimintClient(baseURL, password, activeFederationID)
		fc.UseDefaultGateway()
		done <- fc
	}()
	return <-done
}

func TestFedimintGo(t *testing.T) {
	fc := CreateNewFedimintClient()
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
	data, err = fc.DiscoverVersion(1)
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
	joinData, err := fc.Join(inviteCode, true, true, false)
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

	// `/v1/ln/claim-external-pubkey-tweaked`
	logMethod("/v1/ln/claim-external-pubkey-tweaked")
	claimInvoice, err := fc.Ln.ClaimPubkeyReceiveTweaked(keyPair.PrivateKey, []uint64{1}, nil)
	if err != nil {
		fmt.Println("Error calling CLAIM_PUBKEY_RECEIVE_TWEAKED: ", err)
		return
	}
	logInputAndOutput([]interface{}{keyPair.PrivateKey, []uint64{1}}, claimInvoice)

	// `/v1/ln/invoice-external-pubkey`
	logMethod("/v1/ln/invoice-external-pubkey")
	invoiceInfo, err := fc.Ln.CreateInvoiceForPubkey(keyPair.PublicKey, 10000, "test", nil, nil, nil)
	if err != nil {
		fmt.Println("Error calling CREATE_INVOICE_FOR_PUBKEY: ", err)
		return
	}
	logInputAndOutput([]interface{}{keyPair.PublicKey, 10000, "test"}, invoiceInfo)

	// `/v1/ln/claim-external-pubkey-tweaked`
	logMethod("/v1/ln/claim-external-pubkey-tweaked")
	claimInvoice, err = fc.Ln.ClaimPubkeyReceive(keyPair.PrivateKey, nil)
	if err != nil {
		fmt.Println("Error calling CLAIM_PUBKEY_RECEIVE_TWEAKED: ", err)
		return
	}
	logInputAndOutput([]interface{}{keyPair.PrivateKey}, claimInvoice)
}

// func TestNewFedimintClient(t *testing.T) {
// 	fc := CreateNewFedimintClient()
// 	assert.NotNil(t, fc)

// 	assert.Equal(t, fc.BaseURL, "http://localhost:3333/fedimint/v2")
// 	assert.Equal(t, fc.Password, "password")
// 	assert.Equal(t, fc.ActiveFederationId, "federation123")

// 	assert.Equal(t, fc, fc.Ln.Client)
// 	assert.Equal(t, fc, fc.Onchain.Client)
// 	assert.Equal(t, fc, fc.Mint.Client)
// }

// func TestGetActiveFederationId(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	fedId := fc.GetActiveFederationId()
// 	assert.Equal(t, fedId, "federation123")
// }

// func TestSetActiveFederationId(t *testing.T) {
// 	fc := CreateNewFedimintClient()
// 	new_fedId := "New_federation123"

// 	fedId_prev := fc.ActiveFederationId
// 	fc.SetActiveFederationId(new_fedId, false)
// 	fedId_now := fc.ActiveFederationId
// 	assert.Equal(t, fedId_now, "New_federation123")
// 	assert.NotEqual(t, fedId_now, fedId_prev)
// }

// ////////////
// // Onchain //
// ////////////

// func TestCreateDepositAddress(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	depositAddressRequest := modules.OnchainDepositAddressRequest{
// 		Timeout: 3600,
// 	}

// 	depositResponse, err := fc.Onchain.createDepositAddress(depositAddressRequest.Timeout, &fc.ActiveFederationId)
// 	if err != nil {
// 		assert.Equal(t, depositResponse, nil)
// 		assert.Equal(t, depositResponse.OperationId, nil)
// 		assert.Equal(t, depositResponse.Address, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		assert.NotEqual(t, depositResponse.OperationId, nil)
// 		assert.NotEqual(t, depositResponse.Address, nil)
// 	}

// 	awaitDepositRequest := modules.OnchainAwaitDepositRequest{
// 		OperationId: depositResponse.OperationId,
// 	}

// 	_, err = fc.Onchain.awaitDeposit(awaitDepositRequest.OperationId, &fc.ActiveFederationId)
// 	if err != nil {
// 		fmt.Println("Error awaiting deposit: ", err)
// 		return
// 	}
// }

// func TestWithdraw(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	withdrawRequest := modules.OnchainWithdrawRequest{
// 		Address:   "UNKNOWN",
// 		AmountSat: 10000,
// 	}

// 	withdrawResponse, _ := fc.Onchain.withdraw(withdrawRequest.Address, withdrawRequest.AmountSat, &fc.ActiveFederationId)

// 	assert.NotEqual(t, withdrawResponse, nil)

// 	// Intentionally make an error (like - wrong ActiveFederationId/request)
// 	wrong_fed_id := "12112"
// 	_, err := fc.Onchain.withdraw(withdrawRequest.Address, withdrawRequest.AmountSat, &wrong_fed_id)
// 	assert.NotEqual(t, err, nil)
// }

// func TestAwaitWithdraw(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	depositAddressRequest := modules.OnchainDepositAddressRequest{
// 		Timeout: 3600,
// 	}
// 	depositResponse, _ := fc.Onchain.createDepositAddress(depositAddressRequest.Timeout, &fc.ActiveFederationId)

// 	awaitDepositRequest := modules.OnchainAwaitDepositRequest{
// 		OperationId: depositResponse.OperationId,
// 	}

// 	awaitDepositResponse, err := fc.Onchain.awaitDeposit(awaitDepositRequest.OperationId, &fc.ActiveFederationId)
// 	if err != nil {
// 		assert.Equal(t, awaitDepositResponse, nil)
// 		assert.Equal(t, awaitDepositResponse.Status, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		// println(awaitDepositResponse.Status)
// 		assert.NotEqual(t, awaitDepositResponse.Status, nil)
// 	}

// 	// intentionally giving wrong parameters
// 	wrong_fed_id := "12112"
// 	_, err1 := fc.Onchain.awaitDeposit(awaitDepositRequest.OperationId, &wrong_fed_id)
// 	assert.NotEqual(t, err1, nil)
// }

// //////////
// // mint //
// //////////

// // func TestReissue(t *testing.T) {
// // 	fc := CreateNewFedimintClient()

// // 	oobNotesData := modules.OOBNotes {

// // 	}
// // }

// func TestSpend(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	spendRequest := modules.MintSpendRequest{
// 		AmountMsat:   10000,
// 		AllowOverpay: true,
// 		Timeout:      3600,
// 	}

// 	spendResponse, err := fc.Mint.Spend(spendRequest.AmountMsat, spendRequest.AllowOverpay, spendRequest.Timeout, true, &fc.ActiveFederationId)
// 	if err != nil {
// 		assert.Equal(t, spendResponse, nil)
// 		assert.Equal(t, spendResponse.OperationId, nil)
// 		assert.Equal(t, spendResponse.Notes, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		assert.NotEqual(t, spendResponse, nil)
// 		assert.NotEqual(t, spendResponse.OperationId, nil)
// 		assert.NotEqual(t, spendResponse.Notes, nil)
// 	}

// 	// intentionally giving wrong parameters
// 	wrong_fed_id := "12112"
// 	_, err1 := fc.Mint.Spend(spendRequest.AmountMsat, spendRequest.AllowOverpay, spendRequest.Timeout, true, &wrong_fed_id)
// 	assert.NotEqual(t, err1, nil)
// }

// // func TestValidate(t *testing.T) {
// // 	fc := CreateNewFedimintClient()

// // }

// ////////
// // Ln //
// ////////

// func TestCreateInvoice(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	expiryTime := 3600
// 	GatewayID := "test_GatewayID"
// 	invoiceRequest := modules.LnInvoiceRequest{
// 		AmountMsat:  10000,
// 		Description: "test",
// 		ExpiryTime:  &expiryTime,
// 	}

// 	invoiceResponse, err := fc.Ln.CreateInvoice(invoiceRequest.AmountMsat, invoiceRequest.Description, invoiceRequest.ExpiryTime, &GatewayID, &fc.ActiveFederationId)
// 	if err != nil {
// 		assert.Equal(t, invoiceResponse, nil)
// 		assert.Equal(t, invoiceResponse.OperationId, nil)
// 		assert.Equal(t, invoiceResponse.Invoice, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		assert.NotEqual(t, invoiceResponse, nil)
// 		assert.NotEqual(t, invoiceResponse.OperationId, nil)
// 		assert.NotEqual(t, invoiceResponse.Invoice, nil)
// 	}

// 	// intentionally giving wrong parameters
// 	wrong_fed_id := "12112"
// 	_, err1 := fc.Ln.CreateInvoice(invoiceRequest.AmountMsat, invoiceRequest.Description, invoiceRequest.ExpiryTime, &GatewayID, &wrong_fed_id)
// 	assert.NotEqual(t, err1, nil)
// }

// func TestAwaitInvoice(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	awaitInvoiceRequest := modules.LnAwaitInvoiceRequest{
// 		OperationId: "TestAwaitInvoice",
// 	}

// 	infoResponse, err := fc.Ln.AwaitInvoice(awaitInvoiceRequest.OperationId, &fc.ActiveFederationId)
// 	if err != nil {
// 		assert.Equal(t, infoResponse, nil)
// 		assert.Equal(t, infoResponse.DenominationsMsat, nil)
// 		assert.Equal(t, infoResponse.FederationID, nil)
// 		assert.Equal(t, infoResponse.Meta, nil)
// 		assert.Equal(t, infoResponse.Network, nil)
// 		assert.Equal(t, infoResponse.TotalAmountMsat, nil)
// 		assert.Equal(t, infoResponse.TotalNumNotes, nil)
// 		assert.Equal(t, infoResponse.DenominationsMsat.Tiered, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		assert.Equal(t, infoResponse.FederationID, fc.ActiveFederationId)
// 		assert.NotEqual(t, infoResponse, nil)
// 		assert.NotEqual(t, infoResponse.Meta, nil)
// 		assert.NotEqual(t, infoResponse.Network, nil)
// 		assert.NotEqual(t, infoResponse.TotalAmountMsat, nil)
// 		assert.NotEqual(t, infoResponse.TotalNumNotes, nil)
// 		assert.NotEqual(t, infoResponse.DenominationsMsat.Tiered, nil)
// 	}

// 	// intentionally giving wrong parameters
// 	wrong_fed_id := ""
// 	_, err1 := fc.Ln.AwaitInvoice(awaitInvoiceRequest.OperationId, &wrong_fed_id)
// 	assert.NotEqual(t, err1, nil)
// }

// func TestPay(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	LnurlComment := "test_LnurlComment"
// 	GatewayID := "test_GatewayID"
// 	AmountMsat := uint64(10000)
// 	lnPayRequest := modules.LnPayRequest{
// 		PaymentInfo:  "TestPayment",
// 		AmountMsat:   &AmountMsat,
// 		LnurlComment: &LnurlComment,
// 	}

// 	lnPayResponse, err := fc.Ln.Pay(lnPayRequest.PaymentInfo, lnPayRequest.AmountMsat, lnPayRequest.LnurlComment, &GatewayID, &fc.ActiveFederationId)
// 	if err != nil {
// 		assert.Equal(t, lnPayResponse, nil)
// 		assert.Equal(t, lnPayResponse.ContractId, nil)
// 		assert.Equal(t, lnPayResponse.Fee, nil)
// 		assert.Equal(t, lnPayResponse.PaymentType, nil)
// 		assert.Equal(t, lnPayResponse.PperationId, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		assert.NotEqual(t, lnPayResponse, nil)
// 		assert.NotEqual(t, lnPayResponse.ContractId, nil)
// 		assert.NotEqual(t, lnPayResponse.Fee, nil)
// 		assert.NotEqual(t, lnPayResponse.PaymentType, nil)
// 		assert.NotEqual(t, lnPayResponse.PperationId, nil)
// 	}

// 	// intentionally giving wrong parameters
// 	wrong_fed_id := "12112"
// 	_, err1 := fc.Ln.Pay(lnPayRequest.PaymentInfo, lnPayRequest.AmountMsat, lnPayRequest.LnurlComment, &GatewayID, &wrong_fed_id)
// 	assert.NotEqual(t, err1, nil)
// }

// // func TestAwaitPay(t *testing.T) {
// // 	fc := CreateNewFedimintClient()

// // 	awaitLnPayRequest := modules.AwaitLnPayRequest{
// // 		OperationId: "TestAwaitLnPay",
// // 	}

// // 	lnPayResponse, err := fc.Ln.AwaitPay(awaitLnPayRequest, &fc.ActiveFederationId)
// // 	if err != nil {
// // 		assert.Equal(t, lnPayResponse, nil)
// // 		assert.Equal(t, lnPayResponse.Contract_id, nil)
// // 		assert.Equal(t, lnPayResponse.Fee, nil)
// // 		assert.Equal(t, lnPayResponse.Payment_type, nil)
// // 		assert.Equal(t, lnPayResponse.Pperation_id, nil)
// // 	} else {
// // 		assert.Equal(t, err, nil)
// // 		assert.NotEqual(t, lnPayResponse, nil)
// // 		assert.NotEqual(t, lnPayResponse.Contract_id, nil)
// // 		assert.NotEqual(t, lnPayResponse.Fee, nil)
// // 		assert.NotEqual(t, lnPayResponse.Payment_type, nil)
// // 		assert.NotEqual(t, lnPayResponse.Pperation_id, nil)
// // 	}

// // 	// intentionally giving wrong parameters
// // 	wrong_fed_id := "12112"
// // 	_, err1 := fc.Ln.AwaitPay(awaitLnPayRequest, &wrong_fed_id)
// // 	assert.NotEqual(t, err1, nil)
// // }

// func TestListGateways(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	gatewaysResponse, err := fc.Ln.ListGateways()
// 	if err != nil {
// 		assert.Equal(t, gatewaysResponse, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		assert.NotEqual(t, gatewaysResponse, nil)
// 	}
// }

// func TestSwitchGateway(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	switchGatewayRequest := modules.SwitchGatewayRequest{
// 		GatewayId: "TestGateway1",
// 	}

// 	gatewayResponse, err := fc.Ln.SwitchGateway(switchGatewayRequest, &fc.ActiveFederationId)
// 	if err != nil {
// 		assert.Equal(t, gatewayResponse, nil)
// 		assert.Equal(t, gatewayResponse.Active, true)
// 		assert.NotEqual(t, gatewayResponse.Node_pub_key, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		assert.Equal(t, gatewayResponse.Active, true)
// 		assert.NotEqual(t, gatewayResponse.Node_pub_key, nil)
// 	}

// 	// intentionally giving wrong parameters
// 	wrong_fed_id := "12112"
// 	_, err1 := fc.Ln.SwitchGateway(switchGatewayRequest, &wrong_fed_id)
// 	assert.NotEqual(t, err1, nil)
// }
