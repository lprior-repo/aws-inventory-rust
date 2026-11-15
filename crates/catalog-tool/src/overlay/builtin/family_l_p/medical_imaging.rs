// Pinned Smithy source: .local/aws-models/models/medical-imaging/service/2023-07-19/medical-imaging-2023-07-19.json
use super::policy::{Group, Id, Resource, compile};
use super::super::{name, path};
use crate::overlay::{
    Paginator, Scope, TagEndpoint, TagEndpointParent, TagEndpointRequest, TagEndpointSelector,
};
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CopyImageSet", "CreateDatastore", "DeleteDatastore", "DeleteImageSet", 
            "StartDICOMImportJob", "TagResource", "UntagResource", "UpdateImageSetMetadata",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetDICOMImportJob",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (datastoreId, jobId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListDICOMImportJobs", "SearchImageSets",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (datastoreId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetImageFrame", "GetImageSet", "GetImageSetMetadata",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &[
            "ListImageSetVersions",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (datastoreId, imageSetId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Preserve the real ARN; only absent/null ARNs use the reviewed canonical construction.
    Resource {
        operation: "ListDatastores",
        items: "datastoreSummaries",
        id: Id::PreferArn("datastoreArn",
            "arn:{partition}:medical-imaging:{region}:{account}:datastore/{datastoreId}",
            "HealthImaging datastore ARNs use the required datastoreId under the trusted partition, region and account; the same datastoreArn identifies list and metadata detail responses."),
        scope: Scope::Region, tokens: Some(("nextToken", "nextToken")),
        query: &["datastoreId", "datastoreName", "datastoreStatus", "datastoreArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Metadata-only expansion uses the discovered datastoreId, never imaging content.
    Resource {
        operation: "GetDatastore",
        items: "datastoreProperties",
        id: Id::PreferArn("datastoreArn",
            "arn:{partition}:medical-imaging:{region}:{account}:datastore/{datastoreId}",
            "GetDatastore returns the same datastore resource as ListDatastores; when its optional datastoreArn is absent, required datastoreId and trusted context construct its canonical ARN."),
        scope: Scope::Region, tokens: None,
        query: &["datastoreId", "datastoreName", "datastoreStatus", "datastoreArn", "kmsKeyArn"],
        tags: None,
        bindings: &[("datastoreId", "ListDatastores", "datastoreId")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut inventory = compile("medical-imaging", GROUPS, RESOURCES)?;
    let tags = TagEndpoint {
        parents: vec![TagEndpointParent {
            operation: name("ListDatastores")?,
            request: TagEndpointRequest::Owner { input: path("resourceArn")? },
        }],
        selector: TagEndpointSelector::Map { path: path("tags")? },
    }.operation(Scope::Region, Paginator::None)?;
    inventory.operations.insert(name("ListTagsForResource")?, tags);
    Ok(inventory)
}
