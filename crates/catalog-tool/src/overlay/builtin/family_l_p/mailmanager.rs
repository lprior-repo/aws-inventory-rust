// Pinned Smithy source: .local/aws-models/models/mailmanager/service/2023-10-17/mailmanager-2023-10-17.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "CreateAddonInstance",
            "CreateAddonSubscription",
            "CreateAddressList",
            "CreateAddressListImportJob",
            "CreateArchive",
            "CreateIngressPoint",
            "CreateRelay",
            "CreateRuleSet",
            "CreateTrafficPolicy",
            "DeleteAddonInstance",
            "DeleteAddonSubscription",
            "DeleteAddressList",
            "DeleteArchive",
            "DeleteIngressPoint",
            "DeleteRelay",
            "DeleteRuleSet",
            "DeleteTrafficPolicy",
            "DeregisterMemberFromAddressList",
            "RegisterMemberToAddressList",
            "StartAddressListImportJob",
            "StartArchiveExport",
            "StartArchiveSearch",
            "StopAddressListImportJob",
            "StopArchiveExport",
            "StopArchiveSearch",
            "TagResource",
            "UntagResource",
            "UpdateArchive",
            "UpdateIngressPoint",
            "UpdateRelay",
            "UpdateRuleSet",
            "UpdateTrafficPolicy",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["GetAddonInstance"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (AddonInstanceId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAddonSubscription"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (AddonSubscriptionId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAddressList", "ListAddressListImportJobs"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (AddressListId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAddressListImportJob"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (JobId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetArchive", "ListArchiveExports", "ListArchiveSearches"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ArchiveId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetArchiveExport"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ExportId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetArchiveMessage",
            "GetArchiveMessageContent",
            "GetArchiveSearchResults",
            "ListMembersOfAddressList",
        ],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &["GetArchiveSearch"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (SearchId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetIngressPoint", "GetRelay"],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &["GetMemberOfAddressList"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (AddressListId, Address); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetRuleSet"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (RuleSetId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTrafficPolicy"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (TrafficPolicyId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
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
    // Lists all Add On instances in your account.
    Resource {
        operation: "ListAddonInstances",
        items: "AddonInstances",
        id: Id::Arn("AddonInstanceArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &[
            "AddonInstanceArn",
            "AddonInstanceId",
            "AddonSubscriptionId",
            "AddonName",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists all Add On subscriptions in your account.
    Resource {
        operation: "ListAddonSubscriptions",
        items: "AddonSubscriptions",
        id: Id::Arn("AddonSubscriptionArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["AddonSubscriptionArn", "AddonSubscriptionId", "AddonName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists address lists for this account.
    Resource {
        operation: "ListAddressLists",
        items: "AddressLists",
        id: Id::Arn("AddressListArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["AddressListArn", "AddressListId", "AddressListName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Returns a list of all email archives in your account.
    Resource {
        operation: "ListArchives",
        items: "Archives",
        id: Id::Key(&["ArchiveId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["ArchiveId", "ArchiveName", "ArchiveState"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List all ingress endpoint resources.
    Resource {
        operation: "ListIngressPoints",
        items: "IngressPoints",
        id: Id::Key(&["IngressPointId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["IngressPointId", "IngressPointName", "Status", "Type"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Lists all the existing relay resources.
    Resource {
        operation: "ListRelays",
        items: "Relays",
        id: Id::Key(&["RelayId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["RelayId", "RelayName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List rule sets for this account.
    Resource {
        operation: "ListRuleSets",
        items: "RuleSets",
        id: Id::Key(&["RuleSetId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["RuleSetId", "RuleSetName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List traffic policy resources.
    Resource {
        operation: "ListTrafficPolicies",
        items: "TrafficPolicies",
        id: Id::Key(&["TrafficPolicyId"]),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["TrafficPolicyId", "TrafficPolicyName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("mailmanager", GROUPS, RESOURCES)
}
