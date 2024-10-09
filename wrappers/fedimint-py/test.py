import os
from coincurve import PrivateKey
from FedimintClient import FedimintClient


def log_method(method: str):
    print("--------------------")
    print(f"Method: {method}")


def log_input_and_output(input_data, output):
    print("Input: ", input_data)
    print("Output: ", output)
    print("--------------------")


def new_key_pair():
    private_key = PrivateKey()
    public_key = private_key.public_key.format(compressed=False).hex()
    return {"privateKey": private_key.to_hex(), "publicKey": public_key}


def build_test_client():
    base_url = os.getenv("FEDIMINT_CLIENTD_BASE_URL", "http://127.0.0.1:3333")
    password = os.getenv("FEDIMINT_CLIENTD_PASSWORD", "password")
    active_federation_id = os.getenv(
        "FEDIMINT_CLIENTD_ACTIVE_FEDERATION_ID",
        "15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3",
    )

    client = FedimintClient(base_url, password, active_federation_id)
    client.use_default_gateway()
    print("Default gateway id: ", client.get_active_gateway_id())
    return client


def main():
    fedimint_client = build_test_client()
    key_pair = new_key_pair()
    print("Generated key pair: ", key_pair)

    # ADMIN METHODS
    log_method("/v2/admin/config")
    data = fedimint_client.config()
    log_input_and_output({}, data)

    log_method("/v2/admin/discover-version")
    data = fedimint_client.discover_version()
    log_input_and_output({}, data)

    log_method("/v2/admin/federation-ids")
    data = fedimint_client.federation_ids()
    log_input_and_output({}, data)

    log_method("/v2/admin/info")
    data = fedimint_client.info()
    log_input_and_output({}, data)

    invite_code = os.getenv(
        "INVITE_CODE",
        "fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75",
    )
    log_method("/v2/admin/join")
    data = fedimint_client.join(invite_code, False)
    log_input_and_output({"inviteCode": invite_code}, data)

    log_method("/v2/admin/list-operations")
    data = fedimint_client.list_operations({"limit": 10})
    log_input_and_output({"limit": 10}, data)

    # LIGHTNING METHODS
    log_method("/v2/ln/list-gateways")
    data = fedimint_client.lightning.list_gateways()
    log_input_and_output({}, data)

    log_method("/v2/ln/invoice")
    data = fedimint_client.lightning.create_invoice(10000, "test")
    log_input_and_output({"amountMsat": 10000, "description": "test"}, data)

    log_method("/v2/ln/pay")
    pay_response = fedimint_client.lightning.pay(data["invoice"], None, None)
    log_input_and_output({"paymentInfo": data["invoice"]}, pay_response)

    log_method("/v2/ln/await-invoice")
    pay_data = fedimint_client.lightning.await_invoice(data["operationId"])
    log_input_and_output({"operationId": data["operationId"]}, pay_data)

    log_method("/v2/ln/create-invoice-for-pubkey-tweaked")
    data = fedimint_client.lightning.create_invoice_for_pubkey_tweak(
        pubkey=key_pair["publicKey"], tweak=1, amount_msat=1000, description="test"
    )
    log_input_and_output(
        {
            "pubkey": key_pair["publicKey"],
            "tweak": 1,
            "amountMsat": 1000,
            "description": "test",
        },
        data,
    )

    fedimint_client.lightning.pay(data["invoice"], None, None)
    print("Paid locked invoice!")

    log_method("/v2/ln/claim-external-receive-tweaked")
    data = fedimint_client.lightning.claim_pubkey_tweak_receives(
        private_key=key_pair["privateKey"],
        tweaks=[1],
        federation_id=fedimint_client.get_active_federation_id(),
    )
    log_input_and_output({"privateKey": key_pair["privateKey"], "tweaks": [1]}, data)

    # MINT METHODS
    log_method("/v2/mint/spend")
    mint_data = fedimint_client.mint.spend(3000, True, 1000, False)
    log_input_and_output({"allowOverpay": True, "timeout": 1000}, mint_data)

    log_method("/v2/mint/decode-notes")
    data = fedimint_client.mint.decode_notes(mint_data["notes"])
    log_input_and_output({"notes": mint_data["notes"]}, data)

    log_method("/v2/mint/encode-notes")
    encoded_data = fedimint_client.mint.encode_notes(data["notesJson"])
    log_input_and_output({"notesJson": data}, encoded_data)

    log_method("/v2/mint/validate")
    data = fedimint_client.mint.validate(mint_data["notes"])
    log_input_and_output({"notes": mint_data["notes"]}, data)

    log_method("/v2/mint/reissue")
    data = fedimint_client.mint.reissue(mint_data["notes"])
    log_input_and_output({"notes": mint_data["notes"]}, data)

    log_method("/v2/mint/split")
    data = fedimint_client.mint.split(mint_data["notes"])
    log_input_and_output({"notes": mint_data["notes"]}, data)

    log_method("/v2/mint/combine")
    notes_vec = [data["notes"]]
    notes_values_vec = [
        value for note_dict in notes_vec for value in note_dict.values()
    ]
    data = fedimint_client.mint.combine(notes_values_vec)
    log_input_and_output({"notesVec": notes_values_vec}, data)

    # ONCHAIN METHODS
    log_method("/v2/onchain/deposit-address")
    data = fedimint_client.onchain.create_deposit_address()
    log_input_and_output({}, data)
    log_method("/v2/onchain/withdraw")
    withdraw_data = fedimint_client.onchain.withdraw(data["address"], 1000)
    log_input_and_output({"address": data["address"], "amountSat": 1000}, withdraw_data)

    print("Done: All methods tested successfully!")


if __name__ == "__main__":
    main()
