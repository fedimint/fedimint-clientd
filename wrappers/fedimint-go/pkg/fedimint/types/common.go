package types

type Tiered map[int]interface{}

type TieredSummary struct {
	Tiered Tiered `json:"tiered"`
}

type InfoResponse struct {
	FederationID      string            `json:"federation_id"`
	Network           string            `json:"network"`
	Meta              map[string]string `json:"meta"`
	TotalAmountMsat   int               `json:"total_amount_msat"`
	TotalNumNotes     int               `json:"total_num_notes"`
	DenominationsMsat TieredSummary     `json:"denominations_msat"`
}

type BackupRequest struct {
	Metadata map[string]string `json:"metadata"`
}

type ListOperationsRequest struct {
	Limit int `json:"limit"`
}

type FederationIdsResponse struct {
	FederationIds []string `json:"federationIds"`
}

type JoinRequest struct {
	InviteCode      string `json:"inviteCode"`
	UseManualSecret bool   `json:"useManualSecret"`
}

type JoinResponse struct {
	ThisFederationId string   `json:"thisFederationId"`
	FederationIds    []string `json:"federationIds"`
}

type OperationOutput struct {
	ID            string      `json:"id"`
	CreationTime  string      `json:"creation_time"`
	OperationKind string      `json:"operation_kind"`
	OperationMeta interface{} `json:"operation_meta"`
	Outcome       interface{} `json:"outcome,omitempty"`
}

type FedimintResponse map[string]interface{}
