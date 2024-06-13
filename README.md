# Relayer

Relayer service for the Catalyze platform, which is responsible for relaying messages between the
[Internet Computer Protocol (ICP)](https://internetcomputer.org/) and the Catalyze platform.
Particular information about which role the relayer service plays can be found in the
[History Canister RFC](https://github.com/Catalyze-Software/history/blob/main/rfc.md).

## Overview

In the current state relayer service is responsible for the querying the history canister for the
events and relaying them to the [Matrix server](https://spec.matrix.org/). The relayer service is
written in Rust and uses:

- [`ic-agent`](https://docs.rs/ic-agent/latest/ic_agent/) library for interacting with the Internet
Computer.
- [`matrix_sdk`](https://docs.rs/matrix-sdk/latest/matrix_sdk) library for making requests to the
  Matrix server.
- [`redis`](https://docs.rs/redis/latest/redis/) library for caching the history canister events and
  the last processed event (history point).

The relayer service is designed to be run as a standalone service with the usage of the
multithreading benefits and can be deployed using Docker. The current design of the relayer service
includes the following components:

- **Main thread** - responsible for the main logic of the relayer service, as initialize the third-party
  clients, start the worker threads, and handle the shutdown signal.
- **Producer** - responsible for querying the history canister for the events and sending them to the
  Redis queue, splitting the events by kind to the different queues.
- **Consumer(s)** - responsible for consuming the events from the Redis queue and processing them.
  In the current state, the relayer service has only one consumer, which is responsible for relaying
  the "Group Member Role Change" events to the Matrix server, but it can be extended to have multiple
  consumers for different events, which can be processed in parallel. The current architecture of the
  relayer service allows to easily extend the number of consumers and the processing logic.

## Changelog

For the changelog, see [CHANGELOG.md](./CHANGELOG.md).

## Development

To clone this repository, run the following command:

```shell
git clone git@github.com:Catalyze-Software/relayer.git
git submodule update --init --recursive
```

`git submodule update --init --recursive` is necessary to clone the submodules in this repository.

## Configuration

The relayer service is configured using [`config.toml`](./config.toml) file or environment variables.
Environment variable should be prefixed with `RELAYER_` and should be in uppercase. For example, the
`config.toml` file:

```toml
log_filter="info,reqwest=info,rustls=info,hyper_util=info,hyper=info"
proxy_id="24swh-4iaaa-aaaap-ahevq-cai"
history_id="qejor-xqaaa-aaaap-ahjaa-cai"
matrix_url="https://matrix.staging.catalyze.chat"
redis_url = "redis://localhost:6379"
```

The environment variables:

```shell
RELAYER_LOG_FILTER="info,reqwest=info,rustls=info,hyper_util=info,hyper=info"
RELAYER_PROXY_ID="24swh-4iaaa-aaaap-ahevq-cai"
RELAYER_HISTORY_ID="qejor-xqaaa-aaaap-ahjaa-cai"
RELAYER_MATRIX_URL="https://matrix.staging.catalyze.chat"
RELAYER_REDIS_URL="redis://localhost:6379"
```

Where:

- `log_filter` or `RELAYER_LOG_FILTER` is the log filter for the relayer service. The log filter is
  a comma-separated list of log levels for different modules. The log levels are `trace`, `debug`,
  `info`, `warn`, and `error`. More details about the log filter can be found in the
  [[tracing_subscriber]](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)
   documentation.
- `proxy_id` or `RELAYER_PROXY_ID` is the proxy canister ID, which is used for querying the proxy
  canister (mostly for getting actual `history_point`).
- `history_id` or `RELAYER_HISTORY_ID` is the history canister ID, which is used for querying the
  history canister events.


## License

[GPL-2.0 License](./LICENSE) © [Catalyze Software](https://catalyze.one/)
