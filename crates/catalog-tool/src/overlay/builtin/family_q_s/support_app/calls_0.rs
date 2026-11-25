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
