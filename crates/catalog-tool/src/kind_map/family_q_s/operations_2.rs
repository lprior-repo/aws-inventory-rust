use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "redshift",
        operation: "DescribeTableRestoreStatus",
        kind: "AWS::ApiOnly::RedshiftTableRestore",
    },
    OperationMapping {
        service: "redshift",
        operation: "DescribeClusterDbRevisions",
        kind: "AWS::Redshift::Cluster",
    },
    OperationMapping {
        service: "redshift",
        operation: "ListRecommendations",
        kind: "AWS::ApiOnly::RedshiftRecommendation",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListNamespaces",
        kind: "AWS::RedshiftServerless::Namespace",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListWorkgroups",
        kind: "AWS::RedshiftServerless::Workgroup",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListSnapshots",
        kind: "AWS::RedshiftServerless::Snapshot",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListEndpointAccess",
        kind: "AWS::ApiOnly::RedshiftServerlessEndpointAccess",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListScheduledActions",
        kind: "AWS::ApiOnly::RedshiftServerlessScheduledAction",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListUsageLimits",
        kind: "AWS::ApiOnly::RedshiftServerlessUsageLimit",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListReservations",
        kind: "AWS::ApiOnly::RedshiftServerlessReservation",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListCustomDomainAssociations",
        kind: "AWS::ApiOnly::RedshiftServerlessCustomDomainAssociation",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListManagedWorkgroups",
        kind: "AWS::ApiOnly::RedshiftServerlessManagedWorkgroup",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListRecoveryPoints",
        kind: "AWS::ApiOnly::RedshiftServerlessRecoveryPoint",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListSnapshotCopyConfigurations",
        kind: "AWS::ApiOnly::RedshiftServerlessSnapshotCopyConfiguration",
    },
    OperationMapping {
        service: "redshift-serverless",
        operation: "ListTableRestoreStatus",
        kind: "AWS::ApiOnly::RedshiftServerlessTableRestore",
    },
    OperationMapping {
        service: "rekognition",
        operation: "ListCollections",
        kind: "AWS::Rekognition::Collection",
    },
    OperationMapping {
        service: "rekognition",
        operation: "DescribeProjects",
        kind: "AWS::Rekognition::Project",
    },
    OperationMapping {
        service: "rekognition",
        operation: "ListStreamProcessors",
        kind: "AWS::Rekognition::StreamProcessor",
    },
    OperationMapping {
        service: "rekognition",
        operation: "ListMediaAnalysisJobs",
        kind: "AWS::ApiOnly::RekognitionMediaAnalysisJob",
    },
    OperationMapping {
        service: "repostspace",
        operation: "ListSpaces",
        kind: "AWS::ApiOnly::RePostSpace",
    },
];
