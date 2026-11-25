use super::support::{arn, bound, inventory, key, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{ExecutionBranch, FixedValue, MemberPath, NameError, Scope, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "ecr",
        vec![
            queried(
                inventory(
                    "DescribeRepositories",
                    "repositories",
                    arn("repositoryArn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["repositoryArn", "registryId", "repositoryName"],
            )?,
            queried(
                inventory(
                    "DescribePullThroughCacheRules",
                    "pullThroughCacheRules",
                    key(&["registryId", "ecrRepositoryPrefix"])?,
                    token("nextToken", "nextToken")?,
                )?,
                &[
                    "registryId",
                    "ecrRepositoryPrefix",
                    "credentialArn",
                    "customRoleArn",
                ],
            )?,
            queried(
                inventory(
                    "DescribeRepositoryCreationTemplates",
                    "repositoryCreationTemplates",
                    key(&["prefix"])?,
                    token("nextToken", "nextToken")?,
                )?,
                &["prefix", "customRoleArn"],
            )?,
            queried(
                inventory(
                    "ListReplicationBackfills",
                    "backfills",
                    key(&["replicationBackfillId"])?,
                    token("nextToken", "nextToken")?,
                )?,
                &["replicationBackfillId", "status"],
            )?,
            images()?,
        ],
        &[
            super::s_ecr_decisions_0::ENTRIES,
            super::s_ecr_decisions_1::ENTRIES,
        ],
        &[
            ("GetAuthorizationToken", DenyReason::Secret),
            ("BatchGetImage", DenyReason::Content),
            ("GetDownloadUrlForLayer", DenyReason::Content),
        ],
    )
}

fn images() -> Result<super::support::Entry, NameError> {
    let mut entry = queried(
        bound(
            inventory(
                "DescribeImages",
                "imageDetails",
                key(&["registryId", "repositoryName", "imageDigest"])?,
                token("nextToken", "nextToken")?,
            )?,
            &[("repositoryName", "DescribeRepositories", "repositoryName")],
        )?,
        &["registryId", "repositoryName", "imageDigest", "imageStatus"],
    )?;
    entry.1.branches = ["ACTIVE", "ARCHIVED", "ACTIVATING"]
        .into_iter()
        .map(|status| {
            Ok(ExecutionBranch {
                discriminator: status.to_owned(),
                fixed: [(
                    MemberPath::new("filter.imageStatus")?,
                    FixedValue::String(status.to_owned()),
                )]
                .into_iter()
                .collect(),
                bind: Default::default(),
                emission: None,
                row_policy: None,
                scope: Scope::Region,
                request_region: None,
            })
        })
        .collect::<Result<_, NameError>>()?;
    Ok(entry)
}
