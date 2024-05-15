#!/bin/bash

# User inputs for client URLs and passwords
FROM_CLIENT_URL=$1
FROM_CLIENT_PASSWORD=$2
TO_CLIENT_URL=$3
TO_CLIENT_PASSWORD=$4

# Set the authorization token and content type headers for both clients
FROM_AUTH_HEADER="Authorization: Bearer $FROM_CLIENT_PASSWORD"
TO_AUTH_HEADER="Authorization: Bearer $TO_CLIENT_PASSWORD"
CONTENT_TYPE_HEADER="Content-type: application/json"

# Optional amountMsat from command line argument
AMOUNT_MSAT=$5

# Fetch federation info from both clients
echo "Fetching federation info from source client..."
FEDERATION_INFO_1=$(curl -s -H "$FROM_AUTH_HEADER" "$FROM_CLIENT_URL/admin/info" | jq '.')

echo "Fetching federation info from destination client..."
FEDERATION_INFO_2=$(curl -s -H "$TO_AUTH_HEADER" "$TO_CLIENT_URL/admin/info" | jq '.')

# Extract federation IDs from both clients
FEDERATION_IDS_1=$(echo "$FEDERATION_INFO_1" | jq -r 'keys[]')
FEDERATION_IDS_2=$(echo "$FEDERATION_INFO_2" | jq -r 'keys[]')

# Convert federation IDs into a jq-readable format
FED_IDS_1=$(echo "$FEDERATION_IDS_1" | jq -sR 'split("\n") | map(select(. != ""))')
FED_IDS_2=$(echo "$FEDERATION_IDS_2" | jq -sR 'split("\n") | map(select(. != ""))')

# Compare federation IDs
MISSING_FEDERATION_IDS=$(jq -n --argjson ids1 "$FED_IDS_1" --argjson ids2 "$FED_IDS_2" \
  '$ids1 | map(select(. as $id | $ids2 | index($id) | not))')

if [ "$(echo "$MISSING_FEDERATION_IDS" | jq 'length')" -gt 0 ]; then
  echo "Error: Destination client is missing federation IDs: $MISSING_FEDERATION_IDS"
  exit 1
fi

echo "All federation IDs from source client are present in destination client."

# Print each federation and its balance
echo "Federation Balances:"
echo "$FEDERATION_INFO_1" | jq -r 'to_entries[] | "Federation ID: \(.key) - Federation Name: \(.value.meta.federation_name) - Balance: \(.value.totalAmountMsat) msat"'

# Iterate over each federation
echo "$FEDERATION_INFO_1" | jq -c 'to_entries[]' | while read -r entry; do
  FEDERATION_ID=$(echo "$entry" | jq -r '.key')
  FEDERATION_NAME=$(echo "$entry" | jq -r '.value.meta.federation_name')
  TOTAL_AMOUNT_MSAT=$(echo "$entry" | jq -r '.value.totalAmountMsat')

  echo "Processing Federation ID: $FEDERATION_ID"
  echo "Federation Name: $FEDERATION_NAME"
  echo "Total Amount Msat: $TOTAL_AMOUNT_MSAT"

  if [ "$TOTAL_AMOUNT_MSAT" -eq 0 ]; then
    echo "No funds available in federation $FEDERATION_ID. Skipping spend and reissue steps."
    continue
  fi

  # Use specified amount or full amount if not specified, but do not exceed totalAmountMsat
  if [[ -n "$AMOUNT_MSAT" && "$AMOUNT_MSAT" -le "$TOTAL_AMOUNT_MSAT" ]]; then
    SPEND_AMOUNT_MSAT="$AMOUNT_MSAT"
  else
    SPEND_AMOUNT_MSAT="$TOTAL_AMOUNT_MSAT"
  fi
  echo "Attempting to Spend $SPEND_AMOUNT_MSAT msat in federation $FEDERATION_ID"

  # Spend request with jq
  SPEND_PAYLOAD=$(jq -n --argjson amountMsat "$SPEND_AMOUNT_MSAT" --arg federationId "$FEDERATION_ID" \
    '{amountMsat: $amountMsat | tonumber, allowOverpay: true, timeout: 1000000, include_invite: false, federationId: $federationId}')
  SPEND_RESPONSE=$(curl -s -X POST -H "$FROM_AUTH_HEADER" -H "$CONTENT_TYPE_HEADER" -d "$SPEND_PAYLOAD" "$FROM_CLIENT_URL/mint/spend")
  if [ $? -ne 0 ]; then
    echo "Error during spend operation for federation $FEDERATION_ID"
    continue
  fi
  NOTES=$(echo "$SPEND_RESPONSE" | jq -r '.notes')

  echo "Spend operation completed. Response: $SPEND_RESPONSE"

  # Reissue request
  REISSUE_PAYLOAD=$(jq -n --arg notes "$NOTES" \
    '{notes: $notes}')
  REISSUE_RESPONSE=$(curl -s -X POST -H "$TO_AUTH_HEADER" -H "$CONTENT_TYPE_HEADER" -d "$REISSUE_PAYLOAD" "$TO_CLIENT_URL/mint/reissue")
  if [ $? -ne 0 ]; then
    echo "Error during reissue operation for federation $FEDERATION_ID"
    continue
  fi

  echo "Reissue operation completed. Response: $REISSUE_RESPONSE"
  echo "----------------------------------------"
done

echo "All operations completed."
