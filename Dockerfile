FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev gcc curl unzip

WORKDIR /usr/src/fragrans

# Cache dependency builds: only recompile crates.io deps when lock/manifest changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin \
    && echo "" > src/lib.rs \
    && echo "fn main() {}" > src/main.rs \
    && echo "fn main() {}" > src/bin/migrate_storage_v1.rs \
    && cargo build --release \
    && rm -rf src \
    && rm -f target/release/deps/fragrans* \
    && rm -f target/release/fragrans \
    && rm -f target/release/migrate_storage_v1 \
    && rm -f target/release/deps/migrate_storage_v1*

COPY src ./src
RUN touch src/lib.rs src/main.rs src/bin/migrate_storage_v1.rs \
    && cargo build --release --bin fragrans

FROM alpine:latest
RUN apk add --no-cache libgcc

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
RUN mkdir -p /app/bucket && chown -R appuser:appgroup /app

WORKDIR /app
COPY --from=builder /usr/src/fragrans/target/release/fragrans /usr/local/bin/fragrans

USER appuser
EXPOSE 3821
CMD ["fragrans"]
