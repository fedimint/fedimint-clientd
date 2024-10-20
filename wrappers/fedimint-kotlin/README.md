
# Fedimint SDK for Kotlin

This is a Kotlin client that consumes the Fedimint Http Client (https://github.com/kodylow/fedimint-http-client)[https://github.com/kodylow/fedimint-http-client], communicating with it via HTTP and a password. It's a hacky prototype, but it works. All of the federation handling code happens in the fedimint-http-client, this just exposes a simple API for interacting with the client from Kotlin.

Start the following in the fedimint-http-client .env environment variables:

```bash
FEDIMINT_CLIENTD_DB_PATH="YOUR-DATABASE-PATH"
FEDIMINT_CLIENTD_PASSWORD="YOUR-PASSWORD"
FEDIMINT_CLIENTD_ADDR="127.0.0.1:3333"
FEDIMINT_CLIENTD_MODE="rest"
FEDIMINT_CLIENTD_INVITE_CODE="fed11qgqrgvnhwden5te0v9k8q6rp9ekh2arfdeukuet595cr2ttpd3jhq6rzve6zuer9wchxvetyd938gcewvdhk6tcqqysptkuvknc7erjgf4em3zfh90kffqf9srujn6q53d6r056e4apze5cw27h75"
FEDIMINT_CLIENTD_BASE_URL="127.0.0.1:3333"
```

Then start the fedimint-http-client server:

```bash
cargo run
```

Then you are ready to run the Kotlin client. You have 2 oprions to fire it up:

1. Use an Intellij IDE or Android Studio.

    This is the simplest and best way to work with the Kotlin wrapper. Open the Kotlin project with any Gradle based IDE such as Intellij or Android Studio.

2. Run the following commands from the Kotlin project root folder.

    ```bash
    ./gradlew build
    ./gradlew run
    ```
