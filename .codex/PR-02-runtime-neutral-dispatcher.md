# PR 2: Runtime-neutral secure dispatcher

Status: core dispatcher implemented in `greentic-cap-runtime`; runtime-enforced preemption and linker integration remain in PR 3.

## Goal

Implement fail-closed dispatch through injected stores, policy evaluators, provider invokers, clocks, concurrency gates, and observers.

## Scope

- Trusted binding/status lookup and host-owned invocation context.
- Scope, expiration, revocation, policy-version, operation-map, canonical-CBOR, and schema checks.
- Request/response/metadata bounds, deadlines, cancellation, overload handling, and redacted errors.
- Cycle detection and maximum depth.
- Structured audit events and correlation propagation.

## Security constraints

- No caller-controlled provider or schema fields.
- Re-read binding status and policy at invocation time.
- Reserve concurrency before provider execution and release on every path.
- Bound CBOR nesting and schema work before materializing untrusted values.
- Never return provider error strings or unvalidated metadata.

## Tests

Hermetic tests for success, every safe error, scope isolation, revocation/expiry, policy TOCTOU, malformed/deep/large CBOR, schema mismatch, timeout/cancellation/overload, cycles/depth, redaction, and audit correlation.

## Dependencies

PR 1.

## Acceptance criteria

- A fake generic component invoker receives only the pinned provider reference, mapped operation, bounded payload, deadline, and host context.
- All validation failures occur before provider invocation and emit a terminal audit event.
