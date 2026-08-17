#!/bin/bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
docker compose config -q
docker build -t fragrans:verify .
