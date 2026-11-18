use super::super::service;
use super::{defer, deny, vpc_lattice_plans_1, vpc_lattice_plans_2, vpc_lattice_plans_3};
use crate::{
    classes::DenyReason,
    overlay::{NameError, ServiceOverlay},
};

// Reviewed against pinned Smithy documentation, not operation-name read approval.
pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    service(
        "vpc-lattice",
        vec![
            ("BatchUpdateRule", deny(DenyReason::Mutating)),
            ("CreateAccessLogSubscription", deny(DenyReason::Mutating)),
            ("CreateListener", deny(DenyReason::Mutating)),
            ("CreateResourceConfiguration", deny(DenyReason::Mutating)),
            ("CreateResourceGateway", deny(DenyReason::Mutating)),
            ("CreateRule", deny(DenyReason::Mutating)),
            ("CreateService", deny(DenyReason::Mutating)),
            ("CreateServiceNetwork", deny(DenyReason::Mutating)),
            (
                "CreateServiceNetworkResourceAssociation",
                deny(DenyReason::Mutating),
            ),
            (
                "CreateServiceNetworkServiceAssociation",
                deny(DenyReason::Mutating),
            ),
            (
                "CreateServiceNetworkVpcAssociation",
                deny(DenyReason::Mutating),
            ),
            ("CreateTargetGroup", deny(DenyReason::Mutating)),
            ("DeleteAccessLogSubscription", deny(DenyReason::Mutating)),
            ("DeleteAuthPolicy", deny(DenyReason::Mutating)),
            ("DeleteDomainVerification", deny(DenyReason::Mutating)),
            ("DeleteListener", deny(DenyReason::Mutating)),
            ("DeleteResourceConfiguration", deny(DenyReason::Mutating)),
            (
                "DeleteResourceEndpointAssociation",
                deny(DenyReason::Mutating),
            ),
            ("DeleteResourceGateway", deny(DenyReason::Mutating)),
            ("DeleteResourcePolicy", deny(DenyReason::Mutating)),
            ("DeleteRule", deny(DenyReason::Mutating)),
            ("DeleteService", deny(DenyReason::Mutating)),
            ("DeleteServiceNetwork", deny(DenyReason::Mutating)),
            (
                "DeleteServiceNetworkResourceAssociation",
                deny(DenyReason::Mutating),
            ),
            (
                "DeleteServiceNetworkServiceAssociation",
                deny(DenyReason::Mutating),
            ),
            (
                "DeleteServiceNetworkVpcAssociation",
                deny(DenyReason::Mutating),
            ),
            ("DeleteTargetGroup", deny(DenyReason::Mutating)),
            ("DeregisterTargets", deny(DenyReason::Mutating)),
            (
                "GetAccessLogSubscription",
                defer(
                    "Missing executable inventory binding for accessLogSubscriptionIdentifier: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetAuthPolicy",
                defer(
                    "Missing executable inventory binding for resourceIdentifier: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetDomainVerification",
                vpc_lattice_plans_1::get_domain_verification()?,
            ),
            (
                "GetListener",
                defer(
                    "Missing executable inventory binding for serviceIdentifier, listenerIdentifier: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetResourceConfiguration",
                vpc_lattice_plans_1::get_resource_configuration()?,
            ),
            (
                "GetResourceGateway",
                vpc_lattice_plans_1::get_resource_gateway()?,
            ),
            (
                "GetResourcePolicy",
                defer(
                    "Missing executable inventory binding for resourceArn: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "GetRule",
                defer(
                    "Missing executable inventory binding for serviceIdentifier, listenerIdentifier, ruleIdentifier: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("GetService", vpc_lattice_plans_1::get_service()?),
            (
                "GetServiceNetwork",
                vpc_lattice_plans_1::get_service_network()?,
            ),
            (
                "GetServiceNetworkResourceAssociation",
                vpc_lattice_plans_1::get_service_network_resource_association()?,
            ),
            (
                "GetServiceNetworkServiceAssociation",
                vpc_lattice_plans_1::get_service_network_service_association()?,
            ),
            (
                "GetServiceNetworkVpcAssociation",
                vpc_lattice_plans_1::get_service_network_vpc_association()?,
            ),
            ("GetTargetGroup", vpc_lattice_plans_1::get_target_group()?),
            (
                "ListAccessLogSubscriptions",
                defer(
                    "Missing executable inventory binding for resourceIdentifier: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListDomainVerifications",
                vpc_lattice_plans_2::list_domain_verifications()?,
            ),
            ("ListListeners", vpc_lattice_plans_2::list_listeners()?),
            (
                "ListResourceConfigurations",
                vpc_lattice_plans_2::list_resource_configurations()?,
            ),
            (
                "ListResourceEndpointAssociations",
                vpc_lattice_plans_2::list_resource_endpoint_associations()?,
            ),
            (
                "ListResourceGateways",
                vpc_lattice_plans_2::list_resource_gateways()?,
            ),
            (
                "ListRules",
                defer(
                    "Missing executable inventory binding for serviceIdentifier, listenerIdentifier: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            (
                "ListServiceNetworkResourceAssociations",
                vpc_lattice_plans_2::list_service_network_resource_associations()?,
            ),
            (
                "ListServiceNetworks",
                vpc_lattice_plans_2::list_service_networks()?,
            ),
            (
                "ListServiceNetworkServiceAssociations",
                vpc_lattice_plans_2::list_service_network_service_associations()?,
            ),
            (
                "ListServiceNetworkVpcAssociations",
                vpc_lattice_plans_2::list_service_network_vpc_associations()?,
            ),
            (
                "ListServiceNetworkVpcEndpointAssociations",
                vpc_lattice_plans_2::list_service_network_vpc_endpoint_associations()?,
            ),
            ("ListServices", vpc_lattice_plans_2::list_services()?),
            (
                "ListTagsForResource",
                defer(
                    "Tag-only response omits the resource identity; the runtime does not yet merge bound-parent identity into complete tag-only observations.",
                ),
            ),
            (
                "ListTargetGroups",
                vpc_lattice_plans_3::list_target_groups()?,
            ),
            (
                "ListTargets",
                defer(
                    "Missing executable inventory binding for targetGroupIdentifier: required parent identifiers or request predicates are not present together in a reviewed enumeration item.",
                ),
            ),
            ("PutAuthPolicy", deny(DenyReason::Mutating)),
            ("PutResourcePolicy", deny(DenyReason::Mutating)),
            ("RegisterTargets", deny(DenyReason::Mutating)),
            ("StartDomainVerification", deny(DenyReason::Mutating)),
            ("TagResource", deny(DenyReason::Mutating)),
            ("UntagResource", deny(DenyReason::Mutating)),
            ("UpdateAccessLogSubscription", deny(DenyReason::Mutating)),
            ("UpdateListener", deny(DenyReason::Mutating)),
            ("UpdateResourceConfiguration", deny(DenyReason::Mutating)),
            ("UpdateResourceGateway", deny(DenyReason::Mutating)),
            ("UpdateRule", deny(DenyReason::Mutating)),
            ("UpdateService", deny(DenyReason::Mutating)),
            ("UpdateServiceNetwork", deny(DenyReason::Mutating)),
            (
                "UpdateServiceNetworkVpcAssociation",
                deny(DenyReason::Mutating),
            ),
            ("UpdateTargetGroup", deny(DenyReason::Mutating)),
        ],
    )
}
