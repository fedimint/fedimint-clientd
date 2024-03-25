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
	assert.Equal(t, fc.FederationId, "federation123")

	assert.Equal(t, fc, fc.Ln.Client)
	assert.Equal(t, fc, fc.Wallet.Client)
	assert.Equal(t, fc, fc.Mint.Client)
}

func TestGetActiveFederationId(t *testing.T) {
	fc := CreateNewFedimintClient()

	fedId := fc.getActiveFederationId()
	assert.Equal(t, fedId, "federation123")
}

func TestSetActiveFederationId(t *testing.T) {
	fc := CreateNewFedimintClient()
	new_fedId := "New_federation123"

	fedId_prev := fc.FederationId
	fc.setActiveFederationId(new_fedId)
	fedId_now := fc.FederationId
	assert.Equal(t, fedId_now, "New_federation123")
	assert.NotEqual(t, fedId_now, fedId_prev)
}

////////////
// Wallet //
////////////

func TestCreateDepositAddress(t *testing.T) {
	fc := CreateNewFedimintClient()

	depositAddressRequest := modules.DepositAddressRequest{
		Timeout: 3600,
	}

	depositResponse, err := fc.Wallet.createDepositAddress(depositAddressRequest, &fc.FederationId)
	if err != nil {
		assert.Equal(t, depositResponse, nil)
		assert.Equal(t, depositResponse.OperationID, nil)
		assert.Equal(t, depositResponse.Address, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, depositResponse.OperationID, nil)
		assert.NotEqual(t, depositResponse.Address, nil)
	}

	awaitDepositRequest := modules.AwaitDepositRequest{
		OperationID: depositResponse.OperationID,
	}

	_, err = fc.Wallet.awaitDeposit(awaitDepositRequest, &fc.FederationId)
	if err != nil {
		fmt.Println("Error awaiting deposit: ", err)
		return
	}
}

func TestWithdraw(t *testing.T) {
	fc := CreateNewFedimintClient()

	withdrawRequest := modules.WithdrawRequest{
		Address:    "UNKNOWN",
		AmountMsat: "10000",
	}

	withdrawResponse, _ := fc.Wallet.withdraw(withdrawRequest, &fc.FederationId)

	assert.NotEqual(t, withdrawResponse, nil)

	// Intentionally make an error (like - wrong FederationId/request)
	wrong_fed_id := "12112"
	_, err := fc.Wallet.withdraw(withdrawRequest, &wrong_fed_id)
	assert.NotEqual(t, err, nil)
}

func TestAwaitWithdraw(t *testing.T) {
	fc := CreateNewFedimintClient()

	depositAddressRequest := modules.DepositAddressRequest{
		Timeout: 3600,
	}
	depositResponse, _ := fc.Wallet.createDepositAddress(depositAddressRequest, &fc.FederationId)

	awaitDepositRequest := modules.AwaitDepositRequest{
		OperationID: depositResponse.OperationID,
	}

	awaitDepositResponse, err := fc.Wallet.awaitDeposit(awaitDepositRequest, &fc.FederationId)
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
	_, err1 := fc.Wallet.awaitDeposit(awaitDepositRequest, &wrong_fed_id)
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

	spendRequest := modules.SpendRequest{
		AmountMsat:   10000,
		AllowOverpay: true,
		Timeout:      3600,
	}

	spendResponse, err := fc.Mint.Spend(spendRequest, &fc.FederationId)
	if err != nil {
		assert.Equal(t, spendResponse, nil)
		assert.Equal(t, spendResponse.Operation, nil)
		assert.Equal(t, spendResponse.Notes, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, spendResponse, nil)
		assert.NotEqual(t, spendResponse.Operation, nil)
		assert.NotEqual(t, spendResponse.Notes, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Mint.Spend(spendRequest, &wrong_fed_id)
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

	invoiceRequest := modules.LnInvoiceRequest{
		AmountMsat:  10000,
		Description: "test",
	}

	invoiceResponse, err := fc.Ln.CreateInvoice(invoiceRequest, &fc.FederationId)
	if err != nil {
		assert.Equal(t, invoiceResponse, nil)
		assert.Equal(t, invoiceResponse.OperationID, nil)
		assert.Equal(t, invoiceResponse.Invoice, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, invoiceResponse, nil)
		assert.NotEqual(t, invoiceResponse.OperationID, nil)
		assert.NotEqual(t, invoiceResponse.Invoice, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Ln.CreateInvoice(invoiceRequest, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

func TestAwaitInvoice(t *testing.T) {
	fc := CreateNewFedimintClient()

	awaitInvoiceRequest := modules.AwaitInvoiceRequest{
		OperationID: "TestAwaitInvoice",
	}

	infoResponse, err := fc.Ln.AwaitInvoice(awaitInvoiceRequest, &fc.FederationId)
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
		assert.Equal(t, infoResponse.FederationID, fc.FederationId)
		assert.NotEqual(t, infoResponse, nil)
		assert.NotEqual(t, infoResponse.Meta, nil)
		assert.NotEqual(t, infoResponse.Network, nil)
		assert.NotEqual(t, infoResponse.TotalAmountMsat, nil)
		assert.NotEqual(t, infoResponse.TotalNumNotes, nil)
		assert.NotEqual(t, infoResponse.DenominationsMsat.Tiered, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := ""
	_, err1 := fc.Ln.AwaitInvoice(awaitInvoiceRequest, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

func TestPay(t *testing.T) {
	fc := CreateNewFedimintClient()

	lnPayRequest := modules.LnPayRequest{
		Payment_info:         "TestPayment",
		Finish_in_background: true,
	}

	lnPayResponse, err := fc.Ln.Pay(lnPayRequest, &fc.FederationId)
	if err != nil {
		assert.Equal(t, lnPayResponse, nil)
		assert.Equal(t, lnPayResponse.Contract_id, nil)
		assert.Equal(t, lnPayResponse.Fee, nil)
		assert.Equal(t, lnPayResponse.Payment_type, nil)
		assert.Equal(t, lnPayResponse.Pperation_id, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, lnPayResponse, nil)
		assert.NotEqual(t, lnPayResponse.Contract_id, nil)
		assert.NotEqual(t, lnPayResponse.Fee, nil)
		assert.NotEqual(t, lnPayResponse.Payment_type, nil)
		assert.NotEqual(t, lnPayResponse.Pperation_id, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Ln.Pay(lnPayRequest, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

func TestAwaitPay(t *testing.T) {
	fc := CreateNewFedimintClient()

	awaitLnPayRequest := modules.AwaitLnPayRequest{
		Operation_id: "TestAwaitLnPay",
	}

	lnPayResponse, err := fc.Ln.AwaitPay(awaitLnPayRequest, &fc.FederationId)
	if err != nil {
		assert.Equal(t, lnPayResponse, nil)
		assert.Equal(t, lnPayResponse.Contract_id, nil)
		assert.Equal(t, lnPayResponse.Fee, nil)
		assert.Equal(t, lnPayResponse.Payment_type, nil)
		assert.Equal(t, lnPayResponse.Pperation_id, nil)
	} else {
		assert.Equal(t, err, nil)
		assert.NotEqual(t, lnPayResponse, nil)
		assert.NotEqual(t, lnPayResponse.Contract_id, nil)
		assert.NotEqual(t, lnPayResponse.Fee, nil)
		assert.NotEqual(t, lnPayResponse.Payment_type, nil)
		assert.NotEqual(t, lnPayResponse.Pperation_id, nil)
	}

	// intentionally giving wrong parameters
	wrong_fed_id := "12112"
	_, err1 := fc.Ln.AwaitPay(awaitLnPayRequest, &wrong_fed_id)
	assert.NotEqual(t, err1, nil)
}

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
		Gateway_id: "TestGateway1",
	}

	gatewayResponse, err := fc.Ln.SwitchGateway(switchGatewayRequest, &fc.FederationId)
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
