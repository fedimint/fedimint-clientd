import {
  // BackupRequest,
  InfoResponse,
  ListOperationsRequest,
  FederationIdsResponse,
  OperationOutput,
  DiscoverVersionRequest,
  DiscoverVersionResponse,
  JoinRequest,
  LightningInvoiceExternalPubkeyTweakedRequest,
  LightningClaimPubkeyReceiveTweakedRequest as LightningClaimPubkeyTweakReceivesRequest,
  LightningAwaitInvoiceRequest,
  Gateway,
  LightningInvoiceRequest,
  LightningPayRequest,
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
  JoinResponse,
  LightningInvoiceResponse,
  LightningInvoiceExternalPubkeyTweakedResponse,
  LightningPayResponse,
  LightningPaymentResponse,
} from "./types";

/**
 * Builder pattern for creating a FedimintClient.
 * @param baseUrl - The base URL of the Fedimint API
 * @param password - The password of the Fedimint client
 * @param activeFederationId - The ID of the active federation
 */
export class FedimintClientBuilder {
  private baseUrl: string;
  private password: string;
  private activeFederationId: string;
  private activeGatewayId: string;

  constructor() {
    this.baseUrl = "";
    this.password = "";
    this.activeFederationId = "";
    this.activeGatewayId = "";
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

  setActiveGatewayId(gatewayId: string): FedimintClientBuilder {
    this.activeGatewayId = gatewayId;

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
      this.activeFederationId,
      this.activeGatewayId
    );

    return client;
  }
}

/**
 * FedimintClient provides methods for interacting with a fedimint-clientd's admin, mint, lightning, and onchain methods over HTTP.
 * @param baseUrl - The base URL of the Fedimint Clientd instance, must be running and accessible e.g. http://localhost:3333
 * @param password - The password of the Fedimint client, becomes the bearer token
 * @param activeFederationId - The ID of the active federation to use for module methods
 * @param activeGatewayId - Optional, the ID of the active gateway, if not provided, the first gateway in the active federation will be used
 */
export class FedimintClient {
  private baseUrl: string;
  private password: string;
  private activeFederationId: string;
  private activeGatewayId: string;

  constructor(
    baseUrl: string,
    password: string,
    activeFederationId: string,
    activeGatewayId: string = ""
  ) {
    this.baseUrl = baseUrl + "/v2";
    this.password = password;
    this.activeFederationId = activeFederationId;
    this.activeGatewayId = activeGatewayId;
    console.log(
      "Fedimint Client initialized, must set activeGatewayId after intitalization to use lightning module methods or manually pass in gateways"
    );
  }

  getActiveFederationId(): string {
    return this.activeFederationId;
  }

  /**
   * Set the active federation ID to use for module methods.
   * If useDefaultGateway is true, the first gateway in the active federation will be used.
   * Otherwise, the activeGatewayId must be set by picking a gateway from the list of gateways in the active federation.
   * @param federationId - The ID of the active federation to use for module methods
   * @param useDefaultGateway - Whether to just use the first gateway in the active federation or set the active gateway id manually
   */
  setActiveFederationId(federationId: string, useDefaultGateway: boolean) {
    this.activeFederationId = federationId;
    console.log("Changed active federation id to: ", federationId);

    if (useDefaultGateway) {
      this.lightning.listGateways().then((gateways) => {
        this.activeGatewayId = gateways[0].info.gateway_id;
      });
    } else {
      console.log(
        "Clearing active gateway id, must be set manually on lightning calls or setDefaultGatewayId to true"
      );
      this.activeGatewayId = "";
    }
  }

  getActiveGatewayId(): string {
    return this.activeGatewayId;
  }

  setActiveGatewayId(gatewayId: string) {
    this.activeGatewayId = gatewayId;
  }

  /**
   * Use the first gateway in the active federation as the default gateway for lightning module methods.
   */
  async useDefaultGateway() {
    // hits list_gateways and sets activeGatewayId to the first gateway
    try {
      const gateways = await this.lightning.listGateways();
      console.log("Gateways: ", gateways);
      this.activeGatewayId = gateways[0].info.gateway_id;
      console.log("Set active gateway id to: ", this.activeGatewayId);
    } catch (error) {
      console.error("Error setting default gateway id: ", error);
    }
  }

  /**
   * Makes a GET request to the `baseURL` at the given `endpoint`.
   * Receives a JSON response.
   * Automatically ensures a default federation ID is set if needed.
   * @param endpoint - The endpoint to make the request to.
   */
  private async get<T>(endpoint: string): Promise<T> {
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
  private async post<T>(endpoint: string, body: any): Promise<T> {
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

  /**
   * Makes a POST request to the `baseURL` at the given `endpoint` with the provided `body`.
   * Receives a JSON response.
   * Adds a federation ID to the request if provided or uses the active federation ID.
   * Used for module methods that require a federation ID.
   * @param endpoint - The endpoint to make the request to.
   * @param body - The body of the request.
   * @param federationId - The ID of the federation to use for the request.
   */
  private async postWithFederationId<T>(
    endpoint: string,
    body: any,
    federationId?: string
  ): Promise<T> {
    const effectiveFederationId = federationId || this.activeFederationId;

    return this.post<T>(endpoint, {
      ...body,
      federationId: effectiveFederationId,
    });
  }

  /**
   * Makes a POST request to the `baseURL` at the given `endpoint` with the provided `body`.
   * Receives a JSON response.
   * Adds a gateway ID and federation ID to the request if provided or uses the active gateway ID and active federation ID.
   * Used for lightning module methods that require a gateway ID for a specific federation.
   * @param endpoint - The endpoint to make the request to.
   * @param body - The body of the request.
   * @param gatewayId - The ID of the gateway to use for the request.
   * @param federationId - The ID of the federation to use for the request.
   */
  private async postWithGatewayIdAndFederationId<T>(
    endpoint: string,
    body: any,
    gatewayId?: string,
    federationId?: string
  ): Promise<T> {
    try {
      const effectiveGatewayId = gatewayId || this.activeGatewayId;
      const effectiveFederationId = federationId || this.activeFederationId;

      if (effectiveFederationId === "" || effectiveGatewayId === "") {
        throw new Error(
          "Must set active federation and gateway id before posting with them"
        );
      }

      return this.post<T>(endpoint, {
        ...body,
        federationId: effectiveFederationId,
        gatewayId: effectiveGatewayId,
      });
    } catch (error) {
      throw error;
    }
  }

  /**
   * Fetches wallet information including holdings, tiers, and federation metadata.
   */
  public async info(): Promise<InfoResponse> {
    return await this.get<InfoResponse>("/admin/info");
  }

  /**
   * Returns the client configurations by federationId
   */
  public async config(): Promise<any> {
    return await this.get<any>("/admin/config");
  }

  // --- TODO: UNSUPPORTED METHOD---
  //
  // /**
  //  * Uploads an encrypted snapshot of mint notes to the federation
  //  * @param metadata - The metadata to include in the snapshot
  //  * @param federationId - The ID of the federation to upload the snapshot to
  //  */
  // public async backup(
  //   metadata: BackupRequest,
  //   federationId?: string
  // ): Promise<void> {
  //   await this.postWithFederationId<void>(
  //     "/admin/backup",
  //     metadata,
  //     federationId
  //   );
  // }

  /**
   * Returns the common API version to use to communicate with the federation and modules
   */
  public async discoverVersion(
    threshold?: number
  ): Promise<DiscoverVersionResponse> {
    const request: DiscoverVersionRequest = threshold ? { threshold } : {};

    return this.post<DiscoverVersionResponse>(
      "/admin/discover-version",
      request
    );
  }

  /**
   * Outputs a list of the most recent operations performed by this client on the federation
   * @param limit - The maximum number of operations to return
   * @param federationId - The ID of the federation to list the operations for
   */
  public async listOperations(
    limit: number,
    federationId?: string
  ): Promise<OperationOutput[]> {
    const request: ListOperationsRequest = { limit };

    return await this.postWithFederationId<OperationOutput[]>(
      "/admin/list-operations",
      request,
      federationId
    );
  }

  /**
   * Returns the current set of connected federation IDs
   */
  public async federationIds(): Promise<FederationIdsResponse> {
    return await this.get<FederationIdsResponse>("/admin/federation-ids");
  }

  /**
   * Joins a federation with an inviteCode
   * Returns an array of federation IDs that the client is now connected to
   * If already connected to the federation will just return 200
   * Returns thisFederationId and the list of connected federation IDs in the response body
   * @param inviteCode - The invite code to join the federation with
   * @param setActiveFederationId - Whether to set the active federation ID to the one joined
   * @param useDefaultGateway - Whether to use the first gateway in the active federation or set the active gateway id manually
   * @param useManualSecret - Whether to use the manual secret to join the federation
   */
  public async join(
    inviteCode: string,
    setActiveFederationId: boolean,
    useDefaultGateway: boolean,
    useManualSecret: boolean = false
  ): Promise<JoinResponse> {
    const request: JoinRequest = { inviteCode, useManualSecret };

    const response = await this.post<JoinResponse>("/admin/join", request);

    if (setActiveFederationId) {
      this.setActiveFederationId(response.thisFederationId, useDefaultGateway);
    }

    return response;
  }

  /**
   * A Module for interacting with Lightning
   */
  public lightning = {
    /**
     * Creates a lightning invoice to receive payment via gateway
     */
    createInvoice: async (
      request: LightningInvoiceRequest,
      gatewayId?: string,
      federationId?: string
    ): Promise<LightningInvoiceResponse> => {
      return await this.postWithGatewayIdAndFederationId<LightningInvoiceResponse>(
        "/ln/invoice",
        request,
        gatewayId,
        federationId
      );
    },

    /**
     * Creates a lightning invoice where the gateway contract locks the ecash to a tweakedpubkey
     * Fedimint-clientd tweaks the provided pubkey by the provided tweak to create the lightning contract
     * Useful for creating invoices that pay to another user besides yourself
     * @param pubkey - The pubkey to tweak
     * @param tweak - The tweak to apply to the pubkey
     * @param amountMsat - The amount of ecash to pay
     * @param description - The description of the invoice
     * @param expiryTime - The expiry time of the invoice in seconds
     * @param gatewayId - The ID of the gateway to use for the invoice, if not provided will use the first gateway in the active federation
     * @param federationId - The ID of the federation to use for the invoice, if not provided will use the active federation ID
     */
    createInvoiceForPubkeyTweak: async (
      request: LightningInvoiceExternalPubkeyTweakedRequest,
      gatewayId?: string,
      federationId?: string
    ): Promise<LightningInvoiceResponse> => {
      return await this.postWithGatewayIdAndFederationId<LightningInvoiceExternalPubkeyTweakedResponse>(
        "/ln/invoice-external-pubkey-tweaked",
        request,
        gatewayId,
        federationId
      );
    },

    /**
     * Claims lightning contracts paid to tweaks of a pubkey
     * Provide all the tweaks that were used to create the invoices
     * @param privateKey - The private key of the pubkey to claim
     * @param tweaks - The tweaks of the pubkey to claim
     * @param federationId - The ID of the federation to claim the contracts in. Contracts are on specific federations so this is required.
     */
    claimPubkeyTweakReceives: async (
      request: LightningClaimPubkeyTweakReceivesRequest,
      federationId: string
    ): Promise<LightningPaymentResponse> => {
      return await this.postWithFederationId<LightningPaymentResponse>(
        "/ln/claim-external-receive-tweaked",
        request,
        federationId
      );
    },

    /**
     * Blocking call that waits for a lightning invoice to be paid
     * @param operationId - The contract id of the lightning invoice to wait for, normally the payment hash of the invoice
     * @param federationId - The ID of the federation to wait for the invoice in, if not provided will use the active federation ID
     */
    awaitInvoice: async (
      operationId: string,
      federationId?: string
    ): Promise<LightningPaymentResponse> => {
      const request: LightningAwaitInvoiceRequest = { operationId };

      return await this.postWithFederationId<LightningPaymentResponse>(
        "/ln/await-invoice",
        request,
        federationId
      );
    },

    /**
     * Pays a lightning invoice or Lightningurl via a gateway
     */
    pay: async (
      request: LightningPayRequest,
      gatewayId?: string,
      federationId?: string
    ): Promise<LightningPayResponse> => {
      return await this.postWithGatewayIdAndFederationId<LightningPayResponse>(
        "/ln/pay",
        request,
        gatewayId,
        federationId
      );
    },

    /**
     * Outputs a list of registered lighting lightning gateways
     */
    listGateways: async (): Promise<Gateway[]> =>
      await this.postWithFederationId<Gateway[]>("/ln/list-gateways", {}),
  };

  /**
   * A module for interacting with the ecash mint
   */
  public mint = {
    /**
     * Decodes hex encoded binary ecash notes to json
     */
    decodeNotes: async (notes: string): Promise<MintDecodeNotesResponse> => {
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
    ): Promise<MintEncodeNotesResponse> => {
      const request: MintEncodeNotesRequest = {
        notesJsonStr: JSON.stringify(notesJson),
      };

      return await this.post<MintEncodeNotesResponse>(
        "/mint/encode-notes",
        request
      );
    },

    /**
     * Reissues an ecash note. This is how the client receives notes.
     * @param notes - The notes to reissue
     * @param federationId - The ID of the federation to reissue the notes in, if not provided will use the active federation ID
     */
    reissue: async (
      notes: string,
      federationId?: string
    ): Promise<MintReissueResponse> => {
      const request: MintReissueRequest = { notes };

      return await this.postWithFederationId<MintReissueResponse>(
        "/mint/reissue",
        request,
        federationId
      );
    },

    /**
     * Pulls an ecash note string from the client's wallet and spends it.
     * Once spent, the note is removed from the client's wallet.
     * If a timeout is provided, the wallet will attempt to reissue the note after the timeout in case it wasn't spent.
     * @param amountMsat - The amount of ecash to spend
     * @param allowOverpay - Whether to allow overpaying the amountMsat
     * @param timeout - The number of seconds to wait for the note to be spent, wallet's default timeout is 1 week if not provided
     * @param includeInvite - Whether to include the invite in the note
     * @param federationId - The ID of the federation to spend the note in, if not provided will use the active federation ID
     */
    spend: async (
      request: MintSpendRequest,
      federationId?: string
    ): Promise<MintSpendResponse> => {
      return await this.postWithFederationId<MintSpendResponse>(
        "/mint/spend",
        request,
        federationId
      );
    },

    /**
     * Validates an ecash note string.
     */
    validate: async (
      notes: string,
      federationId?: string
    ): Promise<MintValidateResponse> => {
      const request: MintValidateRequest = { notes };

      return await this.postWithFederationId<MintValidateResponse>(
        "/mint/validate",
        request,
        federationId
      );
    },

    /**
     * Splits an ecash note string into its individual notes.
     * @param notes - The notes to split
     */
    split: async (notes: string): Promise<MintSplitResponse> => {
      const request: MintSplitRequest = { notes };

      return await this.post<MintSplitResponse>("/mint/split", request);
    },

    /**
     * Combines ecash notes into a single note string.
     * @param notesVec - The notes to combine
     */
    combine: async (notesVec: string[]): Promise<MintCombineResponse> => {
      const request: MintCombineRequest = { notesVec };

      return await this.post<MintCombineResponse>("/mint/combine", request);
    },
  };

  /**
   * A module for onchain bitcoin operations.
   * Fedimint's onchain wallet has several restrictions:
   * - When you peg-in bitcoin, the federation will require additional confirmations (normally 6-10),
   *   so you should wait for those confirmations before calling awaitDeposit, which is a blocking call.
   * - Pegging out bitcoin will always be expensive because the Fedimint wallet always tries
   *   to get the transaction confirmed in the next block.
   * It is highly recommended to use the lightning module whenever possible
   * and to only use the onchain module infrequently for major transactions.
   */
  public onchain = {
    /**
     * Creates a new bitcoin deposit address to peg in bitcoin to the federation.
     * @param timeout - The number of seconds for the fedimint-clientd to watch for a deposit to the created address
     */
    createDepositAddress: async (
      timeout: number,
      federationId?: string
    ): Promise<OnchainDepositAddressResponse> => {
      const request: OnchainDepositAddressRequest = { timeout };

      return await this.postWithFederationId<OnchainDepositAddressResponse>(
        "/onchain/deposit-address",
        request,
        federationId
      );
    },

    /**
     * Waits for a peg-in bitcoin deposit to be confirmed by the federation
     * This is a blocking call, and the federation will require additional confirmations (normally 6-10), so you should wait for those confirmations before calling awaitDeposit.
     * @param operationId - The contract id of the deposit to wait for, normally the transaction hash of the deposit
     * @param federationId - The ID of the federation to wait for the deposit in, if not provided will use the active federation ID
     */
    awaitDeposit: async (
      operationId: string,
      federationId?: string
    ): Promise<OnchainAwaitDepositResponse> => {
      const request: OnchainAwaitDepositRequest = { operationId };

      return await this.postWithFederationId<OnchainAwaitDepositResponse>(
        "/onchain/await-deposit",
        request,
        federationId
      );
    },

    /**
     * Initiates a peg-out transaction to withdraw bitcoin onchain from the federation.
     * Peg outs will always be expensive because the Fedimint wallet always tries
     * to get the transaction confirmed in the next block. Improving the transaction efficiency and fees is on the roadmap.
     * For now, it is recommended to use the lightning module whenever possible
     * and to only use the onchain module infrequently for major transactions.
     * @param address - The address to withdraw to
     * @param amountSat - The amount of satoshis to withdraw
     * @param federationId - The ID of the federation to withdraw from, if not provided will use the active federation ID
     */
    withdraw: async (
      address: string,
      amountSat: number | "all",
      federationId?: string
    ): Promise<OnchainWithdrawResponse> => {
      const request: OnchainWithdrawRequest = { address, amountSat };

      return await this.postWithFederationId<OnchainWithdrawResponse>(
        "/onchain/withdraw",
        request,
        federationId
      );
    },
  };
}
