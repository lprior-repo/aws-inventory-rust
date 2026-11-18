use crate::kind_map::ApiOnlyMapping;

pub(super) const ENTRIES: &[ApiOnlyMapping<'static>] = &[
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::WorkSpacesBundle",
        reason: "WorkSpaces bundle configuration has a stable BundleId and no pinned CFN bundle type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::WorkSpacesImage",
        reason: "WorkSpaces desktop image is distinct from an EC2 Image and has no pinned CFN equivalent.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::WorkSpacesThinClientDevice",
        reason: "Registered physical thin client device is distinct from a ThinClient Environment and has no pinned CFN device type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::XRayIndexingRule",
        reason: "XRay indexing sampling configuration is distinct from a request SamplingRule and has no pinned CFN indexing rule type.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::TimestreamBatchLoadTask",
        reason: "Timestream ingestion batch task has a stable task ID and is distinct from Database, Table, and ScheduledQuery; no pinned CFN task type exists.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::Wafv2ManagedRuleSet",
        reason: "WAFv2 managed rule set is a versioned managed resource distinct from a user-defined RuleGroup; no pinned CFN managed rule set type exists.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::TransferAccess",
        reason: "Transfer external identity access is identified by ServerId and ExternalId and is distinct from a service-managed Transfer User; no pinned CFN access type exists.",
    },
    ApiOnlyMapping {
        kind: "AWS::ApiOnly::WorkSpacesWebTrustStoreCertificate",
        reason: "Trust store certificate is identified by trustStoreArn and thumbprint, is a child of TrustStore, and has no pinned CFN certificate resource type.",
    },
];
