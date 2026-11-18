// Pinned model: .local/aws-models/models/elastic-load-balancing/service/2012-06-01/elastic-load-balancing-2012-06-01.json
use super::support::{inventory, key, queried, reviewed, token};
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    reviewed(
        "elastic-load-balancing",
        vec![
            // Describes the specified the load balancers.
            queried(
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
            )?,
        ],
        &[super::s_elastic_load_balancing_decisions_0::ENTRIES],
        &[],
    )
}
