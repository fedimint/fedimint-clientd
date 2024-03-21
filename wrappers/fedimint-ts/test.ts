import { FedimintClientBuilder } from "./FedimintClient";
import dotenv from "dotenv";

dotenv.config();

const logMethod = (method: string) => {
  console.log("--------------------");
  console.log(`\nMethod: ${method}`);
};

const logInputAndOutput = (input: any, output: any) => {
  console.log("Input: ", input);
  console.log("Output: ", output);
  console.log("--------------------\n\n");
};

async function buildTestClient() {
  const baseUrl = process.env.BASE_URL || "http://127.0.0.1:3333";
  const password = process.env.PASSWORD || "password";
  const builder = new FedimintClientBuilder();
  builder.setBaseUrl(baseUrl).setPassword(password).setActiveFederationId(
    "412d2a9338ebeee5957382eb06eac07fa5235087b5a7d5d0a6e18c635394e9ed" // Fedi Alpha Mutinynet
  );

  return await builder.build();
}

// Runs through all of the methods in the Fedimint Client
async function main() {
  const fedimintClient = await buildTestClient();

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
  data = await fedimintClient.join(inviteCode);
  logInputAndOutput({ inviteCode }, data);
  // `/v2/admin/list-operations`
  logMethod("/v2/admin/list-operations");
  data = await fedimintClient.listOperations(10);
  logInputAndOutput({ limit: 10 }, data);

  // LIGHTNING METHODS
  // `/v2/ln/list-gateways`
  logMethod("/v2/ln/list-gateways");
  data = await fedimintClient.ln.listGateways();
  logInputAndOutput({}, data);
  // `/v2/ln/invoice`
  logMethod("/v2/ln/invoice");
  let { operationId, invoice } = await fedimintClient.ln.createInvoice(
    10000,
    "test"
  );
  logInputAndOutput(
    { amountMsat: 10000, description: "test" },
    { operationId, invoice }
  );
  // `/v2/ln/pay`
  logMethod("/v2/ln/pay");
  let payResponse = await fedimintClient.ln.pay(invoice);
  logInputAndOutput({ paymentInfo: invoice }, payResponse);
  // `/v2/ln/await-invoice`
  logMethod("/v2/ln/await-invoice");
  data = await fedimintClient.ln.awaitInvoice(operationId);
  logInputAndOutput({ operationId }, data);

  // MINT METHODS
  // `/v2/mint/spend`
  logMethod("/v2/mint/spend");
  let mintData = await fedimintClient.mint.spend(3000, true, 1000);
  logInputAndOutput(
    { amountMsat: 3000, allowOverpay: true, timeout: 1000 },
    data
  );
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
  // // `/v2/onchain/deposit-address`
  // logMethod("/v2/onchain/deposit-address");
  // data = await fedimintClient.onchain.createDepositAddress(1000);
  // logInputAndOutput({ timeout: 1000 }, data);
  // // `/v2/onchain/await-deposit`
  // logMethod("/v2/onchain/await-deposit");
  // data = await fedimintClient.onchain.awaitDeposit(data.operationId);
  // logInputAndOutput({ operationId: data.operationId }, data);
  // // `/v2/onchain/withdraw`
  // logMethod("/v2/onchain/withdraw");
  // data = await fedimintClient.
}

main().catch(console.error);
