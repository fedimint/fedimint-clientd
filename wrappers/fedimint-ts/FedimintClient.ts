import type {
  BackupRequest,
  InfoResponse,
  ListOperationsRequest,
  FederationIdsResponse,
  OperationOutput,
  DiscoverVersionRequest,
  DiscoverVersionResponse,
  JoinRequest,
  LnInvoiceExternalPubkeyRequest,
  LnInvoiceExternalPubkeyResponse,
  LnInvoiceExternalPubkeyTweakedRequest,
  LnInvoiceExternalPubkeyTweakedResponse,
  LnClaimPubkeyReceiveRequest,
  LnClaimPubkeyReceiveTweakedRequest,
  LnAwaitInvoiceRequest,
  Gateway,
  LnInvoiceRequest,
  LnInvoiceResponse,
  LnPayRequest,
  LnPayResponse,
  MintCombineRequest,
  MintCombineResponse,
  MintReissueRequest,
  MintReissueResponse,
  MintSpendRequest,
  MintSpendResponse,
  MintSplitRequest,
  MintSplitResponse,
  MintValidateRequest,
  MintValidateResponse,
  OnchainAwaitDepositRequest,
  OnchainAwaitDepositResponse,
  OnchainDepositAddressRequest,
  OnchainDepositAddressResponse,
  OnchainWithdrawRequest,
  OnchainWithdrawResponse,
  MintDecodeNotesRequest,
  MintEncodeNotesRequest,
  NotesJson,
  MintEncodeNotesResponse,
  MintDecodeNotesResponse,
} from "./types";

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
    this.baseUrl = baseUrl + "/v2";
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
  public async discoverVersion(
    threshold?: number
  ): FedimintResponse<DiscoverVersionResponse> {
    const request: DiscoverVersionRequest = threshold ? { threshold } : {};
    console.log("request", request);

    return this.post<DiscoverVersionResponse>(
      "/admin/discover-version",
      request
    );
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
    useManualSecret: boolean = false
  ): FedimintResponse<FederationIdsResponse> {
    const request: JoinRequest = { inviteCode, useManualSecret };

    return await this.post<FederationIdsResponse>("/admin/join", request);
  }

  /**
   * Outputs a list of operations that have been performed on the federation
   */
  public async listOperations(
    limit: number,
    federationId?: string
  ): FedimintResponse<OperationOutput[]> {
    const request: ListOperationsRequest = { limit };

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
      amountMsat: number,
      description: string,
      expiryTime?: number,
      federationId?: string
    ): FedimintResponse<LnInvoiceResponse> => {
      const request: LnInvoiceRequest = { amountMsat, description, expiryTime };

      return await this.postWithId<LnInvoiceResponse>(
        "/ln/invoice",
        request,
        federationId
      );
    },

    /**
     * Creates a lightning invoice where the gateway contract locks the ecash to a specific pubkey
     * Useful for creating invoices that pay to another user besides yourself
     */
    createInvoiceForPubkey: async (
      pubkey: string,
      amountMsat: number,
      description: string,
      expiryTime?: number,
      federationId?: string
    ): FedimintResponse<LnInvoiceResponse> => {
      const request: LnInvoiceExternalPubkeyRequest = {
        externalPubkey: pubkey,
        amountMsat,
        description,
        expiryTime,
      };

      return await this.postWithId<LnInvoiceExternalPubkeyResponse>(
        "/ln/invoice-external-pubkey",
        request,
        federationId
      );
    },

    /**
     * Creates a lightning invoice where the gateway contract locks the ecash to a tweakedpubkey
     * Fedimint-clientd tweaks the provided pubkey by the provided tweak, provide the pubkey and tweak
     * Useful for creating invoices that pay to another user besides yourself
     */
    createInvoiceForPubkeyTweak: async (
      pubkey: string,
      tweak: number,
      amountMsat: number,
      description: string,
      expiryTime?: number,
      federationId?: string
    ): FedimintResponse<LnInvoiceResponse> => {
      const request: LnInvoiceExternalPubkeyTweakedRequest = {
        externalPubkey: pubkey,
        tweak,
        amountMsat,
        description,
        expiryTime,
      };

      return await this.postWithId<LnInvoiceExternalPubkeyTweakedResponse>(
        "/ln/invoice-external-pubkey-tweaked",
        request,
        federationId
      );
    },

    /**
     * Claims a lightning contract that was paid to a specific pubkey
     */
    claimPubkeyReceive: async (
      privateKey: string,
      federationId?: string
    ): FedimintResponse<InfoResponse> => {
      const request: LnClaimPubkeyReceiveRequest = { privateKey };

      return await this.postWithId<InfoResponse>(
        "/ln/claim-external-receive",
        request,
        federationId
      );
    },

    /**
     * Claims lightning contracts paid to tweaks of a pubkey
     * Provide all the tweaks that were used to create the invoices
     */
    claimPubkeyReceiveTweaked: async (
      privateKey: string,
      tweaks: number[],
      federationId?: string
    ): FedimintResponse<InfoResponse> => {
      const request: LnClaimPubkeyReceiveTweakedRequest = {
        privateKey,
        tweaks,
      };

      return await this.postWithId<InfoResponse>(
        "/ln/claim-external-receive-tweaked",
        request,
        federationId
      );
    },
    /**
     * Waits for a lightning invoice to be paid
     */
    awaitInvoice: async (
      operationId: string,
      federationId?: string
    ): FedimintResponse<InfoResponse> => {
      const request: LnAwaitInvoiceRequest = { operationId };

      return await this.postWithId<InfoResponse>(
        "/ln/await-invoice",
        request,
        federationId
      );
    },

    /**
     * Pays a lightning invoice or lnurl via a gateway
     */
    pay: async (
      paymentInfo: string,
      amountMsat?: number,
      lnurlComment?: string,
      federationId?: string
    ): FedimintResponse<LnPayResponse> => {
      const request: LnPayRequest = {
        paymentInfo,
        amountMsat,
        lnurlComment,
      };

      return await this.postWithId<LnPayResponse>(
        "/ln/pay",
        request,
        federationId
      );
    },

    /**
     * Outputs a list of registered lighting lightning gateways
     */
    listGateways: async (): FedimintResponse<Gateway[]> =>
      await this.postWithId<Gateway[]>("/ln/list-gateways", {}),
  };

  /**
   * A module for interacting with an ecash mint
   */
  public mint = {
    /**
     * Decodes hex encoded binary ecash notes to json
     */
    decodeNotes: async (
      notes: string
    ): FedimintResponse<MintDecodeNotesResponse> => {
      const request: MintDecodeNotesRequest = {
        notes,
      };

      return await this.post<MintDecodeNotesResponse>(
        "/mint/decode-notes",
        request
      );
    },

    /**
     * Encodes json notes to hex encoded binary notes
     */
    encodeNotes: async (
      notesJson: NotesJson
    ): FedimintResponse<MintEncodeNotesResponse> => {
      const request: MintEncodeNotesRequest = {
        notesJsonStr: JSON.stringify(notesJson),
      };
      console.log("request: ", request);

      return await this.post<MintEncodeNotesResponse>(
        "/mint/encode-notes",
        request
      );
    },

    /**
     * Reissues an ecash note
     */
    reissue: async (
      notes: string,
      federationId?: string
    ): FedimintResponse<MintReissueResponse> => {
      const request: MintReissueRequest = { notes };

      return await this.postWithId<MintReissueResponse>(
        "/mint/reissue",
        request,
        federationId
      );
    },

    /**
     * Spends an ecash note
     */
    spend: async (
      amountMsat: number,
      allowOverpay: boolean,
      timeout: number,
      includeInvite: boolean,
      federationId?: string
    ): FedimintResponse<MintSpendResponse> => {
      const request: MintSpendRequest = {
        amountMsat,
        allowOverpay,
        timeout,
        includeInvite,
      };

      return await this.postWithId<MintSpendResponse>(
        "/mint/spend",
        request,
        federationId
      );
    },

    /**
     * Validates an ecash note
     */
    validate: async (
      notes: string,
      federationId?: string
    ): FedimintResponse<MintValidateResponse> => {
      const request: MintValidateRequest = { notes };

      return await this.postWithId<MintValidateResponse>(
        "/mint/validate",
        request,
        federationId
      );
    },

    /**
     * Splits an ecash note into smaller notes
     */
    split: async (notes: string): FedimintResponse<MintSplitResponse> => {
      const request: MintSplitRequest = { notes };

      return await this.post<MintSplitResponse>("/mint/split", request);
    },

    /**
     * Combines ecash notes
     */
    combine: async (
      notesVec: string[]
    ): FedimintResponse<MintCombineResponse> => {
      const request: MintCombineRequest = { notesVec };

      return await this.post<MintCombineResponse>("/mint/combine", request);
    },
  };

  /**
   * A module for onchain bitcoin operations
   */
  public onchain = {
    /**
     * Creates a new bitcoin deposit address
     */
    createDepositAddress: async (
      timeout: number,
      federationId?: string
    ): FedimintResponse<OnchainDepositAddressResponse> => {
      const request: OnchainDepositAddressRequest = { timeout };

      return await this.postWithId<OnchainDepositAddressResponse>(
        "/wallet/deposit-address",
        request,
        federationId
      );
    },

    /**
     * Waits for a bitcoin deposit to be confirmed
     */
    awaitDeposit: async (
      operationId: string,
      federationId?: string
    ): FedimintResponse<OnchainAwaitDepositResponse> => {
      const request: OnchainAwaitDepositRequest = { operationId };

      return await this.postWithId<OnchainAwaitDepositResponse>(
        "/wallet/await-deposit",
        request,
        federationId
      );
    },

    /**
     * Withdraws bitcoin from the federation
     */
    withdraw: async (
      address: string,
      amountSat: number | "all",
      federationId?: string
    ): FedimintResponse<OnchainWithdrawResponse> => {
      const request: OnchainWithdrawRequest = { address, amountSat };

      return await this.postWithId<OnchainWithdrawResponse>(
        "/wallet/withdraw",
        request,
        federationId
      );
    },
  };
}

export { FedimintClientBuilder, FedimintClient };
