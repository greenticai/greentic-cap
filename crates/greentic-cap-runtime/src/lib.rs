//! Provider-neutral data-plane contracts for resolved Greentic capabilities.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use greentic_cap_types::{CapabilityBinding, CapabilityId, CapabilityProviderOperationMap};
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod dispatcher;
pub use dispatcher::*;

/// Version of the executable binding serialization contract.
pub const EXECUTABLE_BINDING_SCHEMA_VERSION: u32 = 1;

/// An opaque identifier safe to disclose to a consumer.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct BindingId(String);

impl BindingId {
    /// Parses an opaque binding identifier received through the capability ABI.
    pub fn parse(value: impl Into<String>) -> Result<Self, BindingError> {
        let value = value.into();
        let suffix = value
            .strip_prefix("capb_")
            .ok_or(BindingError::InvalidBindingId)?;
        if suffix.len() != 32
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(BindingError::InvalidBindingId);
        }
        Ok(Self(value))
    }

    /// Returns the opaque identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Host-owned deployment scope. Empty tenant and environment values are invalid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BindingScopeV1 {
    pub tenant: String,
    pub environment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
}

/// A policy snapshot checked again by the host at invocation time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BindingPolicyV1 {
    pub reference: String,
    pub version: String,
}

/// Fail-closed resource limits captured at resolution time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BindingLimitsV1 {
    pub timeout_ms: u64,
    pub max_request_bytes: u64,
    pub max_response_bytes: u64,
    pub max_metadata_bytes: u64,
    pub max_cbor_depth: u32,
    pub max_schema_nodes: u32,
    pub max_concurrency: u32,
    pub max_dispatch_depth: u32,
}

impl Default for BindingLimitsV1 {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            max_request_bytes: 1_048_576,
            max_response_bytes: 1_048_576,
            max_metadata_bytes: 16_384,
            max_cbor_depth: 32,
            max_schema_nodes: 4_096,
            max_concurrency: 16,
            max_dispatch_depth: 8,
        }
    }
}

/// One immutable logical-to-component operation mapping.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutableOperationV1 {
    pub component_operation: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

/// Immutable executable state installed by bundle/setup resolution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutableBindingV1 {
    pub schema_version: u32,
    pub binding_id: BindingId,
    pub binding_digest: String,
    pub capability: CapabilityId,
    pub contract_version: String,
    /// Digest-pinned ordinary Greentic component reference.
    pub provider_component_ref: String,
    pub provider_digest: String,
    pub operations: BTreeMap<String, ExecutableOperationV1>,
    pub scope: BindingScopeV1,
    pub policy: BindingPolicyV1,
    pub limits: BindingLimitsV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revocation_id: Option<String>,
    pub resolution_id: String,
}

/// Trusted inputs used to promote a legacy resolver binding to executable state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableBindingInputV1 {
    pub contract_version: String,
    pub provider_digest: String,
    pub scope: BindingScopeV1,
    pub policy: BindingPolicyV1,
    pub limits: BindingLimitsV1,
    pub expires_at_unix_ms: Option<u64>,
    pub revocation_id: Option<String>,
    pub resolution_id: String,
}

/// Binding lifecycle state held in trusted storage, separate from the immutable record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BindingStatusV1 {
    Active,
    Revoked {
        revoked_at_unix_ms: u64,
        reason_code: String,
    },
}

/// ABI-equivalent request for hosts that do not consume generated WIT bindings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityInvocationRequestV1 {
    pub binding_id: BindingId,
    pub operation: String,
    pub payload_cbor: Vec<u8>,
}

/// ABI-equivalent response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityInvocationResponseV1 {
    pub output_cbor: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_cbor: Option<Vec<u8>>,
}

/// Stable safe errors exposed across the guest boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityErrorV1 {
    BindingNotFound,
    OperationNotAllowed,
    ProviderUnavailable,
    InvalidInput,
    InvalidOutput,
    PolicyDenied,
    Timeout,
    Cancelled,
    Overloaded,
    ProtocolError,
    Internal,
}

/// Redacted audit input. It deliberately contains no payload or provider error text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityAuditEventV1 {
    pub correlation_id: String,
    pub binding_id: BindingId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<CapabilityId>,
    pub operation: String,
    pub tenant: String,
    pub environment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    pub outcome: String,
    pub elapsed_ms: u64,
}

/// Construction and artifact validation failures.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BindingError {
    #[error("invalid opaque binding id")]
    InvalidBindingId,
    #[error("legacy binding has no provider")]
    MissingProvider,
    #[error("executable provider reference is not pinned to the supplied sha256 digest")]
    MutableProviderReference,
    #[error("{0} must not be empty")]
    Empty(&'static str),
    #[error("operation map must not be empty")]
    EmptyOperationMap,
    #[error("duplicate logical operation {0}")]
    DuplicateOperation(String),
    #[error("invalid sha256 digest")]
    InvalidDigest,
    #[error("all runtime limits must be greater than zero")]
    InvalidLimits,
    #[error("operation schema exceeds the configured complexity limit")]
    SchemaTooComplex,
    #[error("binding digest or id does not match its contents")]
    Integrity,
    #[error("unsupported executable binding schema version {0}")]
    UnsupportedVersion(u32),
    #[error("CBOR encode failed: {0}")]
    Encode(String),
    #[error("CBOR decode failed: {0}")]
    Decode(String),
}

#[derive(Serialize)]
struct BindingDigestMaterial<'a> {
    schema_version: u32,
    capability: &'a CapabilityId,
    contract_version: &'a str,
    provider_component_ref: &'a str,
    provider_digest: &'a str,
    operations: &'a BTreeMap<String, ExecutableOperationV1>,
    scope: &'a BindingScopeV1,
    policy: &'a BindingPolicyV1,
    limits: &'a BindingLimitsV1,
    expires_at_unix_ms: Option<u64>,
    revocation_id: &'a Option<String>,
    resolution_id: &'a str,
}

impl ExecutableBindingV1 {
    /// Promotes an existing deterministic resolver result using host-supplied immutable data.
    pub fn from_resolved(
        binding: &CapabilityBinding,
        input: ExecutableBindingInputV1,
    ) -> Result<Self, BindingError> {
        let provider = binding
            .provider
            .as_ref()
            .ok_or(BindingError::MissingProvider)?;
        validate_digest(&input.provider_digest)?;
        if !reference_is_pinned(&provider.component_ref, &input.provider_digest) {
            return Err(BindingError::MutableProviderReference);
        }
        let operations = operation_map(&provider.operation_map)?;
        let mut value = Self {
            schema_version: EXECUTABLE_BINDING_SCHEMA_VERSION,
            binding_id: BindingId(String::new()),
            binding_digest: String::new(),
            capability: binding.capability.clone(),
            contract_version: input.contract_version,
            provider_component_ref: provider.component_ref.clone(),
            provider_digest: input.provider_digest.to_ascii_lowercase(),
            operations,
            scope: input.scope,
            policy: input.policy,
            limits: input.limits,
            expires_at_unix_ms: input.expires_at_unix_ms,
            revocation_id: input.revocation_id,
            resolution_id: input.resolution_id,
        };
        value.validate_fields()?;
        value.binding_digest = value.calculate_digest()?;
        value.binding_id = BindingId(format!("capb_{}", &value.binding_digest[7..39]));
        Ok(value)
    }

    /// Validates fields and cryptographic integrity.
    pub fn validate(&self) -> Result<(), BindingError> {
        if self.schema_version != EXECUTABLE_BINDING_SCHEMA_VERSION {
            return Err(BindingError::UnsupportedVersion(self.schema_version));
        }
        self.validate_fields()?;
        let digest = self.calculate_digest()?;
        if digest != self.binding_digest || self.binding_id.0 != format!("capb_{}", &digest[7..39])
        {
            return Err(BindingError::Integrity);
        }
        Ok(())
    }

    fn validate_fields(&self) -> Result<(), BindingError> {
        for (name, value) in [
            ("contract_version", self.contract_version.as_str()),
            ("tenant", self.scope.tenant.as_str()),
            ("environment", self.scope.environment.as_str()),
            ("policy reference", self.policy.reference.as_str()),
            ("policy version", self.policy.version.as_str()),
            ("resolution_id", self.resolution_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(BindingError::Empty(name));
            }
        }
        validate_digest(&self.provider_digest)?;
        if self.operations.is_empty() {
            return Err(BindingError::EmptyOperationMap);
        }
        if self.operations.iter().any(|(logical, op)| {
            logical.trim().is_empty() || op.component_operation.trim().is_empty()
        }) {
            return Err(BindingError::Empty("operation"));
        }
        let l = &self.limits;
        if [
            l.timeout_ms,
            l.max_request_bytes,
            l.max_response_bytes,
            l.max_metadata_bytes,
        ]
        .contains(&0)
            || [
                l.max_cbor_depth,
                l.max_schema_nodes,
                l.max_concurrency,
                l.max_dispatch_depth,
            ]
            .contains(&0)
        {
            return Err(BindingError::InvalidLimits);
        }
        if self.operations.values().any(|operation| {
            json_node_count(&operation.input_schema) > l.max_schema_nodes as usize
                || json_node_count(&operation.output_schema) > l.max_schema_nodes as usize
        }) {
            return Err(BindingError::SchemaTooComplex);
        }
        Ok(())
    }

    fn calculate_digest(&self) -> Result<String, BindingError> {
        let material = BindingDigestMaterial {
            schema_version: self.schema_version,
            capability: &self.capability,
            contract_version: &self.contract_version,
            provider_component_ref: &self.provider_component_ref,
            provider_digest: &self.provider_digest,
            operations: &self.operations,
            scope: &self.scope,
            policy: &self.policy,
            limits: &self.limits,
            expires_at_unix_ms: self.expires_at_unix_ms,
            revocation_id: &self.revocation_id,
            resolution_id: &self.resolution_id,
        };
        let bytes =
            serde_cbor::to_vec(&material).map_err(|e| BindingError::Encode(e.to_string()))?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }
}

fn operation_map(
    entries: &[CapabilityProviderOperationMap],
) -> Result<BTreeMap<String, ExecutableOperationV1>, BindingError> {
    if entries.is_empty() {
        return Err(BindingError::EmptyOperationMap);
    }
    let mut result = BTreeMap::new();
    for entry in entries {
        let value = ExecutableOperationV1 {
            component_operation: entry.component_operation.clone(),
            input_schema: entry.input_schema.clone(),
            output_schema: entry.output_schema.clone(),
        };
        if result
            .insert(entry.contract_operation.clone(), value)
            .is_some()
        {
            return Err(BindingError::DuplicateOperation(
                entry.contract_operation.clone(),
            ));
        }
    }
    Ok(result)
}

fn validate_digest(value: &str) -> Result<(), BindingError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(BindingError::InvalidDigest);
    };
    if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(BindingError::InvalidDigest);
    }
    Ok(())
}

fn reference_is_pinned(reference: &str, digest: &str) -> bool {
    reference
        .to_ascii_lowercase()
        .contains(&digest.to_ascii_lowercase())
}

fn json_node_count(value: &serde_json::Value) -> usize {
    let mut count = 0usize;
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        count = count.saturating_add(1);
        match value {
            serde_json::Value::Array(values) => pending.extend(values),
            serde_json::Value::Object(values) => pending.extend(values.values()),
            _ => {}
        }
    }
    count
}

/// Deterministically encodes and validates an executable binding.
pub fn encode_executable_binding(binding: &ExecutableBindingV1) -> Result<Vec<u8>, BindingError> {
    binding.validate()?;
    serde_cbor::to_vec(binding).map_err(|e| BindingError::Encode(e.to_string()))
}

/// Decodes and verifies an executable binding before returning it.
pub fn decode_executable_binding(bytes: &[u8]) -> Result<ExecutableBindingV1, BindingError> {
    let binding: ExecutableBindingV1 =
        serde_cbor::from_slice(bytes).map_err(|e| BindingError::Decode(e.to_string()))?;
    binding.validate()?;
    Ok(binding)
}

pub fn executable_binding_schema() -> Schema {
    schemars::schema_for!(ExecutableBindingV1)
}
pub fn invocation_request_schema() -> Schema {
    schemars::schema_for!(CapabilityInvocationRequestV1)
}
pub fn invocation_response_schema() -> Schema {
    schemars::schema_for!(CapabilityInvocationResponseV1)
}
pub fn binding_status_schema() -> Schema {
    schemars::schema_for!(BindingStatusV1)
}
pub fn audit_event_schema() -> Schema {
    schemars::schema_for!(CapabilityAuditEventV1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use greentic_cap_types::{CapabilityBindingKind, CapabilityProviderRef};
    use serde_json::json;

    fn binding() -> CapabilityBinding {
        let digest = format!("sha256:{}", "a".repeat(64));
        let mut binding = CapabilityBinding::new(
            CapabilityBindingKind::Requirement,
            "echo.required",
            "echo.offer",
            CapabilityId::new("cap://example.echo@1").unwrap(),
        );
        binding.provider = Some(CapabilityProviderRef {
            component_ref: format!("oci://registry/example/echo@{digest}"),
            operation: "legacy".into(),
            operation_map: vec![
                CapabilityProviderOperationMap {
                    contract_operation: "echo".into(),
                    component_operation: "invoke-echo".into(),
                    input_schema: json!({"type":"object","required":["message"]}),
                    output_schema: json!({"type":"object","required":["message"]}),
                },
                CapabilityProviderOperationMap {
                    contract_operation: "health".into(),
                    component_operation: "health".into(),
                    input_schema: json!({"type":"null"}),
                    output_schema: json!({"type":"boolean"}),
                },
            ],
        });
        binding
    }

    fn input() -> ExecutableBindingInputV1 {
        ExecutableBindingInputV1 {
            contract_version: "1.0.0".into(),
            provider_digest: format!("sha256:{}", "a".repeat(64)),
            scope: BindingScopeV1 {
                tenant: "tenant-a".into(),
                environment: "prod".into(),
                team: Some("team-a".into()),
                profile: Some("default".into()),
            },
            policy: BindingPolicyV1 {
                reference: "policy://echo".into(),
                version: "sha256:policy-v1".into(),
            },
            limits: BindingLimitsV1::default(),
            expires_at_unix_ms: None,
            revocation_id: Some("echo-binding-1".into()),
            resolution_id: "resolution-1".into(),
        }
    }

    #[test]
    fn deterministic_id_digest_and_round_trip() {
        let first = ExecutableBindingV1::from_resolved(&binding(), input()).unwrap();
        let second = ExecutableBindingV1::from_resolved(&binding(), input()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.operations.len(), 2);
        let bytes = encode_executable_binding(&first).unwrap();
        assert_eq!(decode_executable_binding(&bytes).unwrap(), first);
    }

    #[test]
    fn policy_and_limits_are_integrity_protected() {
        let original = ExecutableBindingV1::from_resolved(&binding(), input()).unwrap();
        let mut changed_input = input();
        changed_input.policy.version = "sha256:policy-v2".into();
        let changed = ExecutableBindingV1::from_resolved(&binding(), changed_input).unwrap();
        assert_ne!(original.binding_digest, changed.binding_digest);
        let mut tampered = original;
        tampered.limits.timeout_ms += 1;
        assert_eq!(tampered.validate(), Err(BindingError::Integrity));
    }

    #[test]
    fn mutable_provider_fails_closed() {
        let mut legacy = binding();
        legacy.provider.as_mut().unwrap().component_ref =
            "oci://registry/example/echo:latest".into();
        assert_eq!(
            ExecutableBindingV1::from_resolved(&legacy, input()),
            Err(BindingError::MutableProviderReference)
        );
    }

    #[test]
    fn wit_contract_is_versioned_and_separate() {
        let wit = include_str!("../../../wit/greentic/cap-runtime@1.0.0/package.wit");
        assert!(wit.contains("package greentic:cap-runtime@1.0.0;"));
        assert!(wit.contains("import capability-client;"));
        assert!(!wit.contains("greentic:component"));
        assert_eq!(
            wit,
            include_str!("../../greentic-cap-guest/wit/package.wit"),
            "published guest WIT must match the canonical package"
        );
        assert_eq!(
            wit,
            include_str!("../../greentic-cap-wasmtime/wit/package.wit"),
            "published host WIT must match the canonical package"
        );
    }
}
