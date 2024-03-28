// Client_tests
package fedimint

import (
	"fedimint-go-client/pkg/fedimint/types/modules"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func CreateNewFedimintClient() *FedimintClient {
	// Define test data
	baseURL := "http://localhost:3333"
	password := "password"
	federationID := "federation123"

	fc := NewFedimintClient(baseURL, password, federationID)

	return fc
}

func TestNewFedimintClient(t *testing.T) {
	fc := CreateNewFedimintClient()
	assert.NotNil(t, fc)

	assert.Equal(t, fc.BaseURL, "http://localhost:3333/fedimint/v2")
	assert.Equal(t, fc.Password, "password")
	assert.Equal(t, fc.ActiveFederationId, "federation123")

	assert.Equal(t, fc, fc.Ln.Client)
	assert.Equal(t, fc, fc.Onchain.Client)
	assert.Equal(t, fc, fc.Mint.Client)
}

func TestGetActiveFederationId(t *testing.T) {
	fc := CreateNewFedimintClient()

	fedId := fc.GetActiveFederationId()
	assert.Equal(t, fedId, "federation123")
}

func TestSetActiveFederationId(t *testing.T) {
	fc := CreateNewFedimintClient()
	new_fedId := "New_federation123"

	fedId_prev := fc.ActiveFederationId
	fc.SetActiveFederationId(new_fedId, false)
	fedId_now := fc.ActiveFederationId
	assert.Equal(t, fedId_now, "New_federation123")
	assert.NotEqual(t, fedId_now, fedId_prev)
}

////////////
// Onchain //
////////////

func TestCreateDepositAddress(t *testing.T) {
	fc := CreateNewFedimintClient()

	depositAddressRequest := modules.OnchainDepositAddressRequest{
		Timeout: 3600,
	}

	depositResponse, err := fc.Onchain.createDepositAddress(depositAddressRequest.Timeout, &fc.ActiveFederationId)
	if err != nil {
		assert.Equal(t, depositResponse, nil)
		assert.Equal(t, depositResponse.OperationId, nil)
		assert.Equal(t, depositResponse.Address, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, depositResponse.OperationId, nil)
		assert.NotEqual(t, depositResponse.Address, nil)
	}

	awaitDepositRequest := modules.OnchainAwaitDepositRequest{
		OperationId: depositResponse.OperationId,
	}

	_, err = fc.Onchain.awaitDeposit(awaitDepositRequest.OperationId, &fc.ActiveFederationId)
	if err != nil {
		fmt.Println("Error awaiting deposit: ", err)
		return
	}
}

func TestWithdraw(t *testing.T) {
	fc := CreateNewFedimintClient()

	withdrawRequest := modules.OnchainWithdrawRequest{
		Address:   "UNKNOWN",
		AmountSat: 10000,
	}

	withdrawResponse, _ := fc.Onchain.withdraw(withdrawRequest.Address, withdrawRequest.AmountSat, &fc.ActiveFederationId)

	assert.NotEqual(t, withdrawResponse, nil)

	// Intentionally make an error (like - wrong ActiveFederationId/request)
	wrong_fed_id := "12112"
	_, err := fc.Onchain.withdraw(withdrawRequest.Address, withdrawRequest.AmountSat, &wrong_fed_id)
	assert.NotEqual(t, err, nil)
}

func TestAwaitWithdraw(t *testing.T) {
	fc := CreateNewFedimintClient()

	depositAddressRequest := modules.OnchainDepositAddressRequest{
		Timeout: 3600,
	}
	depositResponse, _ := fc.Onchain.createDepositAddress(depositAddressRequest.Timeout, &fc.ActiveFederationId)

	awaitDepositRequest := modules.OnchainAwaitDepositRequest{
		OperationId: depositResponse.OperationId,
	}

	awaitDepositResponse, err := fc.Onchain.awaitDeposit(awaitDepositRequest.OperationId, &fc.ActiveFederationId)
	if err != nil {
		assert.Equal(t, awaitDepositResponse, nil)
		assert.Equal(t, awaitDepositResponse.Status, nil)
	} else {
		assert.Equal(t, err, nil)
		// println(awaitDepositResponse.Status)
		assert.NotEqual(t, awaitDepositResponse.Status, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Onchain.awaitDeposit(awaitDepositRequest.OperationId, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

//////////
// mint //
//////////

// func TestReissue(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	oobNotesData := modules.OOBNotes {

// 	}
// }

func TestSpend(t *testing.T) {
	fc := CreateNewFedimintClient()

	spendRequest := modules.MintSpendRequest{
		AmountMsat:   10000,
		AllowOverpay: true,
		Timeout:      3600,
	}

	spendResponse, err := fc.Mint.Spend(spendRequest.AmountMsat, spendRequest.AllowOverpay, spendRequest.Timeout, true, &fc.ActiveFederationId)
	if err != nil {
		assert.Equal(t, spendResponse, nil)
		assert.Equal(t, spendResponse.OperationId, nil)
		assert.Equal(t, spendResponse.Notes, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, spendResponse, nil)
		assert.NotEqual(t, spendResponse.OperationId, nil)
		assert.NotEqual(t, spendResponse.Notes, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Mint.Spend(spendRequest.AmountMsat, spendRequest.AllowOverpay, spendRequest.Timeout, true, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

// func TestValidate(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// }

////////
// Ln //
////////

func TestCreateInvoice(t *testing.T) {
	fc := CreateNewFedimintClient()

	expiryTime := 3600
	GatewayID := "test_GatewayID"
	invoiceRequest := modules.LnInvoiceRequest{
		AmountMsat:  10000,
		Description: "test",
		ExpiryTime:  &expiryTime,
	}

	invoiceResponse, err := fc.Ln.CreateInvoice(invoiceRequest.AmountMsat, invoiceRequest.Description, invoiceRequest.ExpiryTime, &GatewayID, &fc.ActiveFederationId)
	if err != nil {
		assert.Equal(t, invoiceResponse, nil)
		assert.Equal(t, invoiceResponse.OperationId, nil)
		assert.Equal(t, invoiceResponse.Invoice, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, invoiceResponse, nil)
		assert.NotEqual(t, invoiceResponse.OperationId, nil)
		assert.NotEqual(t, invoiceResponse.Invoice, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Ln.CreateInvoice(invoiceRequest.AmountMsat, invoiceRequest.Description, invoiceRequest.ExpiryTime, &GatewayID, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

func TestAwaitInvoice(t *testing.T) {
	fc := CreateNewFedimintClient()

	awaitInvoiceRequest := modules.LnAwaitInvoiceRequest{
		OperationId: "TestAwaitInvoice",
	}

	infoResponse, err := fc.Ln.AwaitInvoice(awaitInvoiceRequest.OperationId, &fc.ActiveFederationId)
	if err != nil {
		assert.Equal(t, infoResponse, nil)
		assert.Equal(t, infoResponse.DenominationsMsat, nil)
		assert.Equal(t, infoResponse.FederationID, nil)
		assert.Equal(t, infoResponse.Meta, nil)
		assert.Equal(t, infoResponse.Network, nil)
		assert.Equal(t, infoResponse.TotalAmountMsat, nil)
		assert.Equal(t, infoResponse.TotalNumNotes, nil)
		assert.Equal(t, infoResponse.DenominationsMsat.Tiered, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.Equal(t, infoResponse.FederationID, fc.ActiveFederationId)
		assert.NotEqual(t, infoResponse, nil)
		assert.NotEqual(t, infoResponse.Meta, nil)
		assert.NotEqual(t, infoResponse.Network, nil)
		assert.NotEqual(t, infoResponse.TotalAmountMsat, nil)
		assert.NotEqual(t, infoResponse.TotalNumNotes, nil)
		assert.NotEqual(t, infoResponse.DenominationsMsat.Tiered, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := ""
	_, err1 := fc.Ln.AwaitInvoice(awaitInvoiceRequest.OperationId, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

func TestPay(t *testing.T) {
	fc := CreateNewFedimintClient()

	LnurlComment := "test_LnurlComment"
	GatewayID := "test_GatewayID"
	AmountMsat := uint64(10000)
	lnPayRequest := modules.LnPayRequest{
		PaymentInfo:  "TestPayment",
		AmountMsat:   &AmountMsat,
		LnurlComment: &LnurlComment,
	}

	lnPayResponse, err := fc.Ln.Pay(lnPayRequest.PaymentInfo, lnPayRequest.AmountMsat, lnPayRequest.LnurlComment, &GatewayID, &fc.ActiveFederationId)
	if err != nil {
		assert.Equal(t, lnPayResponse, nil)
		assert.Equal(t, lnPayResponse.ContractId, nil)
		assert.Equal(t, lnPayResponse.Fee, nil)
		assert.Equal(t, lnPayResponse.PaymentType, nil)
		assert.Equal(t, lnPayResponse.PperationId, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, lnPayResponse, nil)
		assert.NotEqual(t, lnPayResponse.ContractId, nil)
		assert.NotEqual(t, lnPayResponse.Fee, nil)
		assert.NotEqual(t, lnPayResponse.PaymentType, nil)
		assert.NotEqual(t, lnPayResponse.PperationId, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Ln.Pay(lnPayRequest.PaymentInfo, lnPayRequest.AmountMsat, lnPayRequest.LnurlComment, &GatewayID, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

// func TestAwaitPay(t *testing.T) {
// 	fc := CreateNewFedimintClient()

// 	awaitLnPayRequest := modules.AwaitLnPayRequest{
// 		OperationId: "TestAwaitLnPay",
// 	}

// 	lnPayResponse, err := fc.Ln.AwaitPay(awaitLnPayRequest, &fc.ActiveFederationId)
// 	if err != nil {
// 		assert.Equal(t, lnPayResponse, nil)
// 		assert.Equal(t, lnPayResponse.Contract_id, nil)
// 		assert.Equal(t, lnPayResponse.Fee, nil)
// 		assert.Equal(t, lnPayResponse.Payment_type, nil)
// 		assert.Equal(t, lnPayResponse.Pperation_id, nil)
// 	} else {
// 		assert.Equal(t, err, nil)
// 		assert.NotEqual(t, lnPayResponse, nil)
// 		assert.NotEqual(t, lnPayResponse.Contract_id, nil)
// 		assert.NotEqual(t, lnPayResponse.Fee, nil)
// 		assert.NotEqual(t, lnPayResponse.Payment_type, nil)
// 		assert.NotEqual(t, lnPayResponse.Pperation_id, nil)
// 	}

// 	// intentionally giving wrong parameters
// 	wrong_fed_id := "12112"
// 	_, err1 := fc.Ln.AwaitPay(awaitLnPayRequest, &wrong_fed_id)
// 	assert.NotEqual(t, err1, nil)
// }

func TestListGateways(t *testing.T) {
	fc := CreateNewFedimintClient()

	gatewaysResponse, err := fc.Ln.ListGateways()
	if err != nil {
		assert.Equal(t, gatewaysResponse, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, gatewaysResponse, nil)
	}
}

func TestSwitchGateway(t *testing.T) {
	fc := CreateNewFedimintClient()

	switchGatewayRequest := modules.SwitchGatewayRequest{
		GatewayId: "TestGateway1",
	}

	gatewayResponse, err := fc.Ln.SwitchGateway(switchGatewayRequest, &fc.ActiveFederationId)
	if err != nil {
		assert.Equal(t, gatewayResponse, nil)
		assert.Equal(t, gatewayResponse.Active, true)
		assert.NotEqual(t, gatewayResponse.Node_pub_key, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.Equal(t, gatewayResponse.Active, true)
		assert.NotEqual(t, gatewayResponse.Node_pub_key, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Ln.SwitchGateway(switchGatewayRequest, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}
