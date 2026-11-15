// Pinned model: .local/aws-models/models/qconnect/service/2020-10-19/qconnect-2020-10-19.json
use super::super::construction::{Inventory, ResourceIdentity, Input};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListAssistants",
        items: "assistantSummaries",
        identity: ResourceIdentity::Arn("assistantArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("tags"),
        queryable: &["assistantId"],
    },
    Inventory {
        operation: "ListKnowledgeBases",
        items: "knowledgeBaseSummaries",
        identity: ResourceIdentity::Arn("knowledgeBaseArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: Some("tags"),
        queryable: &[],
    },
    Inventory {
        operation: "ListAssistantAssociations",
        items: "assistantAssociationSummaries",
        identity: ResourceIdentity::Arn("assistantAssociationArn"),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[Input::Parent("assistantId", "ListAssistants", "assistantId")],
        tags: Some("tags"),
        queryable: &["assistantArn", "associationData.knowledgeBaseAssociation.knowledgeBaseArn"],
    },
];
