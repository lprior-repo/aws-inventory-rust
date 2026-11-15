// Pinned operation purposes: .local/aws-models/models/quicksight/service/2018-04-01/quicksight-2018-04-01.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Updates the permissions for an Amazon QuickSight space.
    ("UpdateSpacePermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds or removes resources from an Amazon QuickSight space.
    ("UpdateSpaceResources", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a template from an existing Amazon Quick Sight analysis or another template.
    ("UpdateTemplate", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the template alias of a template.
    ("UpdateTemplateAlias", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the resource permissions for a template.
    ("UpdateTemplatePermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a theme.
    ("UpdateTheme", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an alias of a theme.
    ("UpdateThemeAlias", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the resource permissions for a theme. Permissions apply to the action to grant or revoke permissions on, for example "quicksight:DescribeTheme" . Theme permissions apply in groupings. Valid groupings include the following for the three levels of permissions, which are user, owner, or no permissions: User "quicksight:DescribeTheme" "quicksight:DescribeThemeAlias" "quicksight:ListThemeAliase
    ("UpdateThemePermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a topic.
    ("UpdateTopic", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the permissions of a topic.
    ("UpdateTopicPermissions", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the permissions of a topic.
    ("UpdateTopicPermissionsV2", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a topic refresh schedule.
    ("UpdateTopicRefreshSchedule", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates the definition of a Q topic.
    ("UpdateTopicV2", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an Amazon Quick Sight user.
    ("UpdateUser", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a custom permissions profile for a user.
    ("UpdateUserCustomPermission", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a VPC connection.
    ("UpdateVPCConnection", Decision::Denied(DenyReason::Mutating)),
];
