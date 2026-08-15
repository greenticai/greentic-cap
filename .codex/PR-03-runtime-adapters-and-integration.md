# PR 3: Guest bindings, host adapters, and integration

Status: guest bindings and the isolated Wasmtime linker adapter are implemented. Bundle/setup wiring and wizard-generated end-to-end archives require coordinated downstream changes and remain outstanding.

## Goal

Integrate the generic dispatcher with Greentic runtimes without coupling core policy and dispatch logic to Wasmtime.

## Scope

- Generate and publish Rust guest bindings for `capability-client`.
- Add an optional Wasmtime linker adapter in a separate crate/feature.
- Adapt the ordinary `greentic:component@1.0.0` invocation envelope.
- Wire bundle/setup emission to executable bindings and trusted binding distribution.
- Add a wizard-generated `cap://example.echo@1` component/pack/bundle fixture.

## Security constraints

- Linker state owns invocation context; guests cannot forge scope or provider data.
- Runtime cancellation and epoch deadlines map to safe capability errors.
- Fixture archives and manifests are produced only by supported wizards using live schemas and `--answers answer.json`.

## Tests

- Guest compilation.
- Wasmtime linker smoke test with a fake ordinary provider component.
- End-to-end bundle/setup binding handoff, revocation, and invocation.
- Compatibility tests against supported component interface versions.

## Dependencies

PRs 1 and 2 plus coordinated releases of runtime/bundle/setup consumers.

## Acceptance criteria

- A guest invokes echo using only a binding ID, operation, and canonical CBOR.
- Host maps it to a digest-pinned ordinary component operation and returns validated bounded CBOR.
- No changes are required in `greentic:component`.
