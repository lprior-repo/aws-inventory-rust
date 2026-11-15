use crate::overlay::{NameError, OperationOverlay, Binding, FixedValue, Identity, ItemSelector, OverlayClass, Paginator, Scope, TokenPair};
use super::super::{empty, path, name};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListAnycastIpLists", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("AnycastIpLists.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("ETag")?, path("LastModifiedTime")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("AnycastIpLists.NextMarker")?, optional: false }], truncated: Some(path("AnycastIpLists.IsTruncated")?) }),
        ..empty()
    }),
("GetAnycastIpList", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("AnycastIpList")?),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        bind: [(path("Id")?, Binding::From { operation: name("ListAnycastIpLists")?, path: path("Id")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListCachePolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("CachePolicyList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("CachePolicy.LastModifiedTime")?],
        fixed: [(path("Type")?, FixedValue::String("custom".to_owned()))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("CachePolicy.Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("CachePolicyList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListCloudFrontOriginAccessIdentities", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("CloudFrontOriginAccessIdentityList.Items")?),
        page_size: Some(100),
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("CloudFrontOriginAccessIdentityList.NextMarker")?, optional: false }], truncated: Some(path("CloudFrontOriginAccessIdentityList.IsTruncated")?) }),
        ..empty()
    }),
("ListConnectionGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ConnectionGroups")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("ETag")?, path("LastModifiedTime")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("GetConnectionGroup", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ConnectionGroup")?),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        tags: Some(path("Tags.Items")?),
        bind: [(path("Identifier")?, Binding::From { operation: name("ListConnectionGroups")?, path: path("Id")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListContinuousDeploymentPolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ContinuousDeploymentPolicyList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("ContinuousDeploymentPolicy.LastModifiedTime")?],
        identity: Some(Identity::Key(vec![path("ContinuousDeploymentPolicy.Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("ContinuousDeploymentPolicyList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListDistributionTenants", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("DistributionTenantList")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("ETag")?, path("LastModifiedTime")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    }),
("ListFieldLevelEncryptionConfigs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("FieldLevelEncryptionList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("FieldLevelEncryptionList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListFieldLevelEncryptionProfiles", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("FieldLevelEncryptionProfileList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("FieldLevelEncryptionProfileList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListFunctions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("FunctionList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("FunctionMetadata.LastModifiedTime")?],
        identity: Some(Identity::Key(vec![path("FunctionMetadata.FunctionARN")?, path("FunctionMetadata.Stage")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("FunctionList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListKeyGroups", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("KeyGroupList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("KeyGroup.LastModifiedTime")?],
        identity: Some(Identity::Key(vec![path("KeyGroup.Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("KeyGroupList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListKeyValueStores", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("KeyValueStoreList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        identity: Some(Identity::Arn(path("ARN")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("KeyValueStoreList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListOriginAccessControls", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("OriginAccessControlList.Items")?),
        page_size: Some(100),
        identity: Some(Identity::Key(vec![path("Id")?])),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("OriginAccessControlList.NextMarker")?, optional: false }], truncated: Some(path("OriginAccessControlList.IsTruncated")?) }),
        ..empty()
    }),
("ListOriginRequestPolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("OriginRequestPolicyList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("OriginRequestPolicy.LastModifiedTime")?],
        fixed: [(path("Type")?, FixedValue::String("custom".to_owned()))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("OriginRequestPolicy.Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("OriginRequestPolicyList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListRealtimeLogConfigs", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("RealtimeLogConfigs.Items")?),
        page_size: Some(100),
        identity: Some(Identity::Arn(path("ARN")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("RealtimeLogConfigs.NextMarker")?, optional: false }], truncated: Some(path("RealtimeLogConfigs.IsTruncated")?) }),
        ..empty()
    }),
("ListResponseHeadersPolicies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("ResponseHeadersPolicyList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("ResponseHeadersPolicy.LastModifiedTime")?],
        fixed: [(path("Type")?, FixedValue::String("custom".to_owned()))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("ResponseHeadersPolicy.Id")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("ResponseHeadersPolicyList.NextMarker")?, optional: false })),
        ..empty()
    }),
("ListStreamingDistributions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("StreamingDistributionList.Items")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        identity: Some(Identity::Arn(path("ARN")?)),
        paginator: Some(Paginator::Tokens { pairs: vec![TokenPair { input: path("Marker")?, output: path("StreamingDistributionList.NextMarker")?, optional: false }], truncated: Some(path("StreamingDistributionList.IsTruncated")?) }),
        ..empty()
    }),
("GetStreamingDistribution", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("StreamingDistribution")?),
        exclude_from_hash: vec![path("LastModifiedTime")?],
        bind: [(path("Id")?, Binding::From { operation: name("ListStreamingDistributions")?, path: path("Id")? })].into_iter().collect(),
        identity: Some(Identity::Arn(path("ARN")?)),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListTrustStores", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("TrustStoreList")?),
        page_size: Some(100),
        exclude_from_hash: vec![path("ETag")?, path("LastModifiedTime")?],
        identity: Some(Identity::Arn(path("Arn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("Marker")?, output: path("NextMarker")?, optional: false })),
        ..empty()
    })
    ])
}
