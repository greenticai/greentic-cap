//! Runtime-neutral, fail-closed capability dispatcher.

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde_cbor::Value as CborValue;

use crate::{
    BindingId, BindingStatusV1, CapabilityAuditEventV1, CapabilityErrorV1,
    CapabilityInvocationRequestV1, CapabilityInvocationResponseV1, ExecutableBindingV1,
};

/// Host-owned identity and execution state. No field comes from the guest request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvocationContextV1 {
    pub tenant: String,
    pub environment: String,
    pub team: Option<String>,
    pub correlation_id: String,
    /// An earlier outer deadline, if one exists.
    pub deadline_unix_ms: Option<u64>,
    pub cancelled: bool,
    /// Binding IDs already active in the host-owned dispatch chain.
    pub dispatch_chain: Vec<BindingId>,
}

/// Binding plus its independently mutable lifecycle state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredBindingV1 {
    pub binding: ExecutableBindingV1,
    pub status: BindingStatusV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingStoreError;

/// Trusted binding lookup. Implementations must provide a coherent binding/status snapshot.
pub trait BindingStore: Send + Sync {
    fn get(&self, id: &BindingId) -> Result<Option<StoredBindingV1>, BindingStoreError>;
}

/// Policy result includes the version actually evaluated to detect policy drift.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyDecisionV1 {
    pub allowed: bool,
    pub evaluated_version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyEvaluationError;

/// Host policy boundary.
pub trait PolicyEvaluator: Send + Sync {
    fn evaluate(
        &self,
        binding: &ExecutableBindingV1,
        operation: &str,
        context: &InvocationContextV1,
    ) -> Result<PolicyDecisionV1, PolicyEvaluationError>;
}

/// Fully trusted provider call assembled by the dispatcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderInvocationV1 {
    pub provider_component_ref: String,
    pub provider_digest: String,
    pub component_operation: String,
    pub payload_cbor: Vec<u8>,
    pub tenant: String,
    pub environment: String,
    pub team: Option<String>,
    pub correlation_id: String,
    pub deadline_unix_ms: u64,
    pub dispatch_chain: Vec<BindingId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderOutputV1 {
    pub output_cbor: Vec<u8>,
    pub metadata_cbor: Option<Vec<u8>>,
}

/// Internal provider failures. Detail is deliberately discarded at the guest boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderInvokeError {
    Unavailable(String),
    Timeout(String),
    Cancelled(String),
    Overloaded(String),
    Protocol(String),
    Internal(String),
}

/// Generic ordinary-component invocation boundary.
pub trait ProviderInvoker: Send + Sync {
    fn invoke(
        &self,
        request: ProviderInvocationV1,
    ) -> Result<ProviderOutputV1, ProviderInvokeError>;
}

/// Protected audit sink. Observer failure never changes or exposes invocation results.
pub trait CapabilityObserver: Send + Sync {
    fn observe(&self, event: CapabilityAuditEventV1);
}

/// Monotonic-enough host clock expressed in Unix milliseconds for deadlines and audit.
pub trait DispatcherClock: Send + Sync {
    fn now_unix_ms(&self) -> u64;
}

/// Runtime-neutral dispatcher with an in-process concurrency gate.
pub struct CapabilityDispatcher<S, P, I, O, C> {
    store: S,
    policy: P,
    invoker: I,
    observer: O,
    clock: C,
    active: Mutex<BTreeMap<BindingId, u32>>,
}

impl<S, P, I, O, C> CapabilityDispatcher<S, P, I, O, C>
where
    S: BindingStore,
    P: PolicyEvaluator,
    I: ProviderInvoker,
    O: CapabilityObserver,
    C: DispatcherClock,
{
    pub fn new(store: S, policy: P, invoker: I, observer: O, clock: C) -> Self {
        Self {
            store,
            policy,
            invoker,
            observer,
            clock,
            active: Mutex::new(BTreeMap::new()),
        }
    }

    /// Resolves, authorizes, validates, maps, invokes, validates, and audits one request.
    pub fn invoke(
        &self,
        request: CapabilityInvocationRequestV1,
        context: &InvocationContextV1,
    ) -> Result<CapabilityInvocationResponseV1, CapabilityErrorV1> {
        let started = self.clock.now_unix_ms();
        let result = self.invoke_inner(&request, context, started);
        let (capability, outcome) = match &result {
            Ok((capability, _)) => (Some(capability.clone()), "success".to_string()),
            Err((capability, error)) => (capability.clone(), error_name(*error).to_string()),
        };
        self.observer.observe(CapabilityAuditEventV1 {
            correlation_id: context.correlation_id.clone(),
            binding_id: request.binding_id.clone(),
            capability,
            operation: request.operation.clone(),
            tenant: context.tenant.clone(),
            environment: context.environment.clone(),
            team: context.team.clone(),
            outcome,
            elapsed_ms: self.clock.now_unix_ms().saturating_sub(started),
        });
        result
            .map(|(_, response)| response)
            .map_err(|(_, error)| error)
    }

    fn invoke_inner(
        &self,
        request: &CapabilityInvocationRequestV1,
        context: &InvocationContextV1,
        now_unix_ms: u64,
    ) -> Result<
        (crate::CapabilityId, CapabilityInvocationResponseV1),
        (Option<crate::CapabilityId>, CapabilityErrorV1),
    > {
        if context.cancelled {
            return Err((None, CapabilityErrorV1::Cancelled));
        }
        let stored = self
            .store
            .get(&request.binding_id)
            .map_err(|_| (None, CapabilityErrorV1::Internal))?
            .ok_or((None, CapabilityErrorV1::BindingNotFound))?;
        let binding = stored.binding;
        let fail = |error| (Some(binding.capability.clone()), error);
        binding
            .validate()
            .map_err(|_| fail(CapabilityErrorV1::ProtocolError))?;
        if !matches!(stored.status, BindingStatusV1::Active) {
            return Err(fail(CapabilityErrorV1::BindingNotFound));
        }
        if binding
            .expires_at_unix_ms
            .is_some_and(|expiry| now_unix_ms >= expiry)
        {
            return Err(fail(CapabilityErrorV1::BindingNotFound));
        }
        if binding.scope.tenant != context.tenant
            || binding.scope.environment != context.environment
            || binding.scope.team != context.team
        {
            return Err(fail(CapabilityErrorV1::BindingNotFound));
        }
        if context
            .dispatch_chain
            .iter()
            .any(|id| id == &request.binding_id)
        {
            return Err(fail(CapabilityErrorV1::PolicyDenied));
        }
        if context.dispatch_chain.len() >= binding.limits.max_dispatch_depth as usize {
            return Err(fail(CapabilityErrorV1::PolicyDenied));
        }
        let operation = binding
            .operations
            .get(&request.operation)
            .ok_or_else(|| fail(CapabilityErrorV1::OperationNotAllowed))?;
        if request.payload_cbor.len() as u64 > binding.limits.max_request_bytes {
            return Err(fail(CapabilityErrorV1::InvalidInput));
        }
        let input = decode_canonical_bounded(&request.payload_cbor, binding.limits.max_cbor_depth)
            .map_err(|_| fail(CapabilityErrorV1::InvalidInput))?;
        validate_schema(&operation.input_schema, &input)
            .map_err(|_| fail(CapabilityErrorV1::InvalidInput))?;
        let decision = self
            .policy
            .evaluate(&binding, &request.operation, context)
            .map_err(|_| fail(CapabilityErrorV1::PolicyDenied))?;
        if !decision.allowed || decision.evaluated_version != binding.policy.version {
            return Err(fail(CapabilityErrorV1::PolicyDenied));
        }
        let own_deadline = now_unix_ms.saturating_add(binding.limits.timeout_ms);
        let deadline = context
            .deadline_unix_ms
            .map_or(own_deadline, |outer| outer.min(own_deadline));
        if deadline <= now_unix_ms {
            return Err(fail(CapabilityErrorV1::Timeout));
        }
        let _permit = self.acquire(&binding).map_err(&fail)?;
        let mut chain = context.dispatch_chain.clone();
        chain.push(request.binding_id.clone());
        let provider_result = self
            .invoker
            .invoke(ProviderInvocationV1 {
                provider_component_ref: binding.provider_component_ref.clone(),
                provider_digest: binding.provider_digest.clone(),
                component_operation: operation.component_operation.clone(),
                payload_cbor: request.payload_cbor.clone(),
                tenant: context.tenant.clone(),
                environment: context.environment.clone(),
                team: context.team.clone(),
                correlation_id: context.correlation_id.clone(),
                deadline_unix_ms: deadline,
                dispatch_chain: chain,
            })
            .map_err(|error| fail(map_provider_error(error)))?;
        if self.clock.now_unix_ms() > deadline {
            return Err(fail(CapabilityErrorV1::Timeout));
        }
        if provider_result.output_cbor.len() as u64 > binding.limits.max_response_bytes
            || provider_result
                .metadata_cbor
                .as_ref()
                .is_some_and(|data| data.len() as u64 > binding.limits.max_metadata_bytes)
        {
            return Err(fail(CapabilityErrorV1::InvalidOutput));
        }
        let output =
            decode_canonical_bounded(&provider_result.output_cbor, binding.limits.max_cbor_depth)
                .map_err(|_| fail(CapabilityErrorV1::InvalidOutput))?;
        validate_schema(&operation.output_schema, &output)
            .map_err(|_| fail(CapabilityErrorV1::InvalidOutput))?;
        if let Some(metadata) = &provider_result.metadata_cbor {
            decode_canonical_bounded(metadata, binding.limits.max_cbor_depth)
                .map_err(|_| fail(CapabilityErrorV1::InvalidOutput))?;
        }
        Ok((
            binding.capability.clone(),
            CapabilityInvocationResponseV1 {
                output_cbor: provider_result.output_cbor,
                metadata_cbor: provider_result.metadata_cbor,
            },
        ))
    }

    fn acquire(
        &self,
        binding: &ExecutableBindingV1,
    ) -> Result<ConcurrencyPermit<'_>, CapabilityErrorV1> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| CapabilityErrorV1::Internal)?;
        let count = active.entry(binding.binding_id.clone()).or_default();
        if *count >= binding.limits.max_concurrency {
            return Err(CapabilityErrorV1::Overloaded);
        }
        *count += 1;
        Ok(ConcurrencyPermit {
            active: &self.active,
            binding_id: binding.binding_id.clone(),
        })
    }
}

struct ConcurrencyPermit<'a> {
    active: &'a Mutex<BTreeMap<BindingId, u32>>,
    binding_id: BindingId,
}

impl Drop for ConcurrencyPermit<'_> {
    fn drop(&mut self) {
        if let Ok(mut active) = self.active.lock()
            && let Some(count) = active.get_mut(&self.binding_id)
        {
            *count = count.saturating_sub(1);
            if *count == 0 {
                active.remove(&self.binding_id);
            }
        }
    }
}

fn decode_canonical_bounded(bytes: &[u8], max_depth: u32) -> Result<serde_json::Value, ()> {
    let value: CborValue = serde_cbor::from_slice(bytes).map_err(|_| ())?;
    if cbor_depth(&value, 1) > max_depth {
        return Err(());
    }
    // Reject indefinite, non-minimal, or non-deterministically ordered encodings.
    if serde_cbor::to_vec(&value).map_err(|_| ())? != bytes {
        return Err(());
    }
    serde_json::to_value(value).map_err(|_| ())
}

fn cbor_depth(value: &CborValue, depth: u32) -> u32 {
    match value {
        CborValue::Array(values) => values
            .iter()
            .map(|value| cbor_depth(value, depth.saturating_add(1)))
            .max()
            .unwrap_or(depth),
        CborValue::Map(values) => values
            .iter()
            .flat_map(|(key, value)| {
                [
                    cbor_depth(key, depth.saturating_add(1)),
                    cbor_depth(value, depth.saturating_add(1)),
                ]
            })
            .max()
            .unwrap_or(depth),
        CborValue::Tag(_, value) => cbor_depth(value, depth.saturating_add(1)),
        _ => depth,
    }
}

fn validate_schema(schema: &serde_json::Value, instance: &serde_json::Value) -> Result<(), ()> {
    let validator = jsonschema::validator_for(schema).map_err(|_| ())?;
    validator.validate(instance).map_err(|_| ())
}

fn map_provider_error(error: ProviderInvokeError) -> CapabilityErrorV1 {
    match error {
        ProviderInvokeError::Unavailable(_) => CapabilityErrorV1::ProviderUnavailable,
        ProviderInvokeError::Timeout(_) => CapabilityErrorV1::Timeout,
        ProviderInvokeError::Cancelled(_) => CapabilityErrorV1::Cancelled,
        ProviderInvokeError::Overloaded(_) => CapabilityErrorV1::Overloaded,
        ProviderInvokeError::Protocol(_) => CapabilityErrorV1::ProtocolError,
        ProviderInvokeError::Internal(_) => CapabilityErrorV1::Internal,
    }
}

fn error_name(error: CapabilityErrorV1) -> &'static str {
    match error {
        CapabilityErrorV1::BindingNotFound => "binding_not_found",
        CapabilityErrorV1::OperationNotAllowed => "operation_not_allowed",
        CapabilityErrorV1::ProviderUnavailable => "provider_unavailable",
        CapabilityErrorV1::InvalidInput => "invalid_input",
        CapabilityErrorV1::InvalidOutput => "invalid_output",
        CapabilityErrorV1::PolicyDenied => "policy_denied",
        CapabilityErrorV1::Timeout => "timeout",
        CapabilityErrorV1::Cancelled => "cancelled",
        CapabilityErrorV1::Overloaded => "overloaded",
        CapabilityErrorV1::ProtocolError => "protocol_error",
        CapabilityErrorV1::Internal => "internal",
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use greentic_cap_types::{
        CapabilityBinding, CapabilityBindingKind, CapabilityId, CapabilityProviderOperationMap,
        CapabilityProviderRef,
    };
    use serde_json::json;

    use super::*;
    use crate::{BindingLimitsV1, BindingPolicyV1, BindingScopeV1, ExecutableBindingInputV1};

    #[derive(Clone)]
    struct Store(Arc<Mutex<Option<StoredBindingV1>>>);
    impl BindingStore for Store {
        fn get(&self, _: &BindingId) -> Result<Option<StoredBindingV1>, BindingStoreError> {
            Ok(self.0.lock().unwrap().clone())
        }
    }

    #[derive(Clone)]
    struct Policy(Arc<Mutex<PolicyDecisionV1>>);
    impl PolicyEvaluator for Policy {
        fn evaluate(
            &self,
            _: &ExecutableBindingV1,
            _: &str,
            _: &InvocationContextV1,
        ) -> Result<PolicyDecisionV1, PolicyEvaluationError> {
            Ok(self.0.lock().unwrap().clone())
        }
    }

    #[derive(Clone)]
    struct Invoker {
        result: Arc<Mutex<Result<ProviderOutputV1, ProviderInvokeError>>>,
        calls: Arc<Mutex<Vec<ProviderInvocationV1>>>,
    }
    impl ProviderInvoker for Invoker {
        fn invoke(
            &self,
            request: ProviderInvocationV1,
        ) -> Result<ProviderOutputV1, ProviderInvokeError> {
            self.calls.lock().unwrap().push(request);
            self.result.lock().unwrap().clone()
        }
    }

    #[derive(Clone, Default)]
    struct Observer(Arc<Mutex<Vec<CapabilityAuditEventV1>>>);
    impl CapabilityObserver for Observer {
        fn observe(&self, event: CapabilityAuditEventV1) {
            self.0.lock().unwrap().push(event);
        }
    }

    #[derive(Clone)]
    struct Clock(Arc<Mutex<u64>>);
    impl DispatcherClock for Clock {
        fn now_unix_ms(&self) -> u64 {
            *self.0.lock().unwrap()
        }
    }

    type TestDispatcher = CapabilityDispatcher<Store, Policy, Invoker, Observer, Clock>;

    struct Fixture {
        dispatcher: TestDispatcher,
        binding: ExecutableBindingV1,
        store: Store,
        policy: Policy,
        invoker: Invoker,
        observer: Observer,
        clock: Clock,
    }

    fn cbor(value: serde_json::Value) -> Vec<u8> {
        serde_cbor::to_vec(&value).unwrap()
    }

    fn make_binding() -> ExecutableBindingV1 {
        let digest = format!("sha256:{}", "b".repeat(64));
        let mut legacy = CapabilityBinding::new(
            CapabilityBindingKind::Requirement,
            "echo.required",
            "echo.offer",
            CapabilityId::new("cap://example.echo@1").unwrap(),
        );
        legacy.provider = Some(CapabilityProviderRef {
            component_ref: format!("oci://example/echo@{digest}"),
            operation: String::new(),
            operation_map: vec![
                CapabilityProviderOperationMap {
                    contract_operation: "echo".into(),
                    component_operation: "component-echo".into(),
                    input_schema: json!({
                        "type": "object",
                        "required": ["message"],
                        "properties": {"message": {"type": "string"}},
                        "additionalProperties": false
                    }),
                    output_schema: json!({
                        "type": "object",
                        "required": ["message"],
                        "properties": {"message": {"type": "string"}},
                        "additionalProperties": false
                    }),
                },
                CapabilityProviderOperationMap {
                    contract_operation: "health".into(),
                    component_operation: "component-health".into(),
                    input_schema: json!({"type": "null"}),
                    output_schema: json!({"type": "boolean"}),
                },
            ],
        });
        ExecutableBindingV1::from_resolved(
            &legacy,
            ExecutableBindingInputV1 {
                contract_version: "1.0.0".into(),
                provider_digest: digest,
                scope: BindingScopeV1 {
                    tenant: "tenant-a".into(),
                    environment: "prod".into(),
                    team: Some("team-a".into()),
                    profile: None,
                },
                policy: BindingPolicyV1 {
                    reference: "policy://echo".into(),
                    version: "policy-v1".into(),
                },
                limits: BindingLimitsV1 {
                    max_request_bytes: 256,
                    max_response_bytes: 256,
                    max_metadata_bytes: 64,
                    ..BindingLimitsV1::default()
                },
                expires_at_unix_ms: Some(10_000),
                revocation_id: Some("revoke-1".into()),
                resolution_id: "resolve-1".into(),
            },
        )
        .unwrap()
    }

    fn fixture() -> Fixture {
        let binding = make_binding();
        let store = Store(Arc::new(Mutex::new(Some(StoredBindingV1 {
            binding: binding.clone(),
            status: BindingStatusV1::Active,
        }))));
        let policy = Policy(Arc::new(Mutex::new(PolicyDecisionV1 {
            allowed: true,
            evaluated_version: "policy-v1".into(),
        })));
        let invoker = Invoker {
            result: Arc::new(Mutex::new(Ok(ProviderOutputV1 {
                output_cbor: cbor(json!({"message": "hello"})),
                metadata_cbor: Some(cbor(json!({"cached": false}))),
            }))),
            calls: Arc::default(),
        };
        let observer = Observer::default();
        let clock = Clock(Arc::new(Mutex::new(1_000)));
        let dispatcher = CapabilityDispatcher::new(
            store.clone(),
            policy.clone(),
            invoker.clone(),
            observer.clone(),
            clock.clone(),
        );
        Fixture {
            dispatcher,
            binding,
            store,
            policy,
            invoker,
            observer,
            clock,
        }
    }

    fn context() -> InvocationContextV1 {
        InvocationContextV1 {
            tenant: "tenant-a".into(),
            environment: "prod".into(),
            team: Some("team-a".into()),
            correlation_id: "corr-1".into(),
            deadline_unix_ms: None,
            cancelled: false,
            dispatch_chain: Vec::new(),
        }
    }

    fn request(binding: &ExecutableBindingV1) -> CapabilityInvocationRequestV1 {
        CapabilityInvocationRequestV1 {
            binding_id: binding.binding_id.clone(),
            operation: "echo".into(),
            payload_cbor: cbor(json!({"message": "hello"})),
        }
    }

    #[test]
    fn dispatches_only_the_pinned_mapped_operation_and_propagates_context() {
        let fixture = fixture();
        let response = fixture
            .dispatcher
            .invoke(request(&fixture.binding), &context())
            .unwrap();
        assert_eq!(response.output_cbor, cbor(json!({"message": "hello"})));
        let calls = fixture.invoker.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].component_operation, "component-echo");
        assert_eq!(calls[0].provider_digest, fixture.binding.provider_digest);
        assert_eq!(calls[0].correlation_id, "corr-1");
        assert_eq!(
            calls[0].dispatch_chain,
            vec![fixture.binding.binding_id.clone()]
        );
        let events = fixture.observer.0.lock().unwrap();
        assert_eq!(events[0].outcome, "success");
        assert_eq!(events[0].correlation_id, "corr-1");
    }

    #[test]
    fn missing_unknown_operation_and_scope_mismatch_fail_before_provider() {
        let fixture = fixture();
        *fixture.store.0.lock().unwrap() = None;
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &context()),
            Err(CapabilityErrorV1::BindingNotFound)
        );
        *fixture.store.0.lock().unwrap() = Some(StoredBindingV1 {
            binding: fixture.binding.clone(),
            status: BindingStatusV1::Active,
        });
        let mut unknown = request(&fixture.binding);
        unknown.operation = "admin".into();
        assert_eq!(
            fixture.dispatcher.invoke(unknown, &context()),
            Err(CapabilityErrorV1::OperationNotAllowed)
        );
        let mut wrong_scope = context();
        wrong_scope.tenant = "tenant-b".into();
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &wrong_scope),
            Err(CapabilityErrorV1::BindingNotFound)
        );
        assert!(fixture.invoker.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn revoked_expired_policy_drift_cycles_and_deadlines_fail_closed() {
        let fixture = fixture();
        *fixture.store.0.lock().unwrap() = Some(StoredBindingV1 {
            binding: fixture.binding.clone(),
            status: BindingStatusV1::Revoked {
                revoked_at_unix_ms: 2,
                reason_code: "security".into(),
            },
        });
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &context()),
            Err(CapabilityErrorV1::BindingNotFound)
        );
        *fixture.store.0.lock().unwrap() = Some(StoredBindingV1 {
            binding: fixture.binding.clone(),
            status: BindingStatusV1::Active,
        });
        *fixture.clock.0.lock().unwrap() = 10_000;
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &context()),
            Err(CapabilityErrorV1::BindingNotFound)
        );
        *fixture.clock.0.lock().unwrap() = 1_000;
        fixture.policy.0.lock().unwrap().evaluated_version = "policy-v2".into();
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &context()),
            Err(CapabilityErrorV1::PolicyDenied)
        );
        fixture.policy.0.lock().unwrap().evaluated_version = "policy-v1".into();
        let mut cycle = context();
        cycle
            .dispatch_chain
            .push(fixture.binding.binding_id.clone());
        assert_eq!(
            fixture.dispatcher.invoke(request(&fixture.binding), &cycle),
            Err(CapabilityErrorV1::PolicyDenied)
        );
        let mut timed_out = context();
        timed_out.deadline_unix_ms = Some(1_000);
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &timed_out),
            Err(CapabilityErrorV1::Timeout)
        );
        assert!(fixture.invoker.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn rejects_malformed_noncanonical_deep_oversized_and_schema_invalid_input() {
        let fixture = fixture();
        let mut deep = json!(0);
        for _ in 0..40 {
            deep = json!([deep]);
        }
        for payload in [
            vec![0xff],
            vec![0x18, 0x01], // valid but non-minimal integer encoding
            cbor(deep),
            vec![0; 257],
            cbor(json!({"wrong": true})),
        ] {
            let mut invalid = request(&fixture.binding);
            invalid.payload_cbor = payload;
            assert_eq!(
                fixture.dispatcher.invoke(invalid, &context()),
                Err(CapabilityErrorV1::InvalidInput)
            );
        }
        assert!(fixture.invoker.calls.lock().unwrap().is_empty());
    }

    #[test]
    fn rejects_bad_provider_output_and_redacts_provider_errors() {
        let fixture = fixture();
        *fixture.invoker.result.lock().unwrap() = Ok(ProviderOutputV1 {
            output_cbor: cbor(json!({"secret": "leak"})),
            metadata_cbor: None,
        });
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &context()),
            Err(CapabilityErrorV1::InvalidOutput)
        );
        *fixture.invoker.result.lock().unwrap() =
            Err(ProviderInvokeError::Unavailable("credential secret".into()));
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &context()),
            Err(CapabilityErrorV1::ProviderUnavailable)
        );
        let events = fixture.observer.0.lock().unwrap();
        assert!(events.iter().all(|event| !event.outcome.contains("secret")));
    }

    #[test]
    fn cancellation_and_concurrency_overload_are_safe() {
        let fixture = fixture();
        let mut cancelled = context();
        cancelled.cancelled = true;
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &cancelled),
            Err(CapabilityErrorV1::Cancelled)
        );
        fixture.dispatcher.active.lock().unwrap().insert(
            fixture.binding.binding_id.clone(),
            fixture.binding.limits.max_concurrency,
        );
        assert_eq!(
            fixture
                .dispatcher
                .invoke(request(&fixture.binding), &context()),
            Err(CapabilityErrorV1::Overloaded)
        );
    }
}
