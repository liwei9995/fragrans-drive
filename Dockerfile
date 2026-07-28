FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev gcc curl unzip

WORKDIR /usr/src/fragrans
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache libgcc

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
RUN mkdir -p /app/bucket && chown -R appuser:appgroup /app

WORKDIR /app
COPY --from=builder /usr/src/fragrans/target/release/fragrans /usr/local/bin/fragrans

USER appuser
EXPOSE 3821
CMD ["fragrans"]