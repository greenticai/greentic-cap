# Dynamic capability invocation

## Control plane and data plane

The control plane declares logical capabilities, checks provider compatibility, resolves an offer, pins the provider artifact, and installs an `ExecutableBindingV1` in trusted host storage. The data plane accepts only an opaque binding ID, logical operation, and canonical CBOR payload through `greentic:cap-runtime@1.0.0/capability-client`.

The capability ABI is intentionally separate from `greentic:component`. Providers continue to expose ordinary component operations; extension providers add functionality by publishing a component and capability offer, without adding host imports or changing the component interface.

```mermaid
sequenceDiagram
    participant C as Consumer
    participant R as Bundle/setup resolver
    participant S as Trusted binding store
    participant H as Capability dispatcher
    participant P as Policy evaluator
    participant I as Generic component invoker
    participant O as Audit observer

    C->>R: require cap://example.echo@1
    R->>R: compatibility + deterministic selection
    R->>R: pin provider digest, schemas, policy, scope, limits
    R->>S: install immutable executable binding
    R-->>C: opaque binding ID
    C->>H: invoke(binding ID, logical operation, canonical CBOR)
    H->>S: resolve binding + current status
    H->>H: scope, lifetime, cycle, bounds, schema checks
    H->>P: authorize pinned policy reference/version
    P-->>H: allow/deny
    H->>I: pinned component ref + mapped operation + deadline
    I-->>H: provider result
    H->>H: output schema and size validation; redact errors
    H->>O: correlated terminal audit event
    H-->>C: validated bounded response or safe error
```

## Trust boundaries

The guest controls only the binding ID, logical operation, and payload. Host-owned context supplies tenant, environment, team, correlation ID, deadline, cancellation, and the active dispatch chain. The trusted binding fixes the provider digest, operation map, schemas, scope, policy version, and resource limits. The dispatcher never accepts replacements for those fields from the guest.

Provider output is untrusted. It is bounded and validated before release to the consumer. Provider errors and metadata are not forwarded unless they conform to the public bounded contract. Audit records exclude payloads and provider error text.

## Binding lifecycle and revocation

A binding is derived deterministically from resolved inputs. Its digest covers every security-relevant field and its opaque ID is derived from that digest. Mutable tags are valid authoring inputs but cannot become executable bindings: setup must first resolve a `sha256` digest and use a component reference containing that digest.

The immutable record may carry expiration and revocation identity. Mutable activation/revocation status lives in trusted storage so revocation can take effect without rewriting the signed resolution artifact. A dispatcher must read current status and current policy state for every call, which closes replay and policy TOCTOU gaps.

## Consumer authoring

A consumer declares a logical requirement such as `cap://example.echo@1`. Setup provides its opaque binding ID through trusted configuration. The component imports `capability-client` and invokes a declared logical operation with canonical CBOR. It cannot name a provider, component operation, tenant, schema, or policy.

## Provider authoring

A provider declares an offer whose operation map relates contract operations to ordinary Greentic component operations and supplies matching input/output schemas. A capability may map multiple logical operations. Provider-specific credentials and behavior stay inside the provider/runtime facilities and never enter a binding.

## Runtime integration

Runtime adapters implement the WIT import by translating it to the runtime-neutral dispatcher. The injected component invoker is responsible for invoking the pinned ordinary component operation and honoring host cancellation/deadline signals. Wasmtime linker support belongs in an adapter crate or optional feature, not in the core binding model.

Rust guests use `greentic-cap-guest`; Wasmtime hosts register the import with `greentic-cap-wasmtime`. The latter accepts an object-safe `CapabilityService` owned by host state, keeping tenant context and provider selection outside guest control. Other runtimes can implement the same WIT contract directly without depending on Wasmtime.

The core dispatcher also rejects a provider result observed after its computed deadline. Runtime adapters must still preempt or cancel execution at the deadline so a non-returning provider cannot retain resources indefinitely.

## Versioning and compatibility

The WIT package and executable binding both start at version 1. Existing `CapabilityBinding` and resolver serialization remain unchanged; promotion to `ExecutableBindingV1` is explicit. Breaking ABI or artifact changes require a new WIT package/artifact schema version. Additive safe errors require coordination with generated guest bindings.

## Error model

The public error set is stable and intentionally low-detail: missing binding, disallowed operation, unavailable provider, invalid input/output, denied policy, timeout, cancellation, overload, protocol error, and internal failure. Detailed diagnostics belong in protected telemetry keyed by correlation ID, not in guest-visible strings.

## Security model

Fail-closed dispatch prevents caller-selected providers, cross-scope binding reuse, operation-map bypass, schema substitution, mutable-tag drift, replay after revocation, recursive cycles, excessive depth, CBOR/schema denial of service, overload, timeout, malformed output, error leakage, and audit gaps. Consumers cannot choose providers directly because doing so would turn their authority to use one logical capability into authority to invoke arbitrary installed components—a confused-deputy boundary violation.
