use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateConfigurationSet",
            "CreateConfigurationSetEventDestination",
            "CreateDedicatedIpPool",
            "CreateDeliverabilityTestReport",
            "CreateEmailIdentity",
            "DeleteConfigurationSet",
            "DeleteConfigurationSetEventDestination",
            "DeleteDedicatedIpPool",
            "DeleteEmailIdentity",
            "PutAccountDedicatedIpWarmupAttributes",
            "PutAccountSendingAttributes",
            "PutConfigurationSetDeliveryOptions",
            "PutConfigurationSetReputationOptions",
            "PutConfigurationSetSendingOptions",
            "PutConfigurationSetTrackingOptions",
            "PutDedicatedIpInPool",
            "PutDedicatedIpWarmupAttributes",
            "PutDeliverabilityDashboardOption",
            "PutEmailIdentityDkimAttributes",
            "PutEmailIdentityFeedbackAttributes",
            "PutEmailIdentityMailFromAttributes",
            "SendEmail",
            "TagResource",
            "UntagResource",
            "UpdateConfigurationSetEventDestination",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetAccount",
            "GetDedicatedIps",
            "GetDeliverabilityDashboardOptions",
            "ListDeliverabilityTestReports",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["GetBlacklistReports"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (BlacklistItemNames); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetConfigurationSetEventDestinations"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ConfigurationSetName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDedicatedIp"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Ip); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDeliverabilityTestReport"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ReportId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDomainDeliverabilityCampaign"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (CampaignId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDomainStatisticsReport"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (Domain, StartDate, EndDate); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetEmailIdentity"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (EmailIdentity); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListDomainDeliverabilityCampaigns"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (StartDate, EndDate, SubscribedDomain); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    Resource {
        operation: "ListConfigurationSets",
        items: "ConfigurationSets",
        id: Id::Key(&["$"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["$"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListDedicatedIpPools",
        items: "DedicatedIpPools",
        id: Id::Key(&["$"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["$"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListEmailIdentities",
        items: "EmailIdentities",
        id: Id::Key(&["IdentityName"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["IdentityName", "IdentityType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "GetConfigurationSet",
        items: "$",
        id: Id::Key(&["ConfigurationSetName"]),
        scope: Scope::Region,
        tokens: None,
        query: &["ConfigurationSetName"],
        tags: Some("Tags"),
        bindings: &[("ConfigurationSetName", "ListConfigurationSets", "$")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pinpoint-email", GROUPS, RESOURCES)
}
