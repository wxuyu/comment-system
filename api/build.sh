#!/usr/bin/env bash
set -euo pipefail
# Build script for Vercel Rust runtime.
# The Vercel Rust builder expects a Cargo workspace at the function root.
cd "$(dirname "$0")"
cargo build --release --bin handler
