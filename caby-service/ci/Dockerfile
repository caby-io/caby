# Build Stage
FROM rust:1.96.0-slim AS build

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libvips-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
# Stub main.rs to cache deps independently of source code changes
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src target/release/caby-service target/release/deps/caby_service*

COPY src ./src
RUN cargo build --release

# Release Stage
FROM debian:12-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libvips ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

ENV CABY_HOME_PATH=/app/cabynet
ENV CABY_DIRECTORY_META_FILENAME=.cabydir
EXPOSE 8080

COPY --from=build /app/target/release/caby-service .
CMD ["./caby-service"]
