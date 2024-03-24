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
	client := buildTestClient()
	keyPair := newKeyPair()
	fmt.Println("Generated key pair: ", keyPair)

	// ADMIN METHODS
	// `/v2/admin/config`
	logMethod("/v2/admin/config")
	data, err := client.Config()
	if err != nil {
		fmt.Println("Error calling config: ", err)
		return
	}
	logInputAndOutput(nil, data)

	// `/v2/admin/discover-version`
	logMethod("/v2/admin/discover-version")
	data, err = client.DiscoverVersion(1)
	if err != nil {
		fmt.Println("Error calling discoverVersion: ", err)
		return
	}
	logInputAndOutput(nil, data)

	// `/v2/admin/federation-ids`
	logMethod("/v2/admin/federation-ids")
	federationIds, err := client.FederationIds()
	if err != nil {
		fmt.Println("Error calling federationIds: ", err)
		return
	}
	logInputAndOutput(nil, federationIds)

	// `/v2/admin/info`
	logMethod("/v2/admin/info")
	infoData, err := client.Info()
	if err != nil {
		fmt.Println("Error calling info: ", err)
		return
	}
	logInputAndOutput(nil, infoData)

	// `/v2/admin/join`
	inviteCode := os.Getenv("FEDIMINT_CLIENTD_INVITE_CODE")
	if inviteCode == "" {
		inviteCode = "fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75"
	}
	logMethod("/v2/admin/join")
	joinData, err := client.Join(inviteCode, true, true, false)
	if err != nil {
		fmt.Println("Error calling join: ", err)
		return
	}
	logInputAndOutput(map[string]interface{}{"inviteCode": inviteCode}, joinData)

	// `/v2/admin/list-operations`
	logMethod("/v2/admin/list-operations")
	listOperationsData, err := client.ListOperations(10, nil)
	if err != nil {
		fmt.Println("Error calling listOperations: ", err)
		return
	}
	logInputAndOutput(map[string]interface{}{"limit": 10}, listOperationsData)

	// LIGHTNING METHODS
	// `/v2/ln/list-gateways`
	logMethod("/v2/ln/list-gateways")
	listGatewaysData, err := client.Ln.ListGateways()
	if err != nil {
		fmt.Println("Error calling listGateways: ", err)
		return
	}
	logInputAndOutput(nil, listGatewaysData)

	// `/v2/ln/invoice`
	logMethod("/v2/ln/invoice")
	invoiceData, err := client.Ln.CreateInvoice(10000, "test")
	if err != nil {
		fmt.Println("Error calling createInvoice: ", err)
		return
	}
	logInputAndOutput(map[string]interface{}{"amountMsat": 10000, "description": "test"}, invoiceData)

	// `/v2/ln/pay`
	logMethod("/v2/ln/pay")
	payData, err := client.Ln.Pay(invoiceData.Invoice, nil)
	if err != nil {
		fmt.Println("Error calling pay: ", err)
		return
	}
	logInputAndOutput(map[string]interface{}{"paymentInfo": invoiceData.Invoice}, payData)

}
