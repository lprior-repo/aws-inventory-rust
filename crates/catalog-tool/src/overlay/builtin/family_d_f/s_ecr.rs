// Pinned model: .local/aws-models/models/ecr/service/2015-09-21/ecr-2015-09-21.json
use super::support::{arn, bound, inventory, key, queried, reviewed, token};
use crate::overlay::{ExecutionBranch, FixedValue, MemberPath, NameError, Scope, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "ecr",
        vec![
        // Describes image repositories in a registry.
        queried(inventory("DescribeRepositories", "repositories", arn("repositoryArn")?, token("nextToken", "nextToken")?)?, &["repositoryArn", "registryId", "repositoryName"])?,
        // Returns the pull through cache rules for a registry.
        queried(inventory("DescribePullThroughCacheRules", "pullThroughCacheRules", key(&["registryId", "ecrRepositoryPrefix"])?, token("nextToken", "nextToken")?)?, &["registryId", "ecrRepositoryPrefix", "credentialArn", "customRoleArn"])?,
        // Returns details about the repository creation templates in a registry.
        queried(inventory("DescribeRepositoryCreationTemplates", "repositoryCreationTemplates", key(&["prefix"])?, token("nextToken", "nextToken")?)?, &["prefix", "customRoleArn"])?,
        // Lists the replication backfills for a registry, including their status and progress.
        queried(inventory("ListReplicationBackfills", "backfills", key(&["replicationBackfillId"])?, token("nextToken", "nextToken")?)?, &["replicationBackfillId", "status"])?,
        // Every modeled lifecycle status has an independent repository-bound pagination chain.
        images()?,
        ],
        &[super::s_ecr_decisions_0::ENTRIES, super::s_ecr_decisions_1::ENTRIES],
        &[
            // Retrieves an authorization token.
            ("GetAuthorizationToken", DenyReason::Secret),
            // Gets detailed information for an image.
            ("BatchGetImage", DenyReason::Content),
            // Retrieves the pre-signed Amazon S3 download URL corresponding to an image layer.
            ("GetDownloadUrlForLayer", DenyReason::Content),
        ],
    )
}

fn images() -> Result<super::support::Entry, NameError> {
    let mut entry = queried(bound(inventory(
        "DescribeImages", "imageDetails", key(&["registryId", "repositoryName", "imageDigest"])?,
        token("nextToken", "nextToken")?,
    )?, &[("repositoryName", "DescribeRepositories", "repositoryName")])?,
        &["registryId", "repositoryName", "imageDigest", "imageStatus"])?;
    entry.1.branches = ["ACTIVE", "ARCHIVED", "ACTIVATING"].into_iter().map(|status| {
        Ok(ExecutionBranch {
            discriminator: status.to_owned(),
            fixed: [(MemberPath::new("filter.imageStatus")?, FixedValue::String(status.to_owned()))].into_iter().collect(),
            bind: Default::default(),
            emission: None,
            row_policy: None,
            scope: Scope::Region,
            request_region: None,
        })
    }).collect::<Result<_, NameError>>()?;
    Ok(entry)
}
