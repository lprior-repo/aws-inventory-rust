// Pinned model: .local/aws-models/models/emr-containers/service/2020-10-01/emr-containers-2020-10-01.json
use super::support::{arn, inventory, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "emr-containers",
        vec![
        // Lists information about the specified virtual cluster.
        queried(tagged(inventory("ListVirtualClusters", "virtualClusters", arn("arn")?, token("nextToken", "nextToken")?)?, "tags")?, &["arn", "id", "name", "state", "securityConfigurationId"])?,
        // Lists security configurations based on a set of parameters.
        queried(tagged(inventory("ListSecurityConfigurations", "securityConfigurations", arn("arn")?, token("nextToken", "nextToken")?)?, "tags")?, &["arn", "id", "name"])?,
        ],
        &[super::s_emr_containers_decisions_0::ENTRIES],
        &[
            // jobTemplateData exposes command arguments, Spark parameters, and parameter default values.
            ("ListJobTemplates", DenyReason::Content),
            // The response includes executable entry points and argument values, not a summary projection.
            ("ListJobRuns", DenyReason::Content),
            // applicationConfiguration.properties is an arbitrary customer value map, not a secret-free summary.
            ("ListManagedEndpoints", DenyReason::Secret),
        ],
    )
}
