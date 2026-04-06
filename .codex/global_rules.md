# Global Codex rules for `greentic-cap`

## Principles
- Keep the repository small, deterministic, and easy to inspect.
- Prefer minimal, well-scoped changes with clear verification.
- Reuse existing Greentic shared crates and conventions before inventing new local types or interfaces.
- Treat this repository as a lightweight Rust binary scaffold unless the codebase clearly grows beyond that.

## Delivery rules
- Every PR should have a clear outcome statement.
- Preserve compatibility with the repo's existing Rust/Cargo conventions.
- Keep CLI and runtime behavior simple unless the repo gains a real product surface.

## Mandatory workflow
1. Maintain `.codex/repo_overview.md` before and after any PR-style change.
2. Run `ci/local_check.sh` at the end of work, or explain precisely why it cannot pass.
3. Keep `.codex/repo_overview.md` consistent with the actual codebase state.
4. If the repo gains shared concepts, check Greentic shared crates first and reuse them where possible.

## Quality expectations
- Use the repo's standard formatter, linter, tests, and build commands where available.
- Do not hide failing checks.
- Document any known gaps, stubs, or partial implementations in the repo overview.
- Add package metadata before attempting release/publish workflows.

## Reuse-first policy
- Prefer shared Greentic crates for common concepts when they exist.
- Avoid duplicating cross-repo models unless there is a clear documented reason.
- Note any deliberate local-only model in the PR summary.
