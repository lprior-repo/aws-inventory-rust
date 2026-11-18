//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/grafana/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![];
    policy::finish(
        "grafana",
        approved,
        DEFERRED,
        "Workspace list/detail can return grafanaToken and are denied in full, not made callable by projection. Other metadata needs independently reviewed workspace discovery and bindings; no resource collection is approved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Secret, SECRET),
        ],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeWorkspaceAuthentication",
    "DescribeWorkspaceConfiguration",
    "ListPermissions",
    "ListTagsForResource",
    "ListVersions",
    "ListWorkspaceServiceAccountTokens",
    "ListWorkspaceServiceAccounts",
];

const MUTATING: &[&str] = &[
    "AssociateLicense",
    "CreateWorkspace",
    "CreateWorkspaceApiKey",
    "CreateWorkspaceServiceAccount",
    "CreateWorkspaceServiceAccountToken",
    "DeleteWorkspace",
    "DeleteWorkspaceApiKey",
    "DeleteWorkspaceServiceAccount",
    "DeleteWorkspaceServiceAccountToken",
    "DisassociateLicense",
    "TagResource",
    "UntagResource",
    "UpdatePermissions",
    "UpdateWorkspace",
    "UpdateWorkspaceAuthentication",
    "UpdateWorkspaceConfiguration",
];

const SECRET: &[&str] = &["DescribeWorkspace", "ListWorkspaces"];
