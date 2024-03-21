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

interface DepositAddressRequest {
  timeout: number;
}

interface DepositAddressResponse {
  operationId: string;
  address: string;
}

interface AwaitDepositRequest {
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

interface AwaitDepositResponse {
  status: { Confirmed: AwaitDepositResponseConfirmed } | { Failed: string };
}

interface WithdrawRequest {
  address: string;
  amountSat: number | "all";
}

interface WithdrawResponse {
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

interface AwaitInvoiceRequest {
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

interface AwaitLnPayRequest {
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

interface ReissueRequest {
  notes: string;
}

interface ReissueResponse {
  amountMsat: number;
}

interface SpendRequest {
  amountMsat: number;
  allowOverpay: boolean;
  timeout: number;
  includeInvite: boolean;
}

interface SpendResponse {
  operation: string;
  notes: string;
}

interface ValidateRequest {
  notes: string;
}

interface ValidateResponse {
  amountMsat: number;
}

interface SplitRequest {
  notes: string;
}

interface SplitResponse {
  notes: Record<number, string>;
}

interface CombineRequest {
  notesVec: string[];
}

interface CombineResponse {
  notes: string;
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
  DepositAddressRequest,
  DepositAddressResponse,
  AwaitDepositRequest,
  AwaitDepositResponse,
  WithdrawRequest,
  WithdrawResponse,
  LnInvoiceRequest,
  LnInvoiceResponse,
  AwaitInvoiceRequest,
  LnPayRequest,
  LnPayResponse,
  AwaitLnPayRequest,
  Gateway,
  SwitchGatewayRequest,
  ReissueRequest,
  ReissueResponse,
  SpendRequest,
  SpendResponse,
  ValidateRequest,
  ValidateResponse,
  SplitRequest,
  SplitResponse,
  CombineRequest,
  CombineResponse,
};
