use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListComponents",
            policy::query(
                policy::fixed(
                    policy::enumerate(
                        "components",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "scope",
                    "PRIVATE",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListComponentVersions",
            policy::query(
                policy::bind(
                    policy::enumerate(
                        "componentVersions",
                        policy::arn("arn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "arn",
                    "ListComponents",
                    "arn",
                )?,
                &["arn"],
            )?,
        ),
        (
            "ListCoreDevices",
            policy::query(
                policy::enumerate(
                    "coreDevices",
                    policy::key("coreDeviceThingName")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["coreDeviceThingName"],
            )?,
        ),
        (
            "ListDeployments",
            policy::query(
                policy::fixed(
                    policy::enumerate(
                        "deployments",
                        policy::key("deploymentId")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "historyFilter",
                    "ALL",
                )?,
                &["deploymentId"],
            )?,
        ),
        (
            "DescribeComponent",
            policy::query(
                policy::tags(
                    policy::bind(
                        policy::enumerate("$", policy::arn("arn")?, None)?,
                        "arn",
                        "ListComponentVersions",
                        "arn",
                    )?,
                    "tags",
                )?,
                &["arn"],
            )?,
        ),
    ];
    policy::finish(
        "greengrassv2",
        approved,
        DEFERRED,
        "Private component containers, their versions, metadata-only descriptions, core-device registrations and deployment summaries are approved. Recipes and artifact URLs are denied; configuration payload expansion is not approved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Content, CONTENT),
        ],
    )
}

const DEFERRED: &[&str] = &[
    "GetConnectivityInfo",
    "GetCoreDevice",
    "GetDeployment",
    "GetServiceRoleForAccount",
    "ListClientDevicesAssociatedWithCoreDevice",
    "ListEffectiveDeployments",
    "ListInstalledComponents",
    "ListTagsForResource",
    "ResolveComponentCandidates",
];

const MUTATING: &[&str] = &[
    "AssociateServiceRoleToAccount",
    "BatchAssociateClientDeviceWithCoreDevice",
    "BatchDisassociateClientDeviceFromCoreDevice",
    "CancelDeployment",
    "CreateComponentVersion",
    "CreateDeployment",
    "DeleteComponent",
    "DeleteCoreDevice",
    "DeleteDeployment",
    "DisassociateServiceRoleFromAccount",
    "TagResource",
    "UntagResource",
    "UpdateConnectivityInfo",
];

const CONTENT: &[&str] = &["GetComponent", "GetComponentVersionArtifact"];
