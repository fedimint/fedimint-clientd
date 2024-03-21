type FederationIdPrefix = string;

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
