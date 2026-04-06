# Repository Overview

## 1. High-Level Purpose
- `greentic-cap` is a Rust workspace for the Greentic capability layer.
- The repo contains the canonical capability data model, schema/CBOR helper layer, pack capability compatibility checks, profile expansion logic, deterministic resolver, a small orchestration façade for driving all three together, and bundle/setup-oriented resolution artifacts.
- The repo also includes lightweight performance and concurrency harnesses for resolver and pack-compatibility hot paths.
- The root package is still a minimal binary scaffold, but the shared capability crates are already functional enough to validate declarations, expand profiles, choose bindings, compare component compatibility, generate schemas and machine-readable outputs for downstream consumers, and run policy/latency regression checks.

## 2. Main Components and Functionality

### `Cargo.toml`
- **Path:** `Cargo.toml`
- **Role:** Workspace root and release package manifest.
- **Key functionality:**
  - Declares the workspace members:
    - `crates/greentic-cap-types`
    - `crates/greentic-cap-schema`
    - `crates/greentic-cap-core`
    - `crates/greentic-cap-profile`
    - `crates/greentic-cap-resolver`
  - Defines shared workspace metadata including version `0.5.0`, Rust 2024 edition, MIT license, repository URL, README, and description.
  - Declares the publishable root package file set with an explicit `include` list covering the README, license, source, docs, and example payloads.
  - Provides root dev-dependencies for Criterion and the workspace crates used by the perf harness.
  - Declares a `perf` benchmark target with `harness = false`.
  - Exposes shared workspace dependencies for `schemars`, `serde`, `serde_json`, `serde_cbor`, and `thiserror`.
  - Keeps the root `greentic-cap` package aligned with the workspace metadata through `workspace = true` fields.
- **Key dependencies / integration points:** Cargo workspace/package resolution, release packaging, and workspace-wide build/test commands.

### `src/main.rs`
- **Path:** `src/main.rs`
- **Role:** Root binary entrypoint.
- **Key functionality:**
  - Defines `fn main()`.
  - Prints `Hello, world!` to standard output.
  - Contains no domain logic yet.
- **Key dependencies / integration points:** Built as the root package's binary target.

### `crates/greentic-cap-types`
- **Path:** `crates/greentic-cap-types`
- **Role:** Canonical capability data model crate.
- **Key functionality:**
  - Defines `CapabilityId` with `cap://` validation and serde support.
  - Defines the core declaration shapes:
    - `CapabilityOffer`
    - `CapabilityRequirement`
    - `CapabilityConsume`
    - `CapabilityProfile`
    - `CapabilityDeclaration`
    - `CapabilityBinding`
    - `CapabilityResolution`
  - Validates duplicate ids, malformed identifiers, invalid profile references, and binding consistency.
  - Supports JSON and CBOR round-tripping in unit tests.
  - Stores provider operation maps and component self-description metadata used by the pack compatibility layer.
- **Key dependencies / integration points:** Uses shared workspace dependencies for `serde`, `schemars`, `serde_json`, `serde_cbor`, and `thiserror`; marked `publish = false`.

### `crates/greentic-cap-schema`
- **Path:** `crates/greentic-cap-schema`
- **Role:** Schema and serialization helper crate for capability declarations.
- **Key functionality:**
  - Re-exports the capability types for downstream consumers.
  - Generates JSON schema values for declarations, profiles, bindings, and resolutions via `schemars`.
  - Provides CBOR encode/decode helpers that validate data before serialization or after deserialization.
  - Defines the versioned pack capability section wrapper and compatibility report types.
  - Checks pack capability offers against component self-descriptions using operation names, provider maps, and exact input/output schema matches.
  - Includes unit tests for schema generation, CBOR round-tripping, and pack/component compatibility.
- **Key dependencies / integration points:** Depends on `greentic-cap-types` and shared serialization/schema crates; marked `publish = false`.

### `crates/greentic-cap-core`
- **Path:** `crates/greentic-cap-core`
- **Role:** Core orchestration crate.
- **Key functionality:**
  - Orchestrates profile expansion and deterministic resolution for a declaration.
  - Produces a `CapabilityOrchestrationReport` with a root resolution and per-profile resolution reports.
  - Provides a root-expansion helper for callers that only need the generic view.
- **Key dependencies / integration points:** Depends on `greentic-cap-profile`, `greentic-cap-resolver`, and the shared capability types; marked `publish = false`.

### `crates/greentic-cap-profile`
- **Path:** `crates/greentic-cap-profile`
- **Role:** Profile expansion crate.
- **Key functionality:**
  - Expands a declaration into a root view or a named profile view.
  - Merges generic requirements/consumes with profile-specific items.
  - Detects conflicting duplicates while normalizing the expanded item lists.
- **Key dependencies / integration points:** Depends on `greentic-cap-types`; marked `publish = false`.

### `crates/greentic-cap-resolver`
- **Path:** `crates/greentic-cap-resolver`
- **Role:** Deterministic resolver crate.
- **Key functionality:**
  - Resolves expanded requirements and consumes to capability offers.
  - Prefers profile-specific offers over generic ones, then provider-backed offers, then stable identifier ordering.
  - Uses a single-pass candidate selection loop and avoids redundant declaration validation on the hot path.
  - Emits machine-readable resolution reports with unresolved and ambiguous selection issues.
- **Key dependencies / integration points:** Depends on `greentic-cap-profile` and `greentic-cap-types`; marked `publish = false`.

### `ci/local_check.sh`
- **Path:** `ci/local_check.sh`
- **Role:** Local quality gate and packaging check.
- **Key functionality:**
  - Runs workspace-wide `fmt`, `clippy`, `test`, `build`, and `doc` checks.
  - Packages and publish-dry-runs publishable crates, skipping crates with `publish = false`.
  - Verifies that the published root package still contains the README, license, source, docs, and example payloads needed for release readiness.
  - Uses portable shell and `grep` for manifest/package checks so it works on GitHub runners without extra utilities.
  - Acts as the single developer entrypoint for local verification.
- **Key dependencies / integration points:** Installed Rust toolchain, Cargo registry access for dry-run publishing.

### `.github/workflows/publish.yml`
- **Path:** `.github/workflows/publish.yml`
- **Role:** Combined CI and release workflow.
- **Key functionality:**
  - Runs formatting and clippy checks on pull requests and branch pushes.
  - Runs the workspace test suite on pull requests and branch pushes.
  - Verifies the release tag matches the workspace package version on release events.
  - Runs `ci/local_check.sh` before release.
  - Publishes the root package to crates.io on pushes to `main`/`master` and on tags or manual dispatch.
  - Treats an already-published version as a successful no-op so repeated pushes do not fail.
- **Key dependencies / integration points:** GitHub Actions, Rust toolchain, cargo cache, and `CARGO_REGISTRY_TOKEN`.

### `.github/actions/rust-setup/action.yml`
- **Path:** `.github/actions/rust-setup/action.yml`
- **Role:** Reusable Rust setup composite action for CI-style jobs.
- **Key functionality:**
  - Installs the stable Rust toolchain with `rustfmt` and `clippy`.
  - Enables the shared Cargo cache.
- **Key dependencies / integration points:** GitHub Actions composite actions.

### `.github/workflows/perf.yml`
- **Path:** `.github/workflows/perf.yml`
- **Role:** Lightweight performance and concurrency workflow.
- **Key functionality:**
  - Triggers on pull requests and pushes to the main branches.
  - Sets up Rust via the shared composite action and installs `greentic-dev` plus coverage tooling with `cargo-binstall`.
  - Runs the perf guard tests and a Criterion benchmark smoke pass.
- **Key dependencies / integration points:** GitHub Actions, `cargo-binstall`, `greentic-dev`, Criterion.

### `.github/workflows/nightly-coverage.yml`
- **Path:** `.github/workflows/nightly-coverage.yml`
- **Role:** Scheduled coverage policy workflow wrapper.
- **Key functionality:**
  - Triggers on a nightly schedule and manual dispatch.
  - Delegates the actual coverage job to the reusable coverage workflow.
- **Key dependencies / integration points:** GitHub Actions scheduling and reusable workflow calls.

### `.github/workflows/_reusable_coverage.yml`
- **Path:** `.github/workflows/_reusable_coverage.yml`
- **Role:** Reusable coverage check workflow.
- **Key functionality:**
  - Sets up the Rust toolchain and Cargo cache.
  - Installs `cargo-binstall` and the coverage tooling binaries.
  - Runs `greentic-dev coverage` so policy violations fail the workflow.
- **Key dependencies / integration points:** GitHub Actions reusable workflows, `cargo-binstall`, and the local coverage policy file.

### `README.md`
- **Path:** `README.md`
- **Role:** Human-facing workspace summary.
- **Key functionality:**
  - Describes the workspace purpose and crate layout.
  - Documents the CI and release entrypoints.
- **Key dependencies / integration points:** Repository onboarding and release guidance.

### `docs/capability_outputs.md`
- **Path:** `docs/capability_outputs.md`
- **Role:** Documentation for generated capability and bundle outputs.
- **Key functionality:**
  - Describes the declaration layer and bundle/setup resolution artifact.
  - Summarizes how downstream tooling can consume the emitted bindings and target summaries.
- **Key dependencies / integration points:** Bundle assembly, setup generation, and wizard workflows.

### `examples/capability/`
- **Path:** `examples/capability/`
- **Role:** Machine-readable example payloads.
- **Key functionality:**
  - Provides example capability declarations, component descriptors, and bundle-resolution output.
  - Demonstrates the JSON shape used by the schema and resolver layers.
- **Key dependencies / integration points:** Documentation, regression testing, and downstream integration references.

### `benches/perf.rs`
- **Path:** `benches/perf.rs`
- **Role:** Criterion smoke benchmark for hot paths.
- **Key functionality:**
  - Measures resolver root selection and pack capability compatibility on synthetic workloads.
  - Serves as a regression signal for the main CPU-bound paths.
- **Key dependencies / integration points:** Criterion and the shared perf fixture module in `tests/perf_support/`.

### `tests/perf_scaling.rs`
- **Path:** `tests/perf_scaling.rs`
- **Role:** Concurrency scaling guard.
- **Key functionality:**
  - Runs the pack compatibility workload with multiple thread counts.
  - Fails if higher thread counts degrade excessively relative to lower counts.
- **Key dependencies / integration points:** Shared perf fixture module and the schema compatibility helper.

### `tests/perf_timeout.rs`
- **Path:** `tests/perf_timeout.rs`
- **Role:** Hang/timeout guard.
- **Key functionality:**
  - Repeats the pack compatibility workload under a fixed deadline.
  - Fails if the operation becomes unexpectedly slow or stalls.
- **Key dependencies / integration points:** Shared perf fixture module and the schema compatibility helper.

### `LICENSE`
- **Path:** `LICENSE`
- **Role:** Repository license file.
- **Key functionality:**
  - Provides the MIT license text for the repo.
- **Key dependencies / integration points:** Cargo packaging and release readiness.

### `coverage-policy.json`
- **Path:** `coverage-policy.json`
- **Role:** Repository-wide coverage policy for Rust source files.
- **Key functionality:**
  - Sets a global and default per-file minimum line coverage threshold.
  - Excludes thin entrypoints and non-source asset directories from coverage enforcement.
  - Applies stricter coverage targets to the high-risk capability modules.
- **Key dependencies / integration points:** Coverage tooling and CI policy enforcement.

## 3. Work In Progress, TODOs, and Stubs
- No `TODO`, `FIXME`, `XXX`, `HACK`, `unimplemented`, or `todo!` markers were found in the repository.
- The root binary is still a stub that prints `Hello, world!`.
- The workspace is still incomplete as a product: the root binary is just a scaffold, and there is no integration yet with packs, bundles, setup, or runtime tooling.
- The profile/resolution stack is implemented, and the pack capability compatibility layer and bundle/setup resolution artifact layer are now present, but the repo still only covers the generic capability model defined locally.

## 4. Broken, Failing, or Conflicting Areas
- No failing tests or build errors were observed.
- `ci/local_check.sh` passes end to end.
- `cargo package` and `cargo publish --dry-run` succeed for the publishable root package.
- The root release package is explicitly constrained to the intended published file set, which keeps docs and examples in the tarball and guards against accidental omissions.
- A repo-level coverage policy now exists and differentiates thin entrypoints from core capability modules.
- A nightly coverage workflow now runs the repo policy through `greentic-dev coverage`.
- A lightweight perf harness now covers the resolver and pack compatibility hot paths, and the largest identified bottlenecks were reduced by removing redundant component validation in the pack compatibility loop and redundant declaration validation in the resolver path.

## 5. Notes for Future Work
- Replace the root `Hello, world!` binary with real workspace entrypoints or remove it if the repo becomes library-only.
- Add integration points for packs, bundles, setup, and runtime tooling once the capability model is consumed downstream.
- Consider whether the root binary should remain or whether the workspace should become library-first once downstream consumers are wired up.
- Add package metadata and publishability decisions for any new workspace members that should ship to crates.io.
