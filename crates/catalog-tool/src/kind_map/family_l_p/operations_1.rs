use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "lightsail", operation: "GetInstances", kind: "AWS::Lightsail::Instance" },
    OperationMapping { service: "lightsail", operation: "GetDisks", kind: "AWS::Lightsail::Disk" },
    OperationMapping { service: "lightsail", operation: "GetDiskSnapshots", kind: "AWS::Lightsail::DiskSnapshot" },
    OperationMapping { service: "lightsail", operation: "GetInstanceSnapshots", kind: "AWS::Lightsail::InstanceSnapshot" },
    OperationMapping { service: "lightsail", operation: "GetLoadBalancers", kind: "AWS::Lightsail::LoadBalancer" },
    OperationMapping { service: "lightsail", operation: "GetRelationalDatabaseSnapshots", kind: "AWS::Lightsail::DatabaseSnapshot" },
    OperationMapping { service: "lightsail", operation: "GetStaticIps", kind: "AWS::Lightsail::StaticIp" },
    OperationMapping { service: "lightsail", operation: "GetDomains", kind: "AWS::Lightsail::Domain" },
    OperationMapping { service: "lightsail", operation: "GetDistributions", kind: "AWS::Lightsail::Distribution" },
    OperationMapping { service: "lightsail", operation: "GetBuckets", kind: "AWS::Lightsail::Bucket" },
    OperationMapping { service: "lightsail", operation: "GetCertificates", kind: "AWS::Lightsail::Certificate" },
    OperationMapping { service: "lightsail", operation: "GetAlarms", kind: "AWS::Lightsail::Alarm" },
    OperationMapping { service: "location", operation: "ListGeofenceCollections", kind: "AWS::Location::GeofenceCollection" },
    OperationMapping { service: "location", operation: "ListMaps", kind: "AWS::Location::Map" },
    OperationMapping { service: "location", operation: "ListPlaceIndexes", kind: "AWS::Location::PlaceIndex" },
    OperationMapping { service: "location", operation: "ListRouteCalculators", kind: "AWS::Location::RouteCalculator" },
    OperationMapping { service: "location", operation: "ListTrackers", kind: "AWS::Location::Tracker" },
    OperationMapping { service: "lookoutequipment", operation: "ListDatasets", kind: "AWS::ApiOnly::LookoutEquipmentDataset" },
    OperationMapping { service: "lookoutequipment", operation: "ListInferenceSchedulers", kind: "AWS::LookoutEquipment::InferenceScheduler" },
    OperationMapping { service: "lookoutequipment", operation: "ListModels", kind: "AWS::ApiOnly::LookoutEquipmentModel" },
];
