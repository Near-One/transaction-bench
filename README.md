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

## Configuration

### Environment Variables

The application can be configured using environment variables:

- `PERIOD`: Default interval between transaction runs (default: 15m)
- `INTERVAL_OVERWRITE`: JSON object to override intervals for specific transaction types

### Custom Transaction Intervals

You can run different transaction types at different intervals using the `INTERVAL_OVERWRITE` environment variable. This is useful when you want to run certain transactions more frequently than others.

Example: Run MpcSignEcdsa every 5 minutes and Swap every 10 minutes, while keeping other transactions at the default 15-minute interval:

```bash
export INTERVAL_OVERWRITE='{"MpcSignEcdsa": "5m", "Swap": "10m"}'
```

The JSON format supports all transaction types:
- `TokenTransferDefault`
- `TokenTransferIncludedFinal` 
- `TokenTransferFinal`
- `FungibleTokenTransfer`
- `Swap`
- `MpcSignEcdsa`
- `MpcSignEddsa`
- `MpcCkd`

Duration formats supported:
- `5m` (5 minutes)
- `10s` (10 seconds)
- `1h` (1 hour)
- `30m` (30 minutes)

## CI
The CI checks that the project compiles successfully at every commit. Docker images are pushed to the registry only by tagged builds.

## Metrics
Metrics are exposed by default on `0.0.0.0:9000`.

## Logs
Logs are printed to `stdout`. Log level can be controlled through the environment variable `RUST_LOG`.
