# PR-05 — Add bundle/setup integration contracts, resolution output, examples, and architecture docs

## Objective / Outcome
Make `greentic-cap` usable by the rest of the platform by defining how bundle and setup consume capability resolution, how bindings are emitted, and how platform docs/examples explain the model.

## Repo status
New repo

## Depends on
- PR-03
- PR-04

## Scope
- Define machine-readable resolution outputs suitable for bundle/setup consumption.
- Specify how resolved bindings are emitted for consumers.
- Add example assets covering:
  - a short-term memory capability with multiple provider candidates
  - a pack that offers one capability while requiring/consuming another
  - a component that requires and consumes a capability
  - a DW-related example built on the generic model without making the repo DW-specific
- Add architecture docs explaining:
  - profiles → requirements → providers → bindings
  - provider dependency graphs
  - separation between correctness checks and profile/preference guidance
- Provide integration notes for `gtc wizard`, `gtc setup`, and bundle assembly.

## Acceptance criteria
- Resolution output is stable and consumable by bundle/setup.
- Examples cover both direct and transitive capability resolution.
- Docs make clear that `greentic-cap` is the shared platform layer beneath DW and other subsystems.

## Codex prompt
```text
Finalize `greentic-cap` with bundle/setup integration outputs and examples.

Required:
- machine-readable resolution output model
- binding output suitable for downstream runtime injection
- examples for offers / requires / consumes across packs and components
- architecture docs showing how the model fits wizard, setup, and bundle assembly
- at least one example with multiple candidate providers for the same capability
- at least one example with a provider that itself depends on another capability

Keep the repo generic and reusable across the platform.
```
