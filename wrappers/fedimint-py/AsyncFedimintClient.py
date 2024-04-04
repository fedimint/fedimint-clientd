import logging
from typing import List
import aiohttp

from models.common import (
    DiscoverVersionRequest,
    DiscoverVersionResponse,
    InfoResponse,
    ListOperationsRequest,
)

from models.lightning import (
    Gateway,
    LightningClaimPubkeReceivesRequest,
    LightningCreateInvoiceRequest,
    LightningCreateInvoiceResponse,
    LightningAwaitInvoiceRequest,
    LightningAwaitInvoiceResponse,
    LightningPayRequest,
    LightningPayResponse,
    LightningAwaitPayRequest,
    LightningAwaitPayResponse,
    ListGatewaysResponse,
    SwitchGatewayRequest,
)

from models.onchain import (
    OnchainAwaitDepositRequest,
    OnchainAwaitDepositResponse,
    OnchainDepositAddressRequest,
    OnchainDepositAddressResponse,
    OnchainWithdrawRequest,
    OnchainWithdrawResponse,
)

from models.mint import (
    MintReissueRequest,
    MintReissueResponse,
    MintSpendRequest,
    MintSpendResponse,
    MintValidateRequest,
    MintValidateResponse,
    MintSplitRequest,
    MintSplitResponse,
    MintCombineRequest,
    MintCombineResponse,
)


class AsyncFedimintClient:
    def __init__(
        self,
        base_url: str,
        password: str,
        active_federationId: str,
        active_gatewayId: str = None,
    ):
        self.base_url = f"{base_url}/fedimint/v2"
        self.password = password
        self.active_federationId = active_federationId
        self.active_gatewayId = active_gatewayId
        self.session = aiohttp.ClientSession()

        self.lightning = self.Lightning(self)
        self.onchain = self.Onchain(self)
        self.mint = self.Mint(self)
        logging.info(
            "Initialized fedimint client, must set active gateway id after intitalization to use lightning module methods or manually pass in gateways"
        )

    def get_active_federationId(self):
        return self.active_federationId

    def set_active_federationId(self, federationId: str):
        self.active_federationId = federationId

    def get_active_gatewayId(self):
        return self.active_gatewayId

    def set_active_gatewayId(self, gatewayId: str):
        self.active_gatewayId = gatewayId

    def use_default_gateway(self):
        # hits list_gateways and sets active_gatewayId to the first gateway
        try:
            gateways = self.lightning.list_gateways()
            logging.info("Gateways: ", gateways)
            self.active_gateway_id = gateways[0].info.gatewayId
            logging.info("Set active gateway id to: ", self.active_gatewayId)
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
        self, endpoint: str, data=None, federationId: str = None
    ):
        if federationId is None:
            federationId = self.get_active_federation_id()

        if data is None:
            data = {}
        data["federationId"] = federationId

        return await self._post(endpoint, data)

    async def _post_with_gateway_id_and_federation_id(
        self,
        endpoint: str,
        data=None,
        gatewayId: str = None,
        federationId: str = None,
    ):

        if gatewayId is None:
            gatewayId = self.get_active_gateway_id()

        if federationId is None:
            federationId = self.get_active_federation_id()

        if federationId is None or gatewayId is None:
            raise Exception(
                "Must set active gateway id and active federation id before calling this method"
            )

        if data is None:
            data = {}
        data["gatewayId"] = gatewayId
        data["federationId"] = federationId

        return await self._post(endpoint, data)

    async def info(self) -> InfoResponse:
        return await self._get("/admin/info")

    async def config(self):
        return await self._get("/admin/config")

    # TODO: Unsupported method
    # async def backup(self, request: BackupRequest, federationId: str = None):
    #     return await self._post_with_id("/admin/backup", request, federationId)

    async def discover_version(self, threshold: int) -> DiscoverVersionResponse:
        request: DiscoverVersionRequest = {"threshold": threshold}
        return await self._post("/admin/discover-version", request)

    async def federation_ids(self):
        return await self._get("/admin/federation-ids")

    async def list_operations(self, request: ListOperationsRequest):
        return await self._post_with_federation_id("/admin/list-operations", request)

    async def join(self, invite_code: str, set_default: bool = False):
        return await self._post(
            "/admin/join", {"inviteCode": invite_code, "setDefault": set_default}
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
        ) -> LightningCreateInvoiceResponse:
            request: LightningCreateInvoiceRequest = {
                "pubkey": pubkey,
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
        ) -> InfoResponse:
            request: LightningClaimPubkeReceivesRequest = {
                "privateKey": private_key,
                "tweaks": tweaks,
            }

            return await self.client._post_with_federation_id(
                "/ln/claim-external-pubkey-tweaked",
                data=request,
                federation_id=federation_id,
            )

        async def await_invoice(
            self, request: LightningAwaitInvoiceRequest, federationId: str = None
        ) -> InfoResponse:
            return await self.client._post_with_gateway_id_and_federation_id(
                "/ln/await-invoice", request, federationId
            )

        async def pay(
            self, request: LightningPayRequest, federationId: str = None
        ) -> LightningPayResponse:
            return await self.client._post_with_gateway_id_and_federation_id(
                "/ln/pay", request, federationId
            )

        async def await_pay(
            self, request: LightningAwaitPayRequest, federationId: str = None
        ):
            return await self.client._post_with_gateway_id_and_federation_id(
                "/ln/await-pay", request, federationId
            )

        async def list_gateways(self) -> List[Gateway]:
            return await self.client._post_with_federation_id("/ln/list-gateways")

    class Onchain:
        def __init__(self, client):
            self.client = client

        async def create_deposit_address(
            self, request: OnchainDepositAddressRequest, federationId: str = None
        ):
            return self.client._post_with_id(
                "/wallet/deposit-address", data=request, federationId=federationId
            )

        async def await_deposit(
            self, request: OnchainAwaitDepositRequest, federationId: str = None
        ):
            return await self.client._post_with_id(
                "/wallet/await-deposit", request, federationId
            )

        async def withdraw(
            self, request: OnchainWithdrawRequest, federationId: str = None
        ):
            return await self.client._post_with_id(
                "/wallet/withdraw", request, federationId
            )

    class Mint:
        def __init__(self, client):
            self.client = client

        async def reissue(self, request: MintReissueRequest, federationId: str = None):
            return await self.client._post_with_id(
                "/mint/reissue", request, federationId
            )

        async def spend(self, request: MintSpendRequest, federationId: str = None):
            return await self.client._post_with_id("/mint/spend", request, federationId)

        async def validate(
            self, request: MintValidateRequest, federationId: str = None
        ):
            return await self.client._post_with_id(
                "/mint/validate", request, federationId
            )

        async def split(self, request: MintSplitRequest, federationId: str = None):
            return await self.client._post_with_id("/mint/split", request, federationId)

        async def combine(self, request: MintCombineRequest, federationId: str = None):
            return await self.client._post_with_id(
                "/mint/combine", request, federationId
            )

    async def close(self):
        await self.session.close()
