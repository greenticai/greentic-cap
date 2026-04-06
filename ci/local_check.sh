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

publishable_packages=()
manifests=("Cargo.toml")
while IFS= read -r manifest; do
  [ -n "$manifest" ] || continue
  manifests+=("$manifest")
done < <(find crates -mindepth 2 -maxdepth 2 -name Cargo.toml | sort)

for manifest in "${manifests[@]}"; do
  if rg -q '^[[:space:]]*publish[[:space:]]*=[[:space:]]*false' "$manifest"; then
    continue
  fi
  package_name="$(sed -n 's/^name = "\(.*\)"/\1/p' "$manifest" | head -n 1)"
  if [ -n "$package_name" ]; then
    publishable_packages+=("$package_name")
  fi
done

for package_name in "${publishable_packages[@]}"; do
  step "cargo package --allow-dirty -p ${package_name}"
  cargo package --allow-dirty -p "$package_name"

  if [ "$package_name" = "greentic-cap" ]; then
    step "Verify package contents for ${package_name}"
    package_listing="$(cargo package --allow-dirty -p "$package_name" --list)"
    required_package_files=(
      "Cargo.toml"
      "Cargo.lock"
      "README.md"
      "LICENSE"
      "src/main.rs"
      "docs/capability_outputs.md"
      "examples/capability/bundle_resolution.json"
      "examples/capability/component_descriptor.json"
      "examples/capability/dw_generic_declaration.json"
      "examples/capability/pack_capabilities.declaration.json"
    )
    for required_file in "${required_package_files[@]}"; do
      if ! printf '%s\n' "$package_listing" | rg -Fxq "$required_file"; then
        printf 'Missing required packaged file: %s\n' "$required_file" >&2
        exit 1
      fi
    done
  fi

  step "cargo publish --dry-run --allow-dirty -p ${package_name}"
  cargo publish --dry-run --allow-dirty -p "$package_name"
done
