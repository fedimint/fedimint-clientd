# fedimint-clientd: A Fedimint Client for Server Side Applications

fedimint-clientd runs a fedimint client with Ecash, Lightning, and Onchain modules to let a server side application hold and use Bitcoin with Fedimint. It exposes a REST API & provides wrappers in typescript, python, golang, and elixir. It uses the `multimint` crate to manage clients connected to multiple Federations from a single `fedimint-clientd` instance.

This project is intended to be an easy-to-use starting point for those interested in adding Fedimint client support to their applications. Fedimint-clientd only exposes Fedimint's default modules, and any more complex Fedimint integration will require custom implementation using [Fedimint's rust crates](https://github.com/fedimint/fedimint).

## Getting Started

1. You can install the cli app with `cargo install fedimint-clientd` or by cloning the repo and running `cargo build --release` in the root directory.

    `fedimint-clientd` runs from the command line and takes a few arguments, which are also available as environment variables. Fedimint uses rocksDB, an embedded key-value store, to store its state. The `--fm_db_path` argument is required and should be an absolute path to a directory where the database will be stored.

    ```shell
    CLI USAGE:
    fedimint-clientd \
      --db-path=/absolute/path/to/dir/to/store/database \
      --password="some-secure-password-that-becomes-the-bearer-token" \
      --addr="127.0.0.1:3333"
      --mode="rest"
      --invite-code="fed1-fedimint-invite-code"
    ```

2. With a Nix environment already setup:
- Clone the repo and run `nix develop` to install and build the dependencies.
- Run `just dev` to fire up the client app.

  Make sure to configure your environment variables as shown below:

  ```shell
  ENV USAGE:
  FEDIMINT_CLIENTD_DB_PATH=/absolute/path/to/dir/to/store/database
  FEDIMINT_CLIENTD_PASSWORD="some-secure-password-that-becomes-the-bearer-token"
  FEDIMINT_CLIENTD_ADDR="127.0.0.1:3333"
  FEDIMINT_CLIENTD_BASE_URL="127.0.0.1:3333"
  FEDIMINT_CLIENTD_MODE="rest"
  # this is the invite code to the Fedi Alpha federation mutinynet,
  # you can replace it with another but its the most useful one for testing so good to at least have it
  FEDIMINT_CLIENTD_INVITE_CODE="fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75"
  ```

## Fedimint Clientd Endpoints

`fedimint-clientd` supports the following endpoints (and has naive websocket support at `/v2/ws`, see code for details until I improve the interface. PRs welcome!). All the endpoints are authed with a Bearer token from the password (from CLI or env). You can hit the endpoints as such with curl, or use the python/typescript/golang wrappers:

```
curl http://localhost:3333/v2/admin/info -H 'Authorization: Bearer some-secure-password-that-becomes-the-bearer-token'
```

### Admin related commands:

- `/v2/admin/info`: Display wallet info (holdings, tiers).
- `/v2/admin/backup`: Upload the (encrypted) snapshot of mint notes to federation.
- `/v2/admin/discover-version`: Discover the common api version to use to communicate with the federation.
- `/v2/admin/restore`: Restore the previously created backup of mint notes (with `backup` command).
- `/v2/admin/list-operations`: List operations.
- `/v2/admin/module`: Call a module subcommand.
- `/v2/admin/config`: Returns the client config.

### Mint related commands:

- `/v2/mint/reissue`: Reissue notes received from a third party to avoid double spends.
- `/v2/mint/spend`: Prepare notes to send to a third party as a payment.
- `/v2/mint/validate`: Verifies the signatures of e-cash notes, but _not_ if they have been spent already.
- `/v2/mint/split`: Splits a string containing multiple e-cash notes (e.g. from the `spend` command) into ones that contain exactly one.
- `/v2/mint/combine`: Combines two or more serialized e-cash notes strings.

### Lightning network related commands:

- `/v2/ln/invoice`: Create a lightning invoice to receive payment via gateway.
- `/v2/ln/await-invoice`: Wait for incoming invoice to be paid.
- `/v2/ln/pay`: Pay a lightning invoice or lnurl via a gateway.
- `/v2/ln/await-pay`: Wait for a lightning payment to complete.
- `/v2/ln/list-gateways`: List registered gateways.
- `/v2/ln/switch-gateway`: Switch active gateway.

### Onchain related commands:

- `/v2/onchain/deposit-address`: Generate a new deposit address, funds sent to it can later be claimed.
- `/v2/onchain/await-deposit`: Wait for deposit on previously generated address.
- `/v2/onchain/withdraw`: Withdraw funds from the federation.

### Extra endpoints:

- `/health`: health check endpoint.
- `/metrics`: exports API metrics using opentelemetry with prometheus exporter (num requests, latency, high-level metrics only)

## Fedimint Clientd on Docker

Fedimint Clientd is not officially supported on Docker. However, you can build, publish and deploy your own docker image by following these steps:

- Build an OCI image

```
nix build .#fedimint-clientd-oci && docker load < ./result
```

- confirm the image is built and loaded in your local docker registry

```
docker image ls

# REPOSITORY          TAG                                IMAGE ID       CREATED        SIZE
# fedimint-clientd    f4cb5pgsw4sn7mg93xwyvx09ib4qg3rd   ff8cf0f0805b   54 years ago   91.1MB1.1GB
# ...
```

- Tag the image image appropriately then publish to your preferred registry

```
docker tag fedimint-clientd <registry>/fedimint-clientd:v0.4.0
docker push <registry>/fedimint-clientd:v0.4.0
```
