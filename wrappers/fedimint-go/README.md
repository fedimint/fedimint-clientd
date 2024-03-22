<img src="assets/fedimint-gophers.png" width="400px" />

# Fedimint SDK for Go

This is a Go client that consumes the Fedimint Http Client (https://github.com/kodylow/fedimint-http-client)[https://github.com/kodylow/fedimint-http-client], communicating with it via HTTP and a password. It's a hacky prototype, but it works until we can get a proper Go client for Fedimint. All of the federation handling code happens in the fedimint-http-client, this just exposes a simple API for interacting with the client from Go (will be mirrored in Python and Go).

Start the following in the fedimint-http-client .env environment variables:

```bash
FEDERATION_INVITE_CODE = 'fed1-some-invite-code'
SECRET_KEY = 'some-secret-key' # generate this with `openssl rand -base64 32`
FM_DB_PATH = '/absolute/path/to/fm.db' # just make this a new dir called `fm_db` in the root of the fedimint-http-client and use the absolute path to thatm it'll create the db file for you on startup
PASSWORD = 'password'
DOMAIN = 'localhost'
PORT = 5000
BASE_URL = 'http://localhost:5000'
```

Then start the fedimint-http-client server:

```bash
cargo run
```

Then you're ready to run the go client, which will use the same base url and password as the fedimint-http-client:

```bash
BASE_URL = 'http://localhost:5000'
PASSWORD = 'password'
```

To install dependencies:
```bash
go get
```

To run (this just runs an example that creates FedimintClient in go and creates an invoice):
@TODO: make this actually export the client for go registry

```bash
go run cmd/main.go
```
