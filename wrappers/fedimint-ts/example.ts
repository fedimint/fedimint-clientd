import { FedimintClientBuilder } from "fedimint-ts";
import dotenv from "dotenv";

dotenv.config();

async function main() {
  const baseUrl = process.env.BASE_URL || "http://localhost:3333";
  const password = process.env.PASSWORD || "password";
  const builder = new FedimintClientBuilder();
  builder
    .setBaseUrl(baseUrl)
    .setPassword(password)
    .setActiveFederationId(
      "412d2a9338ebeee5957382eb06eac07fa5235087b5a7d5d0a6e18c635394e9ed"
    );

  // The FedimintClient has a default federationId set that it'll call any module methods on
  const fedimintClient = await builder.build();

  // You can update the federationId to call methods on a different federation
  const { federationIds } = await fedimintClient.federationIds();
  await fedimintClient.setActiveFederationId(federationIds[0]);

  // Any methods that call on a specific federation can optionally take a federationId as the last argument
  // If no federationId is passed, the default federationId is used
  const _ = await fedimintClient.listOperations(
    { limit: 10 },
    federationIds[1]
  );

  // Admin methods give summaries by federation
  const response = await fedimintClient.info();
  console.log("Response: ", response);
  console.log(
    "Current Total Msats Ecash for active federation: ",
    response[fedimintClient.getActiveFederationId()].totalAmountMsat
  );

  // // All module methods are called on the default federationId if you don't pass in a federationId
}

main().catch(console.error);
