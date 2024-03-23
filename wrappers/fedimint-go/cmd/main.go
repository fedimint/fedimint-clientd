package main

import (
	"fedimint-go-client/pkg/fedimint"
	"fedimint-go-client/pkg/fedimint/types/modules"
	"fmt"
	"os"

	"github.com/joho/godotenv"
)

func main() {
	err := godotenv.Load()
	if err != nil {
		fmt.Println("Error loading .env file")
	}

	baseUrl := os.Getenv("BASE_URL")
	if baseUrl == "" {
		baseUrl = "http://localhost:5000"
	}

	password := os.Getenv("PASSWORD")
	if password == "" {
		password = "password"
	}

	federationId := os.Getenv("FEDERATION_ID")
	if federationId == "" {
		federationId = "defaultId"
	}

	fedimintClient := fedimint.NewFedimintClient(baseUrl, password, federationId)

	info, err := fedimintClient.Info()
	if err != nil {
		fmt.Println("Error getting info: ", err)
		return
	}
	fmt.Println("Current Total Msats Ecash: ", info.TotalAmountMsat)

	invoiceRequest := modules.LnInvoiceRequest{
		AmountMsat:  10000,
		Description: "test",
	}

	invoiceResponse, err := fedimintClient.Ln.CreateInvoice(invoiceRequest, &federationId)
	if err != nil {
		fmt.Println("Error creating invoice: ", err)
		return
	}

	fmt.Println("Created 10 sat Invoice: ", invoiceResponse.Invoice)

	fmt.Println("Waiting for payment...")

	awaitInvoiceRequest := modules.LnAwaitInvoiceRequest{
		OperationID: invoiceResponse.OperationID,
	}

	_, err = fedimintClient.Ln.AwaitInvoice(awaitInvoiceRequest, &federationId)
	if err != nil {
		fmt.Println("Error awaiting invoice: ", err)
		return
	}

	fmt.Println("Payment Received!")
	// fmt.Println("New Total Msats Ecash: ", awaitInvoiceResponse.TotalAmountMsat)
}
