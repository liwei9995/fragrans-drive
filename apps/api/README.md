# Fragrans

A high-performance file storage service rewritten in **Rust**.

## Description

**Fragrans** (Osmanthus fragrans) aims to provide an efficient, secure, and scalable personal file storage solution. The project has been fully rewritten from the original Node.js/NestJS stack to Rust for lower memory usage, higher concurrency, and a smaller deployment footprint.

> "Pale yellow, soft by nature — distant in form, yet its fragrance remains." — Like osmanthus, Fragrans quietly and efficiently handles every file behind the scenes.

## Features

- **High-performance core**: Built on Rust + Axum + Tokio with async I/O.
- **Storage optimizations**:
  - **Content deduplication**: Identical file content is stored once on disk.
  - **Sharded paths**: Uses an `aa/bb/cc/hash` layout to avoid overcrowded directories.
  - **Encrypted at rest**: Physical files are encrypted (AES-based) so data is safe on disk.
- **Media processing**:
  - **Image thumbnails**: Automatically generates WebP/JPEG thumbnails for uploaded images, plus 1600px on-demand derivatives for high-res preview.
  - **Video covers**: Extracts lightweight 360px JPEG covers using FFmpeg (with on-demand lazy backfilling for legacy files and startup scanning).
  - **Video transcoding**: Asynchronous 720p Web preview transcoding with byte-range streaming support.
- **Secure auth & protection**: JWT-based authentication, optional email verification codes, captcha protection, and IP/token-based rate limiting.
- **Minimal deployment**: Multi-stage Docker image, roughly ~30MB (with FFmpeg runtime).

## Tech Stack

- **Language**: Rust (2024 edition)
- **Web framework**: [Axum](https://github.com/tokio-rs/axum)
- **Async runtime**: [Tokio](https://tokio.rs/)
- **Database**: MongoDB (official Rust driver)
- **Crypto & hashing**: `bcrypt`, `aes`, `ctr`, `md5`
- **Media processing**: `image` crate, `ffmpeg`
- **Security**: Token-bucket rate limiting, captcha, SMTP email verification

## Quick Start

### Requirements

- Rust toolchain (1.75+)
- MongoDB (5.0+ recommended)
- FFmpeg (for video thumbnail extraction and 720p transcoding)

### Local development

1. **Configure environment variables**  
   Copy `.env.example` to `.env` and adjust as needed:

   ```bash
   cp .env.example .env
   ```

2. **Start the database**  
   Start MongoDB with the existing Docker Compose setup:

   ```bash
   docker-compose up -d mongo
   ```

3. **Run the project**

   ```bash
   cargo run
   ```

   The service listens on port `3821` by default.

4. **Auto-rebuild during development (optional)**  
   Use [Bacon](https://github.com/Canop/bacon) (recommended alternative to cargo-watch) to check or run on file changes:

   ```bash
   cargo install --locked bacon
   bacon
   ```

   By default this keeps running `cargo check`. For “restart the server on code changes”, use the project’s `bacon.toml` and run:

   ```bash
   bacon run
   ```

   **If the service keeps running after Cmd+C**: Ctrl/Cmd+C may only stop Bacon while the child process (this service) keeps running. Clean up before restarting:

   ```bash
   # Kill leftover fragrans processes by name
   pkill -f fragrans
   # Or free the default port (3821)
   lsof -ti:3821 | xargs kill
   ```

   For simple reloads you can also use `cargo run` and restart with Cmd+C + run again so the process exits cleanly.

### Running tests

```bash
# Run all tests (unit + integration: auth, storage, trash)
cargo test
# Check formatting
cargo fmt --all -- --check
# Lint
cargo clippy
```

## API Documentation

The API follows REST conventions under the `/v1` base path.

### How to explore

OpenAPI docs are generated with **utoipa**. Use the interactive UI to try endpoints:

- **Swagger UI**: [http://localhost:3821/swagger-ui](http://localhost:3821/swagger-ui)
- **OpenAPI JSON**: `/api-docs/openapi.json`

Main endpoints:

| Module      | Path                 | Method   | Description                          |
| ----------- | -------------------- | -------- | ------------------------------------ |
| **Auth**    | `/v1/auth/login`     | POST     | Returns a 15-minute access token and sets an HttpOnly refresh cookie |
| **Auth**    | `/v1/auth/refresh`   | POST     | Rotates the refresh cookie and returns an access token |
| **Auth**    | `/v1/auth/logout`    | POST     | Revokes the current session and clears its cookie |
| **Users**   | `/v1/users`          | GET/POST | User management (token required)     |
| **Storage** | `/v1/storage/upload` | POST     | File upload (token required)         |
| **Storage** | `/v1/storage/list`   | POST     | List files (token required)          |
| **Storage** | `/v1/storage/{id}`   | GET      | Download file (token query supported)|

*Tip: In Swagger UI, click "Authorize" and enter a Bearer token to call protected endpoints.*

## Deployment

### Docker

The repo includes an optimized multi-stage Dockerfile:

```bash
# Build the image
docker build -t fragrans-rust .

# Start with Docker Compose
docker-compose up -d
```

The API is private to the Docker network; the web proxy is published on port `8062`. Serve that proxy over HTTPS so Secure login cookies work. Set `DRIVE_DOMAIN` to the browser-facing HTTPS origin and configure SMTP before enabling registration or password reset. If TLS terminates at an upstream proxy, set `TRUSTED_INGRESS_CIDR` to that proxy's source IP as seen by the web container. The web proxy overwrites `X-Real-IP`; only enable `TRUST_PROXY_HEADERS` when direct API access is blocked.

## Directory layout

- `src/api/`: HTTP handlers and middleware.
- `src/service/`: Business logic wiring domain models and infrastructure.
- `src/domain/`: Domain models (User, Storage).
- `src/infrastructure/`: External integrations (DB, storage I/O, image processing).
- `src/config/`: Configuration loading.
- `src/utils/`: Shared helpers (crypto, hashing).
- `tests/`: Integration test suite.

## Contributing & support

- **Author**: [Aaron Li](https://www.oyiyio.com)
- **License**: MIT
