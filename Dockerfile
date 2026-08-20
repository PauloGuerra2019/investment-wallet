FROM rust:1.89-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/investment-wallet /usr/local/bin/investment-wallet
COPY static ./static
COPY migrations ./migrations
ENV RUST_LOG=info
EXPOSE 3000
CMD ["investment-wallet"]
