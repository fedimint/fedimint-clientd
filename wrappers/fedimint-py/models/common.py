from pydantic import RootModel, BaseModel
from typing import List, Optional, Dict, Any


Tiered = RootModel[Dict[int, Any]]


class TieredSummary(BaseModel):
    tiered: Tiered


class FederationInfo(BaseModel):
    network: str
    meta: Dict[str, str]
    totalAmountMsat: int
    totalNumNotes: int
    denominationsMsat: TieredSummary


InfoResponse = RootModel[Dict[str, FederationInfo]]


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
DiscoverVersionResponse = RootModel[Dict[str, Any]]


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
