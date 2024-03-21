import requests
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


class FedimintClient:
    def __init__(self, base_url: str, password: str, active_federation_id: str):
        self.base_url = f"{base_url}/fedimint/v2"
        self.password = password
        self.active_federation_id = active_federation_id

        self.ln = self.LN(self)
        self.wallet = self.Wallet(self)
        self.mint = self.Mint(self)

    def get_active_federation_id(self):
        return self.active_federation_id

    def set_active_federation_id(self, federation_id: str):
        self.active_federation_id = federation_id

    def _handle_response(self, response):
        if not response.ok:
            raise Exception(
                f"HTTP error! status: {response.status_code}, Body: {response.text}"
            )

    def _get(self, endpoint: str):
        headers = {"Authorization": f"Bearer {self.password}"}
        response = requests.get(f"{self.base_url}{endpoint}", headers=headers)
        self._handle_response(response)
        return response.json()

    def _post(self, endpoint: str, data=None):
        headers = {
            "Authorization": f"Bearer {self.password}",
            "Content-Type": "application/json",
        }
        response = requests.post(
            f"{self.base_url}{endpoint}", json=data, headers=headers
        )
        self._handle_response(response)
        return response.json()

    def _post_with_id(self, endpoint: str, data=None, federation_id: str = None):
        if federation_id is None:
            federation_id = self.get_active_federation_id()

        if data is None:
            data = {}
        data["federationId"] = federation_id

        return self._post(endpoint, data)

    def info(self):
        return self._get("/admin/info")

    def backup(self, request: BackupRequest, federation_id: str = None):
        return self._post_with_id("/admin/backup", request, federation_id)

    def config(self):
        return self.get("/admin/config")

    def discover_version(self):
        return self._get("/admin/discover-version")

    def federation_ids(self):
        return self._get("/admin/federation-ids")

    def list_operations(self, request: ListOperationsRequest):
        return self._fetch_with_auth("/admin/list-operations", "POST", data=request)

    def join(self, invite_code: str, set_default: bool = False):
        return self._post(
            "/admin/join", {"inviteCode": invite_code, "setDefault": set_default}
        )

    # def module(self, name: str):
    #     return self._fetch_with_auth(f'/admin/module', 'POST')

    # def restore(self, request: RestoreRequest):
    #     return self._fetch_with_auth('/admin/restore', 'POST', data=request)

    class LN:
        def __init__(self, client):
            self.client = client

        def create_invoice(self, request: LnInvoiceRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/ln/invoice", data=request, federation_id=federation_id
            )

        def await_invoice(
            self, request: AwaitInvoiceRequest, federation_id: str = None
        ):
            return self.client._post_with_id(
                "/ln/await-invoice", data=request, federation_id=federation_id
            )

        def pay(self, request: LnPayRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/ln/pay", data=request, federation_id=federation_id
            )

        def await_pay(self, request: AwaitLnPayRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/ln/await-pay", data=request, federation_id=federation_id
            )

        def list_gateways(self):
            return self.client._get("/ln/list-gateways")

        def switch_gateway(
            self, request: SwitchGatewayRequest, federation_id: str = None
        ):
            return self.client._post_with_id(
                "/ln/switch-gateway", data=request, federation_id=federation_id
            )

    class Wallet:
        def __init__(self, client):
            self.client = client

        def create_deposit_address(
            self, request: DepositAddressRequest, federation_id: str = None
        ):
            return self.client._post_with_id(
                "/wallet/deposit-address", data=request, federation_id=federation_id
            )

        def await_deposit(
            self, request: AwaitDepositRequest, federation_id: str = None
        ):
            return self.client._post_with_id(
                "/wallet/await-deposit", data=request, federation_id=federation_id
            )

        def withdraw(self, request: WithdrawRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/wallet/withdraw", data=request, federation_id=federation_id
            )

    class Mint:
        def __init__(self, client):
            self.client = client

        def reissue(self, request: ReissueRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/mint/reissue", data=request, federation_id=federation_id
            )

        def spend(self, request: SpendRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/mint/spend", data=request, federation_id=federation_id
            )

        def validate(self, request: ValidateRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/mint/validate", data=request, federation_id=federation_id
            )

        def split(self, request: SplitRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/mint/split", data=request, federation_id=federation_id
            )

        def combine(self, request: CombineRequest, federation_id: str = None):
            return self.client._post_with_id(
                "/mint/combine", data=request, federation_id=federation_id
            )
