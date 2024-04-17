export interface Tiered<T> {
  [amount: number]: T;
}

export interface TieredSummary {
  tiered: Tiered<number>;
}

export interface InfoResponse {
  [federationId: string]: {
    network: string;
    meta: { [key: string]: string };
    totalAmountMsat: number;
    totalNumNotes: number;
    denominationsMsat: TieredSummary;
  };
}

export interface FederationIdsResponse {
  federationIds: string[];
}

export interface DiscoverVersionRequest {
  threshold?: number;
}

export interface DiscoverVersionResponse {
  [federationId: string]: any;
}

export interface JoinRequest {
  inviteCode: string;
  useManualSecret: boolean;
}

export interface JoinResponse {
  thisFederationId: string;
  federationIds: string[];
}

export interface BackupRequest {
  metadata: { [key: string]: string };
}

export interface ListOperationsRequest {
  limit: number;
}

export interface OperationOutput {
  id: string;
  creationTime: string;
  operationKind: string;
  operationMeta: any;
  outcome?: any;
}

export interface OnchainDepositAddressRequest {
  timeout: number;
}

export interface OnchainDepositAddressResponse {
  operationId: string;
  address: string;
}

export interface OnchainAwaitDepositRequest {
  operationId: string;
}

export interface BTCInput {
  previous_output: string;
  script_sig: string;
  sequence: number;
  witness: string[];
}

export interface BTCOutput {
  value: number;
  script_pubkey: string;
}

export interface BTCTransaction {
  version: number;
  lock_time: number;
  input: BTCInput[];
  output: BTCOutput[];
}

export interface AwaitDepositResponseConfirmed {
  btc_transaction: BTCTransaction;
  out_idx: number;
}

export interface OnchainAwaitDepositResponse {
  status: { Confirmed: AwaitDepositResponseConfirmed } | { Failed: string };
}

export interface OnchainWithdrawRequest {
  address: string;
  amountSat: number | "all";
}

export interface OnchainWithdrawResponse {
  txid: string;
  feesSat: number;
}

export interface LightningInvoiceRequest {
  amountMsat: number;
  description: string;
  expiryTime?: number;
}

export interface LightningInvoiceResponse {
  operationId: string;
  invoice: string;
}

export interface LightningInvoiceExternalPubkeyRequest {
  amountMsat: number;
  description: string;
  externalPubkey: string;
  expiryTime?: number;
}

export interface LightningInvoiceExternalPubkeyResponse {
  operationId: string;
  invoice: string;
}

export interface LightningInvoiceExternalPubkeyTweakedRequest {
  amountMsat: number;
  description: string;
  externalPubkey: string;
  tweak: number;
  expiryTime?: number;
}

export interface LightningInvoiceExternalPubkeyTweakedResponse {
  operationId: string;
  invoice: string;
}

export interface LightningClaimPubkeyReceiveRequest {
  privateKey: string;
}

export interface LightningClaimPubkeyReceiveTweakedRequest {
  privateKey: string;
  tweaks: number[];
}

export interface LightningAwaitInvoiceRequest {
  operationId: string;
}

export interface LightningPayRequest {
  paymentInfo: string;
  amountMsat?: number;
  LightningurlComment?: string;
}

export interface LightningPayResponse {
  operationId: string;
  paymentType: string;
  contractId: string;
  fee: number;
}

export interface LightningAwaitPayRequest {
  operationId: string;
}

export interface GatewayInfo {
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

export interface GatewayFees {
  baseMsat: number;
  proportionalMillionths: number;
}

export interface GatewayTTL {
  nanos: number;
  secs: number;
}

export interface Gateway {
  federation_id: string;
  info: GatewayInfo;
  ttl: GatewayTTL;
  vetted: boolean;
}

export interface MintDecodeNotesRequest {
  notes: string;
}

export interface MintDecodeNotesResponse {
  notesJson: NotesJson;
}

export interface MintEncodeNotesRequest {
  notesJsonStr: string;
}

export interface MintEncodeNotesResponse {
  notes: string;
}

export interface MintReissueRequest {
  notes: string;
}

export interface MintReissueResponse {
  amountMsat: number;
}

export interface MintSpendRequest {
  amountMsat: number;
  allowOverpay: boolean;
  timeout: number;
  includeInvite: boolean;
}

export interface MintSpendResponse {
  operation: string;
  notes: string;
}

export interface MintValidateRequest {
  notes: string;
}

export interface MintValidateResponse {
  amountMsat: number;
}

export interface MintSplitRequest {
  notes: string;
}

export interface MintSplitResponse {
  notes: Record<number, string>;
}

export interface MintCombineRequest {
  notesVec: string[];
}

export interface MintCombineResponse {
  notes: string;
}

export interface Note {
  signature: string;
  spend_key: string;
}

export interface NotesJson {
  federation_id_prefix: string;
  notes: {
    [denomination: string]: Note[];
  };
}
