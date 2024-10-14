FROM rust:1.81.0 as builder
RUN apt-get update && apt-get install -y curl libudev-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:12.5-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/main /usr/local/bin/tx-bench
ENTRYPOINT ["tx-bench"]
