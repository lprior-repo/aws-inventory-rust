// Pinned model: .local/aws-models/models/ecr-public/service/2020-10-30/ecr-public-2020-10-30.json
use super::support::{arn, bound, inventory, key, queried, reviewed, scoped, token};
use crate::overlay::{NameError, Scope, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "ecr-public",
        vec![
        // Returns details for a public registry.
        queried(scoped(inventory("DescribeRegistries", "registries", arn("registryArn")?, token("nextToken", "nextToken")?)?, Scope::Account), &["registryArn", "registryId"])?,
        // Describes repositories that are in a public registry.
        queried(scoped(inventory("DescribeRepositories", "repositories", arn("repositoryArn")?, token("nextToken", "nextToken")?)?, Scope::Account), &["repositoryArn", "registryId", "repositoryName"])?,
        // Keep registry and repository context in the image identity; image digests can repeat between repositories.
        queried(scoped(bound(inventory("DescribeImages", "imageDetails", key(&["registryId", "repositoryName", "imageDigest"])?, token("nextToken", "nextToken")?)?, &[("repositoryName", "DescribeRepositories", "repositoryName")])?, Scope::Account), &["registryId", "repositoryName", "imageDigest"])?,
        ],
        &[super::s_ecr_public_decisions_0::ENTRIES],
        &[
            // Retrieves an authorization token.
            ("GetAuthorizationToken", DenyReason::Secret),
            // Retrieve catalog metadata for a repository in a public registry.
            ("GetRepositoryCatalogData", DenyReason::Content),
        ],
    )
}
