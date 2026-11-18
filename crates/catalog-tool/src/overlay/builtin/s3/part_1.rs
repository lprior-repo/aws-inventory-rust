use super::super::empty;
use crate::classes::DenyReason;
use crate::overlay::{NameError, OperationOverlay, OverlayClass};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("GetBucketRequestPayment", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Payer reports requester-pays configuration without consenting to charges. Requires Bucket=ListBuckets.Name, ExpectedBucketOwner=account, inherited bucket identity and regional addressing. Do not set RequestPayer=requester on any collector call.".to_owned()),
        ..empty()
    }),
("GetBucketTagging", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("TagSet.Key and TagSet.Value enrich the same bucket, not independent tag resources. Requires parent-identity inheritance, Bucket=ListBuckets.Name, ExpectedBucketOwner=account, regional addressing and NoSuchTagSet-to-empty-tag semantics. No tag payload is fabricated from ListBuckets.".to_owned()),
        ..empty()
    }),
("GetBucketVersioning", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Status and MFADelete describe bucket versioning; neither identifies the bucket. Requires inherited parent identity, Bucket=ListBuckets.Name, ExpectedBucketOwner=account and regional addressing. An empty output denotes unversioned configuration, not a missing resource.".to_owned()),
        ..empty()
    }),
("GetBucketWebsite", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("RedirectAllRequestsTo, IndexDocument, ErrorDocument and RoutingRules are website configuration, never fetched objects or URLs. Requires inherited bucket identity, owner guard, regional addressing and missing-website handling. Do not follow redirect hosts or expand index/error document keys.".to_owned()),
        ..empty()
    }),
("GetObject", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetObjectAcl", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetObjectAnnotation", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetObjectAttributes", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetObjectLegalHold", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetObjectLockConfiguration", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ObjectLockConfiguration is bucket-level retention-default configuration despite the operation name; it is not GetObjectRetention. Requires Bucket=ListBuckets.Name, ExpectedBucketOwner=account, inherited bucket identity, regional addressing and missing-configuration handling.".to_owned()),
        ..empty()
    }),
("GetObjectRetention", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetObjectTagging", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetObjectTorrent", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("GetPublicAccessBlock", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("PublicAccessBlockConfiguration contains bucket-level booleans, not effective account-wide public-access policy. Requires inherited bucket identity, owner guard, regional addressing and missing-configuration handling; account-level settings belong to a separately reviewed s3-control operation.".to_owned()),
        ..empty()
    }),
("HeadBucket", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("HeadObject", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ListBucketAnalyticsConfigurations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("AnalyticsConfigurationList.Id is unique only within a bucket. Requires parent Bucket+Id identity and Bucket=ListBuckets.Name, ExpectedBucketOwner=account plus per-bucket regional dispatch. Pagination is ContinuationToken -> NextContinuationToken with IsTruncated; items already contain the full configuration including StorageClassAnalysis.DataExport.Destination.S3BucketDestination.Bucket.".to_owned()),
        ..empty()
    }),
("ListBucketIntelligentTieringConfigurations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("IntelligentTieringConfigurationList.Id is bucket-scoped configuration identity, not a global ID. Requires parent Bucket+Id identity, Bucket=ListBuckets.Name, ExpectedBucketOwner=account and regional dispatch. Pagination is ContinuationToken -> NextContinuationToken with IsTruncated; no object contents or tiering action is requested.".to_owned()),
        ..empty()
    }),
("ListBucketInventoryConfigurations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("InventoryConfigurationList.Id requires parent Bucket+Id identity, owner guard and bucket-region/control endpoint dispatch. Pagination is ContinuationToken -> NextContinuationToken with IsTruncated. InventoryConfigurationList.Destination.S3BucketDestination.Bucket is a destination ARN; read only its configuration, never the exported inventory files.".to_owned()),
        ..empty()
    }),
("ListBucketMetricsConfigurations", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("MetricsConfigurationList.Id requires parent Bucket+Id identity, owner guard and bucket-region/control endpoint dispatch. Pagination is ContinuationToken -> NextContinuationToken with IsTruncated. Reads metric configuration only, never metric time series; list item parity must precede skipping GetBucketMetricsConfiguration.".to_owned()),
        ..empty()
    })
    ])
}
