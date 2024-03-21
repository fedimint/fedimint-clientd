from pydantic import RootModel, BaseModel
from typing import Optional, Dict, Any


class Tiered(RootModel):
    root: Dict[int, Any]


class TieredSummary(BaseModel):
    tiered: Tiered


class InfoResponse(BaseModel):
    federation_id: str
    network: str
    meta: Dict[str, str]
    total_amount_msat: int
    total_num_notes: int
    denominations_msat: TieredSummary


class BackupRequest(BaseModel):
    metadata: Dict[str, str]


class ListOperationsRequest(BaseModel):
    limit: int


class OperationOutput(BaseModel):
    id: str
    creation_time: str
    operation_kind: str
    operation_meta: Any
    outcome: Optional[Any]
