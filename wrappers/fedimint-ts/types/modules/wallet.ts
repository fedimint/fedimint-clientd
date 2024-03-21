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

interface AwaitDepositResponse {
    status: string;
}

interface WithdrawRequest {
    address: string;
    amountMsat: number | 'all';
}

interface WithdrawResponse {
    txid: string;
    feesSat: number;
}

export type {
    DepositAddressRequest,
    DepositAddressResponse,
    AwaitDepositRequest,
    AwaitDepositResponse,
    WithdrawRequest,
    WithdrawResponse,
}
