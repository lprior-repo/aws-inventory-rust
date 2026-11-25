use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateBroker",
            "CreateConfiguration",
            "CreateTags",
            "CreateUser",
            "DeleteBroker",
            "DeleteConfiguration",
            "DeleteTags",
            "DeleteUser",
            "Promote",
            "RebootBroker",
            "UpdateBroker",
            "UpdateConfiguration",
            "UpdateUser",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["DescribeBrokerEngineTypes", "DescribeBrokerInstanceOptions"],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["DescribeConfiguration", "ListConfigurationRevisions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ConfigurationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeConfigurationRevision"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ConfigurationId, ConfigurationRevision); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeSharedResources", "ListUsers"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (BrokerId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeUser"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (BrokerId, Username); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTags"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListBrokers",
        items: "BrokerSummaries",
        id: Id::Arn("BrokerArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "BrokerArn",
            "BrokerId",
            "BrokerName",
            "BrokerState",
            "EngineType",
            "HostInstanceType",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListConfigurations",
        items: "Configurations",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "EngineType", "Id", "Name"],
        tags: Some("Tags"),
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "DescribeBroker",
        items: "$",
        id: Id::Arn("BrokerArn"),
        scope: Scope::Region,
        tokens: None,
        query: &[
            "BrokerArn",
            "BrokerId",
            "BrokerName",
            "BrokerState",
            "EngineType",
            "HostInstanceType",
            "PendingHostInstanceType",
            "StorageType",
        ],
        tags: Some("Tags"),
        bindings: &[("BrokerId", "ListBrokers", "BrokerId")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mq", GROUPS, RESOURCES)
}
