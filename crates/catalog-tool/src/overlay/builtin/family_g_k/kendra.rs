//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/kendra/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{Identity, IdentityField, MemberPath, NameError, OperationOverlay, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListIndices", policy::query(policy::enumerate("IndexConfigurationSummaryItems", policy::key("Id")?, Some(("NextToken", "NextToken")))?, &["Id"])?),
        ("ListDataSources", child("SummaryItems", "Id")?),
        ("ListFaqs", child("FaqSummaryItems", "Id")?),
        ("ListExperiences", child("SummaryItems", "Id")?),
        ("ListQuerySuggestionsBlockLists", child("BlockListSummaryItems", "Id")?),
        ("ListThesauri", child("ThesaurusSummaryItems", "Id")?),
        ("ListFeaturedResultsSets", child("FeaturedResultsSetSummaryItems", "FeaturedResultsSetId")?),
        // The complete summary contains only Id, not document ACLs or user/group lists.
        ("ListAccessControlConfigurations", child("AccessControlConfigurations", "Id")?),
    ];
    policy::finish(
        "kendra", approved, DEFERRED,
        "Index identifiers and index-qualified child summaries contain configuration/status metadata, not connector credentials, FAQ text, blocked phrases, thesaurus contents or featured query/document bodies. Experience HOME endpoints are hosted URLs, not authenticated search results. Featured-results continuation is explicit without a paginator trait. Query, Retrieve and suggestions remain content-denied.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Content, CONTENT)],
    )
}

fn child(items: &str, id: &str) -> Result<OperationOverlay, NameError> {
    policy::query(policy::bind(
        policy::enumerate(items, Identity::CorrelatedKey(vec![
            IdentityField::Request(MemberPath::new("IndexId")?),
            IdentityField::Selected(MemberPath::selected(id)?),
        ]), Some(("NextToken", "NextToken")))?,
        "IndexId", "ListIndices", "Id",
    )?, &[id])
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "BatchGetDocumentStatus", "DescribeAccessControlConfiguration", "DescribeDataSource",
    "DescribeExperience", "DescribeFaq", "DescribeFeaturedResultsSet", "DescribeIndex",
    "DescribePrincipalMapping", "DescribeQuerySuggestionsBlockList", "DescribeQuerySuggestionsConfig",
    "DescribeThesaurus", "GetSnapshots", "ListDataSourceSyncJobs",
    "ListEntityPersonas", "ListExperienceEntities", "ListGroupsOlderThanOrderingId",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AssociateEntitiesToExperience", "AssociatePersonasToEntities", "BatchDeleteDocument",
    "BatchDeleteFeaturedResultsSet", "BatchPutDocument", "ClearQuerySuggestions",
    "CreateAccessControlConfiguration", "CreateDataSource", "CreateExperience", "CreateFaq",
    "CreateFeaturedResultsSet", "CreateIndex", "CreateQuerySuggestionsBlockList", "CreateThesaurus",
    "DeleteAccessControlConfiguration", "DeleteDataSource", "DeleteExperience", "DeleteFaq", "DeleteIndex",
    "DeletePrincipalMapping", "DeleteQuerySuggestionsBlockList", "DeleteThesaurus",
    "DisassociateEntitiesFromExperience", "DisassociatePersonasFromEntities", "PutPrincipalMapping",
    "StartDataSourceSyncJob", "StopDataSourceSyncJob", "SubmitFeedback", "TagResource", "UntagResource",
    "UpdateAccessControlConfiguration", "UpdateDataSource", "UpdateExperience", "UpdateFeaturedResultsSet",
    "UpdateIndex", "UpdateQuerySuggestionsBlockList", "UpdateQuerySuggestionsConfig", "UpdateThesaurus",
];

const CONTENT: &[&str] = &[
    "GetQuerySuggestions", "Query", "Retrieve",
];
