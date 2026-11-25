use super::support::{arn, bound, inventory, key, queried, reviewed, scoped, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "ecr-public",
        vec![
            queried(
                scoped(
                    inventory(
                        "DescribeRegistries",
                        "registries",
                        arn("registryArn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    Scope::Account,
                ),
                &["registryArn", "registryId"],
            )?,
            queried(
                scoped(
                    inventory(
                        "DescribeRepositories",
                        "repositories",
                        arn("repositoryArn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    Scope::Account,
                ),
                &["repositoryArn", "registryId", "repositoryName"],
            )?,
            queried(
                scoped(
                    bound(
                        inventory(
                            "DescribeImages",
                            "imageDetails",
                            key(&["registryId", "repositoryName", "imageDigest"])?,
                            token("nextToken", "nextToken")?,
                        )?,
                        &[("repositoryName", "DescribeRepositories", "repositoryName")],
                    )?,
                    Scope::Account,
                ),
                &["registryId", "repositoryName", "imageDigest"],
            )?,
        ],
        &[super::s_ecr_public_decisions_0::ENTRIES],
        &[
            ("GetAuthorizationToken", DenyReason::Secret),
            ("GetRepositoryCatalogData", DenyReason::Content),
        ],
    )
}
