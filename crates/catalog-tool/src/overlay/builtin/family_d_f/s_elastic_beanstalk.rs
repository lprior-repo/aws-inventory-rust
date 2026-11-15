// Pinned model: .local/aws-models/models/elastic-beanstalk/service/2010-12-01/elastic-beanstalk-2010-12-01.json
use super::support::{arn, inventory, queried, reviewed, token};
use crate::overlay::{NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "elastic-beanstalk",
        vec![
        // Returns the descriptions of existing applications.
        queried(inventory("DescribeApplications", "Applications", arn("ApplicationArn")?, Paginator::None)?, &["ApplicationArn", "ApplicationName"])?,
        // Retrieve a list of application versions.
        queried(inventory("DescribeApplicationVersions", "ApplicationVersions", arn("ApplicationVersionArn")?, token("NextToken", "NextToken")?)?, &["ApplicationVersionArn", "ApplicationName", "Status"])?,
        // Returns descriptions for existing environments.
        queried(inventory("DescribeEnvironments", "Environments", arn("EnvironmentArn")?, token("NextToken", "NextToken")?)?, &["EnvironmentArn", "OperationsRole", "EnvironmentName", "EnvironmentId", "ApplicationName", "SolutionStackName", "TemplateName", "CNAME", "Status", "HealthStatus"])?,
        ],
        &[super::s_elastic_beanstalk_decisions_0::ENTRIES, super::s_elastic_beanstalk_decisions_1::ENTRIES],
        &[

        ],
    )
}
