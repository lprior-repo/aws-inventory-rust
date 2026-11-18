// Pinned model: .local/aws-models/models/datasync/service/2018-11-09/datasync-2018-11-09.json
use super::support::{arn, bound, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "datasync",
        vec![
            // Returns a list of DataSync agents that belong to an Amazon Web Services account in the Amazon Web Services Region specified in the request.
            queried(
                inventory(
                    "ListAgents",
                    "Agents",
                    arn("AgentArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["AgentArn", "Name", "Status"],
            )?,
            // Returns a list of source and destination locations.
            queried(
                inventory(
                    "ListLocations",
                    "Locations",
                    arn("LocationArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["LocationArn"],
            )?,
            // Returns a list of the DataSync tasks you created.
            queried(
                inventory(
                    "ListTasks",
                    "Tasks",
                    arn("TaskArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["TaskArn", "Status", "Name"],
            )?,
            // Returns a list of executions for an DataSync transfer task.
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
