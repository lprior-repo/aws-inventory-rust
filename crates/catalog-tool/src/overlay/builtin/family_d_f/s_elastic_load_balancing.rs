use super::support::{inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "elastic-load-balancing",
        vec![queried(
            inventory(
                "DescribeLoadBalancers",
                "LoadBalancerDescriptions",
                key(&["LoadBalancerName"])?,
                token("Marker", "NextMarker")?,
            )?,
            &[
                "LoadBalancerName",
                "DNSName",
                "CanonicalHostedZoneName",
                "CanonicalHostedZoneNameID",
                "VPCId",
            ],
        )?],
        &[super::s_elastic_load_balancing_decisions_0::ENTRIES],
        &[],
    )
}
