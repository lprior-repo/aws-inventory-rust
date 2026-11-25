use super::super::{empty, path};
use crate::overlay::{
    FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator,
    Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListBuckets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Account),
        items: Some(ItemSelector::parse("Buckets")?),
        queryable: vec![path("Name")?, path("BucketRegion")?, path("CreationDate")?],
        fixed: [(path("MaxBuckets")?, FixedValue::Integer(10000))].into_iter().collect(),
        identity: Some(Identity::Template("arn:{partition}:s3:::{Name}".to_owned())),
        paginator: Some(Paginator::Token(TokenPair { input: path("ContinuationToken")?, output: path("ContinuationToken")?, optional: false })),
        ..empty()
    }),
("GetBucketAbac", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AbacStatus is bucket configuration, not object data. Bind Bucket from ListBuckets.Name and ExpectedBucketOwner from account; output has no bucket identity. Requires parent-identity inheritance and bucket-region/addressing dispatch before expanding this singleton.".to_owned()),
        ..empty()
    }),
("GetBucketAccelerateConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Status is transfer-acceleration configuration; RequestCharged is request telemetry, not configuration. No RequestPayer opt-in is authorized. Requires ListBuckets.Name bucket binding, account owner guard, inherited bucket identity and regional addressing; exclude RequestCharged only after the singleton merge is supported.".to_owned()),
        ..empty()
    }),
("GetBucketAcl", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Owner and Grants describe bucket access controls. Requires ListBuckets.Name plus ExpectedBucketOwner=account, inherited bucket identity and bucket regional addressing. Do not substitute GetObjectAcl or treat each grantee as a bucket.".to_owned()),
        ..empty()
    }),
("GetBucketAnalyticsConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AnalyticsConfiguration.Id binds from ListBucketAnalyticsConfigurations.AnalyticsConfigurationList.Id within the same parent bucket. Requires correlated Bucket+Id bindings, bucket-region dispatch and parent-aware configuration identity; the complete list already returns the configuration, so later skip this redundant get only after list parity is proven.".to_owned()),
        ..empty()
    }),
("GetBucketCors", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("CORSRules is a configuration list without stable rule IDs; it must enrich the parent bucket, not create anonymous resources. Requires Bucket=ListBuckets.Name, ExpectedBucketOwner=account, inherited bucket identity and regional addressing.".to_owned()),
        ..empty()
    }),
("GetBucketEncryption", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ServerSideEncryptionConfiguration.Rules describes default encryption, not key material. Requires inherited bucket identity, owner guard, correct bucket endpoint and modeled absent-configuration handling. KMS relationship path is ServerSideEncryptionConfiguration.Rules.ApplyServerSideEncryptionByDefault.KMSMasterKeyID; aliases and key IDs require resolution before claiming an ARN edge.".to_owned()),
        ..empty()
    }),
("GetBucketIntelligentTieringConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("IntelligentTieringConfiguration.Id binds from ListBucketIntelligentTieringConfigurations.IntelligentTieringConfigurationList.Id for the same bucket. Requires correlated Bucket+Id identity and bindings plus regional dispatch; this duplicates the full list item and must not be skipped without proven list parity.".to_owned()),
        ..empty()
    }),
("GetBucketInventoryConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("InventoryConfiguration.Id binds from ListBucketInventoryConfigurations.InventoryConfigurationList.Id for the same bucket. Requires correlated Bucket+Id identity, owner guard, region and general-purpose versus directory control endpoints. Reads export configuration only, never the inventory report objects.".to_owned()),
        ..empty()
    }),
("GetBucketLifecycleConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Rules and TransitionDefaultMinimumObjectSize are bucket lifecycle configuration; Rules.ID is optional, so rules cannot independently identify resources. Requires parent bucket enrichment/identity, owner guard and regional/control endpoint dispatch. No lifecycle action is executed.".to_owned()),
        ..empty()
    }),
("GetBucketLocation", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("LocationConstraint is a region hint, not a resource identity. Requires Bucket=ListBuckets.Name and ExpectedBucketOwner=account plus output-to-endpoint binding; empty constraint means us-east-1 and legacy EU means eu-west-1. HeadBucket is preferred by the model but remains denied by the shared Head prefix classifier.".to_owned()),
        ..empty()
    }),
("GetBucketLogging", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("LoggingEnabled.TargetBucket is a destination bucket name; TargetPrefix and TargetObjectKeyFormat are configuration, never object listings. Requires inherited source-bucket identity, regional addressing, owner guard and name-to-bucket-ARN relationship resolution. Empty LoggingEnabled must remain disabled configuration rather than failed inventory.".to_owned()),
        ..empty()
    }),
("GetBucketMetadataConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GetBucketMetadataConfigurationResult.MetadataConfigurationResult is V2 metadata-table configuration, not table records. Requires inherited Bucket identity, owner guard and regional addressing. Preserve structured configuration and table/IAM references without reading journal, inventory or annotation table contents.".to_owned()),
        ..empty()
    }),
("GetBucketMetadataTableConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("GetBucketMetadataTableConfigurationResult.MetadataTableConfigurationResult.S3TablesDestinationResult.TableBucketArn and TableArn describe V1 metadata destinations, not table records. Requires inherited bucket identity, owner guard, regional addressing and V1/V2 outcome handling; model documents HTTP 405 for a V2 configuration read through this V1 API.".to_owned()),
        ..empty()
    }),
("GetBucketMetricsConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("MetricsConfiguration.Id binds from ListBucketMetricsConfigurations.MetricsConfigurationList.Id within the same bucket. This is request-metrics configuration, not CloudWatch metric values. Requires correlated Bucket+Id identity, owner guard and regional/control endpoint dispatch; do not add a paid metric-data call.".to_owned()),
        ..empty()
    }),
("GetBucketNotificationConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("TopicConfigurations.TopicArn, QueueConfigurations.QueueArn and LambdaFunctionConfigurations.LambdaFunctionArn are relationship targets; EventBridgeConfiguration can be empty but enabled. Requires inherited bucket identity, regional addressing, owner guard and typed relation support; never invoke a destination or read its contents.".to_owned()),
        ..empty()
    }),
("GetBucketOwnershipControls", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("OwnershipControls.Rules describes bucket object-ownership policy, not object instances. Requires Bucket=ListBuckets.Name, ExpectedBucketOwner=account, inherited identity, regional addressing and explicit missing-configuration handling.".to_owned()),
        ..empty()
    }),
("GetBucketPolicy", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Policy is a bucket-policy document string, not object content. Requires Bucket=ListBuckets.Name, owner guard, inherited bucket identity, regional/control endpoint dispatch and missing-policy handling. Preserve the opaque modeled string; no runtime JSON parsing or policy-derived resource edges are introduced.".to_owned()),
        ..empty()
    }),
("GetBucketPolicyStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PolicyStatus.IsPublic is bucket metadata, not an identity. Requires parent bucket enrichment with inherited identity, Bucket=ListBuckets.Name, ExpectedBucketOwner=account, regional addressing and explicit absent-policy handling.".to_owned()),
        ..empty()
    }),
("GetBucketReplication", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ReplicationConfiguration.Role and Rules.Destination.Bucket are IAM-role and bucket-ARN references; Rules.Destination.EncryptionConfiguration.ReplicaKmsKeyID may be an ID or ARN. Requires inherited source bucket identity, regional dispatch, owner guard, absent-configuration handling and reference resolution; never perform replication or read object contents.".to_owned()),
        ..empty()
    })
    ])
}
