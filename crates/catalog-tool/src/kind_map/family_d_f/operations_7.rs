use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "forecast", operation: "ListWhatIfForecasts", kind: "AWS::ApiOnly::ForecastWhatIfForecast" },
    OperationMapping { service: "frauddetector", operation: "GetDetectors", kind: "AWS::FraudDetector::Detector" },
    OperationMapping { service: "frauddetector", operation: "GetEntityTypes", kind: "AWS::FraudDetector::EntityType" },
    OperationMapping { service: "frauddetector", operation: "GetEventTypes", kind: "AWS::FraudDetector::EventType" },
    OperationMapping { service: "frauddetector", operation: "GetLabels", kind: "AWS::FraudDetector::Label" },
    OperationMapping { service: "frauddetector", operation: "GetModels", kind: "AWS::ApiOnly::FraudDetectorModel" },
    OperationMapping { service: "frauddetector", operation: "GetOutcomes", kind: "AWS::FraudDetector::Outcome" },
    OperationMapping { service: "frauddetector", operation: "GetListsMetadata", kind: "AWS::FraudDetector::List" },
    OperationMapping { service: "fsx", operation: "DescribeBackups", kind: "AWS::ApiOnly::FSxBackup" },
    OperationMapping { service: "fsx", operation: "DescribeDataRepositoryAssociations", kind: "AWS::FSx::DataRepositoryAssociation" },
    OperationMapping { service: "fsx", operation: "DescribeFileCaches", kind: "AWS::FSx::FileCache" },
    OperationMapping { service: "fsx", operation: "DescribeFileSystems", kind: "AWS::FSx::FileSystem" },
    OperationMapping { service: "fsx", operation: "DescribeS3AccessPointAttachments", kind: "AWS::FSx::S3AccessPointAttachment" },
    OperationMapping { service: "fsx", operation: "DescribeSnapshots", kind: "AWS::FSx::Snapshot" },
    OperationMapping { service: "fsx", operation: "DescribeStorageVirtualMachines", kind: "AWS::FSx::StorageVirtualMachine" },
    OperationMapping { service: "fsx", operation: "DescribeVolumes", kind: "AWS::FSx::Volume" },
];
