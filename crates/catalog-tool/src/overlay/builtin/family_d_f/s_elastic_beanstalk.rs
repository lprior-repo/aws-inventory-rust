use super::support::{arn, inventory, queried, reviewed, token};
use crate::overlay::{NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "elastic-beanstalk",
        vec![
            queried(
                inventory(
                    "DescribeApplications",
                    "Applications",
                    arn("ApplicationArn")?,
                    Paginator::None,
                )?,
                &["ApplicationArn", "ApplicationName"],
            )?,
            queried(
                inventory(
                    "DescribeApplicationVersions",
                    "ApplicationVersions",
                    arn("ApplicationVersionArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &["ApplicationVersionArn", "ApplicationName", "Status"],
            )?,
            queried(
                inventory(
                    "DescribeEnvironments",
                    "Environments",
                    arn("EnvironmentArn")?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "EnvironmentArn",
                    "OperationsRole",
                    "EnvironmentName",
                    "EnvironmentId",
                    "ApplicationName",
                    "SolutionStackName",
                    "TemplateName",
                    "CNAME",
                    "Status",
                    "HealthStatus",
                ],
            )?,
        ],
        &[
            super::s_elastic_beanstalk_decisions_0::ENTRIES,
            super::s_elastic_beanstalk_decisions_1::ENTRIES,
        ],
        &[],
    )
}
