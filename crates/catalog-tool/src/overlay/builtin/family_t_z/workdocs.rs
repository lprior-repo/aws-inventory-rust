use super::super::service;
use super::{defer, deny};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "workdocs",
        vec![
            ("AbortDocumentVersionUpload", deny(DenyReason::Mutating)),
            (
                "ActivateUser",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            ("AddResourcePermissions", deny(DenyReason::Mutating)),
            ("CreateComment", deny(DenyReason::Mutating)),
            ("CreateCustomMetadata", deny(DenyReason::Mutating)),
            ("CreateFolder", deny(DenyReason::Mutating)),
            ("CreateLabels", deny(DenyReason::Mutating)),
            ("CreateNotificationSubscription", deny(DenyReason::Mutating)),
            ("CreateUser", deny(DenyReason::Mutating)),
            (
                "DeactivateUser",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            ("DeleteComment", deny(DenyReason::Mutating)),
            ("DeleteCustomMetadata", deny(DenyReason::Mutating)),
            ("DeleteDocument", deny(DenyReason::Mutating)),
            ("DeleteDocumentVersion", deny(DenyReason::Mutating)),
            ("DeleteFolder", deny(DenyReason::Mutating)),
            ("DeleteFolderContents", deny(DenyReason::Mutating)),
            ("DeleteLabels", deny(DenyReason::Mutating)),
            ("DeleteNotificationSubscription", deny(DenyReason::Mutating)),
            ("DeleteUser", deny(DenyReason::Mutating)),
            ("DescribeActivities", deny(DenyReason::Content)),
            ("DescribeComments", deny(DenyReason::Content)),
            ("DescribeDocumentVersions", deny(DenyReason::Content)),
            ("DescribeFolderContents", deny(DenyReason::Content)),
            (
                "DescribeGroups",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            (
                "DescribeNotificationSubscriptions",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            (
                "DescribeResourcePermissions",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            (
                "DescribeRootFolders",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            (
                "DescribeUsers",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            (
                "GetCurrentUser",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            ("GetDocument", deny(DenyReason::Content)),
            ("GetDocumentPath", deny(DenyReason::Content)),
            ("GetDocumentVersion", deny(DenyReason::Content)),
            (
                "GetFolder",
                defer(
                    "Administrative calls require an organization ID or user authentication token; this API has no account organization enumeration root.",
                ),
            ),
            ("GetFolderPath", deny(DenyReason::Content)),
            ("GetResources", deny(DenyReason::Content)),
            ("InitiateDocumentVersionUpload", deny(DenyReason::Mutating)),
            ("RemoveAllResourcePermissions", deny(DenyReason::Mutating)),
            ("RemoveResourcePermission", deny(DenyReason::Mutating)),
            ("RestoreDocumentVersions", deny(DenyReason::Mutating)),
            ("SearchResources", deny(DenyReason::Content)),
            ("UpdateDocument", deny(DenyReason::Mutating)),
            ("UpdateDocumentVersion", deny(DenyReason::Mutating)),
            ("UpdateFolder", deny(DenyReason::Mutating)),
            ("UpdateUser", deny(DenyReason::Mutating)),
        ],
    )
}
