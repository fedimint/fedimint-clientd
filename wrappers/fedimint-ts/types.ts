interface Tiered<T> {
  [amount: number]: T;
}

interface TieredSummary {
  tiered: Tiered<number>;
}

interface InfoResponse {
  federationId: string;
  network: string;
  meta: { [key: string]: string };
  totalAmountMsat: number;
  totalNumNotes: number;
  denominationsMsat: TieredSummary;
}

interface FederationIdsResponse {
  federationIds: string[];
}

interface DiscoverVersionRequest {
  threshold?: number;
}

interface DiscoverVersionResponse {
  [federationId: string]: any;
}

interface JoinRequest {
  inviteCode: string;
  useManualSecret: boolean;
}

interface BackupRequest {
  metadata: { [key: string]: string };
}

interface ListOperationsRequest {
  limit: number;
}

interface OperationOutput {
  id: string;
  creationTime: string;
  operationKind: string;
  operationMeta: any;
  outcome?: any;
}

interface OnchainDepositAddressRequest {
  timeout: number;
}

interface OnchainDepositAddressResponse {
  operationId: string;
  address: string;
}

interface OnchainAwaitDepositRequest {
  operationId: string;
}

interface BTCInput {
  previous_output: string;
  script_sig: string;
  sequence: number;
  witness: string[];
}

interface BTCOutput {
  value: number;
  script_pubkey: string;
}

interface BTCTransaction {
  version: number;
  lock_time: number;
  input: BTCInput[];
  output: BTCOutput[];
}

interface AwaitDepositResponseConfirmed {
  btc_transaction: BTCTransaction;
  out_idx: number;
}

interface OnchainAwaitDepositResponse {
  status: { Confirmed: AwaitDepositResponseConfirmed } | { Failed: string };
}

interface OnchainWithdrawRequest {
  address: string;
  amountSat: number | "all";
}

interface OnchainWithdrawResponse {
  txid: string;
  feesSat: number;
}

interface LnInvoiceRequest {
  amountMsat: number;
  description: string;
  expiryTime?: number;
}

interface LnInvoiceResponse {
  operationId: string;
  invoice: string;
}

interface LnInvoiceExternalPubkeyRequest {
  amountMsat: number;
  description: string;
  externalPubkey: string;
  expiryTime?: number;
}

interface LnInvoiceExternalPubkeyResponse {
  operationId: string;
  invoice: string;
}

interface LnInvoiceExternalPubkeyTweakedRequest {
  amountMsat: number;
  description: string;
  externalPubkey: string;
  tweak: number;
  expiryTime?: number;
}

interface LnInvoiceExternalPubkeyTweakedResponse {
  operationId: string;
  invoice: string;
}

interface LnClaimPubkeyReceiveRequest {
  privateKey: string;
}

interface LnClaimPubkeyReceiveTweakedRequest {
  privateKey: string;
  tweaks: number[];
}

interface LnAwaitInvoiceRequest {
  operationId: string;
}

interface LnPayRequest {
  paymentInfo: string;
  amountMsat?: number;
  lnurlComment?: string;
}

interface LnPayResponse {
  operationId: string;
  paymentType: string;
  contractId: string;
  fee: number;
}

interface LnAwaitPayRequest {
  operationId: string;
}

interface Gateway {
  nodePubKey: string;
  active: boolean;
}

interface SwitchGatewayRequest {
  gatewayId: string;
}

type FederationIdPrefix = string;

interface TieredMulti<T> {
  [amount: number]: T[];
}

interface MintDecodeNotesRequest {
  notes: string;
}

interface MintDecodeNotesResponse {
  notesJson: NotesJson;
}

interface MintEncodeNotesRequest {
  notesJsonStr: string;
}

interface MintEncodeNotesResponse {
  notes: string;
}

interface MintReissueRequest {
  notes: string;
}

interface MintReissueResponse {
  amountMsat: number;
}

interface MintSpendRequest {
  amountMsat: number;
  allowOverpay: boolean;
  timeout: number;
  includeInvite: boolean;
}

interface MintSpendResponse {
  operation: string;
  notes: string;
}

interface MintValidateRequest {
  notes: string;
}

interface MintValidateResponse {
  amountMsat: number;
}

interface MintSplitRequest {
  notes: string;
}

interface MintSplitResponse {
  notes: Record<number, string>;
}

interface MintCombineRequest {
  notesVec: string[];
}

interface MintCombineResponse {
  notes: string;
}

interface NotesJson {
  federation_id_prefix: string;
  notes: {
    [denomination: string]: Array<{
      signature: string;
      spend_key: string;
    }>;
  };
}

export type {
  Tiered,
  TieredSummary,
  InfoResponse,
  FederationIdsResponse,
  DiscoverVersionRequest,
  DiscoverVersionResponse,
  JoinRequest,
  BackupRequest,
  ListOperationsRequest,
  OperationOutput,
  OnchainDepositAddressRequest,
  OnchainDepositAddressResponse,
  OnchainAwaitDepositRequest,
  OnchainAwaitDepositResponse,
  OnchainWithdrawRequest,
  OnchainWithdrawResponse,
  LnInvoiceRequest,
  LnInvoiceResponse,
  LnInvoiceExternalPubkeyRequest,
  LnInvoiceExternalPubkeyResponse,
  LnInvoiceExternalPubkeyTweakedRequest,
  LnInvoiceExternalPubkeyTweakedResponse,
  LnClaimPubkeyReceiveRequest,
  LnClaimPubkeyReceiveTweakedRequest,
  LnAwaitInvoiceRequest,
  LnPayRequest,
  LnPayResponse,
  LnAwaitPayRequest,
  Gateway,
  NotesJson,
  SwitchGatewayRequest,
  MintDecodeNotesRequest,
  MintDecodeNotesResponse,
  MintEncodeNotesRequest,
  MintEncodeNotesResponse,
  MintReissueRequest,
  MintReissueResponse,
  MintSpendRequest,
  MintSpendResponse,
  MintValidateRequest,
  MintValidateResponse,
  MintSplitRequest,
  MintSplitResponse,
  MintCombineRequest,
  MintCombineResponse,
};
