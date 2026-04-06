#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

step() {
  printf '\n==> %s\n' "$1"
}

step "cargo fmt --all -- --check"
cargo fmt --all -- --check

step "cargo clippy --workspace --all-targets --all-features -- -D warnings"
cargo clippy --workspace --all-targets --all-features -- -D warnings

step "cargo test --workspace --all-features"
cargo test --workspace --all-features

step "cargo build --workspace --all-features"
cargo build --workspace --all-features

step "cargo doc --workspace --no-deps --all-features"
cargo doc --workspace --no-deps --all-features
