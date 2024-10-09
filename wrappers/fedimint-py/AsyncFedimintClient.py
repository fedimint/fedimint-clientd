import json
import logging
from typing import List, Literal, Optional, Union
import asyncio
import aiohttp
import atexit

from models.common import (
    DiscoverVersionResponse,
    InfoResponse,
    ListOperationsRequest,
)

from models.lightning import (
    Gateway,
    LightningClaimPubkeReceivesRequest,
    LightningAwaitInvoiceRequest,
    LightningCreateInvoiceRequest,
    LightningCreateInvoiceResponse,
    LightningInvoiceForPubkeyTweakRequest,
    LightningInvoiceForPubkeyTweakResponse,
    LightningPayRequest,
    LightningPayResponse,
    LightningAwaitPayRequest,
    LightningPaymentResponse,
)

from models.onchain import (
    OnchainAwaitDepositRequest,
    OnchainAwaitDepositResponse,
    OnchainDepositAddressResponse,
    OnchainWithdrawRequest,
    OnchainWithdrawResponse,
)

from models.mint import (
    MintDecodeNotesRequest,
    MintDecodeNotesResponse,
    MintEncodeNotesRequest,
    MintEncodeNotesResponse,
    MintReissueRequest,
    MintReissueResponse,
    MintSpendRequest,
    MintSpendResponse,
    MintValidateRequest,
    MintValidateResponse,
    MintSplitRequest,
    MintCombineRequest,
    NotesJson,
)


class AsyncFedimintClient:

    def __init__(
        self,
        base_url: str,
        password: str,
        active_federation_id: str,
        active_gateway_id: str = None,
    ):
        self.base_url = f"{base_url}/v2"
        self.password = password
        self.active_federation_id = active_federation_id
        self.active_gateway_id = active_gateway_id
        self.session = aiohttp.ClientSession()
        atexit.register(self.close_sync)  # Register the cleanup function

        self.lightning = self.Lightning(self)
        self.onchain = self.Onchain(self)
        self.mint = self.Mint(self)
        logging.info(
            "Initialized fedimint client, must set active gateway id after intitalization to use lightning module methods or manually pass in gateways"
        )

    async def close(self):
        await self.session.close()

    def close_sync(self):
        try:
            loop = asyncio.get_event_loop()
        except RuntimeError as e:
            # If no event loop is available, create a new one for cleanup
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)

        if loop.is_running():
            loop.create_task(self.close())
        else:
            loop.run_until_complete(self.close())

    def get_active_federation_id(self):
        return self.active_federation_id

    def set_active_federation_id(self, federation_id: str):
        self.active_federation_id = federation_id

    def get_active_gateway_id(self):
        return self.active_gateway_id

    def set_active_gateway_id(self, gateway_id: str):
        self.active_gateway_id = gateway_id

    async def use_default_gateway(self):
        # hits list_gateways and sets active_gatewayId to the first gateway
        try:
            gateways = await self.lightning.list_gateways()
            logging.info("Gateways: ", gateways)
            self.active_gateway_id = gateways[0]["info"]["gateway_id"]
            logging.info("Set active gateway id to: ", self.active_gateway_id)
        except Exception as e:
            logging.error("Error setting default gateway id: ", e)

    async def _handle_response(self, response):
        if response.status != 200:
            text = await response.text()
            raise Exception(f"HTTP error! status: {response.status}, Body: {text}")

    async def _get(self, endpoint: str):
        headers = {"Authorization": f"Bearer {self.password}"}
        async with self.session.get(
            f"{self.base_url}{endpoint}", headers=headers
        ) as response:
            await self._handle_response(response)
            return await response.json()

    async def _post(self, endpoint: str, data=None):
        headers = {
            "Authorization": f"Bearer {self.password}",
            "Content-Type": "application/json",
        }
        async with self.session.post(
            f"{self.base_url}{endpoint}", json=data, headers=headers
        ) as response:
            await self._handle_response(response)
            return await response.json()

    async def _post_with_federation_id(
        self, endpoint: str, data=None, federation_id: str = None
    ):
        if federation_id is None:
            federation_id = self.get_active_federation_id()

        if data is None:
            data = {}
        data["federationId"] = federation_id

        return await self._post(endpoint, data)

    async def _post_with_gateway_id_and_federation_id(
        self,
        endpoint: str,
        data=None,
        gateway_id: str = None,
        federation_id: str = None,
    ):

        if gateway_id is None:
            gateway_id = self.get_active_gateway_id()

        if federation_id is None:
            federation_id = self.get_active_federation_id()

        if federation_id is None or gateway_id is None:
            raise Exception(
                "Must set active gateway id and active federation id before calling this method"
            )

        if data is None:
            data = {}
        data["gatewayId"] = gateway_id
        data["federationId"] = federation_id

        return await self._post(endpoint, data)

    async def info(self) -> InfoResponse:
        return await self._get("/admin/info")

    async def config(self):
        return await self._get("/admin/config")

    # TODO: Unsupported method
    # async def backup(self, request: BackupRequest, federationId: str = None):
    #     return await self._post_with_id("/admin/backup", request, federationId)

    async def discover_version(
        self, federation_id: str = None
    ) -> DiscoverVersionResponse:
        return await self._post_with_federation_id(
            "/admin/discover-version", {}, federation_id
        )

    async def federation_ids(self):
        return await self._get("/admin/federation-ids")

    async def list_operations(self, request: ListOperationsRequest):
        return await self._post_with_federation_id("/admin/list-operations", request)

    async def join(self, invite_code: str, use_manual_secret: bool = False):
        return await self._post(
            "/admin/join",
            {"inviteCode": invite_code, "useManualSecret": use_manual_secret},
        )

    class Lightning:
        def __init__(self, client):
            self.client = client

        async def create_invoice(
            self,
            amount_msat: int,
            description: str,
            expiry_time: int = None,
            gateway_id: str = None,
            federation_id: str = None,
        ) -> LightningCreateInvoiceResponse:
            request: LightningCreateInvoiceRequest = {
                "amountMsat": amount_msat,
                "description": description,
                "expiryTime": expiry_time,
            }

            return await self.client._post_with_gateway_id_and_federation_id(
                "/ln/invoice",
                data=request,
                gateway_id=gateway_id,
                federation_id=federation_id,
            )

        async def create_invoice_for_pubkey_tweak(
            self,
            pubkey: str,
            tweak: int,
            amount_msat: int,
            description: str,
            expiry_time: int = None,
            gateway_id: str = None,
            federation_id: str = None,
        ) -> LightningInvoiceForPubkeyTweakResponse:
            request: LightningInvoiceForPubkeyTweakRequest = {
                "externalPubkey": pubkey,
                "tweak": tweak,
                "amountMsat": amount_msat,
                "description": description,
                "expiryTime": expiry_time,
            }

            return await self.client._post_with_gateway_id_and_federation_id(
                "/ln/invoice-external-pubkey-tweaked",
                data=request,
                gateway_id=gateway_id,
                federation_id=federation_id,
            )

        async def claim_pubkey_tweak_receives(
            self,
            private_key: str,
            tweaks: List[int],
            federation_id: str = None,
        ) -> LightningPaymentResponse:
            request: LightningClaimPubkeReceivesRequest = {
                "privateKey": private_key,
                "tweaks": tweaks,
            }

            return await self.client._post_with_federation_id(
                "/ln/claim-external-receive-tweaked",
                data=request,
                federation_id=federation_id,
            )

        async def await_invoice(
            self, operation_id: str, federation_id: str = None
        ) -> LightningPaymentResponse:
            request: LightningAwaitInvoiceRequest = {"operationId": operation_id}
            return await self.client._post_with_gateway_id_and_federation_id(
                "/ln/await-invoice", request, federation_id=federation_id
            )

        async def pay(
            self,
            payment_info: str,
            amount_msat: Optional[int],
            lightning_url_comment: Optional[str],
            gateway_id: str = None,
            federation_id: str = None,
        ) -> LightningPayResponse:
            request: LightningPayRequest = {
                "paymentInfo": payment_info,
                "amountMsat": amount_msat,
                "lightningUrlComment": lightning_url_comment,
            }
            return await self.client._post_with_gateway_id_and_federation_id(
                "/ln/pay", request, gateway_id=gateway_id, federation_id=federation_id
            )

        async def await_pay(
            self, operation_id: str, gateway_id: str = None, federation_id: str = None
        ):
            request: LightningAwaitPayRequest = {"operationId": operation_id}
            return await self.client._post_with_gateway_id_and_federation_id(
                "/ln/await-pay",
                request,
                gateway_id=gateway_id,
                federation_id=federation_id,
            )

        async def list_gateways(self) -> List[Gateway]:
            return await self.client._post_with_federation_id("/ln/list-gateways")

    class Mint:
        def __init__(self, client):
            self.client = client

        async def decode_notes(
            self, notes: str, federation_id: str = None
        ) -> MintDecodeNotesResponse:
            request: MintDecodeNotesRequest = {"notes": notes}
            return await self.client._post_with_federation_id(
                "/mint/decode-notes", request, federation_id
            )

        async def encode_notes(
            self, notes_json: NotesJson, federation_id: str = None
        ) -> MintEncodeNotesResponse:
            request: MintEncodeNotesRequest = {"notesJsonStr": json.dumps(notes_json)}

            return await self.client._post_with_federation_id(
                "/mint/encode-notes", request, federation_id
            )

        async def reissue(
            self, notes: str, federation_id: str = None
        ) -> MintReissueResponse:
            request: MintReissueRequest = {"notes": notes}
            return await self.client._post_with_federation_id(
                "/mint/reissue", request, federation_id
            )

        async def spend(
            self,
            amount_msat: int,
            allow_overpay: bool,
            timeout: int,
            include_invite: bool,
            federation_id: str = None,
        ) -> MintSpendResponse:
            request: MintSpendRequest = {
                "amountMsat": amount_msat,
                "allowOverpay": allow_overpay,
                "timeout": timeout,
                "includeInvite": include_invite,
            }
            return await self.client._post_with_federation_id(
                "/mint/spend", request, federation_id
            )

        async def validate(
            self, notes: str, federation_id: str = None
        ) -> MintValidateResponse:
            request: MintValidateRequest = {"notes": notes}
            return await self.client._post_with_federation_id(
                "/mint/validate", request, federation_id
            )

        async def split(self, notes: str, federation_id: str = None):
            request: MintSplitRequest = {"notes": notes}
            return await self.client._post_with_federation_id(
                "/mint/split", request, federation_id
            )

        async def combine(self, notes_vec: List[str], federation_id: str = None):
            request: MintCombineRequest = {"notesVec": notes_vec}
            return await self.client._post_with_federation_id(
                "/mint/combine", request, federation_id
            )

    class Onchain:
        def __init__(self, client):
            self.client = client

        async def create_deposit_address(self, federation_id: str = None):
            return await self.client._post_with_federation_id(
                "/onchain/deposit-address", {}, federation_id
            )

        async def await_deposit(
            self, operation_id: str, federation_id: str = None
        ) -> OnchainAwaitDepositResponse:
            request: OnchainAwaitDepositRequest = {"operationId": operation_id}
            return await self.client._post_with_federation_id(
                "/onchain/await-deposit", request, federation_id
            )

        async def withdraw(
            self,
            address: str,
            amount_sat: Union[int, Literal["all"]],
            federationId: str = None,
        ) -> OnchainWithdrawResponse:
            request: OnchainWithdrawRequest = {
                "address": address,
                "amountSat": amount_sat,
            }
            return await self.client._post_with_federation_id(
                "/onchain/withdraw", request, federationId
            )
