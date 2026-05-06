FROM rust:1.80-slim as builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo fetch
RUN cargo build --release -p brain-api --verbose

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/brain-api /usr/local/bin/brain-api
CMD ["brain-api"]
