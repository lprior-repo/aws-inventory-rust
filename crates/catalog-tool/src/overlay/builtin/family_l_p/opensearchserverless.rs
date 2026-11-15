// Pinned Smithy source: .local/aws-models/models/opensearchserverless/service/2021-11-01/opensearchserverless-2021-11-01.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "BatchGetCollection", "BatchGetCollectionGroup", 
            "BatchGetEffectiveLifecyclePolicy", "BatchGetLifecyclePolicy", 
            "BatchGetVpcEndpoint", "CancelIngestionJob", "CreateAccessPolicy", 
            "CreateCollection", "CreateCollectionGroup", "CreateIndex", "CreateIngestionJob", 
            "CreateIngestionStream", "CreateLifecyclePolicy", "CreateSecurityConfig", 
            "CreateSecurityPolicy", "CreateVpcEndpoint", "DeleteAccessPolicy", 
            "DeleteCollection", "DeleteCollectionGroup", "DeleteIndex", 
            "DeleteIngestionStream", "DeleteLifecyclePolicy", "DeleteSecurityConfig", 
            "DeleteSecurityPolicy", "DeleteVpcEndpoint", "TagResource", "UntagResource", 
            "UpdateAccessPolicy", "UpdateAccountSettings", "UpdateCollection", 
            "UpdateCollectionGroup", "UpdateIndex", "UpdateLifecyclePolicy", 
            "UpdateSecurityConfig", "UpdateSecurityPolicy", "UpdateVpcEndpoint",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetAccessPolicy", "GetSecurityPolicy",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (type, name); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetAccountSettings", "GetPoliciesStats",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "GetIndex",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (id, indexName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetIngestionJob",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (collectionId, ingestionJobId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetIngestionStream",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (collectionId, ingestionStreamId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetSecurityConfig",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (id); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListIngestionJobs", "ListIngestionStreams",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (collectionId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists all OpenSearch Serverless collections. For more information, see Creating and managing Amazon OpenSearch Serverless collections . Make sure to include an empty request body {
    Resource {
        operation: "ListCollections",
        items: "collectionSummaries", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "id", "name", "status", "kmsKeyArn", "collectionGroupName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of collection groups. For more information, see Creating and managing Amazon OpenSearch Serverless collections .
    Resource {
        operation: "ListCollectionGroups",
        items: "collectionGroupSummaries", id: Id::Arn("arn"),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "id", "name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns the OpenSearch Serverless-managed interface VPC endpoints associated with the current account. For more information, see Access Amazon OpenSearch Serverless using an interf
    Resource {
        operation: "ListVpcEndpoints",
        items: "vpcEndpointSummaries", id: Id::Key(&["id"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["id", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListAccessPolicies", items: "accessPolicySummaries", id: Id::Correlated(&["type"], &["name"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["type", "name", "policyVersion"], tags: None, bindings: &[], fixed: &[],
    },
    Resource {
        operation: "ListLifecyclePolicies", items: "lifecyclePolicySummaries", id: Id::Correlated(&["type"], &["name"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["type", "name", "policyVersion"], tags: None, bindings: &[], fixed: &[],
    },
    Resource {
        operation: "ListSecurityPolicies", items: "securityPolicySummaries", id: Id::Correlated(&["type"], &["name"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["type", "name", "policyVersion"], tags: None, bindings: &[], fixed: &[],
    },
    Resource {
        operation: "ListSecurityConfigs", items: "securityConfigSummaries", id: Id::Key(&["id"]),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["id", "type", "configVersion"], tags: None, bindings: &[], fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    use crate::overlay::{ExecutionBranch, FixedValue};
    use super::super::{name, path};
    let mut service = compile("opensearchserverless", GROUPS, RESOURCES)?;
    [
        ("ListAccessPolicies", &["data"][..]),
        ("ListLifecyclePolicies", &["retention"][..]),
        ("ListSecurityPolicies", &["encryption", "network"][..]),
        ("ListSecurityConfigs", &["saml", "iamidentitycenter", "iamfederation"][..]),
    ].into_iter().try_for_each(|(operation, types)| -> Result<(), NameError> {
        if let Some(plan) = service.operations.get_mut(&name(operation)?) {
            plan.branches = types.iter().map(|kind| Ok(ExecutionBranch {
                discriminator: (*kind).to_owned(),
                fixed: [(path("type")?, FixedValue::String((*kind).to_owned()))].into_iter().collect(),
                bind: Default::default(), emission: None, row_policy: None,
                scope: Scope::Region, request_region: None,
            })).collect::<Result<_, NameError>>()?;
        }
        Ok(())
    })?;
    Ok(service)
}
