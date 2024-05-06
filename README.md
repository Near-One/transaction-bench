# Transaction latency benchmark

An application to measure the end user latency of transactions on the Near blockchain.

## Supported transactions
- Token transfer to another account
- Token transfer to self

## Usage
Run locally with `cargo` or build and run as a docker image:
```
docker build -t tx-bench .

docker run --rm -it tx-bench
```

Check the program's help: `cargo run -- -h`.

List the supported transactions: `cargo run -- list`.

Run a single transaction once: `cargo run -- test self_token_transfer <SIGNER_ID>`.

Run the bechmarks until the program is manually halted: `cargo run -- run <SIGNER_ID>`.

## CI
The CI checks that the project compiles successfully at every commit. Docker images are pushed to the registry only by tagged builds.

## Metrics
Metrics are exposed by default on `0.0.0.0:9000`.

## Logs
Logs are printed to `stdout`. Log level can be controlled through the environment variable `RUST_LOG`.