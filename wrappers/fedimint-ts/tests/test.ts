import dotenv from "dotenv";
import { randomBytes } from "crypto";
import * as secp256k1 from "secp256k1";
import {
  FedimintClientBuilder,
  LightningClaimPubkeyReceiveTweakedRequest,
  LightningInvoiceExternalPubkeyTweakedRequest,
  LightningInvoiceRequest,
  LightningPayRequest,
  MintSpendRequest,
} from "../src";

dotenv.config();

const logMethod = (method: string) => {
  console.log("--------------------");
  console.log(`Method: ${method}`);
};

const logInputAndOutput = (input: any, output: any) => {
  console.log("Input: ", input);
  console.log("Output: ", output);
  console.log("--------------------");
};

interface KeyPair {
  privateKey: string;
  publicKey: string;
}

const newKeyPair = (): KeyPair => {
  let privateKey: Buffer;
  do {
    privateKey = randomBytes(32);
  } while (!secp256k1.privateKeyVerify(privateKey));

  const publicKey = secp256k1.publicKeyCreate(privateKey);

  return {
    privateKey: privateKey.toString("hex"),
    publicKey: Buffer.from(publicKey).toString("hex"),
  };
};

async function buildTestClient() {
  const baseUrl =
    process.env.FEDIMINT_CLIENTD_BASE_URL || "http://127.0.0.1:3333";
  const password = process.env.FEDIMINT_CLIENTD_PASSWORD || "password";
  const activeFederationId =
    process.env.FEDIMINT_CLIENTD_ACTIVE_FEDERATION_ID ||
    "15db8cb4f1ec8e484d73b889372bec94812580f929e8148b7437d359af422cd3";
  const builder = new FedimintClientBuilder();
  builder
    .setBaseUrl(baseUrl)
    .setPassword(password)
    .setActiveFederationId(activeFederationId);

  const client = await builder.build();
  console.log(
    "Created client. Base url: ",
    baseUrl,
    ", password: ",
    password,
    "Active federation id: ",
    client.getActiveFederationId()
  );
  await client.useDefaultGateway();

  console.log("Default gateway id: ", client.getActiveGatewayId());

  return client;
}

// Runs through all of the methods in the Fedimint Client
async function main() {
  const fedimintClient = await buildTestClient();
  const keyPair = newKeyPair();
  console.log("Generated key pair: ", keyPair);

  // ADMIN METHODS
  // `/v2/admin/config`
  logMethod("/v2/admin/config");
  let data = await fedimintClient.config();
  logInputAndOutput({}, data);
  // `/v2/admin/discover-version`
  logMethod("/v2/admin/discover-version");
  data = await fedimintClient.discoverVersion();
  logInputAndOutput({}, data);
  // `/v2/admin/federation-ids
  logMethod("/v2/admin/federation-ids");
  const { federationIds } = await fedimintClient.federationIds();
  logInputAndOutput({}, federationIds);
  // `/v2/admin/info`
  logMethod("/v2/admin/info");
  data = await fedimintClient.info();
  logInputAndOutput({}, data);
  // `/v2/admin/join`
  let inviteCode =
    process.env.INVITE_CODE ||
    "fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75";
  logMethod("/v2/admin/join");
  data = await fedimintClient.join(inviteCode, true, true);
  logInputAndOutput({ inviteCode }, data);
  // `/v2/admin/list-operations`
  logMethod("/v2/admin/list-operations");
  data = await fedimintClient.listOperations(10);
  logInputAndOutput({ limit: 10 }, data);

  // LIGHTNING METHODS
  // `/v2/ln/list-gateways`
  logMethod("/v2/ln/list-gateways");
  data = await fedimintClient.lightning.listGateways();
  logInputAndOutput({}, data);
  // `/v2/ln/invoice`
  logMethod("/v2/ln/invoice");
  let lightningInvoiceRequest: LightningInvoiceRequest = {
    amountMsat: 10000,
    description: "test",
    expiryTime: 3600,
  };
  let { operationId, invoice } = await fedimintClient.lightning.createInvoice(
    lightningInvoiceRequest
  );
  logInputAndOutput(
    { amountMsat: 10000, description: "test" },
    { operationId, invoice }
  );
  // `/v2/ln/pay`
  logMethod("/v2/ln/pay");
  let lightningPayRequest: LightningPayRequest = {
    paymentInfo: invoice,
  };
  let payResponse = await fedimintClient.lightning.pay(lightningPayRequest);
  logInputAndOutput({ paymentInfo: invoice }, payResponse);
  // `/v2/ln/await-invoice`
  logMethod("/v2/ln/await-invoice");
  data = await fedimintClient.lightning.awaitInvoice(operationId);
  logInputAndOutput({ operationId }, data);
  // `/v1/ln/invoice-external-pubkey-tweaked`
  logMethod("/v1/ln/invoice-external-pubkey-tweaked");
  let lightningInvoiceExternalPubkeyTweakedRequest: LightningInvoiceExternalPubkeyTweakedRequest =
    {
      externalPubkey: keyPair.publicKey,
      tweak: 1,
      amountMsat: 1000,
      description: "test",
      expiryTime: 3600,
    };
  data = await fedimintClient.lightning.createInvoiceForPubkeyTweak(
    lightningInvoiceExternalPubkeyTweakedRequest
  );
  logInputAndOutput(
    {
      externalPubkey: keyPair.publicKey,
      tweak: 1,
      amountMsat: 10000,
      description: "test",
    },
    data
  );
  // pay the invoice
  let payRequest: LightningPayRequest = {
    paymentInfo: data.invoice,
  };
  payResponse = await fedimintClient.lightning.pay(payRequest);
  // `/v1/ln/claim-external-pubkey-tweaked`
  logMethod("/v1/ln/claim-external-pubkey-tweaked");
  let lightningClaimPubkeyTweakedRequest: LightningClaimPubkeyReceiveTweakedRequest =
    {
      privateKey: keyPair.privateKey,
      tweaks: [1],
    };
  data = await fedimintClient.lightning.claimPubkeyTweakReceives(
    lightningClaimPubkeyTweakedRequest,
    fedimintClient.getActiveFederationId()
  );
  logInputAndOutput({ privateKey: keyPair.privateKey, tweaks: [1] }, data);

  // MINT METHODS
  // `/v2/mint/spend`
  logMethod("/v2/mint/spend");
  let spendRequest: MintSpendRequest = {
    amountMsat: 3000,
    allowOverpay: true,
    timeout: 1000,
    includeInvite: true,
  };
  let mintData = await fedimintClient.mint.spend(spendRequest);
  logInputAndOutput(
    { amountMsat: 3000, allowOverpay: true, timeout: 1000 },
    data
  );
  // `/v2/mint/decode-notes`
  logMethod("/v2/mint/decode-notes");
  data = await fedimintClient.mint.decodeNotes(mintData.notes);
  logInputAndOutput({ notes: mintData.notes }, data);
  // `/v2/mint/encode-notes`
  logMethod("/v2/mint/encode-notes");
  data = await fedimintClient.mint.encodeNotes(data.notesJson);
  logInputAndOutput({ notesJson: data.notesJson }, data);
  // `/v2/mint/validate`
  logMethod("/v2/mint/validate");
  data = await fedimintClient.mint.validate(mintData.notes);
  logInputAndOutput({ notes: mintData.notes }, data);
  // `/v2/mint/reissue`
  logMethod("/v2/mint/reissue");
  data = await fedimintClient.mint.reissue(mintData.notes);
  logInputAndOutput({ notes: mintData.notes }, data);
  // `/v2/mint/split`
  logMethod("/v2/mint/split");
  data = await fedimintClient.mint.split(mintData.notes);
  logInputAndOutput({ notes: mintData.notes }, data);
  // `/v2/mint/combine`
  logMethod("/v2/mint/combine");
  const notesVec = Object.values(data.notes) as string[];
  data = await fedimintClient.mint.combine(notesVec);
  logInputAndOutput({ notesVec }, data);

  // ONCHAIN METHODS
  // `/v2/onchain/deposit-address`
  logMethod("/v2/onchain/deposit-address");
  data = await fedimintClient.onchain.createDepositAddress(1000);
  logInputAndOutput({ timeout: 1000 }, data);
  // `/v2/onchain/withdraw`
  logMethod("/v2/onchain/withdraw");
  data = await fedimintClient.onchain.withdraw(data.address, 1000);
  logInputAndOutput({ address: data.address, amountSat: 1000 }, data);
  // // `/v2/onchain/await-deposit`
  // logMethod("/v2/onchain/await-deposit");
  // data = await fedimintClient.onchain.awaitDeposit(data.operationId);
  // logInputAndOutput({ operationId: data.operationId }, data);

  console.log("Done: All methods tested successfully!");
}

main().catch(console.error);
