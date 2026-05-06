FROM rust:1.80-slim as builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo fetch
RUN cargo build --release -p brain-ingestion --verbose

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/brain-ingestion /usr/local/bin/brain-ingestion
COPY sources.json .
CMD ["brain-ingestion"]
