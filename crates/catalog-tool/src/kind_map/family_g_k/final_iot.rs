use crate::kind_map::{ApiOnlyMapping, CfnMapping, OperationMapping, RelationMapping, RelationPath, RelationSource, RelationTarget};

pub(super) const CFN: &[CfnMapping<'static>] = &[
    CfnMapping::canonical("AWS::IoT::Authorizer"),
    CfnMapping::canonical("AWS::IoT::CACertificate"),
    CfnMapping::canonical("AWS::IoT::Certificate"),
    CfnMapping::canonical("AWS::IoT::CertificateProvider"),
    CfnMapping::canonical("AWS::IoT::Command"),
    CfnMapping::canonical("AWS::IoT::FleetMetric"),
    CfnMapping::canonical("AWS::IoT::MitigationAction"),
    CfnMapping::canonical("AWS::IoT::JobTemplate"),
    CfnMapping::canonical("AWS::IoT::SoftwarePackage"),
    CfnMapping::canonical("AWS::IoT::SoftwarePackageVersion"),
    CfnMapping::canonical("AWS::IoT::ProvisioningTemplate"),
    CfnMapping::canonical("AWS::IoT::ScheduledAudit"),
    CfnMapping::canonical("AWS::IoT::Stream"),
    CfnMapping::canonical("AWS::IoT::TopicRule"),
    CfnMapping::canonical("AWS::IoT::OTAUpdate"),
    CfnMapping::canonical("AWS::IoT::Job"),
];

pub(super) const API: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping { kind: "AWS::ApiOnly::IoTPolicyVersion", reason: "Pinned registry has no policy-version kind; metadata version is qualified by the actual bound local policy name, not a second Policy row." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::IoTAuditSuppression", reason: "Pinned registry has no Device Defender audit suppression kind. A suppression belongs to the account and is identified by checkName plus the canonical typed resourceIdentifier structure, never by its referenced resource ARN alone." },
    ApiOnlyMapping { kind: "AWS::ApiOnly::IoTProvisioningTemplateVersion", reason: "Pinned CFN registry has no provisioning-template version kind. The integer version identifier is qualified by its actual bound account-owned template name; no template document is read." },
];

pub(super) const OPERATIONS: &[OperationMapping<'static>] = &[
    OperationMapping { service: "iot", operation: "ListAuthorizers", kind: "AWS::IoT::Authorizer" },
    OperationMapping { service: "iot", operation: "ListCACertificates", kind: "AWS::IoT::CACertificate" },
    OperationMapping { service: "iot", operation: "ListCertificates", kind: "AWS::IoT::Certificate" },
    OperationMapping { service: "iot", operation: "ListCertificateProviders", kind: "AWS::IoT::CertificateProvider" },
    OperationMapping { service: "iot", operation: "ListCommands", kind: "AWS::IoT::Command" },
    OperationMapping { service: "iot", operation: "ListFleetMetrics", kind: "AWS::IoT::FleetMetric" },
    OperationMapping { service: "iot", operation: "ListMitigationActions", kind: "AWS::IoT::MitigationAction" },
    OperationMapping { service: "iot", operation: "ListJobTemplates", kind: "AWS::IoT::JobTemplate" },
    OperationMapping { service: "iot", operation: "ListPackages", kind: "AWS::IoT::SoftwarePackage" },
    OperationMapping { service: "iot", operation: "ListPackageVersions", kind: "AWS::IoT::SoftwarePackageVersion" },
    OperationMapping { service: "iot", operation: "ListProvisioningTemplates", kind: "AWS::IoT::ProvisioningTemplate" },
    OperationMapping { service: "iot", operation: "ListScheduledAudits", kind: "AWS::IoT::ScheduledAudit" },
    OperationMapping { service: "iot", operation: "ListStreams", kind: "AWS::IoT::Stream" },
    OperationMapping { service: "iot", operation: "ListTopicRules", kind: "AWS::IoT::TopicRule" },
    OperationMapping { service: "iot", operation: "ListOTAUpdates", kind: "AWS::IoT::OTAUpdate" },
    OperationMapping { service: "iot", operation: "ListJobs", kind: "AWS::IoT::Job" },
    OperationMapping { service: "iot", operation: "ListPolicyVersions", kind: "AWS::ApiOnly::IoTPolicyVersion" },
    OperationMapping { service: "iot", operation: "ListProvisioningTemplateVersions", kind: "AWS::ApiOnly::IoTProvisioningTemplateVersion" },
    OperationMapping { service: "iot", operation: "ListAuditSuppressions", kind: "AWS::ApiOnly::IoTAuditSuppression" },
];

pub(super) const REQUEST_RELATIONS: &[RelationMapping<'static>] = &[
    RelationMapping { source_kind: "AWS::IoT::SoftwarePackageVersion", source: RelationSource::Request(RelationPath::Fields("packageName")), target_kind: "AWS::IoT::SoftwarePackage", target: RelationTarget::SelectedOutput(RelationPath::Fields("packageName")), relation: "VERSION_OF", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::IoTPolicyVersion", source: RelationSource::Request(RelationPath::Fields("policyName")), target_kind: "AWS::IoT::Policy", target: RelationTarget::SelectedOutput(RelationPath::Fields("policyName")), relation: "VERSION_OF", scope: "account_region", guard: None },
    RelationMapping { source_kind: "AWS::ApiOnly::IoTProvisioningTemplateVersion", source: RelationSource::Request(RelationPath::Fields("templateName")), target_kind: "AWS::IoT::ProvisioningTemplate", target: RelationTarget::SelectedOutput(RelationPath::Fields("templateName")), relation: "VERSION_OF", scope: "account_region", guard: None },
];
