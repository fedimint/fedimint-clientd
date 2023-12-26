<img src="assets/fedimint-http.png" width="500">

# fedimint-http: A Fedimint Client Server

fedimint-http exposes a REST API to interact with the Fedimint client.

Soon(TM): maps [Cashu NUT](https://github.com/cashubts/nuts) endpoints to fedimint client.

Supported NUTs:

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

