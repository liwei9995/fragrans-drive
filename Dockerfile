FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev gcc

WORKDIR /usr/src/fragrans
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache libgcc

COPY --from=builder /usr/src/fragrans/target/release/fragrans /usr/local/bin/fragrans
EXPOSE 3821
CMD ["fragrans"]