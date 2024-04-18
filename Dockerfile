FROM rust:1.77.0 as builder
RUN apt-get update && apt-get install -y curl libudev-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/app
RUN sh -c '(curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/near-cli-rs/releases/download/v0.8.1/near-cli-rs-installer.sh | sh) || \ 
    cargo install near-cli-rs' 
COPY . .
RUN cargo install --path .

FROM debian:12.5-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/main /usr/local/bin/tx-bench
COPY --from=builder /usr/local/cargo/bin/near /usr/local/bin/near
ENTRYPOINT ["tx-bench"]
