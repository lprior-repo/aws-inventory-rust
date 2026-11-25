//! Pinned model: api-models-aws 46135f8468cd5cb60d983b97290c9f6661e359c6, models/global-accelerator/service.
//! Review is source-only; no live service or emulator support claim.
use super::policy;
use crate::overlay::{
    MemberPath, NameError, OperationName, Paginator, RequestRouting, Scope, ServiceOverlay,
    TagEndpoint, TagEndpointParent, TagEndpointRequest, TagEndpointSelector,
};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut approved = Vec::new();
    approved.extend(accelerator_rows()?);
    approved.extend(detail_rows()?);
    approved.extend(tag_byoip_rows()?);
    set_routing(&mut approved);
    policy::finish(
        "global-accelerator",
        approved,
        DEFERRED,
        "Account-scoped standard/custom accelerators, listeners, endpoint groups and cross-account attachments are metadata. Requests and signing use the service's us-west-2 control plane without changing global resource identity. Paginated child listeners and endpoint groups bind only the corresponding routing family's parent.",
        &[(policy::DenyReason::Mutating, MUTATING)],
    )
}

fn accelerator_rows() -> Result<Vec<(&'static str, crate::overlay::OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListAccelerators",
            policy::query(
                policy::scope(
                    policy::enumerate(
                        "Accelerators",
                        policy::arn("AcceleratorArn")?,
                        Some(("NextToken", "NextToken")),
                    )?,
                    crate::overlay::Scope::Account,
                ),
                &["AcceleratorArn"],
            )?,
        ),
        (
            "ListCrossAccountAttachments",
            policy::query(
                policy::scope(
                    policy::enumerate(
                        "CrossAccountAttachments",
                        policy::arn("AttachmentArn")?,
                        Some(("NextToken", "NextToken")),
                    )?,
                    crate::overlay::Scope::Account,
                ),
                &["AttachmentArn"],
            )?,
        ),
        (
            "ListCustomRoutingAccelerators",
            policy::query(
                policy::scope(
                    policy::enumerate(
                        "Accelerators",
                        policy::arn("AcceleratorArn")?,
                        Some(("NextToken", "NextToken")),
                    )?,
                    crate::overlay::Scope::Account,
                ),
                &["AcceleratorArn"],
            )?,
        ),
    ])
}

fn detail_rows() -> Result<Vec<(&'static str, crate::overlay::OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DescribeAccelerator",
            policy::query(
                policy::scope(
                    policy::bind(
                        policy::enumerate("Accelerator", policy::arn("AcceleratorArn")?, None)?,
                        "AcceleratorArn",
                        "ListAccelerators",
                        "AcceleratorArn",
                    )?,
                    crate::overlay::Scope::Account,
                ),
                &["AcceleratorArn"],
            )?,
        ),
        (
            "ListListeners",
            child(
                "Listeners",
                "ListenerArn",
                "AcceleratorArn",
                "ListAccelerators",
            )?,
        ),
        (
            "ListEndpointGroups",
            child(
                "EndpointGroups",
                "EndpointGroupArn",
                "ListenerArn",
                "ListListeners",
            )?,
        ),
        (
            "ListCustomRoutingListeners",
            child(
                "Listeners",
                "ListenerArn",
                "AcceleratorArn",
                "ListCustomRoutingAccelerators",
            )?,
        ),
        (
            "ListCustomRoutingEndpointGroups",
            child(
                "EndpointGroups",
                "EndpointGroupArn",
                "ListenerArn",
                "ListCustomRoutingListeners",
            )?,
        ),
    ])
}

fn tag_byoip_rows() -> Result<Vec<(&'static str, crate::overlay::OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListTagsForResource",
            TagEndpoint {
                parents: ["ListAccelerators", "ListCustomRoutingAccelerators"]
                    .into_iter()
                    .map(|operation| {
                        Ok(TagEndpointParent {
                            operation: OperationName::new(operation)?,
                            request: TagEndpointRequest::Member {
                                input: MemberPath::new("ResourceArn")?,
                                parent: MemberPath::new("AcceleratorArn")?,
                            },
                        })
                    })
                    .collect::<Result<_, NameError>>()?,
                selector: TagEndpointSelector::List {
                    path: MemberPath::new("Tags")?,
                    key: MemberPath::new("Key")?,
                    value: MemberPath::new("Value")?,
                },
            }
            .operation(Scope::Account, Paginator::None)?,
        ),
        (
            "ListByoipCidrs",
            policy::query(
                policy::scope(
                    policy::enumerate(
                        "ByoipCidrs",
                        policy::key("Cidr")?,
                        Some(("NextToken", "NextToken")),
                    )?,
                    crate::overlay::Scope::Account,
                ),
                &["Cidr"],
            )?,
        ),
    ])
}

fn set_routing(approved: &mut Vec<(&'static str, crate::overlay::OperationOverlay)>) {
    for (_, operation) in approved.iter_mut() {
        operation.routing = RequestRouting::FixedRegion {
            region: "us-west-2".to_owned(),
        };
    }
}

fn child(
    items: &str,
    identity: &str,
    input: &str,
    parent: &str,
) -> Result<crate::overlay::OperationOverlay, NameError> {
    policy::query(
        policy::scope(
            policy::bind(
                policy::enumerate(
                    items,
                    policy::arn(identity)?,
                    Some(("NextToken", "NextToken")),
                )?,
                input,
                parent,
                input,
            )?,
            Scope::Account,
        ),
        &[identity],
    )
}

// Explicit pinned operation dispositions. No runtime operation-name classifier approves calls.
// Mutation sets change resource/lifecycle/authorization state; content and credential reads stay denied.
const DEFERRED: &[&str] = &[
    "DescribeAcceleratorAttributes",
    "DescribeCrossAccountAttachment",
    "DescribeCustomRoutingAccelerator",
    "DescribeCustomRoutingAcceleratorAttributes",
    "DescribeCustomRoutingEndpointGroup",
    "DescribeCustomRoutingListener",
    "DescribeEndpointGroup",
    "DescribeListener",
    "ListCrossAccountResourceAccounts",
    "ListCrossAccountResources",
    "ListCustomRoutingPortMappings",
    "ListCustomRoutingPortMappingsByDestination",
];

const MUTATING: &[&str] = &[
    "AddCustomRoutingEndpoints",
    "AddEndpoints",
    "AdvertiseByoipCidr",
    "AllowCustomRoutingTraffic",
    "CreateAccelerator",
    "CreateCrossAccountAttachment",
    "CreateCustomRoutingAccelerator",
    "CreateCustomRoutingEndpointGroup",
    "CreateCustomRoutingListener",
    "CreateEndpointGroup",
    "CreateListener",
    "DeleteAccelerator",
    "DeleteCrossAccountAttachment",
    "DeleteCustomRoutingAccelerator",
    "DeleteCustomRoutingEndpointGroup",
    "DeleteCustomRoutingListener",
    "DeleteEndpointGroup",
    "DeleteListener",
    "DenyCustomRoutingTraffic",
    "DeprovisionByoipCidr",
    "ProvisionByoipCidr",
    "RemoveCustomRoutingEndpoints",
    "RemoveEndpoints",
    "TagResource",
    "UntagResource",
    "UpdateAccelerator",
    "UpdateAcceleratorAttributes",
    "UpdateCrossAccountAttachment",
    "UpdateCustomRoutingAccelerator",
    "UpdateCustomRoutingAcceleratorAttributes",
    "UpdateCustomRoutingListener",
    "UpdateEndpointGroup",
    "UpdateListener",
    "WithdrawByoipCidr",
];
