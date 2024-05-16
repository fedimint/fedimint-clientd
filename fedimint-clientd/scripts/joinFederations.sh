#!/bin/bash

# Check if the correct number of arguments are passed
if [ "$#" -ne 3 ]; then
  echo "Usage: $0 <url_base> <password> <file_with_invite_codes>"
  exit 1
fi

URL_BASE=$1
PASSWORD=$2
INVITE_CODES_FILE=$3

# Read each invite code from the file and send a join request
while IFS= read -r inviteCode; do
  curl -v -X POST -H "Authorization: Bearer $PASSWORD" "${URL_BASE}/v2/admin/join" \
    -H "Content-type: application/json" \
    -d "{\"inviteCode\": \"$inviteCode\", \"useManualSecret\": false}"
done <"$INVITE_CODES_FILE"
