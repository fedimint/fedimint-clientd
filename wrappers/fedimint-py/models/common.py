from pydantic import RootModel, BaseModel
from typing import List, Optional, Dict, Any


class Tiered(RootModel):
    root: Dict[int, Any]


class TieredSummary(BaseModel):
    tiered: Tiered


class FederationInfo(BaseModel):
    network: str
    meta: Dict[str, str]
    totalAmountMsat: int
    totalNumNotes: int
    denominationsMsat: TieredSummary


class InfoResponse(BaseModel):
    __root__: Dict[str, FederationInfo]


class BackupRequest(BaseModel):
    metadata: Dict[str, str]


class ListOperationsRequest(BaseModel):
    limit: int


class OperationOutput(BaseModel):
    id: str
    creationTime: str
    operationKind: str
    operationMeta: Any
    outcome: Optional[Any]


class DiscoverVersionRequest(BaseModel):
    threshold: Optional[int]


# Returns a dictionary of federation_ids and their api versions
class DiscoverVersionResponse(BaseModel):
    __root__: Dict[str, Any]


class JoinRequest(BaseModel):
    inviteCode: str
    useManualSecret: bool


class JoinResponse(BaseModel):
    thisFederationId: str
    federationIds: List[str]


class ListOperationsRequest(BaseModel):
    limit: int


class OperationOutput(BaseModel):
    id: str
    creationTime: str
    operationKind: str
    operationMeta: Any
    outcome: Optional[Any]
