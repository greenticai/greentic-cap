# PR-04 — Add `pack.cbor` capability declarations and operation/schema compatibility model

## Objective / Outcome
Ground capability matching in existing Greentic primitives by defining how packs declare capabilities in `pack.cbor` and how offered capabilities map to real self-described component operations with typed schemas.

## Repo status
New repo

## Depends on
- PR-02
- PR-03

## Scope
- Define the `pack.cbor` capability section for:
  - `offers`
  - `requires`
  - `consumes`
- For offered capabilities, define provider metadata including:
  - provider component identifier
  - operation map from logical capability contract to real component operations
- Define capability contracts as operation sets rather than free-form traits.
- Add compatibility checks using:
  - pack metadata
  - component self-description
  - operation names
  - typed input/output schemas
- Keep optional profile/preference selection separate from hard contract validity.

## Acceptance criteria
- A capability can be declared in `pack.cbor` and mapped to component operations.
- Compatibility checks can validate that a provider genuinely satisfies a required capability contract.
- The model reuses existing self-description and typed schema primitives rather than inventing a parallel metadata language.

## Codex prompt
```text
Add the `pack.cbor` capability declaration model and operation/schema compatibility checks.

Required:
- define capability declarations in `pack.cbor` for offers / requires / consumes
- define offered capability -> provider component -> operation map structure
- model capability contracts as required operation sets
- implement compatibility checks using component self-description and typed schemas
- add examples such as:
  - `cap://memory.short-term` offered by redis/postgres-backed providers
  - a consuming component binding to that capability

Avoid a heavyweight trait system. Reuse existing Greentic primitives.
```
