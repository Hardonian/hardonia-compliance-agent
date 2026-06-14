# Multi-stage build for Compliance Agent
FROM rust:1.78-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin compliance-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/compliance-api /usr/local/bin/
EXPOSE 8080
CMD ["compliance-api"]