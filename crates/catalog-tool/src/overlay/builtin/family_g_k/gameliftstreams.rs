use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListApplications",
            policy::query(
                policy::enumerate(
                    "Items",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
        (
            "ListStreamGroups",
            policy::query(
                policy::enumerate(
                    "Items",
                    policy::arn("Arn")?,
                    Some(("NextToken", "NextToken")),
                )?,
                &["Arn"],
            )?,
        ),
    ];
    policy::finish(
        "gameliftstreams",
        approved,
        DEFERRED,
        "Applications and stream-group summaries are metadata. Stream-session URLs and session details need an independent capability/content review; collection is limited to the two approved resource types.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

const DEFERRED: &[&str] = &[
    "GetApplication",
    "GetStreamGroup",
    "GetStreamSession",
    "GetStreamUrl",
    "ListApplicationShaderCaches",
    "ListStreamSessions",
    "ListStreamSessionsByAccount",
    "ListStreamUrls",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "AddStreamGroupLocations",
    "AssociateApplications",
    "CreateApplication",
    "CreateStreamGroup",
    "CreateStreamSessionAdminShell",
    "CreateStreamSessionConnection",
    "CreateStreamUrl",
    "DeleteApplication",
    "DeleteStreamGroup",
    "DisassociateApplications",
    "ExportStreamSessionFiles",
    "RemoveStreamGroupLocations",
    "RevokeStreamUrl",
    "StartStreamSession",
    "TagResource",
    "TerminateStreamSession",
    "UntagResource",
    "UpdateApplication",
    "UpdateStreamGroup",
];
