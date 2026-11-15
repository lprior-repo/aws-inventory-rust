// Pinned Smithy source: .local/aws-models/models/pinpoint-email/service/2018-07-26/pinpoint-email-2018-07-26.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{classes::DenyReason, overlay::{NameError, OverlayClass, ServiceOverlay}};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateConfigurationSet", "CreateConfigurationSetEventDestination", 
            "CreateDedicatedIpPool", "CreateDeliverabilityTestReport", "CreateEmailIdentity", 
            "DeleteConfigurationSet", "DeleteConfigurationSetEventDestination", 
            "DeleteDedicatedIpPool", "DeleteEmailIdentity", 
            "PutAccountDedicatedIpWarmupAttributes", "PutAccountSendingAttributes", 
            "PutConfigurationSetDeliveryOptions", "PutConfigurationSetReputationOptions", 
            "PutConfigurationSetSendingOptions", "PutConfigurationSetTrackingOptions", 
            "PutDedicatedIpInPool", "PutDedicatedIpWarmupAttributes", 
            "PutDeliverabilityDashboardOption", "PutEmailIdentityDkimAttributes", 
            "PutEmailIdentityFeedbackAttributes", "PutEmailIdentityMailFromAttributes", 
            "SendEmail", "TagResource", "UntagResource", 
            "UpdateConfigurationSetEventDestination",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetAccount", "GetDedicatedIps", "GetDeliverabilityDashboardOptions", 
            "ListDeliverabilityTestReports",
        ],
        class: OverlayClass::Defer,
        reason: Some("The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract."),
    },
    Group {
        operations: &[
            "GetBlacklistReports",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (BlacklistItemNames); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
             "GetConfigurationSetEventDestinations",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ConfigurationSetName); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDedicatedIp",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Ip); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDeliverabilityTestReport",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ReportId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDomainDeliverabilityCampaign",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (CampaignId); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetDomainStatisticsReport",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (Domain, StartDate, EndDate); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "GetEmailIdentity",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (EmailIdentity); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListDomainDeliverabilityCampaigns",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (StartDate, EndDate, SubscribedDomain); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
    Group {
        operations: &[
            "ListTagsForResource",
        ],
        class: OverlayClass::Defer,
        reason: Some("Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner."),
    },
];

const RESOURCES: &[Resource] = &[
    // List all of the configuration sets associated with your Amazon Pinpoint account in the current region. In Amazon Pinpoint, configuration sets are groups of rules that you can apply
    Resource {
        operation: "ListConfigurationSets",
        items: "ConfigurationSets", id: Id::Key(&["$"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["$"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List all of the dedicated IP pools that exist in your Amazon Pinpoint account in the current AWS Region.
    Resource {
        operation: "ListDedicatedIpPools",
        items: "DedicatedIpPools", id: Id::Key(&["$"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["$"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of all of the email identities that are associated with your Amazon Pinpoint account. An identity can be either an email address or a domain. This operation returns 
    Resource {
        operation: "ListEmailIdentities",
        items: "EmailIdentities", id: Id::Key(&["IdentityName"]),
        scope: Scope::Region, tokens: Some(("NextToken", "NextToken")),
        query: &["IdentityName", "IdentityType"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Preserve the singleton metadata response, including its inline tag snapshot.
    Resource {
        operation: "GetConfigurationSet",
        items: "$", id: Id::Key(&["ConfigurationSetName"]),
        scope: Scope::Region, tokens: None,
        query: &["ConfigurationSetName"],
        tags: Some("Tags"),
        bindings: &[("ConfigurationSetName", "ListConfigurationSets", "$")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("pinpoint-email", GROUPS, RESOURCES)
}
