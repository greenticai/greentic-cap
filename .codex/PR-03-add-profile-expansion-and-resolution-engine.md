# PR-03 — Add capability profile expansion and deterministic resolution engine

## Objective / Outcome
Implement the generic engine that expands capability profiles into concrete requirements and resolves those requirements to concrete providers using existing Greentic artifact refs.

## Repo status
New repo

## Depends on
- PR-02

## Scope
- Implement profile expansion:
  - profile → requirements
  - nested/composed profiles if supported
  - duplicate normalization and merge rules
- Implement deterministic resolution:
  - match requirements to offered capabilities
  - support provider refs using `./`, `file://`, `https://`, `oci://`, `store://`, `repo://`
  - detect unresolved capabilities
  - detect ambiguous providers / conflicts
  - emit resolution output and bindings
- Support dependency graphs where provider packs may themselves require/consume other capabilities.
- Keep resolution deterministic and machine-readable.

## Acceptance criteria
- Profiles expand reproducibly.
- Resolution can satisfy simple and graph-shaped capability dependencies.
- Unresolved and conflicting capability situations are clearly reported.
- Output includes enough information for bundle/setup integration.

## Codex prompt
```text
Implement profile expansion and the deterministic capability resolver.

Required:
- expand profiles into requirements
- merge requirements across assets
- resolve requirements to offered capabilities
- provider refs must use existing Greentic artifact ref schemes, not `pack://`
- support provider dependency graphs
- emit machine-readable resolution output including bindings, unresolved entries, and conflicts
- add unit tests for:
  - simple resolution
  - duplicate merging
  - ambiguous providers
  - unresolved requirements
  - transitive provider dependencies

Do not rely on ad hoc heuristics for correctness. Keep resolution deterministic.
```
