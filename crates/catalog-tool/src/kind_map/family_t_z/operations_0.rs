use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "textract",
        operation: "ListAdapters",
        kind: "AWS::Textract::Adapter",
    },
    OperationMapping {
        service: "textract",
        operation: "ListAdapterVersions",
        kind: "AWS::ApiOnly::TextractAdapterVersion",
    },
    OperationMapping {
        service: "timestream-influxdb",
        operation: "ListDbBackups",
        kind: "AWS::Timestream::InfluxDBBackup",
    },
    OperationMapping {
        service: "timestream-influxdb",
        operation: "ListDbClusters",
        kind: "AWS::Timestream::InfluxDBCluster",
    },
    OperationMapping {
        service: "timestream-influxdb",
        operation: "ListDbInstances",
        kind: "AWS::Timestream::InfluxDBInstance",
    },
    OperationMapping {
        service: "timestream-influxdb",
        operation: "ListDbParameterGroups",
        kind: "AWS::ApiOnly::TimestreamInfluxDBParameterGroup",
    },
    OperationMapping {
        service: "transcribe",
        operation: "ListVocabularies",
        kind: "AWS::Transcribe::Vocabulary",
    },
    OperationMapping {
        service: "transcribe",
        operation: "ListMedicalVocabularies",
        kind: "AWS::Transcribe::MedicalVocabulary",
    },
    OperationMapping {
        service: "transcribe",
        operation: "ListVocabularyFilters",
        kind: "AWS::Transcribe::VocabularyFilter",
    },
    OperationMapping {
        service: "transfer",
        operation: "DescribeAgreement",
        kind: "AWS::Transfer::Agreement",
    },
    OperationMapping {
        service: "transfer",
        operation: "ListCertificates",
        kind: "AWS::Transfer::Certificate",
    },
    OperationMapping {
        service: "transfer",
        operation: "ListConnectors",
        kind: "AWS::Transfer::Connector",
    },
    OperationMapping {
        service: "transfer",
        operation: "ListProfiles",
        kind: "AWS::Transfer::Profile",
    },
    OperationMapping {
        service: "transfer",
        operation: "ListServers",
        kind: "AWS::Transfer::Server",
    },
    OperationMapping {
        service: "transfer",
        operation: "ListWebApps",
        kind: "AWS::Transfer::WebApp",
    },
    OperationMapping {
        service: "transfer",
        operation: "ListWorkflows",
        kind: "AWS::Transfer::Workflow",
    },
    OperationMapping {
        service: "translate",
        operation: "ListParallelData",
        kind: "AWS::Translate::ParallelData",
    },
    OperationMapping {
        service: "translate",
        operation: "ListTerminologies",
        kind: "AWS::Translate::Terminology",
    },
    OperationMapping {
        service: "verifiedpermissions",
        operation: "ListPolicyStores",
        kind: "AWS::VerifiedPermissions::PolicyStore",
    },
    OperationMapping {
        service: "verifiedpermissions",
        operation: "ListPolicyStoreAliases",
        kind: "AWS::VerifiedPermissions::PolicyStoreAlias",
    },
    OperationMapping {
        service: "voice-id",
        operation: "ListDomains",
        kind: "AWS::VoiceID::Domain",
    },
];
