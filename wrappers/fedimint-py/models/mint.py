from pydantic import BaseModel, RootModel
from typing import Optional, Dict, List, Union


class FederationIdPrefix(RootModel):
    root: List[int]


class Fp(RootModel):
    root: List[int]


class Choice(RootModel):
    root: int


class G1Affine(BaseModel):
    x: Fp
    y: Fp
    infinity: Choice


class Signature(RootModel):
    root: G1Affine


class KeyPair(RootModel):
    root: List[int]


class SpendableNote(BaseModel):
    signature: Signature
    spend_key: KeyPair


class TieredMulti(RootModel):
    root: Dict[int, List[SpendableNote]]


class OOBNotesData(BaseModel):
    Notes: Optional[TieredMulti]
    FederationIdPrefix: Optional[FederationIdPrefix]
    Default: Optional[Dict[str, Union[int, List[int]]]]


class OOBNotes(RootModel):
    root: List[OOBNotesData]


class ReissueRequest(BaseModel):
    notes: OOBNotes


class ReissueResponse(BaseModel):
    amount_msat: int


class SpendRequest(BaseModel):
    amount_msat: int
    allow_overpay: bool
    timeout: int


class SpendResponse(BaseModel):
    operation: str
    notes: OOBNotes


class ValidateRequest(BaseModel):
    notes: OOBNotes


class ValidateResponse(BaseModel):
    amount_msat: int


class SplitRequest(BaseModel):
    notes: OOBNotes


class SplitResponse(BaseModel):
    notes: Dict[int, OOBNotes]


class CombineRequest(BaseModel):
    notes: List[OOBNotes]


class CombineResponse(BaseModel):
    notes: OOBNotes
