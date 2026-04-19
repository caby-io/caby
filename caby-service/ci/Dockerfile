# Build Stage
FROM rust:1.94.1-slim AS build

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
# Stub main.rs to cache deps independently of source code changes
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src target/release/caby-service target/release/deps/caby_service*

COPY src ./src
RUN cargo build --release

# Release Stage
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=build /app/target/release/caby-service .
CMD ["./caby-service"]
