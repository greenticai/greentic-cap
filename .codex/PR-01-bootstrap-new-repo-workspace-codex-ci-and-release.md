# PR-01 — Bootstrap new `greentic-cap` repo with workspace, `.codex`, CI, crate-release baseline, and quality tooling

## Objective / Outcome
Create the new `greentic-cap` repository with a production-ready Rust workspace baseline, repo-local `.codex` guidance, org-aligned CI, crate publishing readiness, quality scripts, and a clear architecture skeleton.

## Repo status
New repo

## Depends on
- None

## Scope
- Create the Cargo workspace and initial crate placeholders:
  - `greentic-cap-types`
  - `greentic-cap-schema`
  - `greentic-cap-core`
  - `greentic-cap-profile`
  - `greentic-cap-resolver`
- Add `.codex/GLOBAL_RULES.md` and `.codex/REPO_OVERVIEW.md`.
- Wire CI to org reusable workflows for Rust setup, fmt, clippy, tests, and cache/binstall where appropriate.
- Add crate publishing / release workflow baseline suitable for library crates.
- Add `tools/i18n.sh` only if the repo ships user-facing schema/help text needing extraction/validation; otherwise stub it with a rationale.
- Add nightly workflows for coverage and performance; e2e may be a lightweight stub if no external runtime exists yet.
- Add a README explaining the repo’s role in the wider Greentic architecture.

## Acceptance criteria
- The repo builds as a Rust workspace scaffold.
- CI is wired through org standards where possible.
- Release flow is appropriate for publishing crates rather than forcing six-platform CLI artifacts unless a real CLI exists.
- Coverage/perf workflows exist or are stubbed compatibly with org reusable workflows.

## Codex prompt
```text
Create the new `greentic-cap` repository baseline.

Required:
- Cargo workspace with planned crate structure
- `.codex/GLOBAL_RULES.md`
- `.codex/REPO_OVERVIEW.md`
- CI wired to org reusable workflows for Rust setup, fmt, clippy, tests
- crate release / publish workflow baseline
- nightly coverage and performance workflow wiring
- README with repo role and crate responsibilities
- optional `tools/i18n.sh` only if justified

Do not force a standalone CLI unless there is a concrete need.
Prefer extending org `.github` reusable workflows rather than copying heavy local YAML.
```
