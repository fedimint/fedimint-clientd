# Fedimint SDK in Typescript

This is a TypeScript client that consumes the [Fedimint Http Client](https://github.com/kodylow/fedimint-http), communicating with it via HTTP and a password. It's a hacky prototype, but it works until we can get a proper TS client for Fedimint. All of the federation handling code happens in the fedimint-http-client, this just exposes a simple API for interacting with the client from TypeScript (will be mirrored in Python and Go).

## Usage

```typescript
import dotenv from "dotenv";
import { FedimintClientBuilder } from "fedimint-ts";

dotenv.config();

const baseUrl = process.env.BASE_URL || "http://localhost:3000";
const password = process.env.PASSWORD || "password";
const builder = new FedimintClientBuilder();
builder.setBaseUrl(baseUrl).setPassword(password);

// If you pass in an invite code, it will be set as the default federation
if (process.env.INVITE_CODE) {
  builder.setInviteCode(process.env.INVITE_CODE);
}

// The FedimintClient has a default federationId set that it'll call any module methods on
const fedimintClient = await builder.build();

// You can update the federationId to call methods on a different federation
const { federation_ids } = await fedimintClient.federationIds();
await fedimintClient.setDefaultFederationId(federation_ids[0]);

// Any methods that call on a specific federation can optionally take a federationId as the last argument
// If no federationId is passed, the default federationId is used
const _ = await fedimintClient.listOperations({ limit: 10 }, federation_ids[1]);

// Admin methods give summaries by federation
fedimintClient.info().then((response) => {
  console.log("Current Total Msats Ecash: ", response.total_amount_msat);
});

// All module methods are called on the default federationId if you don't pass in a federationId
const { operation_id, invoice } = await fedimintClient.ln.createInvoice({
  amount_msat: 10000,
  description: "test",
});

console.log("Created 10 sat Invoice: ", invoice);

console.log("Waiting for payment...");

fedimintClient.ln.awaitInvoice({ operation_id }).then((response) => {
  console.log("Payment Received!");
  console.log("New Total Msats Ecash: ", response.total_amount_msat);
});
```

## Development

Install dependencies with `bun install` or `npm install`.

Follow these [steps to setup a `fedimint-clientd` server](https://github.com/fedimint/fedimint-clientd?tab=readme-ov-file#getting-started) and then you're ready to run the typescript client.
 `fedimint-ts` will use the same base url and password as the `fedimint-clientd`:

```bash
BASE_URL = 'http://localhost:5000'
PASSWORD = 'password'
```

Then run your code that follows the pattern of `tests/info-example.ts`.

```bash
bun run tests/info-example.ts
```
