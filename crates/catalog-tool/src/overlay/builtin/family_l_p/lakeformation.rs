// Pinned Smithy source: .local/aws-models/models/lakeformation/service/2017-03-31/lakeformation-2017-03-31.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AddLFTagsToResource", "AttachRLSPolicy", "BatchGrantPermissions", 
            "BatchRevokePermissions", "CancelTransaction", "CommitTransaction", 
            "CreateDataCellsFilter", "CreateLFTag", "CreateLFTagExpression", 
            "CreateLakeFormationIdentityCenterConfiguration", "CreateLakeFormationOptIn", 
            "CreateRLSPolicy", "DeleteDataCellsFilter", "DeleteLFTag", "DeleteLFTagExpression", 
            "DeleteLakeFormationIdentityCenterConfiguration", "DeleteLakeFormationOptIn", 
            "DeleteObjectsOnCancel", "DeleteRLSPolicy", "DeregisterResource", 
            "DetachRLSPolicy", "ExtendTransaction", "GrantPermissions", "PutDataLakeSettings", 
            "RegisterResource", "RemoveLFTagsFromResource", "RevokePermissions", 
            "StartQueryPlanning", "StartTransaction", "UpdateDataCellsFilter", "UpdateLFTag", 
            "UpdateLFTagExpression", "UpdateLakeFormationIdentityCenterConfiguration", 
            "UpdateRLSPolicy", "UpdateResource", "UpdateTableObjects", 
            "UpdateTableStorageOptimizer",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "AssumeDecoratedRoleWithSAML", "GetGlueFunctionSessionContext", 
            "GetTemporaryDataLocationCredentials", "GetTemporaryGlueFunctionCredentials", 
            "GetTemporaryGluePartitionCredentials", "GetTemporaryGlueTableCredentials",
        ],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeLakeFormationIdentityCenterConfiguration", "GetDataLakePrincipal", 
            "GetDataLakeSettings", "ListDataCellsFilter", "ListLakeFormationOptIns", 
            "ListPermissions", "ListRLSPolicies", "ListRLSPolicyAttachments", 
            "ListTransactions",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
             "GetEffectivePermissionsForPath",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "DescribeTransaction",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (TransactionId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDataCellsFilter",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (TableCatalogId, DatabaseName, TableName, Name); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetLFTag",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (TagKey); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetLFTagExpression", "GetRLSPolicy",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Name); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetQueryState", "GetQueryStatistics",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (QueryId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetResourceLFTags",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Resource); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetTableObjects", "GetWorkUnitResults", "GetWorkUnits",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &[
            "ListTableStorageOptimizers",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (DatabaseName, TableName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "SearchDatabasesByLFTags", "SearchTablesByLFTags",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Expression); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Lists the resources registered to be managed by the Data Catalog.
    Resource {
        operation: "ListResources",
        items: "ResourceInfoList", id: Id::Arn("ResourceArn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["ResourceArn", "RoleArn", "VerificationStatus"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists LF-tags that the requester has permission to view.
    Resource {
        operation: "ListLFTags",
        items: "LFTags", id: Id::Key(&["CatalogId", "TagKey"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["CatalogId", "TagKey"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns the LF-Tag expressions in caller’s account filtered based on caller's permissions. Data Lake and read only admins implicitly can see all tag expressions in their account, e
    Resource {
        operation: "ListLFTagExpressions",
        items: "LFTagExpressions", id: Id::Key(&["CatalogId", "Name"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["CatalogId", "Name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Same pinned response resource shape as ListResources; no content-bearing expansion.
    Resource {
        operation: "DescribeResource",
        items: "ResourceInfo", id: Id::Arn("ResourceArn"),
        scope: Scope::Region, tokens: None,
        query: &["ResourceArn", "RoleArn", "VerificationStatus"],
        tags: None,
        bindings: &[("ResourceArn", "ListResources", "ResourceArn")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lakeformation", GROUPS, RESOURCES)
}
