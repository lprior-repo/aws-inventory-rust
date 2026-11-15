// Pinned model: .local/aws-models/models/elasticsearch-service/service/2015-01-01/elasticsearch-service-2015-01-01.json
use super::support::{inventory, key, queried, reviewed, token};
use crate::overlay::{FixedValue, NameError, Paginator, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let mut domains = queried(inventory("ListDomainNames", "DomainNames", key(&["DomainName"])?, Paginator::None)?, &["DomainName", "EngineType"])?;
    // The legacy and current APIs enumerate the same physical domains unless filtered.
    domains.1.fixed.insert(super::super::path("EngineType")?, FixedValue::String("Elasticsearch".to_owned()));
    reviewed(
        "elasticsearch-service",
        vec![
        // Returns the name of all Elasticsearch domains owned by the current user's account.
        domains,
        // Retrieves all Amazon OpenSearch Service-managed VPC endpoints in the current account and Region.
        queried(inventory("ListVpcEndpoints", "VpcEndpointSummaryList", key(&["VpcEndpointId"])?, token("NextToken", "NextToken")?)?, &["VpcEndpointId", "Status"])?,
        // Lists all the inbound cross-cluster search connections for a destination domain.
        queried(inventory("DescribeInboundCrossClusterSearchConnections", "CrossClusterSearchConnections", key(&["CrossClusterSearchConnectionId"])?, token("NextToken", "NextToken")?)?, &["CrossClusterSearchConnectionId"])?,
        // Lists all the outbound cross-cluster search connections for a source domain.
        queried(inventory("DescribeOutboundCrossClusterSearchConnections", "CrossClusterSearchConnections", key(&["CrossClusterSearchConnectionId"])?, token("NextToken", "NextToken")?)?, &["CrossClusterSearchConnectionId"])?,
        // Describes all packages available to Amazon ES.
        queried(inventory("DescribePackages", "PackageDetailsList", key(&["PackageID"])?, token("NextToken", "NextToken")?)?, &["PackageID", "PackageName", "PackageType", "PackageStatus"])?,
        // Describes already-owned reservations, never an offering purchase or a search query.
        queried(inventory("DescribeReservedElasticsearchInstances", "ReservedElasticsearchInstances", key(&["ReservedElasticsearchInstanceId"])?, token("NextToken", "NextToken")?)?, &["ReservedElasticsearchInstanceId", "ReservationName", "State"])?,
        ],
        &[super::s_elasticsearch_service_decisions_0::ENTRIES, super::s_elasticsearch_service_decisions_1::ENTRIES],
        &[

        ],
    )
}
