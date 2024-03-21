import type {
  BackupRequest,
  InfoResponse,
  ListOperationsRequest,
  FederationIdsResponse,
  OperationOutput,
} from "./types/common";
import type {
  AwaitInvoiceRequest,
  AwaitLnPayRequest,
  Gateway,
  LnInvoiceRequest,
  LnInvoiceResponse,
  LnPayRequest,
  LnPayResponse,
  SwitchGatewayRequest,
} from "./types/modules/ln";
import type {
  CombineRequest,
  CombineResponse,
  ReissueRequest,
  ReissueResponse,
  SpendRequest,
  SpendResponse,
  SplitRequest,
  SplitResponse,
  ValidateRequest,
  ValidateResponse,
} from "./types/modules/mint";
import type {
  AwaitDepositRequest,
  AwaitDepositResponse,
  DepositAddressRequest,
  DepositAddressResponse,
  WithdrawRequest,
  WithdrawResponse,
} from "./types/modules/wallet";

type FedimintResponse<T> = Promise<T>;

class FedimintClientBuilder {
  private baseUrl: string;
  private password: string;
  private activeFederationId: string;

  constructor() {
    this.baseUrl = "";
    this.password = "";
    this.activeFederationId = "";
  }

  setBaseUrl(baseUrl: string): FedimintClientBuilder {
    this.baseUrl = baseUrl;

    return this;
  }

  setPassword(password: string): FedimintClientBuilder {
    this.password = password;

    return this;
  }

  setActiveFederationId(federationId: string): FedimintClientBuilder {
    this.activeFederationId = federationId;

    return this;
  }

  build(): FedimintClient {
    if (
      this.baseUrl === "" ||
      this.password === "" ||
      this.activeFederationId === ""
    ) {
      throw new Error("baseUrl, password, and activeFederationId must be set");
    }

    const client = new FedimintClient(
      this.baseUrl,
      this.password,
      this.activeFederationId
    );

    return client;
  }
}

class FedimintClient {
  private baseUrl: string;
  private password: string;
  private activeFederationId: string;

  constructor(baseUrl: string, password: string, activeFederationId: string) {
    this.baseUrl = baseUrl + "/fedimint/v2";
    this.password = password;
    this.activeFederationId = activeFederationId;
  }

  getActiveFederationId(): string {
    return this.activeFederationId;
  }

  setActiveFederationId(federationId: string) {
    this.activeFederationId = federationId;
  }

  /**
   * Makes a GET request to the `baseURL` at the given `endpoint`.
   * Receives a JSON response.
   * Automatically ensures a default federation ID is set if needed.
   * @param endpoint - The endpoint to make the request to.
   */
  private async get<T>(endpoint: string): FedimintResponse<T> {
    const res = await fetch(`${this.baseUrl}${endpoint}`, {
      method: "GET",
      headers: { Authorization: `Bearer ${this.password}` },
    });

    if (!res.ok) {
      throw new Error(
        `GET request failed. Status: ${res.status}, Body: ${await res.text()}`
      );
    }

    return (await res.json()) as T;
  }

  /**
   * Makes a POST request to the `baseURL` at the given `endpoint` with the provided `body`.
   * Receives a JSON response.
   * Automatically ensures a default federation ID is set if needed.
   * @param endpoint - The endpoint to make the request to.
   * @param body - The body of the request.
   */
  private async post<T>(endpoint: string, body: any): FedimintResponse<T> {
    const res = await fetch(`${this.baseUrl}${endpoint}`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${this.password}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
    });

    if (!res.ok) {
      throw new Error(
        `POST request failed. Status: ${res.status}, Body: ${await res.text()}`
      );
    }

    return (await res.json()) as T;
  }

  // Adjust postWithId to not require federationId as a mandatory parameter
  // since ensureactiveFederationId will ensure it's set.
  private async postWithId<T>(
    endpoint: string,
    body: any,
    federationId?: string
  ): FedimintResponse<T> {
    // Note: No need to call ensureactiveFederationId here since post already does.
    const effectiveFederationId = federationId || this.activeFederationId;

    return this.post<T>(endpoint, {
      ...body,
      federationId: effectiveFederationId,
    });
  }

  /**
   * Uploads the encrypted snapshot of mint notest to the federation
   */
  public async backup(
    metadata: BackupRequest,
    federationId?: string
  ): FedimintResponse<void> {
    await this.postWithId<void>("/admin/backup", metadata, federationId);
  }

  /**
   * Returns the client configurations by federationId
   */
  public async config(): FedimintResponse<any> {
    return await this.get<any>("/admin/config");
  }

  /**
   * Returns the API version to use to communicate with the federation
   */
  public async discoverVersion(): FedimintResponse<string> {
    return this.get<string>("/admin/discover-version");
  }

  /**
   * Returns the current set of connected federation IDs
   */
  public async federationIds(): FedimintResponse<FederationIdsResponse> {
    return await this.get<FederationIdsResponse>("/admin/federation-ids");
  }

  /**
   * Fetches wallet information including holdings, tiers, and federation metadata.
   */
  public async info(): FedimintResponse<InfoResponse> {
    return await this.get<InfoResponse>("/admin/info");
  }

  /**
   * Joins a federation with an inviteCode
   * Returns an array of federation IDs that the client is now connected to
   */
  public async join(
    inviteCode: string,
    setDefault: boolean
  ): FedimintResponse<FederationIdsResponse> {
    return await this.post<FederationIdsResponse>("/admin/join", {
      inviteCode,
      setDefault,
    });
  }

  /**
   * Outputs a list of operations that have been performed on the federation
   */
  public async listOperations(
    request: ListOperationsRequest,
    federationId?: string
  ): FedimintResponse<OperationOutput[]> {
    return await this.postWithId<OperationOutput[]>(
      "/admin/list-operations",
      request,
      federationId
    );
  }

  /**
   * A Module for interacting with Lightning
   */
  public ln = {
    /**
     * Creates a lightning invoice to receive payment via gateway
     */
    createInvoice: async (
      request: LnInvoiceRequest,
      federationId?: string
    ): FedimintResponse<LnInvoiceResponse> =>
      await this.postWithId<LnInvoiceResponse>(
        "/ln/invoice",
        request,
        federationId
      ),

    /**
     * Waits for a lightning invoice to be paid
     */
    awaitInvoice: async (
      request: AwaitInvoiceRequest,
      federationId?: string
    ): FedimintResponse<InfoResponse> =>
      await this.postWithId<InfoResponse>(
        "/ln/await-invoice",
        request,
        federationId
      ),

    /**
     * Pays a lightning invoice or lnurl via a gateway
     */
    pay: async (
      request: LnPayRequest,
      federationId?: string
    ): FedimintResponse<LnPayResponse> =>
      await this.postWithId<LnPayResponse>("/ln/pay", request, federationId),

    /**
     * Waits for a lightning payment to complete
     */
    awaitPay: async (
      request: AwaitLnPayRequest,
      federationId?: string
    ): FedimintResponse<LnPayResponse> =>
      await this.postWithId<LnPayResponse>(
        "/ln/await-pay",
        request,
        federationId
      ),

    /**
     * Outputs a list of registered lighting lightning gateways
     */
    listGateways: async (): FedimintResponse<Gateway[]> =>
      await this.postWithId<Gateway[]>("/ln/list-gateways", {}),

    /**
     * Switches the active lightning gateway
     */
    switchGateway: async (
      request: SwitchGatewayRequest,
      federationId?: string
    ): FedimintResponse<Gateway> =>
      await this.postWithId<Gateway>(
        "/ln/switch-gateway",
        request,
        federationId
      ),
  };

  /**
   * A module for creating a bitcoin deposit address
   */
  public wallet = {
    /**
     * Creates a new bitcoin deposit address
     */
    createDepositAddress: async (
      request: DepositAddressRequest,
      federationId?: string
    ): FedimintResponse<DepositAddressResponse> =>
      await this.postWithId<DepositAddressResponse>(
        "/wallet/deposit-address",
        request,
        federationId
      ),

    /**
     * Waits for a bitcoin deposit to be confirmed
     */
    awaitDeposit: async (
      request: AwaitDepositRequest,
      federationId?: string
    ): FedimintResponse<AwaitDepositResponse> =>
      await this.postWithId<AwaitDepositResponse>(
        "/wallet/await-deposit",
        request,
        federationId
      ),

    /**
     * Withdraws bitcoin from the federation
     */
    withdraw: async (
      request: WithdrawRequest,
      federationId?: string
    ): FedimintResponse<WithdrawResponse> =>
      await this.postWithId<WithdrawResponse>(
        "/wallet/withdraw",
        request,
        federationId
      ),
  };

  /**
   * A module for interacting with an ecash mint
   */
  public mint = {
    /**
     * Reissues an ecash note
     */
    reissue: async (
      request: ReissueRequest,
      federationId?: string
    ): FedimintResponse<ReissueResponse> =>
      await this.postWithId<ReissueResponse>(
        "/mint/reissue",
        request,
        federationId
      ),

    /**
     * Spends an ecash note
     */
    spend: async (
      request: SpendRequest,
      federationId?: string
    ): FedimintResponse<SpendResponse> =>
      await this.postWithId<SpendResponse>(
        "/mint/spend",
        request,
        federationId
      ),

    /**
     * Validates an ecash note
     */
    validate: async (
      request: ValidateRequest,
      federationId?: string
    ): FedimintResponse<ValidateResponse> =>
      await this.postWithId<ValidateResponse>(
        "/mint/validate",
        request,
        federationId
      ),

    /**
     * Splits an ecash note
     */
    split: async (request: SplitRequest): FedimintResponse<SplitResponse> =>
      await this.post<SplitResponse>("/mint/split", request),

    /**
     * Combines ecash notes
     */
    combine: async (
      request: CombineRequest
    ): FedimintResponse<CombineResponse> =>
      await this.post<CombineResponse>("/mint/combine", request),
  };
}

export { FedimintClientBuilder, FedimintClient };
