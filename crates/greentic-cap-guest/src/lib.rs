//! Guest-side bindings for `greentic:cap-runtime@1.0.0`.

#![forbid(unsafe_code)]

#[cfg(target_arch = "wasm32")]
wit_bindgen::generate!({
    path: "wit",
    world: "capability-consumer",
    generate_all,
});

/// Invoke a resolved capability using only its opaque binding ID and logical operation.
#[cfg(target_arch = "wasm32")]
pub fn invoke(
    binding_id: impl Into<String>,
    operation: impl Into<String>,
    payload_cbor: Vec<u8>,
) -> Result<
    greentic::cap_runtime::capability_client::Response,
    greentic::cap_runtime::capability_client::CapabilityError,
> {
    greentic::cap_runtime::capability_client::invoke(
        &greentic::cap_runtime::capability_client::Request {
            binding_id: binding_id.into(),
            operation: operation.into(),
            payload_cbor,
        },
    )
}
