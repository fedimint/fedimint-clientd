interface Tiered<T> {
  [amount: number]: T;
}

interface TieredSummary {
  tiered: Tiered<number>;
}

interface InfoResponse {
  [federationId: string]: {
    network: string;
    meta: { [key: string]: string };
    totalAmountMsat: number;
    totalNumNotes: number;
    denominationsMsat: TieredSummary;
  };
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

interface JoinResponse {
  thisFederationId: string;
  federationIds: string[];
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

interface LightningInvoiceRequest {
  amountMsat: number;
  description: string;
  expiryTime?: number;
}

interface LightningInvoiceResponse {
  operationId: string;
  invoice: string;
}

interface LightningInvoiceExternalPubkeyRequest {
  amountMsat: number;
  description: string;
  externalPubkey: string;
  expiryTime?: number;
}

interface LightningInvoiceExternalPubkeyResponse {
  operationId: string;
  invoice: string;
}

interface LightningInvoiceExternalPubkeyTweakedRequest {
  amountMsat: number;
  description: string;
  externalPubkey: string;
  tweak: number;
  expiryTime?: number;
}

interface LightningInvoiceExternalPubkeyTweakedResponse {
  operationId: string;
  invoice: string;
}

interface LightningClaimPubkeyReceiveRequest {
  privateKey: string;
}

interface LightningClaimPubkeyReceiveTweakedRequest {
  privateKey: string;
  tweaks: number[];
}

interface LightningAwaitInvoiceRequest {
  operationId: string;
}

interface LightningPayRequest {
  paymentInfo: string;
  amountMsat?: number;
  LightningurlComment?: string;
}

interface LightningPayResponse {
  operationId: string;
  paymentType: string;
  contractId: string;
  fee: number;
}

interface LightningAwaitPayRequest {
  operationId: string;
}

interface GatewayInfo {
  api: string;
  fees: GatewayFees;
  gateway_id: string;
  gateway_redeem_key: string;
  lightning_alias: string;
  mint_channel_id: number;
  node_pub_key: string;
  route_hints: any[]; // Adjust the type according to the actual structure of route hints
  supports_private_payments: boolean;
}

interface GatewayFees {
  baseMsat: number;
  proportionalMillionths: number;
}

interface GatewayTTL {
  nanos: number;
  secs: number;
}

interface Gateway {
  federation_id: string;
  info: GatewayInfo;
  ttl: GatewayTTL;
  vetted: boolean;
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

interface Note {
  signature: string;
  spend_key: string;
}

interface NotesJson {
  federation_id_prefix: string;
  notes: {
    [denomination: string]: Note[];
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
  JoinResponse,
  BackupRequest,
  ListOperationsRequest,
  OperationOutput,
  OnchainDepositAddressRequest,
  OnchainDepositAddressResponse,
  OnchainAwaitDepositRequest,
  OnchainAwaitDepositResponse,
  OnchainWithdrawRequest,
  OnchainWithdrawResponse,
  LightningInvoiceRequest,
  LightningInvoiceResponse,
  LightningInvoiceExternalPubkeyRequest,
  LightningInvoiceExternalPubkeyResponse,
  LightningInvoiceExternalPubkeyTweakedRequest,
  LightningInvoiceExternalPubkeyTweakedResponse,
  LightningClaimPubkeyReceiveRequest,
  LightningClaimPubkeyReceiveTweakedRequest,
  LightningAwaitInvoiceRequest,
  LightningPayRequest,
  LightningPayResponse,
  LightningAwaitPayRequest,
  Gateway,
  NotesJson,
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
