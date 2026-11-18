use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator, Scope,
    TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetFunctionCodeSigningConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires FunctionName from a safe complete function discovery source; ListFunctions is secret-bearing. Output FunctionName is a possible region/account-scoped key, but CodeSigningConfigArn identifies a different shared resource. Do not replace discovery with the code-signed or capacity-provider subset.".to_owned()),
        ..empty()
    }),
("GetFunctionConcurrency", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires safe complete FunctionName discovery and response identity inherited from that input: output contains only ReservedConcurrentExecutions. Current identity paths address response members, not request/parent identity. No pagination.".to_owned()),
        ..empty()
    }),
("GetFunctionEventInvokeConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Output root FunctionArn is usable, but FunctionName requires safe complete function discovery; version/alias Qualifier needs correlated discovery to cover qualified configurations. ListFunctionEventInvokeConfigs is the complete per-function paginated alternative once discovery is safe.".to_owned()),
        ..empty()
    }),
("GetFunctionRecursionConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires safe complete unqualified FunctionName discovery and input-inherited identity: RecursiveLoop is the only output member and cannot identify a function. Current response-only identity cannot encode this configuration attachment.".to_owned()),
        ..empty()
    }),
("GetFunctionScalingConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires correlated unqualified FunctionName plus required published Qualifier. ListVersionsByFunction exposes Environment.Variables; capacity-provider member FunctionArn is qualified and cannot be split by the current binding schema. Output root FunctionArn is usable after safe version discovery and typed ARN/qualifier extraction.".to_owned()),
        ..empty()
    }),
("GetFunctionUrlConfig", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Returns endpoint configuration, not function invocation content, and root FunctionArn is usable; requires safe complete FunctionName and alias Qualifier discovery. Do not send requests to returned FunctionUrl. ListFunctionUrlConfigs is the paginated per-function alternative once discovery is safe.".to_owned()),
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
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Aliases[].AliasArn is a valid identity and pagination is Marker/NextMarker/MaxItems, but required FunctionName lacks a safe complete discovery source. ListFunctions is secret-bearing; code-signing/capacity-provider/event-mapping subsets would silently omit functions.".to_owned()),
        ..empty()
    }),
("ListDurableExecutionsByFunction", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("DurableExecutions[] contains execution metadata only (DurableExecutionArn, FunctionArn, status, timestamps, KMSKeyArn), with Marker/NextMarker/MaxItems pagination. Required FunctionName lacks safe complete discovery; do not invoke functions to generate executions or enable the separate payload/history/state APIs.".to_owned()),
        ..empty()
    }),
("ListFunctionEventInvokeConfigs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("FunctionEventInvokeConfigs[].FunctionArn is usable with Marker/NextMarker/MaxItems and page size 50; required FunctionName lacks safe complete discovery. Do not substitute a subset of code-signed, mapped, or capacity-provider functions.".to_owned()),
        ..empty()
    }),
("ListFunctionUrlConfigs", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("FunctionUrlConfigs[].FunctionArn is usable with Marker/NextMarker/MaxItems and page size 50; required FunctionName lacks safe complete discovery. FunctionUrl is configuration and must never be invoked by inventory.".to_owned()),
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
        defer_reason: Some("ProvisionedConcurrencyConfigs[].FunctionArn identifies the qualified configuration owner; Marker/NextMarker/MaxItems pagination and page size 50 are supported. Required FunctionName lacks safe complete discovery; do not use concurrency counts/status as resource identity.".to_owned()),
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
