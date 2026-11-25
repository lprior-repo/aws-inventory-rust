use super::support::{arn, bound, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "datasync",
        vec![
            queried(
                inventory(
                    "ListAgents",
                    "Agents",
                    arn("AgentArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["AgentArn", "Name", "Status"],
            )?,
            queried(
                inventory(
                    "ListLocations",
                    "Locations",
                    arn("LocationArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["LocationArn"],
            )?,
            queried(
                inventory(
                    "ListTasks",
                    "Tasks",
                    arn("TaskArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["TaskArn", "Status", "Name"],
            )?,
            queried(
                bound(
                    inventory(
                        "ListTaskExecutions",
                        "TaskExecutions",
                        arn("TaskExecutionArn")?,
                        token("NextToken", "NextToken")?,
                    )?,
                    &[("TaskArn", "ListTasks", "TaskArn")],
                )?,
                &["TaskExecutionArn", "Status"],
            )?,
        ],
        &[
            super::s_datasync_decisions_0::ENTRIES,
            super::s_datasync_decisions_1::ENTRIES,
        ],
        &[],
    )
}
