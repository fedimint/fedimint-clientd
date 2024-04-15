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
	SpendKey  KeyPair   `json:"spendKey"`
}

type NotesJson struct {
	FederationIdPrefix string                             `json:"federation_id_prefix"`
	Notes              map[string][]SignatureSpendKeyPair `json:"notes"`
}

type SignatureSpendKeyPair struct {
	Signature string `json:"signature"`
	SpendKey  string `json:"spend_key"`
}

type DecodeNotesRequest struct {
	Notes string `json:"notes"`
}

type DecodeNotesResponse struct {
	NotesJson NotesJson `json:"notesJson"`
}

type EncodeNotesRequest struct {
	NotesJsonStr string `json:"notesJsonStr"`
}

type EncodeNotesResponse struct {
	Notes string `json:"notes"`
}
type MintReissueRequest struct {
	Notes string `json:"notes"`
}

type MintReissueResponse struct {
	AmountMsat uint64 `json:"amountMsat"`
}

type MintSpendRequest struct {
	AmountMsat    uint64 `json:"amountMsat"`
	AllowOverpay  bool   `json:"allowOverpay"`
	Timeout       uint64 `json:"timeout"`
	IncludeInvite bool   `json:"includeInvite"`
}

type MintSpendResponse struct {
	OperationId string `json:"operationId"`
	Notes       string `json:"notes"`
}

type MintValidateRequest struct {
	Notes string `json:"notes"`
}

type MintValidateResponse struct {
	AmountMsat uint64 `json:"amountMsat"`
}

type MintSplitRequest struct {
	Notes string `json:"notes"`
}

type MintSplitResponse struct {
	Notes map[uint64]string `json:"notes"`
}

type MintCombineRequest struct {
	NotesVec []string `json:"notesVec"`
}

type MintCombineResponse struct {
	Notes string `json:"notes"`
}
