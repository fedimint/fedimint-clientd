from pydantic import BaseModel


class DepositAddressRequest(BaseModel):
    timeout: int


class DepositAddressResponse(BaseModel):
    operation_id: str
    address: str


class AwaitDepositRequest(BaseModel):
    operation_id: str


class AwaitDepositResponse(BaseModel):
    status: str


class WithdrawRequest(BaseModel):
    address: str
    amount_msat: str


class WithdrawResponse(BaseModel):
    txid: str
    fees_sat: int
