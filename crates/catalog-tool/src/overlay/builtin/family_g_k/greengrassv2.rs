//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/greengrassv2/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        ("ListComponents", policy::query(policy::fixed(policy::enumerate("components", policy::arn("arn")?, Some(("nextToken", "nextToken")))?, "scope", "PRIVATE")?, &["arn"])?),
        ("ListComponentVersions", policy::query(policy::bind(policy::enumerate("componentVersions", policy::arn("arn")?, Some(("nextToken", "nextToken")))?, "arn", "ListComponents", "arn")?, &["arn"])?),
        ("ListCoreDevices", policy::query(policy::enumerate("coreDevices", policy::key("coreDeviceThingName")?, Some(("nextToken", "nextToken")))?, &["coreDeviceThingName"])?),
        ("ListDeployments", policy::query(policy::fixed(policy::enumerate("deployments", policy::key("deploymentId")?, Some(("nextToken", "nextToken")))?, "historyFilter", "ALL")?, &["deploymentId"])?),
        ("DescribeComponent", policy::query(policy::tags(policy::bind(policy::enumerate("$", policy::arn("arn")?, None)?, "arn", "ListComponentVersions", "arn")?, "tags")?, &["arn"])?),
    ];
    policy::finish(
        "greengrassv2", approved, DEFERRED,
        "Private component containers, their versions, metadata-only descriptions, core-device registrations and deployment summaries are approved. Recipes and artifact URLs are denied; configuration payload expansion is not approved.",
        &[(policy::DenyReason::Mutating, MUTATING), (policy::DenyReason::Content, CONTENT)],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "GetConnectivityInfo", "GetCoreDevice", "GetDeployment", "GetServiceRoleForAccount",
    "ListClientDevicesAssociatedWithCoreDevice", "ListEffectiveDeployments", "ListInstalledComponents",
    "ListTagsForResource", "ResolveComponentCandidates",
];

const MUTATING: &[&str] = &[
    "AssociateServiceRoleToAccount", "BatchAssociateClientDeviceWithCoreDevice",
    "BatchDisassociateClientDeviceFromCoreDevice", "CancelDeployment", "CreateComponentVersion",
    "CreateDeployment", "DeleteComponent", "DeleteCoreDevice", "DeleteDeployment",
    "DisassociateServiceRoleFromAccount", "TagResource", "UntagResource", "UpdateConnectivityInfo",
];

const CONTENT: &[&str] = &[
    "GetComponent", "GetComponentVersionArtifact",
];
