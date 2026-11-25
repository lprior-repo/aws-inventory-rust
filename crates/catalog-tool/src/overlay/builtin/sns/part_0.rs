use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    ArnResourceKind, Binding, Identity, IdentityBranch, IdentityChoice, IdentityField,
    IdentityLeaf, IdentityPredicate, ItemSelector, MapKeyExclusion, NameError, OperationOverlay,
    OverlayClass, Paginator, ResourceProjection, Scope, TagEndpoint, TagEndpointParent,
    TagEndpointRequest, TagEndpointSelector, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListTopics", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Topics")?),
        identity: Some(Identity::Arn(path("TopicArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("ListOriginationNumbers", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("PhoneNumbers")?),
        page_size: Some(30),
        identity: Some(Identity::Key(vec![path("Iso2CountryCode")?, path("PhoneNumber")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("GetTopicAttributes", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::Root),
        identity: Some(Identity::MapArn { path: path("Attributes")?, key: "TopicArn".to_owned(), resource_kind: ArnResourceKind::SnsTopic, request_url: None }),
        bind: std::collections::BTreeMap::from([(path("TopicArn")?, Binding::From { operation: name("ListTopics")?, path: path("TopicArn")? })]),
        exclude_map_keys: ["SubscriptionsConfirmed", "SubscriptionsPending", "SubscriptionsDeleted"]
            .into_iter().map(|key| Ok(MapKeyExclusion { path: path("Attributes")?, key: key.to_owned() }))
            .collect::<Result<Vec<_>, NameError>>()?,
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetDataProtectionPolicy", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::Root),
        identity: Some(Identity::CorrelatedKey(vec![IdentityField::RequestArn(path("ResourceArn")?)])),
        bind: std::collections::BTreeMap::from([(path("ResourceArn")?, Binding::From { operation: name("ListTopics")?, path: path("TopicArn")? })]),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("ListTagsForResource", TagEndpoint {
        parents: vec![TagEndpointParent { operation: name("ListTopics")?, request: TagEndpointRequest::Member { input: path("ResourceArn")?, parent: path("TopicArn")? } }],
        selector: TagEndpointSelector::List { path: path("Tags")?, key: path("Key")?, value: path("Value")? },
    }.operation(Scope::Region, Paginator::None)?),
("ListSubscriptions", subscriptions(None)?),
("ListSubscriptionsByTopic", subscriptions(Some("ListTopics"))?),
("GetSubscriptionAttributes", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::Root),
        identity: Some(Identity::MapArn { path: path("Attributes")?, key: "SubscriptionArn".to_owned(), resource_kind: ArnResourceKind::SnsSubscription, request_url: None }),
        projection: Some(ResourceProjection::SensitiveMapDigest { path: path("Attributes")?, key: "Endpoint".to_owned() }),
        bind: std::collections::BTreeMap::from([(path("SubscriptionArn")?, Binding::FromArn { operation: name("ListSubscriptions")?, path: path("SubscriptionArn")?, arn_service: "sns".to_owned(), resource_kind: ArnResourceKind::SnsSubscription })]),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("GetSMSAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Safe SMS settings return an attributes map with no stable identity. Needs account-region singleton identity and regional settings kind; runtime compiler rejects Identity::Template. Future request can explicitly select MonthlySpendLimit, DeliveryStatusIAMRole, DeliveryStatusSuccessSamplingRate, DefaultSenderID, DefaultSMSType and UsageReportS3Bucket; bucket names do not authorize reading usage-report content.".to_owned()),
        ..empty()
    }),
("GetSMSSandboxAccountStatus", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Read-only regional account status returns only IsInSandbox. Needs account-region singleton identity and settings kind; IsInSandbox is mutable status, not identity, and runtime compiler rejects Identity::Template.".to_owned()),
        ..empty()
    }),
("ListPlatformApplications", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("PlatformApplications")?),
        identity: Some(Identity::Arn(path("PlatformApplicationArn")?)),
        exclude_response_members: vec![path("PlatformApplications.Attributes")?],
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("GetPlatformApplicationAttributes", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("ListPlatformApplications now safely discovers parent ARNs by omitting Attributes. This detail output contains only the open Attributes map: whole-member omission would leave no metadata, while selective map-key allowlisting and request-bound identity/parent enrichment remain unsupported. Arbitrary attribute values are not proven safe.".to_owned()),
        ..empty()
    }),
("GetEndpointAttributes", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListEndpointsByPlatformApplication", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListPhoneNumbersOptedOut", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("ListSMSSandboxPhoneNumbers", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("CheckIfPhoneNumberIsOptedOut", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("AddPermission", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ConfirmSubscription", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreatePlatformApplication", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}

fn subscriptions(parent: Option<&str>) -> Result<OperationOverlay, NameError> {
    let pending = IdentityLeaf::Key(vec![
        path("TopicArn")?,
        path("Protocol")?,
        path("Endpoint")?,
    ]);
    let identity = Identity::Choice(IdentityChoice::Discriminated(vec![
        IdentityBranch::new(
            IdentityPredicate::TextEquals {
                path: path("SubscriptionArn")?,
                value: "PendingConfirmation".to_owned(),
            },
            pending.clone(),
        )?,
        IdentityBranch::new(
            IdentityPredicate::TextEquals {
                path: path("SubscriptionArn")?,
                value: "Deleted".to_owned(),
            },
            pending,
        )?,
        IdentityBranch::new(
            IdentityPredicate::Present(path("SubscriptionArn")?),
            IdentityLeaf::OwnedArn {
                path: path("SubscriptionArn")?,
                owner: path("Owner")?,
                resource_kind: ArnResourceKind::SnsSubscription,
            },
        )?,
    ]));
    let mut operation = OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("Subscriptions")?),
        identity: Some(identity),
        projection: Some(ResourceProjection::SensitiveDigest(path("Endpoint")?)),
        paginator: Some(Paginator::Token(TokenPair {
            input: path("NextToken")?,
            output: path("NextToken")?,
            optional: false,
        })),
        ..empty()
    };
    if let Some(parent) = parent {
        operation.class = Some(OverlayClass::Expand);
        operation.bind.insert(
            path("TopicArn")?,
            Binding::From {
                operation: name(parent)?,
                path: path("TopicArn")?,
            },
        );
    }
    Ok(operation)
}
