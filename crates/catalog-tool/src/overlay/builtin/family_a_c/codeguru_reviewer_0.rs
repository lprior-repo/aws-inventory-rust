// Pinned source: .local/aws-models/models/codeguru-reviewer/service/2019-09-19/codeguru-reviewer-2019-09-19.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("AssociateRepository", deferred("Model requires Repository. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Use to associate an Amazon Web Services CodeCommit repository or a repository managed by Amazon Web Services CodeStar Connections with Amazon CodeGuru Reviewer. When you associate a repository, CodeGuru Reviewer reviews source…")),
        ("CreateCodeReview", deferred("Model requires Name, RepositoryAssociationArn, Type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Use to create a code review with a CodeReviewType of RepositoryAnalysis . This type of code review analyzes all code under a specified branch in an associated repository. PullRequest code reviews are automatically triggered by a…")),
        ("DescribeCodeReview", deferred("Model requires CodeReviewArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the metadata associated with the code review along with its status.")),
        ("DescribeRecommendationFeedback", deferred("Model requires CodeReviewArn, RecommendationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes the customer feedback for a CodeGuru Reviewer recommendation.")),
        // Model: Returns a RepositoryAssociation object that contains information about the requested repository association.
        ("DescribeRepositoryAssociation", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("AssociationArn")?, Binding::From { operation: name("ListRepositoryAssociations")?, path: path("AssociationArn")? })].into_iter().collect(),
            queryable: vec![path("AssociationArn")?],
            ..read("RepositoryAssociation", Identity::Arn(path("AssociationArn")?), Scope::Region, Paginator::None)?
        }),
        ("DisassociateRepository", deferred("Model requires AssociationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the association between Amazon CodeGuru Reviewer and a repository.")),
        ("ListCodeReviews", deferred("Model requires Type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all the code reviews that the customer has created in the past 90 days.")),
        ("ListRecommendationFeedback", deferred("Model requires CodeReviewArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of RecommendationFeedbackSummary objects that contain customer recommendation feedback for all CodeGuru Reviewer users.")),
        ("ListRecommendations", deferred("Model requires CodeReviewArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the list of all recommendations for a completed code review.")),
        // Model: Returns a list of RepositoryAssociationSummary objects that contain summary information about a repository association. You can filter the returned list by ProviderType , Name , State , and Owner .
        ("ListRepositoryAssociations", OperationOverlay {
            queryable: vec![path("AssociationArn")?],
            ..read("RepositoryAssociationSummaries", Identity::Arn(path("AssociationArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("ListTagsForResource", deferred("Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the list of tags associated with an associated repository resource.")),
        ("PutRecommendationFeedback", deferred("Model requires CodeReviewArn, RecommendationId, Reactions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stores customer feedback for a CodeGuru Reviewer recommendation. When this API is called again with different reactions the previous feedback is overwritten.")),
        ("TagResource", deferred("Model requires resourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds one or more tags to an associated repository.")),
        ("UntagResource", deferred("Model requires resourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a tag from an associated repository.")),
    ])
}
