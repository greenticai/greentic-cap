# greentic-cap

`greentic-cap` is the Greentic capability workspace scaffold. It starts as a small Rust
workspace that will eventually hold the shared capability model, schema helpers, profile
expansion logic, and deterministic resolution engine used across packs, bundles, setup, and
runtime tooling.

## Workspace layout

- `crates/greentic-cap-types` - canonical capability data model, validation, and resolution types
- `crates/greentic-cap-schema` - JSON schema, CBOR, and pack compatibility helpers for capability declarations
- `crates/greentic-cap-profile` - profile expansion and normalization logic
- `crates/greentic-cap-resolver` - deterministic capability resolution and conflict reporting
- `crates/greentic-cap-core` - orchestration layer that expands and resolves declarations end to end
- `docs/capability_outputs.md` - bundle/setup-oriented output shapes and downstream notes
- `examples/capability/` - machine-readable examples for declarations, compatibility, and resolution

## CI and Releases

- `ci/local_check.sh` runs the local quality gate: format, clippy, tests, build, and docs.
- GitHub Actions provides:
  - `publish.yml` for pull requests, branch pushes, release verification, and crates.io publishing
  - `perf.yml` for lightweight performance and concurrency guards
  - `nightly-coverage.yml` for scheduled policy checks against `coverage-policy.json`
- Release tags are expected to match the workspace package version, prefixed with `v`.
- Pushes to `main` or `master` publish the workspace crates to crates.io in dependency order, and
  a version that is already published is treated as a successful no-op.
- The published packages include the source and, for the root package, the README, license, docs,
  and example payloads used by the capability layer.

## Performance

- `benches/perf.rs` contains Criterion smoke benchmarks for resolver and pack-compatibility hot paths.
- `tests/perf_scaling.rs` and `tests/perf_timeout.rs` provide fast concurrency and timeout guards.
- The pack compatibility path now validates the component once per pack and reuses that result across offers.
- The resolver hot path now avoids redundant declaration validation and uses a single-pass best-offer selection loop.

## Status

The shared capability crates are implemented enough to expand profiles, choose deterministic
bindings, compare component compatibility, and emit machine-readable resolution and bundle
artifacts. The root binary remains a stub, and the workspace still reserves room for future
integration work with packs, bundles, setup, and runtime tooling.
