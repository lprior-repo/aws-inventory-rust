// Pinned model: .local/aws-models/models/support-app/service/2021-08-20/support-app-2021-08-20.json
use super::super::construction::{Inventory, ResourceIdentity};
use crate::overlay::Scope;

pub(super) const CALLS: &[Inventory] = &[
    Inventory {
        operation: "ListSlackChannelConfigurations",
        items: "slackChannelConfigurations",
        identity: ResourceIdentity::Key(&["teamId", "channelId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
    Inventory {
        operation: "ListSlackWorkspaceConfigurations",
        items: "slackWorkspaceConfigurations",
        identity: ResourceIdentity::Key(&["teamId"]),
        token: Some(("nextToken", "nextToken")),
        scope: Scope::Region,
        input: &[],
        tags: None,
        queryable: &[],
    },
];
