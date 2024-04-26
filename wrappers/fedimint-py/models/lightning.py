from pydantic import BaseModel
from typing import Any, List, Optional, Union
from enum import Enum


class LightningCreateInvoiceRequest(BaseModel):
    amountMsat: int
    description: str
    expiryTime: Optional[int]


class LightningCreateInvoiceResponse(BaseModel):
    operationId: str
    invoice: str


class LightningInvoiceForPubkeyTweakRequest(BaseModel):
    amountMsat: int
    description: str
    externalPubkey: str
    tweak: int
    expiryTime: Optional[int]


class LightningInvoiceForPubkeyTweakResponse(BaseModel):
    operationId: str
    invoice: str


class LightningClaimPubkeReceivesRequest(BaseModel):
    privateKey: str
    tweaks: List[int]


class LightningAwaitInvoiceRequest(BaseModel):
    operationId: str


class LightningPayRequest(BaseModel):
    paymentInfo: str
    amountMsat: Optional[int]
    lightningUrlComment: Optional[str]


class LightningPayResponse(BaseModel):
    operationId: str
    paymentType: str
    contractId: str
    fee: int


class LightningAwaitPayRequest(BaseModel):
    operationId: str


class LnReceiveState(Enum):
    Created = "Created"
    WaitingForPayment = "WaitingForPayment"
    Canceled = "Canceled"
    Funded = "Funded"
    AwaitingFunds = "AwaitingFunds"
    Claimed = "Claimed"


class WaitingForPayment(BaseModel):
    invoice: str
    timeout: int


class Canceled(BaseModel):
    reason: str


class LightningPaymentResponse(BaseModel):
    state: LnReceiveState
    details: Optional[Union[WaitingForPayment, Canceled]] = None


class GatewayFees(BaseModel):
    baseMsat: int
    proportionalMillionths: int


class GatewayInfo(BaseModel):
    api: str
    fees: GatewayFees
    gatewayId: str
    gatewayRedeemKey: str
    lightningAlias: str
    mintChannelId: int
    nodePubKey: str
    routeHints: List[Any]
    supportsPrivatePayments: bool


class GatewayTTL(BaseModel):
    nanos: int
    secs: int


class Gateway(BaseModel):
    federationId: str
    info: GatewayInfo
    ttl: GatewayTTL
    vetted: bool
