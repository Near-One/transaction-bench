# Transaction latency benchmark

An application to measure the end user latency of transactions on the Near blockchain.

## Supported transactions
- Token transfer with default parameters
- Token transfer with [`wait_until: IncludedFinal`](https://docs.near.org/api/rpc/transactions#tx-status-result)
- Token transfer with [`wait_until: Final`](https://docs.near.org/api/rpc/transactions#tx-status-result)
- Swap NEAR -> USDT
- FT USDT transfer
- MPC Sign requests

## Usage
Run locally with `cargo` or build and run as a docker image:
```
docker build -t tx-bench .
docker run --rm -it tx-bench
```

You can also run it without Docker, follow the usual Rust workflow.

## CI
The CI checks that the project compiles successfully at every commit. Docker images are pushed to the registry only by tagged builds.

## Metrics
Metrics are exposed by default on `0.0.0.0:9000`.

## Logs
Logs are printed to `stdout`. Log level can be controlled through the environment variable `RUST_LOG`.
