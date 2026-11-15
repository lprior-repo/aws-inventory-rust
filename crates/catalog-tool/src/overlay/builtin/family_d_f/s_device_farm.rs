// Pinned model: .local/aws-models/models/device-farm/service/2015-06-23/device-farm-2015-06-23.json
use super::support::{arn, inventory, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};
use crate::classes::DenyReason;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "device-farm",
        vec![
        // Returns information about the private device instances associated with one or more AWS accounts.
        queried(inventory("ListDeviceInstances", "deviceInstances", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "status", "udid"])?,
        // Returns information about all the instance profiles in an AWS account.
        queried(inventory("ListInstanceProfiles", "instanceProfiles", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "name"])?,
        // Gets a list of all Selenium testing projects in your account.
        queried(inventory("ListTestGridProjects", "testGridProjects", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "name"])?,
        // Returns information about all Amazon Virtual Private Cloud (VPC) endpoint configurations in the AWS account.
        queried(inventory("ListVPCEConfigurations", "vpceConfigurations", arn("arn")?, token("nextToken", "nextToken")?)?, &["arn", "vpceConfigurationName", "vpceServiceName", "serviceDnsName"])?,
        ],
        &[super::s_device_farm_decisions_0::ENTRIES, super::s_device_farm_decisions_1::ENTRIES],
        &[
            // Projects includes environmentVariables[].value; no documented omission of secret-bearing environment values. Output projection does not authorize this API.
            ("ListProjects", DenyReason::Secret),
            // Gets information about a project.
            ("GetProject", DenyReason::Secret),
            // Gets information about artifacts.
            ("ListArtifacts", DenyReason::Content),
            // Gets information about uploads, given an AWS Device Farm project ARN.
            ("ListUploads", DenyReason::Content),
            // Gets information about an upload.
            ("GetUpload", DenyReason::Content),
            // A session is an instance of a browser created through a RemoteWebDriver with the URL from CreateTestGridUrlResult .
            ("GetTestGridSession", DenyReason::Content),
            // seleniumProperties contains the customer's complete Selenium options and parameter JSON.
            ("ListTestGridSessions", DenyReason::Content),
            // Creates a signed, short-term URL that can be passed to a Selenium RemoteWebDriver constructor.
            ("CreateTestGridUrl", DenyReason::Secret),
        ],
    )
}
