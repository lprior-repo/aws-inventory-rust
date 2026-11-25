use super::support::{arn, inventory, queried, reviewed, tagged, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "emr-containers",
        vec![
            queried(
                tagged(
                    inventory(
                        "ListVirtualClusters",
                        "virtualClusters",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &["arn", "id", "name", "state", "securityConfigurationId"],
            )?,
            queried(
                tagged(
                    inventory(
                        "ListSecurityConfigurations",
                        "securityConfigurations",
                        arn("arn")?,
                        token("nextToken", "nextToken")?,
                    )?,
                    "tags",
                )?,
                &["arn", "id", "name"],
            )?,
        ],
        &[super::s_emr_containers_decisions_0::ENTRIES],
        &[
            ("ListJobTemplates", DenyReason::Content),
            ("ListJobRuns", DenyReason::Content),
            ("ListManagedEndpoints", DenyReason::Secret),
        ],
    )
}
