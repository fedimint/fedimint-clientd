import aiohttp
from models.common import (
    InfoResponse,
    ListOperationsRequest,
    OperationOutput,
    BackupRequest,
)
from models.ln import (
    LnInvoiceRequest,
    LnInvoiceResponse,
    AwaitInvoiceRequest,
    LnPayRequest,
    LnPayResponse,
    AwaitLnPayRequest,
    Gateway,
    SwitchGatewayRequest,
)
from models.wallet import DepositAddressRequest, AwaitDepositRequest, WithdrawRequest
from models.mint import (
    ReissueRequest,
    SpendRequest,
    ValidateRequest,
    SplitRequest,
    CombineRequest,
)


class AsyncFedimintClient:
    def __init__(self, base_url: str, password: str, active_federation_id: str):
        self.base_url = f"{base_url}/fedimint/v2"
        self.password = password
        self.active_federation_id = active_federation_id
        self.session = aiohttp.ClientSession()

        self.ln = self.LN(self)
        self.wallet = self.Wallet(self)
        self.mint = self.Mint(self)

    def get_active_federation_id(self):
        return self.active_federation_id

    def set_active_federation_id(self, federation_id: str):
        self.active_federation_id = federation_id

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

    async def _post_with_id(self, endpoint: str, data=None, federation_id: str = None):
        if federation_id is None:
            federation_id = self.get_active_federation_id()

        if data is None:
            data = {}
        data["federationId"] = federation_id

        return await self._post(endpoint, data)

    async def info(self):
        return await self._get("/admin/info")

    async def backup(self, request: BackupRequest, federation_id: str = None):
        return await self._post_with_id("/admin/backup", request, federation_id)

    async def config(self):
        return await self._get("/admin/config")

    async def discover_version(self):
        return await self._get("/admin/discover-version")

    async def federation_ids(self):
        return await self._get("/admin/federation-ids")

    async def list_operations(self, request: ListOperationsRequest):
        return await self._post_with_id("/admin/list-operations", request)

    async def join(self, invite_code: str, set_default: bool = False):
        return await self._post(
            "/admin/join", {"inviteCode": invite_code, "setDefault": set_default}
        )

    class LN:
        def __init__(self, client):
            self.client = client

        async def create_invoice(
            self, request: LnInvoiceRequest, federation_id: str = None
        ):
            return await self.client._post_with_id(
                "/ln/invoice", request, federation_id
            )

        async def await_invoice(
            self, request: AwaitInvoiceRequest, federation_id: str = None
        ):
            return await self.client._post_with_id(
                "/ln/await-invoice", request, federation_id
            )

        async def pay(self, request: LnPayRequest, federation_id: str = None):
            return await self.client._post_with_id("/ln/pay", request, federation_id)

        async def await_pay(
            self, request: AwaitLnPayRequest, federation_id: str = None
        ):
            return await self.client._post_with_id(
                "/ln/await-pay", request, federation_id
            )

        async def list_gateways(self):
            return await self.client._get("/ln/list-gateways")

        async def switch_gateway(
            self, request: SwitchGatewayRequest, federation_id: str = None
        ):
            return await self.client._post_with_id(
                "/ln/switch-gateway", request, federation_id
            )

    class Wallet:
        def __init__(self, client):
            self.client = client

        async def create_deposit_address(
            self, request: DepositAddressRequest, federation_id: str = None
        ):
            return self.client._post_with_id(
                "/wallet/deposit-address", data=request, federation_id=federation_id
            )

        async def await_deposit(
            self, request: AwaitDepositRequest, federation_id: str = None
        ):
            return await self.client._post_with_id(
                "/wallet/await-deposit", request, federation_id
            )

        async def withdraw(self, request: WithdrawRequest, federation_id: str = None):
            return await self.client._post_with_id(
                "/wallet/withdraw", request, federation_id
            )

    class Mint:
        def __init__(self, client):
            self.client = client

        async def reissue(self, request: ReissueRequest, federation_id: str = None):
            return await self.client._post_with_id(
                "/mint/reissue", request, federation_id
            )

        async def spend(self, request: SpendRequest, federation_id: str = None):
            return await self.client._post_with_id(
                "/mint/spend", request, federation_id
            )

        async def validate(self, request: ValidateRequest, federation_id: str = None):
            return await self.client._post_with_id(
                "/mint/validate", request, federation_id
            )

        async def split(self, request: SplitRequest, federation_id: str = None):
            return await self.client._post_with_id(
                "/mint/split", request, federation_id
            )

        async def combine(self, request: CombineRequest, federation_id: str = None):
            return await self.client._post_with_id(
                "/mint/combine", request, federation_id
            )

    async def close(self):
        await self.session.close()
