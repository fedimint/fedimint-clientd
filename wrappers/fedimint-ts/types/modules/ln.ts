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

interface ListGatewaysResponse {
    [federationId: string]: Gateway[];
}

interface SwitchGatewayRequest {
    gatewayId: string;
}

export type {
    LnInvoiceRequest,
    LnInvoiceResponse,
    AwaitInvoiceRequest,
    LnPayRequest,
    LnPayResponse,
    AwaitLnPayRequest,
    Gateway,
    ListGatewaysResponse,
    SwitchGatewayRequest,
}
