//! Wasmtime-specific adapter for the runtime-neutral capability dispatcher.

#![forbid(unsafe_code)]

mod bindings {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "capability-consumer",
    });
}

use greentic_cap_runtime::{
    BindingId, CapabilityErrorV1, CapabilityInvocationRequestV1, CapabilityInvocationResponseV1,
};

pub use bindings::greentic::cap_runtime::capability_client::{CapabilityError, Request, Response};

/// Object-safe boundary implemented by host state around a configured dispatcher.
pub trait CapabilityService: Send {
    fn invoke(
        &mut self,
        request: CapabilityInvocationRequestV1,
    ) -> Result<CapabilityInvocationResponseV1, CapabilityErrorV1>;
}

/// Registers the capability-client import on a Wasmtime component linker.
pub fn add_to_linker<T: 'static>(
    linker: &mut wasmtime::component::Linker<T>,
    get: fn(&mut T) -> &mut dyn CapabilityService,
) -> wasmtime::Result<()> {
    let mut instance = linker.instance("greentic:cap-runtime/capability-client@1.0.0")?;
    instance.func_wrap(
        "invoke",
        move |mut caller: wasmtime::StoreContextMut<'_, T>, (request,): (Request,)| {
            let request = match BindingId::parse(request.binding_id) {
                Ok(binding_id) => CapabilityInvocationRequestV1 {
                    binding_id,
                    operation: request.operation,
                    payload_cbor: request.payload_cbor,
                },
                Err(_) => return Ok((Err(CapabilityError::InvalidInput),)),
            };
            let result = get(caller.data_mut())
                .invoke(request)
                .map(
                    |CapabilityInvocationResponseV1 {
                         output_cbor,
                         metadata_cbor,
                     }| Response {
                        output_cbor,
                        metadata_cbor,
                    },
                )
                .map_err(map_error);
            Ok((result,))
        },
    )?;
    Ok(())
}

fn map_error(error: CapabilityErrorV1) -> CapabilityError {
    match error {
        CapabilityErrorV1::BindingNotFound => CapabilityError::BindingNotFound,
        CapabilityErrorV1::OperationNotAllowed => CapabilityError::OperationNotAllowed,
        CapabilityErrorV1::ProviderUnavailable => CapabilityError::ProviderUnavailable,
        CapabilityErrorV1::InvalidInput => CapabilityError::InvalidInput,
        CapabilityErrorV1::InvalidOutput => CapabilityError::InvalidOutput,
        CapabilityErrorV1::PolicyDenied => CapabilityError::PolicyDenied,
        CapabilityErrorV1::Timeout => CapabilityError::Timeout,
        CapabilityErrorV1::Cancelled => CapabilityError::Cancelled,
        CapabilityErrorV1::Overloaded => CapabilityError::Overloaded,
        CapabilityErrorV1::ProtocolError => CapabilityError::ProtocolError,
        CapabilityErrorV1::Internal => CapabilityError::Internal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeService;
    impl CapabilityService for FakeService {
        fn invoke(
            &mut self,
            request: CapabilityInvocationRequestV1,
        ) -> Result<CapabilityInvocationResponseV1, CapabilityErrorV1> {
            Ok(CapabilityInvocationResponseV1 {
                output_cbor: request.payload_cbor,
                metadata_cbor: None,
            })
        }
    }

    struct State {
        service: FakeService,
    }

    #[test]
    fn registers_capability_client_on_linker() {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        let engine = wasmtime::Engine::new(&config).unwrap();
        let mut linker = wasmtime::component::Linker::<State>::new(&engine);
        add_to_linker(&mut linker, |state| &mut state.service).unwrap();
    }

    #[test]
    fn maps_every_safe_error_without_detail() {
        let errors = [
            CapabilityErrorV1::BindingNotFound,
            CapabilityErrorV1::OperationNotAllowed,
            CapabilityErrorV1::ProviderUnavailable,
            CapabilityErrorV1::InvalidInput,
            CapabilityErrorV1::InvalidOutput,
            CapabilityErrorV1::PolicyDenied,
            CapabilityErrorV1::Timeout,
            CapabilityErrorV1::Cancelled,
            CapabilityErrorV1::Overloaded,
            CapabilityErrorV1::ProtocolError,
            CapabilityErrorV1::Internal,
        ];
        for error in errors {
            let _ = map_error(error);
        }
    }
}
