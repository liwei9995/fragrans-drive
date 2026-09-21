<p align="center">
  <a href="https://www.oyiyio.com/" rel="noopener" target="_blank">
    <img width="150" src="./apps/web/public/logo.svg" alt="Fragrans Drive" />
  </a>
</p>

<h1 align="center">Fragrans Drive</h1>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.3.0-blue" alt="version 0.3.0" />
</p>

Fragrans Drive is a full-stack personal cloud drive application based on Vue 3, Element Plus, TypeScript, and a Rust backend, designed for private deployment.

> The project is actively under development. This repository is structured as a monorepo containing both the frontend (`apps/web`) and the backend (`apps/api`).

## Features

- **Authentication & Security**: Email-based authentication with persistent login state, optional email verification codes, captcha protection, and API rate limiting
- **File Management**: Folder browsing, paginated loading, breadcrumb navigation, and batch operations (create, rename, move, delete)
- **Convenient Interactions**: Automatic text selection in rename and folder creation dialogs for instant typing
- **Uploads**: File selection and drag-and-drop uploads with real-time queue and progress tracking
- **Video Processing & Streaming**: Automatic video cover thumbnail extraction (FFmpeg), on-demand 720p web transcoding, and seamless quality switching
- **High-Performance Image Preview**: On-demand 1600px derivative caching, LQIP placeholders, and idle neighbor preloading
- **Mobile-First Experience**: Responsive layout with dynamic viewport height (`100dvh`), Safe Area insets (`env(safe-area-inset-*)`), and mobile floating action button

Watch the [Introduction Video](https://www.youtube.com/embed/Uzeur9v44LE) to see the basic features in action.

## Tech Stack

| Category | Technologies |
| --- | --- |
| **Frontend** | Vue 3.5, Vue Router 5, Pinia 4, Element Plus 2.14, Vite 8, TypeScript 6 |
| **Backend** | Rust 2024, Axum, MongoDB, FFmpeg |
| **Code Quality** | Biome 2, vue-tsc 3 |
| **Testing** | Vitest 4, Vue Test Utils, Playwright |
| **Deployment** | Docker, Docker Compose, CircleCI, Aliyun ACR |

## Requirements

- Node.js 22 or higher
- pnpm 12.5.1 (Enabling via Corepack is recommended)
- Rust toolchain (for backend development)
- FFmpeg (required for local backend video thumbnailing & transcoding)
- Docker & Docker Compose (for production deployment)

## Local Development

```bash
# Enable corepack and install dependencies for the monorepo
corepack enable
pnpm install --frozen-lockfile

# Start the frontend development server
pnpm --filter @fragrans/web dev
```

The frontend development server runs at <http://localhost:5173> by default.

For the backend:
```bash
# Navigate to the backend directory and start the API
cd apps/api
cargo run
```
The backend API listens on `127.0.0.1:3821` by default.

## Deployment & CI/CD

The project utilizes an enterprise-grade CI/CD pipeline powered by **CircleCI**.

When code is merged into the `main` branch:
1. **Build**: CircleCI automatically builds the Docker images for both the Web and API applications using `setup_remote_docker`.
2. **Multi-tagging**: The images are injected with Git Revision data and tagged with an immutable identifier (`prod-<timestamp>-<sha>`), a semantic version, and a moving `prod-latest` tag.
3. **Registry**: The images are pushed to the Aliyun Container Registry (ACR).
4. **Deploy**: CircleCI connects to the deployment server, writes the precise immutable tag into the `.env` file, and triggers `docker-compose pull && docker-compose up -d` to spin up the exact built versions.

This completely eliminates manual server-side builds, drastically reduces server load, and guarantees safe, instant rollbacks.

## Project Structure

```text
.
├── apps/
│   ├── api/              # Rust backend application (Axum + MongoDB)
│   └── web/              # Vue 3 frontend application
├── .circleci/            # CI/CD pipeline configuration
├── docker-compose.yaml   # Unified production orchestration configuration
├── pnpm-workspace.yaml   # Monorepo workspace configuration
└── README.md
```
