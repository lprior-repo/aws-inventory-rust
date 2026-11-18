// Pinned Smithy source: .local/aws-models/models/network-firewall/service/2020-11-12/network-firewall-2020-11-12.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptNetworkFirewallTransitGatewayAttachment",
            "AssociateAvailabilityZones",
            "AssociateFirewallPolicy",
            "AssociateSubnets",
            "CreateContainerAssociation",
            "CreateFirewall",
            "CreateFirewallPolicy",
            "CreateFirewallPolicyV2",
            "CreateRuleGroup",
            "CreateTLSInspectionConfiguration",
            "CreateVpcEndpointAssociation",
            "DeleteContainerAssociation",
            "DeleteFirewall",
            "DeleteFirewallPolicy",
            "DeleteFirewallPolicyV2",
            "DeleteNetworkFirewallTransitGatewayAttachment",
            "DeleteProxy",
            "DeleteResourcePolicy",
            "DeleteRuleGroup",
            "DeleteTLSInspectionConfiguration",
            "DeleteVpcEndpointAssociation",
            "DisassociateAvailabilityZones",
            "DisassociateSubnets",
            "PutResourcePolicy",
            "RejectNetworkFirewallTransitGatewayAttachment",
            "StartAnalysisReport",
            "StartFlowCapture",
            "StartFlowFlush",
            "TagResource",
            "UntagResource",
            "UpdateAvailabilityZoneChangeProtection",
            "UpdateContainerAssociation",
            "UpdateFirewallAnalysisSettings",
            "UpdateFirewallDeleteProtection",
            "UpdateFirewallDescription",
            "UpdateFirewallEncryptionConfiguration",
            "UpdateFirewallPolicy",
            "UpdateFirewallPolicyChangeProtection",
            "UpdateFirewallPolicyV2",
            "UpdateLoggingConfiguration",
            "UpdateProxySettings",
            "UpdateRuleGroup",
            "UpdateSubnetChangeProtection",
            "UpdateTLSInspectionConfiguration",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "DescribeContainerAssociation",
            "DescribeFirewall",
            "DescribeFirewallMetadata",
            "DescribeFirewallPolicy",
            "DescribeFirewallPolicyV2",
            "DescribeLoggingConfiguration",
            "DescribeProxy",
            "DescribeRuleGroup",
            "DescribeRuleGroupMetadata",
            "DescribeRuleGroupSummary",
            "DescribeTLSInspectionConfiguration",
            "ListAnalysisReports",
            "ListContainerAssociations",
            "ListFirewallPoliciesV2",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &["DescribeFlowOperation", "ListFlowOperationResults"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (FirewallArn, FlowOperationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeResourcePolicy", "ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (ResourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["DescribeVpcEndpointAssociation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (VpcEndpointAssociationArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetAnalysisReportResults"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (AnalysisReportId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListFlowOperations"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (FirewallArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // Retrieves the metadata for the firewalls that you have defined. If you provide VPC identifiers in your request, this returns only the firewalls for those VPCs. Depending on your se
    Resource {
        operation: "ListFirewalls",
        items: "Firewalls",
        id: Id::Arn("FirewallArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["FirewallArn", "FirewallName", "TransitGatewayAttachmentId"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves the metadata for the firewall policies that you have defined. Depending on your setting for max results and the number of firewall policies, a single call might not retur
    Resource {
        operation: "ListFirewallPolicies",
        items: "FirewallPolicies",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves the metadata for the rule groups that you have defined. Depending on your setting for max results and the number of rule groups, a single call might not return the full l
    Resource {
        operation: "ListRuleGroups",
        items: "RuleGroups",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name", "VendorName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves the metadata for the TLS inspection configurations that you have defined. Depending on your setting for max results and the number of TLS inspection configurations, a sin
    Resource {
        operation: "ListTLSInspectionConfigurations",
        items: "TLSInspectionConfigurations",
        id: Id::Arn("Arn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["Arn", "Name"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Retrieves the metadata for the VPC endpoint associations that you have defined. If you specify a fireawll, this returns only the endpoint associations for that firewall. Depending
    Resource {
        operation: "ListVpcEndpointAssociations",
        items: "VpcEndpointAssociations",
        id: Id::Arn("VpcEndpointAssociationArn"),
        scope: Scope::Region,
        tokens: Some(("NextToken", "NextToken")),
        query: &["VpcEndpointAssociationArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("network-firewall", GROUPS, RESOURCES)
}
