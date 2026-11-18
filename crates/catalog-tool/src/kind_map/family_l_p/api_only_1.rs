use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::MigrationHomeRegionControl",
        reason: "MigrationHomeRegionControl is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::MigrationOrchestratorWorkflow",
        reason: "MigrationOrchestratorWorkflow is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::MigrationProgressUpdateStream",
        reason: "MigrationProgressUpdateStream is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::MigrationSourceServer",
        reason: "MigrationSourceServer is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::MigrationStrategyCollector",
        reason: "MigrationStrategyCollector is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::MigrationVcenterClient",
        reason: "MigrationVcenterClient is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::MwaaServerlessWorkflow",
        reason: "MwaaServerlessWorkflow is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::NeptuneClusterEndpoint",
        reason: "NeptuneClusterEndpoint is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::NeptuneClusterSnapshot",
        reason: "NeptuneClusterSnapshot is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::NetworkManagerAttachment",
        reason: "Native attachment summary is polymorphic across VPC, Connect, VPN, transit-gateway-route-table and Direct Connect attachments; one operation cannot map all results to one concrete CFN attachment type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::NetworkManagerConnection",
        reason: "NetworkManagerConnection is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::NetworkManagerPeering",
        reason: "Native peering summaries require subtype dispatch; preserve generic peering identity rather than mislabel every subtype AWS::NetworkManager::TransitGatewayPeering.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OpenSearchDirectQueryDataSource",
        reason: "OpenSearchDirectQueryDataSource is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OpenSearchDomainVpcEndpoint",
        reason: "OpenSearchDomainVpcEndpoint is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OracleAutonomousDatabase",
        reason: "OracleAutonomousDatabase is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OracleExadbVmCluster",
        reason: "OracleExadbVmCluster is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::OracleExascaleStorageVault",
        reason: "OracleExascaleStorageVault is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::PartnerEngagement",
        reason: "PartnerEngagement is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::PartnerMarketplaceRevenueShare",
        reason: "PartnerMarketplaceRevenueShare is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::PartnerRevenueAttribution",
        reason: "PartnerRevenueAttribution is a native metadata resource absent from the pinned 1881 CloudFormation schemas; preserve its own stable identity rather than aliasing an unrelated CFN resource.",
    },
];
