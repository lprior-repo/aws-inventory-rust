use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
    TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListCapacityProviders", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityProviders")?),
        page_size: Some(50),
        identity: Some(Identity::Arn(path("CapacityProviderArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("GetCapacityProvider", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CapacityProvider")?),
        identity: Some(Identity::Arn(path("CapacityProviderArn")?)),
        paginator: Some(Paginator::None),
        bind: [(path("CapacityProviderName")?, Binding::From { operation: name("ListCapacityProviders")?, path: path("CapacityProviderArn")? })].into_iter().collect(),
        ..empty()
    }),
("ListFunctionVersionsByCapacityProvider", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FunctionVersions")?),
        page_size: Some(50),
        identity: Some(Identity::Arn(path("FunctionArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        bind: [(path("CapacityProviderName")?, Binding::From { operation: name("ListCapacityProviders")?, path: path("CapacityProviderArn")? })].into_iter().collect(),
        ..empty()
    }),
("ListCodeSigningConfigs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CodeSigningConfigs")?),
        page_size: Some(50),
        identity: Some(Identity::Arn(path("CodeSigningConfigArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("GetCodeSigningConfig", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("CodeSigningConfig")?),
        identity: Some(Identity::Arn(path("CodeSigningConfigArn")?)),
        paginator: Some(Paginator::None),
        bind: [(path("CodeSigningConfigArn")?, Binding::From { operation: name("ListCodeSigningConfigs")?, path: path("CodeSigningConfigArn")? })].into_iter().collect(),
        ..empty()
    }),
("ListEventSourceMappings", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("EventSourceMappings")?),
        page_size: Some(50),
        identity: Some(Identity::Arn(path("EventSourceMappingArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("GetEventSourceMapping", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Arn(path("EventSourceMappingArn")?)),
        paginator: Some(Paginator::None),
        bind: [(path("UUID")?, Binding::From { operation: name("ListEventSourceMappings")?, path: path("UUID")? })].into_iter().collect(),
        ..empty()
    }),
("ListLayers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Layers")?),
        page_size: Some(50),
        identity: Some(Identity::Arn(path("LayerArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("ListLayerVersions", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("LayerVersions")?),
        page_size: Some(50),
        identity: Some(Identity::Arn(path("LayerVersionArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        bind: [(path("LayerName")?, Binding::From { operation: name("ListLayers")?, path: path("LayerArn")? })].into_iter().collect(),
        ..empty()
    }),
("GetFunction", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetLayerVersion", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetLayerVersionByArn", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetFunctionConfiguration", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("$")?),
        identity: Some(Identity::Arn(path("FunctionArn")?)),
        exclude_response_members: vec![path("Environment")?, path("ImageConfigResponse")?],
        bind: [(path("FunctionName")?, Binding::From { operation: name("ListFunctions")?, path: path("FunctionName")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListFunctions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Functions")?),
        identity: Some(Identity::Arn(path("FunctionArn")?)),
        exclude_response_members: vec![path("Functions.Environment")?, path("Functions.ImageConfigResponse")?],
        page_size: Some(50),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("ListVersionsByFunction", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("GetDurableExecution", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetDurableExecutionHistory", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetDurableExecutionState", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetAccountSettings", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("GetAlias", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GetAlias requires correlated FunctionName and Name; ListAliases items contain Name/AliasArn but not the unqualified parent FunctionName. Need a safe complete function discovery source and ancestor-bound input correlation; do not cross-join alias names between functions. Output root AliasArn is otherwise a valid identity; no pagination.".to_owned()),
        ..empty()
    })
    ])
}
