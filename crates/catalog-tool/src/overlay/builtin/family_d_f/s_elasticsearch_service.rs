use super::support::{inventory, key, queried, reviewed, token};
use crate::overlay::{FixedValue, NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut domains = queried(
        inventory(
            "ListDomainNames",
            "DomainNames",
            key(&["DomainName"])?,
            Paginator::None,
        )?,
        &["DomainName", "EngineType"],
    )?;
    domains.1.fixed.insert(
        super::super::path("EngineType")?,
        FixedValue::String("Elasticsearch".to_owned()),
    );
    reviewed(
        "elasticsearch-service",
        vec![
            domains,
            queried(
                inventory(
                    "ListVpcEndpoints",
                    "VpcEndpointSummaryList",
                    key(&["VpcEndpointId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["VpcEndpointId", "Status"],
            )?,
            queried(
                inventory(
                    "DescribeInboundCrossClusterSearchConnections",
                    "CrossClusterSearchConnections",
                    key(&["CrossClusterSearchConnectionId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["CrossClusterSearchConnectionId"],
            )?,
            queried(
                inventory(
                    "DescribeOutboundCrossClusterSearchConnections",
                    "CrossClusterSearchConnections",
                    key(&["CrossClusterSearchConnectionId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["CrossClusterSearchConnectionId"],
            )?,
            queried(
                inventory(
                    "DescribePackages",
                    "PackageDetailsList",
                    key(&["PackageID"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &["PackageID", "PackageName", "PackageType", "PackageStatus"],
            )?,
            queried(
                inventory(
                    "DescribeReservedElasticsearchInstances",
                    "ReservedElasticsearchInstances",
                    key(&["ReservedElasticsearchInstanceId"])?,
                    token("NextToken", "NextToken")?,
                )?,
                &[
                    "ReservedElasticsearchInstanceId",
                    "ReservationName",
                    "State",
                ],
            )?,
        ],
        &[
            super::s_elasticsearch_service_decisions_0::ENTRIES,
            super::s_elasticsearch_service_decisions_1::ENTRIES,
        ],
        &[],
    )
}
