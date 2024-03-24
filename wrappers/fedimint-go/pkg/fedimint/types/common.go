package types

import "encoding/json"

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
	Limit uint16 `json:"limit"`
}

// OperationOutput mirrors the Rust OperationOutput struct for JSON unmarshalling
type OperationOutput struct {
	ID            string           `json:"id"`
	CreationTime  string           `json:"creationTime"`
	OperationKind string           `json:"operationKind"`
	OperationMeta json.RawMessage  `json:"operationMeta"`     // Use json.RawMessage for arbitrary JSON
	Outcome       *json.RawMessage `json:"outcome,omitempty"` // Pointer to handle optional field
}

// ListOperationsResponse represents the JSON response structure from the listOperations endpoint
type ListOperationsResponse struct {
	Operations []OperationOutput `json:"operations"`
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

type DiscoverVersionRequest struct {
	Threshold uint16 `json:"threshold"`
}

type FedimintResponse map[string]interface{}
