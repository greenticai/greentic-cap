use std::collections::BTreeMap;

use greentic_cap_schema::{
    CapabilityComponentDescriptor, CapabilityComponentOperation, PackCapabilitySectionV1,
};
use greentic_cap_types::{
    CapabilityConsume, CapabilityDeclaration, CapabilityId, CapabilityOffer, CapabilityProfile,
    CapabilityProviderOperationMap, CapabilityProviderRef, CapabilityRequirement,
};

/// Shared fixture for performance guards around pack compatibility.
pub struct PackPerfCase {
    pub section: PackCapabilitySectionV1,
    pub component: CapabilityComponentDescriptor,
}

/// Shared fixture for resolution benchmarks.
pub struct ResolvePerfCase {
    pub declaration: CapabilityDeclaration,
}

fn cap(value: &str) -> CapabilityId {
    CapabilityId::new(value).expect("valid capability id")
}

fn component_operation(name: &str) -> CapabilityComponentOperation {
    CapabilityComponentOperation {
        name: name.to_string(),
        input_schema: serde_json::json!({"type": "string"}),
        output_schema: serde_json::json!({"type": "string"}),
    }
}

fn offer(index: usize, capability_index: usize, profile: Option<&str>) -> CapabilityOffer {
    let mut offer = CapabilityOffer::new(
        format!("offer.{index}"),
        cap(&format!("cap://bench.capability.{capability_index}")),
    );
    if let Some(profile) = profile {
        offer.profiles.push(profile.to_string());
    }
    let component_operation = format!("operation.{capability_index}");
    offer.provider = Some(CapabilityProviderRef {
        component_ref: "component:bench".to_string(),
        operation: component_operation.clone(),
        operation_map: vec![CapabilityProviderOperationMap {
            contract_operation: format!("contract.{capability_index}"),
            component_operation,
            input_schema: serde_json::json!({"type": "string"}),
            output_schema: serde_json::json!({"type": "string"}),
        }],
    });
    offer
}

/// Builds a synthetic declaration with enough work to exercise resolver and compatibility paths.
pub fn resolution_case() -> ResolvePerfCase {
    let mut declaration = CapabilityDeclaration::new();
    let profiles = [
        "profile.alpha",
        "profile.beta",
        "profile.gamma",
        "profile.delta",
    ];

    for index in 0..32 {
        declaration.offers.push(offer(
            index,
            index % 8,
            Some(profiles[index % profiles.len()]),
        ));
    }

    for (index, profile_id) in profiles.iter().enumerate() {
        let capability_index = index % 8;
        declaration.profiles.push(CapabilityProfile {
            id: (*profile_id).to_string(),
            description: Some(format!("profile {index}")),
            requires: vec![CapabilityRequirement {
                id: format!("require.{index}"),
                capability: cap(&format!("cap://bench.capability.{capability_index}")),
                profiles: vec![(*profile_id).to_string()],
                optional: false,
                description: Some(format!("requirement {index}")),
                metadata: BTreeMap::new(),
            }],
            consumes: vec![CapabilityConsume {
                id: format!("consume.{index}"),
                capability: cap(&format!("cap://bench.capability.{capability_index}")),
                profiles: vec![(*profile_id).to_string()],
                mode: greentic_cap_types::CapabilityConsumeMode::Shared,
                description: Some(format!("consume {index}")),
                metadata: BTreeMap::new(),
            }],
        });
    }

    for index in 0..8 {
        let capability_index = index;
        declaration.requires.push(CapabilityRequirement {
            id: format!("require.root.{index}"),
            capability: cap(&format!("cap://bench.capability.{capability_index}")),
            profiles: Vec::new(),
            optional: false,
            description: Some(format!("root requirement {index}")),
            metadata: BTreeMap::new(),
        });
        declaration.consumes.push(CapabilityConsume {
            id: format!("consume.root.{index}"),
            capability: cap(&format!("cap://bench.capability.{capability_index}")),
            profiles: Vec::new(),
            mode: greentic_cap_types::CapabilityConsumeMode::Shared,
            description: Some(format!("root consume {index}")),
            metadata: BTreeMap::new(),
        });
    }

    ResolvePerfCase { declaration }
}

/// Builds a synthetic pack capability section and matching component descriptor.
pub fn pack_case() -> PackPerfCase {
    let mut declaration = resolution_case().declaration;
    declaration.offers.clear();

    for index in 0..48 {
        declaration.offers.push(offer(index, index % 8, None));
    }

    let section = PackCapabilitySectionV1::new(declaration);
    let component = CapabilityComponentDescriptor {
        component_ref: "component:bench".to_string(),
        version: "1.0.0".to_string(),
        operations: (0..8)
            .map(|index| component_operation(&format!("operation.{index}")))
            .collect(),
        capabilities: (0..8)
            .map(|index| cap(&format!("cap://bench.capability.{index}")))
            .collect(),
        metadata: BTreeMap::new(),
    };

    PackPerfCase { section, component }
}
