# PR 1: Executable bindings and capability-client ABI

## Goal

Define the stable, provider-neutral data-plane contract and immutable artifact that later runtime dispatch consumes.

## Scope

- Add `greentic:cap-runtime@1.0.0` WIT, separate from `greentic:component`.
- Add versioned executable binding, invocation, response, status, limits, scope, policy, and audit types.
- Produce deterministic opaque binding IDs and binding digests from canonical CBOR.
- Convert existing resolver bindings without changing their serialized representation.
- Publish JSON schemas and canonical CBOR round-trip helpers.

## Security constraints

- Executable providers must be digest pinned; mutable authoring references fail closed.
- IDs and digests cover provider, operation map, schemas, scope, policy and limits.
- Artifacts contain no secrets and callers receive only the opaque binding ID.
- Validation rejects empty operation maps, duplicate operations, invalid limits, and incomplete scope.

## Tests

- Stable IDs/digests and serialization independent of map insertion order.
- Digest changes for provider, schema, policy, scope, operation, or limit changes.
- Multiple-operation and CBOR round trips.
- Mutable provider rejection and legacy resolver compatibility.
- WIT syntax/contract regression test.

## Dependencies

None. This is the foundation for PR 2.

## Acceptance criteria

- Existing APIs and tests remain compatible.
- An existing resolved binding can be explicitly promoted to a validated executable binding.
- Repeated construction produces byte-identical artifacts, IDs, and digests.
