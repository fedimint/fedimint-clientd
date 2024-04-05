from typing import Dict, List, Literal, Union
from pydantic import BaseModel


class OnchainDepositAddressRequest(BaseModel):
    timeout: int


class OnchainDepositAddressResponse(BaseModel):
    operation_id: str
    address: str


class OnchainAwaitDepositRequest(BaseModel):
    operation_id: str


class BTCInput(BaseModel):
    previous_output: str
    script_sig: str
    sequence: int
    witness: List[str]


class BTCOutput(BaseModel):
    value: int
    script_pubkey: str


class BTCTransaction(BaseModel):
    version: int
    lock_time: int
    input: List[BTCInput]
    output: List[BTCOutput]


class AwaitDepositResponseConfirmed(BaseModel):
    btc_transaction: BTCTransaction
    out_idx: int


class OnchainAwaitDepositResponse(BaseModel):
    status: Union[Dict[str, AwaitDepositResponseConfirmed], Dict[str, str]]


class OnchainWithdrawRequest(BaseModel):
    address: str
    amount_sat: Union[int, Literal["all"]]


class OnchainWithdrawResponse(BaseModel):
    txid: str
    fees_sat: int
