<img src="assets/federated-cashu.jpg" width="500">

# fedimint-http: A Fedimint HTTP Client (and Cashu Proxy)

fedimint-http exposes a REST API to interact with the Fedimint client.

Set the variables in `.env` by copying `example.env`, then run `cargo run`. It's also set up as a clap app so you can start the server with command line args as well.

## Fedimint Client Endpoints

The Fedimint client supports the following endpoints (and has naive websocket support at `/fedimint/v2/ws`, see code for details until I improve the interface. PRs welcome!)

### Admin related commands:

- `/fedimint/v2/admin/info`: Display wallet info (holdings, tiers).
- `/fedimint/v2/admin/backup`: Upload the (encrypted) snapshot of mint notes to federation.
- `/fedimint/v2/admin/discover-version`: Discover the common api version to use to communicate with the federation.
- `/fedimint/v2/admin/restore`: Restore the previously created backup of mint notes (with `backup` command).
- `/fedimint/v2/admin/list-operations`: List operations.
- `/fedimint/v2/admin/module`: Call a module subcommand.
- `/fedimint/v2/admin/config`: Returns the client config.

### Mint related commands:

- `/fedimint/v2/mint/reissue`: Reissue notes received from a third party to avoid double spends.
- `/fedimint/v2/mint/spend`: Prepare notes to send to a third party as a payment.
- `/fedimint/v2/mint/validate`: Verifies the signatures of e-cash notes, but *not* if they have been spent already.
- `/fedimint/v2/mint/split`: Splits a string containing multiple e-cash notes (e.g. from the `spend` command) into ones that contain exactly one.
- `/fedimint/v2/mint/combine`: Combines two or more serialized e-cash notes strings.

### Lightning network related commands:

- `/fedimint/v2/ln/invoice`: Create a lightning invoice to receive payment via gateway.
- `/fedimint/v2/ln/await-invoice`: Wait for incoming invoice to be paid.
- `/fedimint/v2/ln/pay`: Pay a lightning invoice or lnurl via a gateway.
- `/fedimint/v2/ln/await-pay`: Wait for a lightning payment to complete.
- `/fedimint/v2/ln/list-gateways`: List registered gateways.
- `/fedimint/v2/ln/switch-gateway`: Switch active gateway.

### Onchain related commands:

- `/fedimint/v2/onchain/deposit-address`: Generate a new deposit address, funds sent to it can later be claimed.
- `/fedimint/v2/onchain/await-deposit`: Wait for deposit on previously generated address.
- `/fedimint/v2/onchain/withdraw`: Withdraw funds from the federation.

### Extra endpoints:

- `/health`: health check endpoint.
- `/metrics`: exports API metrics using opentelemetry with prometheus exporter (num requests, latency, high-level metrics only)


Soon(TM): maps [Cashu NUT](https://github.com/cashubtc/nuts) endpoints to fedimint client.

# Supported Cashu NUTs: (Notation, Utilization, and Terminology)

- [ ] NUT-00: Notation, Utilization, and Terminology
  - Fedimint ecash does not currently encode the federation endpoint as part of the ecash, just the federation id. Fedimint encourages longer running relationships based off its trust model so doesnt currently support on the fly issuance / reissuance. Can coerce a mapping but doesnt exactly match. returns a federation id instead
- [ ] NUT-01: Mint public key exchange
  - [ ] `/v1/keys`: supportable
  - [ ] `/v1/keys/{keyset-id}`: supportable (fedimint only maintains 1 keyset)
  - Fedimint does not currently rotate keysets. Responds with single keyset mapping in Cashu format.
- [ ] NUT-02: Keysets and keyset ID
  - [ ] `/v1/keysets`: supportable
- [ ] NUT-03: Swap tokens
  - [ ] `/v1/swap`: supportable
  - Equivalent to Fedimint Reissue. Proofs are slightly different but functionally equivalent.
- [ ] NUT-04: Mint tokens
  - [ ] `/v1/mint/quote/{method}`: supportable
      - [ ] method=bolt11: supportable via lngateway
      - [ ] method=onchain: supportable via pegin
  - [ ] `/v1/mint/quote/{method}/{quote_id}`: supportable
  - [ ] `/v1/mint/{method}`: supportable
    - Fedimint client handles these a little differently but can probably coerce the flow, dont get why it requires the 2nd round after status is completed, should just return the notes there.
- [ ] NUT-05: Melting tokens
  - [ ] `/v1/melt/quote/{method}`: supportable
      - [ ] method=bolt11: supportable via lngateway
      - [ ] method=onchain: supportable via pegout
  - [ ] `/v1/melt/quote/{method}/{quote_id}`: supportable
- [ ] NUT-06: Mint information
  - [ ] `/v1/info`: supportable

