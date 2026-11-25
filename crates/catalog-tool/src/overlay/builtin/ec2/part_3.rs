use super::super::{empty, path};
use crate::classes::DenyReason;
use crate::overlay::{
    FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass, Paginator,
    Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("DescribeFpgaImages", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("FpgaImages")?),
        fixed: [(path("Owners")?, FixedValue::Strings(vec!["self".to_owned()]))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("FpgaImageId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeHostReservationOfferings", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::ReferenceData)),
        ..empty()
    }),
("DescribeHostReservations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("HostReservationSet")?),
        identity: Some(Identity::Key(vec![path("HostReservationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeHosts", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Hosts")?),
        identity: Some(Identity::Key(vec![path("HostId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIamInstanceProfileAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("IamInstanceProfileAssociations")?),
        identity: Some(Identity::Key(vec![path("AssociationId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeIdFormat", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Statuses")?),
        identity: Some(Identity::Key(vec![path("Resource")?])),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeIdentityIdFormat", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required PrincipalArn is not enumerated by EC2 and Statuses[] omits that principal; needs cross-service principal enumeration and request-context identity.".to_owned()),
        ..empty()
    }),
("DescribeImageAttribute", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires ImageId and Attribute; UefiData can expose variable-store contents, so complete safe metadata coverage requires bounded attribute fanout excluding uefiData and contextual result identity.".to_owned()),
        ..empty()
    }),
("DescribeImageReferences", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required ImageIds is a list but DescribeImages emits scalar ImageId per item; needs reviewed scalar-to-list batching and cross-region/cross-account reference ownership from ImageReferences[].Arn.".to_owned()),
        ..empty()
    }),
("DescribeImageUsageReportEntries", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ImageUsageReportEntries")?),
        identity: Some(Identity::Key(vec![path("ReportId")?, path("AccountId")?, path("ImageId")?, path("ResourceType")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeImageUsageReports", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("ImageUsageReports")?),
        identity: Some(Identity::Key(vec![path("ReportId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeImages", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Images")?),
        fixed: [(path("Owners")?, FixedValue::Strings(vec!["self".to_owned()])), (path("IncludeDeprecated")?, FixedValue::Boolean(true)), (path("IncludeDisabled")?, FixedValue::Boolean(true))].into_iter().collect(),
        identity: Some(Identity::Key(vec![path("ImageId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeImportImageTasks", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeImportSnapshotTasks", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeInstanceAttribute", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Requires InstanceId and Attribute; UserData is startup content, so requires non-userData attribute fanout and per-attribute contextual identity after nested instance enumeration is supported.".to_owned()),
        ..empty()
    }),
("DescribeInstanceConnectEndpoints", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InstanceConnectEndpoints")?),
        identity: Some(Identity::Key(vec![path("InstanceConnectEndpointId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeInstanceCreditSpecifications", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InstanceCreditSpecifications")?),
        identity: Some(Identity::Key(vec![path("InstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeInstanceEventNotificationAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("InstanceTagAttribute is an account/region settings singleton with no stable output identifier; requires context-derived identity.".to_owned()),
        ..empty()
    }),
("DescribeInstanceEventWindows", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InstanceEventWindows")?),
        identity: Some(Identity::Key(vec![path("InstanceEventWindowId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("DescribeInstanceImageMetadata", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("InstanceImageMetadata")?),
        identity: Some(Identity::Key(vec![path("InstanceId")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    })
    ])
}
