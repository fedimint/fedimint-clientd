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
  amountMsat: number | "all";
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
  finishInBackground: boolean;
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

interface FederationIdPrefix {
  0: number; // Assuming u8 is equivalent to number in TypeScript
  1: number;
  2: number;
  3: number;
}

interface TieredMulti<T> {
  [amount: number]: T[]; // Assuming Amount is equivalent to number in TypeScript
}

interface Signature {
  0: G1Affine;
}

interface G1Affine {
  x: Fp;
  y: Fp;
  infinity: Choice;
}

interface Fp {
  0: number[]; // Assuming u64 is equivalent to number in TypeScript
}

interface Choice {
  0: number; // Assuming u8 is equivalent to number in TypeScript
}

interface KeyPair {
  0: number[]; // Assuming c_uchar is equivalent to number in TypeScript
}

interface OOBNotesData {
  Notes?: TieredMulti<SpendableNote>;
  FederationIdPrefix?: FederationIdPrefix;
  Default?: {
    variant: number; // Assuming u64 is equivalent to number in TypeScript
    bytes: number[]; // Assuming Vec<u8> is equivalent to number[] in TypeScript
  };
}

interface OOBNotes {
  0: OOBNotesData[];
}

interface SpendableNote {
  signature: Signature;
  spendKey: KeyPair;
}

interface ReissueRequest {
  notes: OOBNotes;
}

interface ReissueResponse {
  amountMsat: number;
}

interface SpendRequest {
  amountMsat: number;
  allowOverpay: boolean;
  timeout: number;
}

interface SpendResponse {
  operation: string;
  notes: OOBNotes;
}

interface ValidateRequest {
  notes: OOBNotes;
}

interface ValidateResponse {
  amountMsat: number;
}

interface SplitRequest {
  notes: OOBNotes;
}

interface SplitResponse {
  notes: Record<number, OOBNotes>;
}

interface CombineRequest {
  notes: OOBNotes[];
}

interface CombineResponse {
  notes: OOBNotes;
}

export type {
  Tiered,
  TieredSummary,
  InfoResponse,
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
