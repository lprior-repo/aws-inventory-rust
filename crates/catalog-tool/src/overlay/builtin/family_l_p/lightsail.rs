// Pinned Smithy source: .local/aws-models/models/lightsail/service/2016-11-28/lightsail-2016-11-28.json
use super::policy::{Group, compile};
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AllocateStaticIp",
            "AttachCertificateToDistribution",
            "AttachDisk",
            "AttachInstancesToLoadBalancer",
            "AttachLoadBalancerTlsCertificate",
            "AttachStaticIp",
            "CloseInstancePublicPorts",
            "CopySnapshot",
            "CreateBucket",
            "CreateBucketAccessKey",
            "CreateCertificate",
            "CreateCloudFormationStack",
            "CreateContactMethod",
            "CreateContainerService",
            "CreateContainerServiceDeployment",
            "CreateContainerServiceRegistryLogin",
            "CreateDisk",
            "CreateDiskFromSnapshot",
            "CreateDiskSnapshot",
            "CreateDistribution",
            "CreateDomain",
            "CreateDomainEntry",
            "CreateGUISessionAccessDetails",
            "CreateInstanceSnapshot",
            "CreateInstances",
            "CreateInstancesFromSnapshot",
            "CreateKeyPair",
            "CreateLoadBalancer",
            "CreateLoadBalancerTlsCertificate",
            "CreateRelationalDatabase",
            "CreateRelationalDatabaseFromSnapshot",
            "CreateRelationalDatabaseSnapshot",
            "CreateSubdomain",
            "DeleteAlarm",
            "DeleteAutoSnapshot",
            "DeleteBucket",
            "DeleteBucketAccessKey",
            "DeleteCertificate",
            "DeleteContactMethod",
            "DeleteContainerImage",
            "DeleteContainerService",
            "DeleteDisk",
            "DeleteDiskSnapshot",
            "DeleteDistribution",
            "DeleteDomain",
            "DeleteDomainEntry",
            "DeleteInstance",
            "DeleteInstanceSnapshot",
            "DeleteKeyPair",
            "DeleteKnownHostKeys",
            "DeleteLoadBalancer",
            "DeleteLoadBalancerTlsCertificate",
            "DeleteRelationalDatabase",
            "DeleteRelationalDatabaseSnapshot",
            "DeleteSubdomain",
            "DetachCertificateFromDistribution",
            "DetachDisk",
            "DetachInstancesFromLoadBalancer",
            "DetachStaticIp",
            "DisableAddOn",
            "EnableAddOn",
            "ExportSnapshot",
            "ImportKeyPair",
            "PutAlarm",
            "PutInstancePublicPorts",
            "RebootInstance",
            "RebootRelationalDatabase",
            "RegisterContainerImage",
            "ReleaseStaticIp",
            "ResetDistributionCache",
            "SendContactMethodVerification",
            "SetIpAddressType",
            "SetResourceAccessForBucket",
            "SetupInstanceHttps",
            "StartGUISession",
            "StartInstance",
            "StartRelationalDatabase",
            "StopGUISession",
            "StopInstance",
            "StopRelationalDatabase",
            "TagResource",
            "UntagResource",
            "UpdateBucket",
            "UpdateBucketBundle",
            "UpdateContainerService",
            "UpdateDistribution",
            "UpdateDistributionBundle",
            "UpdateDomainEntry",
            "UpdateInstanceMetadataOptions",
            "UpdateLoadBalancerAttribute",
            "UpdateRelationalDatabase",
            "UpdateRelationalDatabaseParameters",
            "UpdateSubdomainAssociation",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &["CheckSubdomainAvailability"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (subdomainName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "DownloadDefaultKeyPair",
            "GetBucketAccessKeys",
            "GetContainerServiceDeployments",
            "GetContainerServices",
            "GetInstanceAccessDetails",
            "GetRelationalDatabase",
            "GetRelationalDatabaseMasterUserPassword",
            "GetRelationalDatabases",
        ],
        class: OverlayClass::Deny(DenyReason::Secret),
        reason: None,
    },
    Group {
        operations: &[
            "GetActiveNames",
            "GetBlueprints",
            "GetBucketBundles",
            "GetBundles",
            "GetCloudFormationStackRecords",
            "GetContactMethods",
            "GetContainerAPIMetadata",
            "GetContainerServicePowers",
            "GetDistributionBundles",
            "GetDistributionLatestCacheReset",
            "GetExportSnapshotRecords",
            "GetKeyPairs",
            "GetLoadBalancerTlsPolicies",
            "GetOperations",
            "GetProfile",
            "GetRegions",
            "GetRelationalDatabaseBlueprints",
            "GetRelationalDatabaseBundles",
            "GetSubdomains",
            "IsVpcPeered",
            "PeerVpc",
            "UnpeerVpc",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &[
            "GetAutoSnapshots",
            "GetOperationsForResource",
            "GetSetupHistory",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetBucketMetricData"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (bucketName, metricName, startTime, endTime, period, statistics, unit); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetContainerImages"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (serviceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetContainerLog", "GetRelationalDatabaseLogEvents"],
        class: OverlayClass::Deny(DenyReason::Content),
        reason: None,
    },
    Group {
        operations: &["GetContainerServiceMetricData"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (serviceName, metricName, startTime, endTime, period, statistics); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetCostEstimate"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceName, startTime, endTime); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDistributionMetricData"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (distributionName, metricName, startTime, endTime, period, unit, statistics); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetInstancePortStates", "GetInstanceState"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (instanceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetInstanceMetricData"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (instanceName, metricName, period, startTime, endTime, unit, statistics); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetKeyPair"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (keyPairName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetLoadBalancerTlsCertificates"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (loadBalancerName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetLoadBalancerMetricData"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (loadBalancerName, metricName, period, startTime, endTime, unit, statistics); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetOperation"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (operationId); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetRelationalDatabaseEvents",
            "GetRelationalDatabaseLogStreams",
            "GetRelationalDatabaseParameters",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (relationalDatabaseName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetRelationalDatabaseMetricData"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (relationalDatabaseName, metricName, period, startTime, endTime, unit, statistics); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["OpenInstancePublicPorts"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (portInfo, instanceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["TestAlarm"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (alarmName, state); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

#[path = "lightsail_resources.rs"]
mod resources;
use resources::RESOURCES;

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    compile("lightsail", GROUPS, RESOURCES)
}
