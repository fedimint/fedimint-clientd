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

interface BackupRequest {
    metadata: { [key: string]: string };
}

interface ListOperationsRequest {
    limit: number;
}

interface FederationIdsResponse {
    federationIds: string[];
}

interface OperationOutput {
    id: string;
    creationTime: string;
    operationKind: string;
    operationMeta: any;
    outcome?: any;
}

export type {
    Tiered,
    TieredSummary,
    InfoResponse,
    BackupRequest,
    ListOperationsRequest,
    FederationIdsResponse,
    OperationOutput,
}
