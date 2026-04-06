# PR-02 — Add core capability types, schemas, and validation model

## Objective / Outcome
Establish the canonical data model for capabilities so all Greentic subsystems can express what they offer, require, and consume in a shared way.

## Repo status
New repo

## Depends on
- PR-01

## Scope
- Implement canonical types for:
  - `CapabilityId`
  - `CapabilityOffer`
  - `CapabilityRequirement`
  - `CapabilityConsume`
  - `CapabilityProfile`
  - `CapabilityResolution`
  - `CapabilityBinding`
- Add JSON/CBOR-friendly schemas for request/profile/resolution structures.
- Define the top-level declaration shape:
  - `capabilities.offers`
  - `capabilities.requires`
  - `capabilities.consumes`
- Add validation for ID format, duplicate entries, invalid profile references, and malformed requirement declarations.
- Keep the model generic across DW, deployers, bundles, packs, and components.

## Acceptance criteria
- Types and schemas compile and round-trip cleanly.
- Validation rejects malformed capability declarations.
- The model clearly supports packs offering/requiring/consuming capabilities and components requiring/consuming capabilities.

## Codex prompt
```text
Implement the canonical capability model in `greentic-cap`.

Required:
- types for offers / requires / consumes / profiles / resolution / bindings
- schema definitions and serde support
- validation for `cap://...` IDs and declaration structure
- examples showing:
  - a pack offering a capability
  - a pack requiring and consuming a capability
  - a component requiring and consuming a capability

Keep the model platform-generic. Do not bake in DW-only assumptions.
```
