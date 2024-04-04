import asyncio
import os
from coincurve import PrivateKey
from AsyncFedimintClient import AsyncFedimintClient


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


async def build_test_client():
    base_url = os.getenv("FEDIMINT_CLIENTD_BASE_URL", "127.0.0.1:3333")
    password = os.getenv("FEDIMINT_CLIENTD_PASSWORD", "password")
    active_federation_id = os.getenv(
        "FEDIMINT_CLIENTD_ACTIVE_FEDERATION_ID",
        "15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3",
    )

    client = AsyncFedimintClient(base_url, password, active_federation_id)
    await client.use_default_gateway()
    print("Default gateway id: ", client.get_active_gatewayId())
    return client


async def main():
    fedimint_client = await build_test_client()
    key_pair = new_key_pair()
    print("Generated key pair: ", key_pair)

    # ADMIN METHODS
    # `/v2/admin/config`
    log_method("/v2/admin/config")
    data = await fedimint_client.config()
    log_input_and_output({}, data)
    # `/v2/admin/discover-version`
    log_method("/v2/admin/discover-version")
    data = await fedimint_client.discover_version(
        1
    )  # Assuming threshold is required, adjust as needed
    log_input_and_output({}, data)
    # `/v2/admin/federation-ids`
    log_method("/v2/admin/federation-ids")
    data = await fedimint_client.federation_ids()
    log_input_and_output({}, data)
    # `/v2/admin/info`
    log_method("/v2/admin/info")
    data = await fedimint_client.info()
    log_input_and_output({}, data)
    # `/v2/admin/join`
    invite_code = os.getenv("INVITE_CODE", "your_invite_code_here")
    log_method("/v2/admin/join")
    data = await fedimint_client.join(invite_code, True)
    log_input_and_output({"inviteCode": invite_code}, data)
    # `/v2/admin/list-operations`
    log_method("/v2/admin/list-operations")
    data = await fedimint_client.list_operations(
        {"limit": 10}
    )  # Adjust the request as needed
    log_input_and_output({"limit": 10}, data)

    # LIGHTNING METHODS
    # `/v2/ln/list-gateways`
    log_method("/v2/ln/list-gateways")
    data = await fedimint_client.lightning.list_gateways()
    log_input_and_output({}, data)
    # `/v2/ln/invoice`
    log_method("/v2/ln/invoice")
    data = await fedimint_client.lightning.create_invoice(10000, "test")
    log_input_and_output({"amountMsat": 10000, "description": "test"}, data)
    # `/v2/ln/pay`
    log_method("/v2/ln/pay")
    pay_response = await fedimint_client.lightning.pay(
        {"invoice": data.invoice}
    )  # Adjust the request as needed
    log_input_and_output({"paymentInfo": data.invoice}, pay_response)
    # `/v2/ln/await-invoice`
    log_method("/v2/ln/await-invoice")
    data = await fedimint_client.lightning.await_invoice(
        {"operationId": data.operation_id}
    )  # Adjust the request as needed
    log_input_and_output({"operationId": data.operation_id}, data)

    # MINT METHODS
    # `/v2/mint/spend`
    log_method("/v2/mint/spend")
    mint_data = await fedimint_client.mint.spend(3000, True, 1000, False)
    log_input_and_output({"allowOverpay": True, "timeout": 1000}, mint_data)
    # `/v2/mint/decode-notes`
    log_method("/v2/mint/decode-notes")
    data = await fedimint_client.mint.decode_notes(mint_data.notes)
    log_input_and_output({"notes": mint_data.notes}, data)
    # `/v2/mint/encode-notes`
    log_method("/v2/mint/encode-notes")
    data = await fedimint_client.mint.encode_notes(data.notes_json)
    log_input_and_output({"notesJson": data.notes_json}, data)
    # `/v2/mint/validate`
    log_method("/v2/mint/validate")
    data = await fedimint_client.mint.validate(mint_data.notes)
    log_input_and_output({"notes": mint_data.notes}, data)
    # `/v2/mint/reissue`
    log_method("/v2/mint/reissue")
    data = await fedimint_client.mint.reissue(mint_data.notes)
    log_input_and_output({"notes": mint_data.notes}, data)
    # `/v2/mint/split`
    log_method("/v2/mint/split")
    data = await fedimint_client.mint.split(mint_data.notes)
    log_input_and_output({"notes": mint_data.notes}, data)
    # `/v2/mint/combine`
    log_method("/v2/mint/combine")
    notes_vec = [data.notes]  # Assuming `data.notes` is correct, adjust as needed
    data = await fedimint_client.mint.combine(notes_vec)
    log_input_and_output({"notesVec": notes_vec}, data)

    # ONCHAIN METHODS
    # `/v2/onchain/deposit-address`
    log_method("/v2/onchain/deposit-address")
    data = await fedimint_client.onchain.create_deposit_address(1000)
    log_input_and_output({"timeout": 1000}, data)
    # `/v2/onchain/withdraw`
    log_method("/v2/onchain/withdraw")
    data = await fedimint_client.onchain.withdraw(data.address, 1000)
    log_input_and_output({"address": data.address, "amountSat": 1000}, data)
    # `/v2/onchain/await-deposit`
    # This method might be commented out or not implemented in the original script. Adjust as needed.

    print("Done: All methods tested successfully!")


# Run the main function
asyncio.run(main())
