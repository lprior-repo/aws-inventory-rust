// Pinned Smithy source: .local/aws-models/models/omics/service/2022-11-28/omics-2022-11-28.json
use super::policy::{Group, Id, Input, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AbortMultipartReadSetUpload",
            "AcceptShare",
            "BatchDeleteReadSet",
            "CancelAnnotationImportJob",
            "CancelRun",
            "CancelRunBatch",
            "CancelVariantImportJob",
            "CompleteMultipartReadSetUpload",
            "CreateAnnotationStore",
            "CreateAnnotationStoreVersion",
            "CreateConfiguration",
            "CreateMultipartReadSetUpload",
            "CreateReferenceStore",
            "CreateRunCache",
            "CreateRunGroup",
            "CreateSequenceStore",
            "CreateShare",
            "CreateVariantStore",
            "CreateWorkflow",
            "CreateWorkflowVersion",
            "DeleteAnnotationStore",
            "DeleteAnnotationStoreVersions",
            "DeleteBatch",
            "DeleteConfiguration",
            "DeleteReference",
            "DeleteReferenceStore",
            "DeleteRun",
            "DeleteRunBatch",
            "DeleteRunCache",
            "DeleteRunGroup",
            "DeleteS3AccessPolicy",
            "DeleteSequenceStore",
            "DeleteShare",
            "DeleteVariantStore",
            "DeleteWorkflow",
            "DeleteWorkflowVersion",
            "PutS3AccessPolicy",
            "StartAnnotationImportJob",
            "StartReadSetActivationJob",
            "StartReadSetExportJob",
            "StartReadSetImportJob",
            "StartReferenceImportJob",
            "StartRun",
            "StartRunBatch",
            "StartVariantImportJob",
            "TagResource",
            "UntagResource",
            "UpdateAnnotationStore",
            "UpdateAnnotationStoreVersion",
            "UpdateRunCache",
            "UpdateRunGroup",
            "UpdateSequenceStore",
            "UpdateVariantStore",
            "UpdateWorkflow",
            "UpdateWorkflowVersion",
            "UploadReadSetPart",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetAnnotationImportJob", "GetVariantImportJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (jobId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAnnotationStore", "GetConfiguration", "GetVariantStore"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (name); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAnnotationStoreVersion"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (name, versionName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetBatch", "ListRunsInBatch"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (batchId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetReadSet",
            "GetReference",
            "GetRunTask",
            "GetWorkflow",
            "GetWorkflowVersion",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &[
            "GetReadSetActivationJob",
            "GetReadSetImportJob",
            "GetReadSetMetadata",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (id, sequenceStoreId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetReadSetExportJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (sequenceStoreId, id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetReferenceImportJob", "GetReferenceMetadata"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (id, referenceStoreId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetReferenceStore",
            "GetRun",
            "GetRunCache",
            "GetRunGroup",
            "GetSequenceStore",
            "ListRunTasks",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetS3AccessPolicy"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (s3AccessPointArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetShare"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (shareId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "ListAnnotationImportJobs",
            "ListBatch",
            "ListConfigurations",
            "ListRuns",
            "ListVariantImportJobs",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &[
            "ListMultipartReadSetUploads",
            "ListReadSetActivationJobs",
            "ListReadSetExportJobs",
            "ListReadSetImportJobs",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (sequenceStoreId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListReadSetUploadParts"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (sequenceStoreId, uploadId, partSource); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListReferenceImportJobs"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (referenceStoreId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListShares"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceOwner); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    // Read-set listings include clinical subject/sample identifiers and sequence-derived data.
    Group {
        operations: &["ListReadSets"],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
];

const RESOURCES: &[Resource] = &[
    // Amazon Web Services HealthOmics variant stores and annotation stores are no longer open to new customers. Existing customers can continue to use the service as normal. For more inf
    Resource {
        operation: "ListAnnotationStores",
        items: "annotationStores",
        id: Id::Arn("storeArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["storeArn", "id", "status", "name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves a list of reference stores linked to your account and returns their metadata in JSON format. For more information, see Creating a reference store in the Amazon Web Servic
    Resource {
        operation: "ListReferenceStores",
        items: "referenceStores",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "id", "name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves a list of sequence stores and returns each sequence store's metadata. For more information, see Creating a HealthOmics sequence store in the Amazon Web Services HealthOmi
    Resource {
        operation: "ListSequenceStores",
        items: "sequenceStores",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "id", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Amazon Web Services HealthOmics variant stores and annotation stores are no longer open to new customers. Existing customers can continue to use the service as normal. For more inf
    Resource {
        operation: "ListVariantStores",
        items: "variantStores",
        id: Id::Arn("storeArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["storeArn", "id", "status", "name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves a list of existing workflows. You can filter for specific workflows by their name and type. Using the type parameter, specify PRIVATE to retrieve a list of private workfl
    Resource {
        operation: "ListWorkflows",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("startingToken", "nextToken")),
        query: &["arn", "id", "name", "status", "type"],
        tags: None,
        bindings: &[],
        fixed: &[Input::String("type", "PRIVATE")],
    },
    // Retrieves a list of all run groups and returns the metadata for each run group.
    Resource {
        operation: "ListRunGroups",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("startingToken", "nextToken")),
        query: &["arn", "id", "name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves a list of your run caches and the metadata for each cache.
    Resource {
        operation: "ListRunCaches",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("startingToken", "nextToken")),
        query: &["arn", "id", "name", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListAnnotationStoreVersions",
        items: "annotationStoreVersions",
        id: Id::Arn("versionArn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["versionArn", "id", "name", "versionName", "status"],
        tags: None,
        bindings: &[("name", "ListAnnotationStores", "name")],
        fixed: &[],
    },
    Resource {
        operation: "ListReferences",
        items: "references",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "id", "referenceStoreId", "status"],
        tags: None,
        bindings: &[("referenceStoreId", "ListReferenceStores", "id")],
        fixed: &[],
    },
    Resource {
        operation: "ListWorkflowVersions",
        items: "items",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("startingToken", "nextToken")),
        query: &["arn", "workflowId", "versionName", "status"],
        tags: None,
        bindings: &[("workflowId", "ListWorkflows", "id")],
        fixed: &[Input::String("type", "PRIVATE")],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("omics", GROUPS, RESOURCES)
}
