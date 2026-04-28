# Build Stage
FROM rust:1.95.0-slim AS build

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

# workaround for no mkdir in distroless
WORKDIR /app/cabynet
WORKDIR /app

ENV CABY_HOME_PATH=/app/cabynet
ENV CABY_DIRECTORY_META_FILENAME=.cabydir
EXPOSE 8080

COPY --from=build /app/target/release/caby-service .
CMD ["./caby-service"]
