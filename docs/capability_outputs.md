# Capability Outputs

This repository now emits two machine-readable layers:

1. Capability declarations and compatibility data.
2. Bundle/setup-oriented resolution artifacts.

## Capability Declaration Layer

The shared types in `crates/greentic-cap-types` and `crates/greentic-cap-schema` cover:

- capability offers, requirements, consumes, and profiles
- deterministic bindings between requests and offers
- pack capability sections
- component self-descriptions
- compatibility reports for pack/component matching

## Bundle/Setup Resolution Artifact

`CapabilityBundleResolutionV1` mirrors the bundle-side `resolved_targets` surface:

- `bundle_id`, `bundle_name`, `requested_mode`, `locale`, and `artifact_extension`
- generated file lists for resolved and setup manifests
- per-target summaries with:
  - `path`
  - `tenant`
  - `team`
  - policy locations
  - emitted binding records
- root and per-profile resolver reports
- compatibility reports for pack/component matching

The emitted binding records are intentionally flat so bundle and setup tooling can write them
directly into resolved manifests, audit logs, or state directories without re-deriving resolver
state.

## Downstream Use

- `gtc wizard` can use the compatibility and binding records while scaffolding packs and setup
  metadata.
- `gtc setup` can convert bundle-resolution artifacts into resolved manifests and setup state
  files.
- Bundle assembly tooling can use the `resolved_targets` summaries to generate `resolved/` and
  `state/resolved/` artifacts in a deterministic order.
