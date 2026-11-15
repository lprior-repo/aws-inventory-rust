use super::super::policy::{Id, Resource};
use crate::overlay::Scope;

pub(super) const RESOURCES: &[Resource] = &[
    // Produces list of channels that have been created
    Resource {
        operation: "ListChannels",
        items: "Channels", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "RoleArn", "State"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // List input devices
    Resource {
        operation: "ListInputDevices",
        items: "InputDevices", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ConnectionState", "DeviceSettingsSyncState", "DeviceUpdateStatus", "Id", "Name", "Type", "OutputType"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Produces a list of Input Security Groups for an account
    Resource {
        operation: "ListInputSecurityGroups",
        items: "InputSecurityGroups", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "State"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Produces list of inputs that have been created
    Resource {
        operation: "ListInputs",
        items: "Inputs", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "InputSourceType", "Name", "RoleArn", "State", "Type"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Retrieve a list of the existing multiplexes.
    Resource {
        operation: "ListMultiplexes",
        items: "Multiplexes", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "State"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Retrieve the list of Networks.
    Resource {
        operation: "ListNetworks",
        items: "Networks", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "State"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieve the list of Clusters.
    Resource {
        operation: "ListClusters",
        items: "Clusters", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "ClusterType", "Id", "InstanceRoleArn", "Name", "State"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List all the SdiSources in the AWS account.
    Resource {
        operation: "ListSdiSources",
        items: "SdiSources", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name", "State", "Type"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists signal maps.
    Resource {
        operation: "ListSignalMaps",
        items: "SignalMaps", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "MonitorDeploymentStatus", "Name", "Status"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists cloudwatch alarm templates.
    Resource {
        operation: "ListCloudWatchAlarmTemplates",
        items: "CloudWatchAlarmTemplates", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "GroupId", "Id", "MetricName", "Name", "TargetResourceType"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists cloudwatch alarm template groups.
    Resource {
        operation: "ListCloudWatchAlarmTemplateGroups",
        items: "CloudWatchAlarmTemplateGroups", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists eventbridge rule templates.
    Resource {
        operation: "ListEventBridgeRuleTemplates",
        items: "EventBridgeRuleTemplates", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "EventType", "GroupId", "Id", "Name"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    // Lists eventbridge rule template groups.
    Resource {
        operation: "ListEventBridgeRuleTemplateGroups",
        items: "EventBridgeRuleTemplateGroups", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "Name"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListChannelPlacementGroups", items: "ChannelPlacementGroups", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "ClusterId", "Name", "State"], tags: None,
        bindings: &[("ClusterId", "ListClusters", "Id")], fixed: &[],
    },
    Resource {
        operation: "ListNodes", items: "Nodes", id: Id::Arn("Arn"),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Id", "ClusterId", "Name", "State"], tags: None,
        bindings: &[("ClusterId", "ListClusters", "Id")], fixed: &[],
    },
];
