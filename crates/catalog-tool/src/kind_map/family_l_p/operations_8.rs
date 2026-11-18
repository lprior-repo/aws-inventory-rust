use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "observabilityadmin",
        operation: "ListCentralizationRulesForOrganization",
        kind: "AWS::ObservabilityAdmin::OrganizationCentralizationRule",
    },
    OperationMapping {
        service: "observabilityadmin",
        operation: "ListTelemetryRules",
        kind: "AWS::ObservabilityAdmin::TelemetryRule",
    },
    OperationMapping {
        service: "observabilityadmin",
        operation: "ListTelemetryRulesForOrganization",
        kind: "AWS::ObservabilityAdmin::OrganizationTelemetryRule",
    },
    OperationMapping {
        service: "observabilityadmin",
        operation: "ListTelemetryPipelines",
        kind: "AWS::ObservabilityAdmin::TelemetryPipelines",
    },
    OperationMapping {
        service: "observabilityadmin",
        operation: "ListS3TableIntegrations",
        kind: "AWS::ObservabilityAdmin::S3TableIntegration",
    },
    OperationMapping {
        service: "observabilityadmin",
        operation: "ListDatasetIntegrations",
        kind: "AWS::ObservabilityAdmin::DatasetIntegration",
    },
    OperationMapping {
        service: "odb",
        operation: "ListCloudAutonomousVmClusters",
        kind: "AWS::ODB::CloudAutonomousVmCluster",
    },
    OperationMapping {
        service: "odb",
        operation: "ListCloudExadataInfrastructures",
        kind: "AWS::ODB::CloudExadataInfrastructure",
    },
    OperationMapping {
        service: "odb",
        operation: "ListCloudVmClusters",
        kind: "AWS::ODB::CloudVmCluster",
    },
    OperationMapping {
        service: "odb",
        operation: "ListOdbNetworks",
        kind: "AWS::ODB::OdbNetwork",
    },
    OperationMapping {
        service: "odb",
        operation: "ListOdbPeeringConnections",
        kind: "AWS::ODB::OdbPeeringConnection",
    },
    OperationMapping {
        service: "odb",
        operation: "ListAutonomousDatabases",
        kind: "AWS::ApiOnly::OracleAutonomousDatabase",
    },
    OperationMapping {
        service: "odb",
        operation: "ListExadbVmClusters",
        kind: "AWS::ApiOnly::OracleExadbVmCluster",
    },
    OperationMapping {
        service: "odb",
        operation: "ListExascaleDbStorageVaults",
        kind: "AWS::ApiOnly::OracleExascaleStorageVault",
    },
    OperationMapping {
        service: "omics",
        operation: "ListAnnotationStores",
        kind: "AWS::Omics::AnnotationStore",
    },
    OperationMapping {
        service: "omics",
        operation: "ListReferenceStores",
        kind: "AWS::Omics::ReferenceStore",
    },
    OperationMapping {
        service: "omics",
        operation: "ListSequenceStores",
        kind: "AWS::Omics::SequenceStore",
    },
    OperationMapping {
        service: "omics",
        operation: "ListVariantStores",
        kind: "AWS::Omics::VariantStore",
    },
    OperationMapping {
        service: "omics",
        operation: "ListWorkflows",
        kind: "AWS::Omics::Workflow",
    },
    OperationMapping {
        service: "omics",
        operation: "ListRunGroups",
        kind: "AWS::Omics::RunGroup",
    },
];
