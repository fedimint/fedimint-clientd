package modules

type FederationIdPrefix struct {
	Value [4]byte `json:"value"`
}

type TieredMulti struct {
	Amount []interface{} `json:"amount"`
}

type Signature struct {
	Zero G1Affine `json:"zero"`
}

type G1Affine struct {
	X        Fp     `json:"x"`
	Y        Fp     `json:"y"`
	Infinity Choice `json:"infinity"`
}

type Fp struct {
	Zero []uint64 `json:"zero"`
}

type Choice struct {
	Zero uint8 `json:"zero"`
}

type KeyPair struct {
	Zero []uint8 `json:"zero"`
}

type OOBNotesData struct {
	Notes              *TieredMulti        `json:"notes"`
	FederationIdPrefix *FederationIdPrefix `json:"federation_id_prefix"`
	Default            struct {
		Variant uint64  `json:"variant"`
		Bytes   []uint8 `json:"bytes"`
	} `json:"default"`
}

type OOBNotes struct {
	Zero []OOBNotesData `json:"zero"`
}

type SpendableNote struct {
	Signature Signature `json:"signature"`
	SpendKey  KeyPair   `json:"spend_key"`
}

// @> `ReissueRequest` notes should be string? as fedimint-ts does uses string.
type ReissueRequest struct {
	Notes OOBNotes `json:"notes"`
}

type ReissueResponse struct {
	AmountMsat uint64 `json:"amount_msat"`
}

type SpendRequest struct {
	AmountMsat   uint64 `json:"amount_msat"`
	AllowOverpay bool   `json:"allow_overpay"`
	Timeout      uint64 `json:"timeout"`
}

type SpendResponse struct {
	Operation string   `json:"operation"`
	Notes     OOBNotes `json:"notes"`
}

type ValidateRequest struct {
	Notes OOBNotes `json:"notes"`
}

type ValidateResponse struct {
	AmountMsat uint64 `json:"amount_msat"`
}

type SplitRequest struct {
	Notes OOBNotes `json:"notes"`
}

type SplitResponse struct {
	Notes map[uint64]OOBNotes `json:"notes"`
}

type CombineRequest struct {
	Notes []OOBNotes `json:"notes"`
}

type CombineResponse struct {
	Notes OOBNotes `json:"notes"`
}
