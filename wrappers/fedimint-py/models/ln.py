from pydantic import BaseModel
from typing import Optional


class LnInvoiceRequest(BaseModel):
    amount_msat: int
    description: str
    expiry_time: Optional[int]


class LnInvoiceResponse(BaseModel):
    operation_id: str
    invoice: str


class AwaitInvoiceRequest(BaseModel):
    operation_id: str


class LnPayRequest(BaseModel):
    payment_info: str
    amount_msat: Optional[int]
    finish_in_background: bool
    lnurl_comment: Optional[str]


class LnPayResponse(BaseModel):
    operation_id: str
    payment_type: str
    contract_id: str
    fee: int


class AwaitLnPayRequest(BaseModel):
    operation_id: str


class Gateway(BaseModel):
    node_pub_key: str
    active: bool


class SwitchGatewayRequest(BaseModel):
    gateway_id: str
