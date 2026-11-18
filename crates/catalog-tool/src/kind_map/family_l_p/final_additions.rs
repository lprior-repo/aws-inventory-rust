use crate::kind_map::{MappingSource, OperationMapping};

pub(super) const SOURCE: MappingSource<'static> = MappingSource {
    operations: &[
        OperationMapping {
            service: "medical-imaging",
            operation: "GetDatastore",
            kind: "AWS::HealthImaging::Datastore",
        },
        OperationMapping {
            service: "license-manager",
            operation: "GetLicenseConfiguration",
            kind: "AWS::LicenseManager::LicenseConfiguration",
        },
        OperationMapping {
            service: "lakeformation",
            operation: "DescribeResource",
            kind: "AWS::LakeFormation::Resource",
        },
        OperationMapping {
            service: "license-manager",
            operation: "GetLicense",
            kind: "AWS::LicenseManager::License",
        },
        OperationMapping {
            service: "license-manager",
            operation: "GetLicenseAssetGroup",
            kind: "AWS::LicenseManager::LicenseAssetGroup",
        },
        OperationMapping {
            service: "license-manager",
            operation: "GetLicenseManagerReportGenerator",
            kind: "AWS::LicenseManager::ReportGenerator",
        },
        OperationMapping {
            service: "lightsail",
            operation: "GetDisk",
            kind: "AWS::Lightsail::Disk",
        },
        OperationMapping {
            service: "lightsail",
            operation: "GetDiskSnapshot",
            kind: "AWS::Lightsail::DiskSnapshot",
        },
        OperationMapping {
            service: "lightsail",
            operation: "GetDomain",
            kind: "AWS::Lightsail::Domain",
        },
        OperationMapping {
            service: "lightsail",
            operation: "GetInstance",
            kind: "AWS::Lightsail::Instance",
        },
        OperationMapping {
            service: "lightsail",
            operation: "GetInstanceSnapshot",
            kind: "AWS::Lightsail::InstanceSnapshot",
        },
        OperationMapping {
            service: "lightsail",
            operation: "GetLoadBalancer",
            kind: "AWS::Lightsail::LoadBalancer",
        },
        OperationMapping {
            service: "lightsail",
            operation: "GetRelationalDatabaseSnapshot",
            kind: "AWS::Lightsail::DatabaseSnapshot",
        },
        OperationMapping {
            service: "lightsail",
            operation: "GetStaticIp",
            kind: "AWS::Lightsail::StaticIp",
        },
        OperationMapping {
            service: "mediaconvert",
            operation: "GetQueue",
            kind: "AWS::MediaConvert::Queue",
        },
        OperationMapping {
            service: "mediastore",
            operation: "DescribeContainer",
            kind: "AWS::MediaStore::Container",
        },
        OperationMapping {
            service: "notificationscontacts",
            operation: "GetEmailContact",
            kind: "AWS::NotificationsContacts::EmailContact",
        },
        OperationMapping {
            service: "outposts",
            operation: "GetOutpost",
            kind: "AWS::Outposts::Outpost",
        },
        OperationMapping {
            service: "outposts",
            operation: "GetSite",
            kind: "AWS::Outposts::Site",
        },
        OperationMapping {
            service: "payment-cryptography",
            operation: "GetAlias",
            kind: "AWS::PaymentCryptography::Alias",
        },
        OperationMapping {
            service: "pinpoint-email",
            operation: "GetConfigurationSet",
            kind: "AWS::PinpointEmail::ConfigurationSet",
        },
        OperationMapping {
            service: "pinpoint",
            operation: "GetApp",
            kind: "AWS::Pinpoint::App",
        },
        OperationMapping {
            service: "pinpoint",
            operation: "GetRecommenderConfiguration",
            kind: "AWS::ApiOnly::PinpointRecommenderConfiguration",
        },
    ],
    ..super::super::EMPTY
};
