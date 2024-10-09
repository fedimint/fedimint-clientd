#!/bin/bash

# Check if amount is provided
if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <amountMsat>"
  exit 1
fi

AMOUNT_MSAT=$1
FEDERATION_ID="15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3"
GATEWAY_ID="035f2f7912e0f570841d5c0d8976a40af0dcca5609198436f596e78d2c851ee58a"

# Create an invoice and capture the response
INVOICE_RESPONSE=$(curl -s -X POST http://localhost:3333/v2/ln/invoice \
  -H "Authorization: Bearer password" \
  -H "Content-Type: application/json" \
  -d "{\"amountMsat\": $AMOUNT_MSAT, \"description\": \"Invoice for amount $AMOUNT_MSAT msat\", \"gatewayId\": \"$GATEWAY_ID\", \"federationId\": \"$FEDERATION_ID\"}")

# Extract and print just the invoice from the response
INVOICE=$(echo "$INVOICE_RESPONSE" | jq -r '.invoice')
echo "$INVOICE"
