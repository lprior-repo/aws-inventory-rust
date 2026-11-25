use super::support::{arn, inventory, queried, reviewed, token};
use crate::classes::DenyReason;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "device-farm",
        vec![
            queried(
                inventory(
                    "ListDeviceInstances",
                    "deviceInstances",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "status", "udid"],
            )?,
            queried(
                inventory(
                    "ListInstanceProfiles",
                    "instanceProfiles",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "name"],
            )?,
            queried(
                inventory(
                    "ListTestGridProjects",
                    "testGridProjects",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &["arn", "name"],
            )?,
            queried(
                inventory(
                    "ListVPCEConfigurations",
                    "vpceConfigurations",
                    arn("arn")?,
                    token("nextToken", "nextToken")?,
                )?,
                &[
                    "arn",
                    "vpceConfigurationName",
                    "vpceServiceName",
                    "serviceDnsName",
                ],
            )?,
        ],
        &[
            super::s_device_farm_decisions_0::ENTRIES,
            super::s_device_farm_decisions_1::ENTRIES,
        ],
        &[
            ("ListProjects", DenyReason::Secret),
            ("GetProject", DenyReason::Secret),
            ("ListArtifacts", DenyReason::Content),
            ("ListUploads", DenyReason::Content),
            ("GetUpload", DenyReason::Content),
            ("GetTestGridSession", DenyReason::Content),
            ("ListTestGridSessions", DenyReason::Content),
            ("CreateTestGridUrl", DenyReason::Secret),
        ],
    )
}
