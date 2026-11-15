// Pinned Smithy source: .local/aws-models/models/partnercentral-selling/service/2022-07-26/partnercentral-selling-2022-07-26.json
use super::policy::{Group, Id, Resource, Input, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptEngagementInvitation", "AssignOpportunity", "AssociateOpportunity", 
            "CreateEngagement", "CreateEngagementContext", "CreateEngagementInvitation", 
            "CreateOpportunity", "CreateResourceSnapshot", "CreateResourceSnapshotJob", 
            "DeleteResourceSnapshotJob", "DisassociateOpportunity", "PutSellingSystemSettings", 
            "RejectEngagementInvitation", "StartEngagementByAcceptingInvitationTask", 
            "StartEngagementFromOpportunityTask", "StartOpportunityFromEngagementTask", 
            "StartProspectingFromEngagementTask", "StartResourceSnapshotJob", 
            "StopResourceSnapshotJob", "SubmitOpportunity", "TagResource", "UntagResource", 
            "UpdateEngagementContext", "UpdateOpportunity",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetAwsOpportunitySummary",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Catalog, RelatedOpportunityIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetEngagement", "GetEngagementInvitation", "GetOpportunity", 
            "ListEngagementMembers",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Catalog, Identifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetProspectingFromEngagementTask",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Catalog, TaskIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetResourceSnapshot",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Catalog, EngagementIdentifier, ResourceType, ResourceIdentifier, ResourceSnapshotTemplateIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetResourceSnapshotJob",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Catalog, ResourceSnapshotJobIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetSellingSystemSettings", "ListEngagementByAcceptingInvitationTasks", 
            "ListEngagementFromOpportunityTasks", "ListEngagementResourceAssociations", 
            "ListOpportunities", "ListOpportunityFromEngagementTasks", 
            "ListProspectingFromEngagementTasks", "ListResourceSnapshotJobs",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Catalog); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListEngagementInvitations",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Catalog, ParticipantType); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListResourceSnapshots",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Catalog, EngagementIdentifier); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // Retrieves a list of Partner Solutions that the partner registered on Partner Central. This API is used to generate a list of solutions that an end user selects from for association
    Resource {
        operation: "ListSolutions",
        items: "SolutionSummaries", id: Id::Arn("Arn"),
        scope: Scope::Account, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "Status", "AwsMarketplaceSolutionArn"],
        tags: None,
        bindings: &[],
        fixed: &[Input::String("Catalog", "AWS")],
    },
    // This action allows users to retrieve a list of Engagement records from Partner Central. This action can be used to manage and track various engagements across different stages of t
    Resource {
        operation: "ListEngagements",
        items: "EngagementSummaryList", id: Id::Arn("Arn"),
        scope: Scope::Account, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id"],
        tags: None,
        bindings: &[],
        fixed: &[Input::String("Catalog", "AWS")],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("partnercentral-selling", GROUPS, RESOURCES)
}
