# Fedimintex

# Fedimintex: Fedimint SDK in Elixir

This is an elixir library that consumes Fedimint HTTP (https://github.com/kodylow/fedimint-http-client)[https://github.com/kodylow/fedimint-http-client], communicating with it via REST endpoints + password. It's a hacky prototype, but it works until we can get a proper elixir client for Fedimint. All of the federation handling code happens in the fedimint-http, this just exposes a simple API for interacting with the client from elixir (mirrored in Go, Python, and TS).

Start the following in the fedimint-http-client `.env` file:

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

Then you're ready to use the elixir client, which will use the same base url and password as the fedimint-http-client, so you'll need to set those in your elixir project's `.env` file:

```bash
export BASE_URL='http://localhost:5000'
export PASSWORD='password'
```

Source the `.env` file and enter the iex shell:

```bash
source .env
iex -S mix
```

Then you can use the client:

```bash
iex > client = Fedimintex.new()
iex > invoice = Fedimintex.ln.create_invoice(client, 1000)
# pay the invoice
iex > Fedimintex.ln.await_invoice
```

## Installation

If [available in Hex](https://hex.pm/docs/publish), the package can be installed
by adding `fedimintex` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:fedimintex, "~> 0.1.0"}
  ]
end
```

Documentation can be generated with [ExDoc](https://github.com/elixir-lang/ex_doc)
and published on [HexDocs](https://hexdocs.pm). Once published, the docs can
be found at <https://hexdocs.pm/fedimintex>.
