use super::super::{empty, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Identity, IdentityChoice, IdentityLeaf, ItemSelector, NameError, OperationOverlay,
    OverlayClass, Paginator, RequestRouting, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListDirectoryBuckets", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        routing: RequestRouting::S3ExpressControl,
        items: Some(ItemSelector::parse("Buckets")?),
        identity: Some(Identity::Choice(IdentityChoice::PreferArn {
            path: path("BucketArn")?, fallback: IdentityLeaf::Key(vec![path("Name")?]),
            review: "ListDirectoryBuckets returns caller-owned regional directory buckets; BucketArn is optional and Name is the regional bucket identity. BucketRegion is not populated for this operation.".to_owned(),
        })),
        queryable: vec![path("Name")?, path("BucketArn")?],
        paginator: Some(Paginator::Token(TokenPair { input: path("ContinuationToken")?, output: path("ContinuationToken")?, optional: false })),
        ..empty()
    }),
("ListMultipartUploads", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListObjectAnnotations", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListObjectVersions", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListObjects", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListObjectsV2", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListParts", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("SelectObjectContent", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("AbortMultipartUpload", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CompleteMultipartUpload", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CopyObject", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateBucket", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateBucketMetadataConfiguration", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateBucketMetadataTableConfiguration", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateMultipartUpload", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateSession", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Secret)),
        ..empty()
    }),
("DeleteBucket", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteBucketAnalyticsConfiguration", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteBucketCors", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteBucketEncryption", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
