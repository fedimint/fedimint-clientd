from pydantic import BaseModel
from typing import Dict, List


class MintDecodeNotesRequest(BaseModel):
    notes: str


class Note(BaseModel):
    signature: str
    spendKey: str


class NotesJson(BaseModel):
    federation_id_prefix: str
    notes: Dict[str, List[Note]]


class MintDecodeNotesResponse(BaseModel):
    notes: NotesJson


class MintEncodeNotesRequest(BaseModel):
    notesJsonStr: str


class MintEncodeNotesResponse(BaseModel):
    notesJson: str


class MintReissueRequest(BaseModel):
    notes: str


class MintReissueResponse(BaseModel):
    amountMsat: int


class MintSpendRequest(BaseModel):
    amountMsat: int
    allowOverpay: bool
    timeout: int
    includeInvite: bool


class MintSpendResponse(BaseModel):
    operation: str
    notes: str


class MintValidateRequest(BaseModel):
    notes: str


class MintValidateResponse(BaseModel):
    amountMsat: int


class MintSplitRequest(BaseModel):
    notes: str


class MintSplitResponse(BaseModel):
    notes: Dict[int, str]


class MintCombineRequest(BaseModel):
    notesVec: List[str]


class MintCombineResponse(BaseModel):
    notes: str
