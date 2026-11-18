use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "omics",
        operation: "ListRunCaches",
        kind: "AWS::Omics::RunCache",
    },
    OperationMapping {
        service: "opensearch",
        operation: "ListDomainNames",
        kind: "AWS::OpenSearchService::Domain",
    },
    OperationMapping {
        service: "opensearch",
        operation: "ListApplications",
        kind: "AWS::OpenSearchService::Application",
    },
    OperationMapping {
        service: "opensearch",
        operation: "ListVpcEndpoints",
        kind: "AWS::ApiOnly::OpenSearchDomainVpcEndpoint",
    },
    OperationMapping {
        service: "opensearch",
        operation: "ListDirectQueryDataSources",
        kind: "AWS::ApiOnly::OpenSearchDirectQueryDataSource",
    },
    OperationMapping {
        service: "opensearchserverless",
        operation: "ListCollections",
        kind: "AWS::OpenSearchServerless::Collection",
    },
    OperationMapping {
        service: "opensearchserverless",
        operation: "ListCollectionGroups",
        kind: "AWS::OpenSearchServerless::CollectionGroup",
    },
    OperationMapping {
        service: "opensearchserverless",
        operation: "ListVpcEndpoints",
        kind: "AWS::OpenSearchServerless::VpcEndpoint",
    },
    OperationMapping {
        service: "osis",
        operation: "ListPipelines",
        kind: "AWS::OSIS::Pipeline",
    },
    OperationMapping {
        service: "osis",
        operation: "ListPipelineEndpoints",
        kind: "AWS::OSIS::PipelineEndpoint",
    },
    OperationMapping {
        service: "outposts",
        operation: "ListOutposts",
        kind: "AWS::Outposts::Outpost",
    },
    OperationMapping {
        service: "outposts",
        operation: "ListSites",
        kind: "AWS::Outposts::Site",
    },
    OperationMapping {
        service: "payment-cryptography",
        operation: "ListKeys",
        kind: "AWS::PaymentCryptography::Key",
    },
    OperationMapping {
        service: "payment-cryptography",
        operation: "ListAliases",
        kind: "AWS::PaymentCryptography::Alias",
    },
    OperationMapping {
        service: "pca-connector-ad",
        operation: "ListConnectors",
        kind: "AWS::PCAConnectorAD::Connector",
    },
    OperationMapping {
        service: "pca-connector-ad",
        operation: "ListDirectoryRegistrations",
        kind: "AWS::PCAConnectorAD::DirectoryRegistration",
    },
    OperationMapping {
        service: "pca-connector-scep",
        operation: "ListConnectors",
        kind: "AWS::PCAConnectorSCEP::Connector",
    },
    OperationMapping {
        service: "pca-connector-scep",
        operation: "ListChallengeMetadata",
        kind: "AWS::PCAConnectorSCEP::Challenge",
    },
    OperationMapping {
        service: "pcs",
        operation: "ListClusters",
        kind: "AWS::PCS::Cluster",
    },
    OperationMapping {
        service: "pcs",
        operation: "ListComputeNodeGroups",
        kind: "AWS::PCS::ComputeNodeGroup",
    },
];
