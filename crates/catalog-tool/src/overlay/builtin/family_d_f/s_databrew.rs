// Pinned model: .local/aws-models/models/databrew/service/2017-07-25/databrew-2017-07-25.json
use super::support::{inventory, key, queried, reviewed, tagged, token};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "databrew",
        vec![
        // Lists all of the DataBrew jobs that are defined.
        queried(tagged(inventory("ListJobs", "Jobs", key(&["Name"])?, token("NextToken", "NextToken")?)?, "Tags")?, &["ResourceArn", "RoleArn", "EncryptionKeyArn", "AccountId", "DatasetName", "Name", "Type", "ProjectName"])?,
        // Lists all of the DataBrew projects that are defined.
        queried(tagged(inventory("ListProjects", "Projects", key(&["Name"])?, token("NextToken", "NextToken")?)?, "Tags")?, &["ResourceArn", "RoleArn", "AccountId", "DatasetName", "Name", "RecipeName"])?,
        // List all rulesets available in the current account or rulesets associated with a specific resource (dataset).
        queried(tagged(inventory("ListRulesets", "Rulesets", key(&["Name"])?, token("NextToken", "NextToken")?)?, "Tags")?, &["ResourceArn", "AccountId", "Name"])?,
        // Lists the DataBrew schedules that are defined.
        queried(tagged(inventory("ListSchedules", "Schedules", key(&["Name"])?, token("NextToken", "NextToken")?)?, "Tags")?, &["ResourceArn", "AccountId", "Name"])?,
        ],
        &[super::s_databrew_decisions_0::ENTRIES, super::s_databrew_decisions_1::ENTRIES],
        &[
            // Datasets[].Input.DatabaseInputDefinition.QueryString returns stored custom SQL, not just dataset metadata.
            ("ListDatasets", DenyReason::Content),
        ],
    )
}
