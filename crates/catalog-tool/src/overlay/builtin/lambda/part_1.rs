use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
    TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetFunctionCodeSigningConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions now supplies complete FunctionName discovery. This output identifies the function by FunctionName while the canonical function identity is FunctionArn; CodeSigningConfigArn belongs to a different shared resource. Requires parent-identity enrichment rather than publishing a duplicate function or misidentifying the signing configuration.".to_owned()),
        ..empty()
    }),
("GetFunctionConcurrency", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions supplies FunctionName, but output contains only ReservedConcurrentExecutions. Requires canonical function identity inherited from the bound input and parent enrichment; current identity paths address response members. No pagination.".to_owned()),
        ..empty()
    }),
("GetFunctionEventInvokeConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions supplies FunctionName and root FunctionArn is usable, but complete version/alias Qualifier coverage requires correlated discovery and a reviewed qualified-configuration kind. ListFunctionEventInvokeConfigs is the paginated per-function alternative; do not collapse qualified configurations onto the unqualified function identity.".to_owned()),
        ..empty()
    }),
("GetFunctionRecursionConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions supplies unqualified FunctionName, but RecursiveLoop is the only output member and cannot identify a function. Requires canonical parent-input identity and configuration enrichment rather than a second function resource.".to_owned()),
        ..empty()
    }),
("GetFunctionScalingConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires correlated unqualified FunctionName plus required published Qualifier. ListVersionsByFunction exposes Environment.Variables; capacity-provider member FunctionArn is qualified and cannot be split by the current binding schema. Output root FunctionArn is usable after safe version discovery and typed ARN/qualifier extraction.".to_owned()),
        ..empty()
    }),
("GetFunctionUrlConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions and ListAliases discover function and alias metadata, but complete Qualifier correlation and a reviewed URL-configuration kind are required to preserve qualified configurations without overwriting function records. FunctionUrl is configuration only; inventory must never invoke it.".to_owned()),
        ..empty()
    }),
("GetLayerVersionPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires LayerName and VersionNumber correlated across ListLayers parent and ListLayerVersions child; child carries LayerVersionArn/Version, not unversioned LayerName. Need ancestor input binding or typed ARN extraction, integer Version binding, and input-inherited attachment identity because output contains only Policy/RevisionId. Policy is resource configuration, not executable code.".to_owned()),
        ..empty()
    }),
("GetPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Resource-based IAM policy is configuration, but requires safe complete FunctionName/Qualifier discovery and input-inherited attachment identity; Policy/RevisionId output contains no function identity. Current response-only identity cannot encode this attachment; do not use policy text or revision as identity.".to_owned()),
        ..empty()
    }),
("GetProvisionedConcurrencyConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires correlated FunctionName and required alias/version Qualifier plus input-inherited identity: output has concurrency/status fields only. Need safe function/version/alias discovery and attachment identity; ListProvisionedConcurrencyConfigs is the per-function paginated alternative.".to_owned()),
        ..empty()
    }),
("GetResourcePolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Resource policy is configuration, but output Policy/RevisionId has no ResourceArn. Need identity inherited from the required ResourceArn input and complete typed discovery of supported functions/versions/aliases; current response-only identity and one-source binding cannot express the resource union.".to_owned()),
        ..empty()
    }),
("GetRuntimeManagementConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Output root FunctionArn identifies the configuration owner; safe complete FunctionName and correlated optional version/alias Qualifier discovery is missing because FunctionConfiguration listings expose Environment.Variables. RuntimeVersionArn identifies a different object, not the function.".to_owned()),
        ..empty()
    }),
("ListAliases", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Aliases")?),
        identity: Some(Identity::Arn(path("AliasArn")?)),
        bind: [(path("FunctionName")?, Binding::From { operation: name("ListFunctions")?, path: path("FunctionName")? })].into_iter().collect(),
        page_size: Some(50),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("ListDurableExecutionsByFunction", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions supplies FunctionName. DurableExecutions contains execution metadata, not function resources; enabling it additionally requires a reviewed execution kind and lifecycle/scope semantics. Never invoke functions to generate executions or enable the separate payload/history/state APIs.".to_owned()),
        ..empty()
    }),
("ListFunctionEventInvokeConfigs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions supplies FunctionName, and FunctionEventInvokeConfigs has FunctionArn plus Marker/NextMarker/MaxItems. Qualified configuration rows need a reviewed configuration kind and owner relation; treating them as canonical function resources would conflate configuration and function identity.".to_owned()),
        ..empty()
    }),
("ListFunctionUrlConfigs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions supplies FunctionName, and FunctionUrlConfigs has FunctionArn plus Marker/NextMarker/MaxItems. Qualified URL configurations need a reviewed configuration kind and owner relation rather than overwriting function records. Never invoke returned FunctionUrl values.".to_owned()),
        ..empty()
    }),
("ListFunctionsByCodeSigningConfig", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FunctionArns")?),
        identity: Some(Identity::Arn(path("$")?)),
        bind: [(path("CodeSigningConfigArn")?, Binding::From {
            operation: name("ListCodeSigningConfigs")?, path: path("CodeSigningConfigArn")?,
        })].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair {
            input: path("Marker")?, output: path("NextMarker")?, optional: false,
        })),
        ..empty()
    }),
("ListProvisionedConcurrencyConfigs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Sanitized ListFunctions supplies FunctionName. ProvisionedConcurrencyConfigs identifies qualified configuration owners with FunctionArn, not unqualified function resources; requires a reviewed configuration kind and qualified-owner relation. Marker/NextMarker/MaxItems is supported; concurrency counts/status cannot be resource identity.".to_owned()),
        ..empty()
    }),
("ListTags", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires Resource ARN and returns only a Tags map; no response identity and no paginator. Need input-inherited attachment identity, tag merge semantics, and union fan-out across function/event-source-mapping/code-signing-config/capacity-provider/network-connector resource types accepted by TaggableResource. Current single-source binding cannot express the complete resource union.".to_owned()),
        ..empty()
    }),
("AddLayerVersionPermission", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("AddPermission", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
