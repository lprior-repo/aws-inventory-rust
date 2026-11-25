use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    ArnResourceKind, Binding, Emission, FixedValue, Identity, ItemSelector, MapKeyExclusion,
    NameError, OperationOverlay, OverlayClass, Paginator, Scope, SelectionKind, TagEndpoint,
    TagEndpointParent, TagEndpointRequest, TagEndpointSelector, TokenPair, UrlResourceKind,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("AddPermission", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CancelMessageMoveTask", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ChangeMessageVisibility", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ChangeMessageVisibilityBatch", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateQueue", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteMessage", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteMessageBatch", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("DeleteQueue", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("GetQueueAttributes", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::Root),
        identity: Some(Identity::MapArn { path: path("Attributes")?, key: "QueueArn".to_owned(), resource_kind: ArnResourceKind::SqsQueue, request_url: Some(path("QueueUrl")?) }),
        paginator: Some(Paginator::None),
        bind: std::collections::BTreeMap::from([(path("QueueUrl")?, Binding::FromUrl { operation: name("ListQueues")?, path: path("$")?, resource_kind: UrlResourceKind::SqsQueue })]),
        fixed: std::collections::BTreeMap::from([(path("AttributeNames")?, FixedValue::Strings([
            "QueueArn", "Policy", "VisibilityTimeout", "MaximumMessageSize", "MessageRetentionPeriod",
            "DelaySeconds", "ReceiveMessageWaitTimeSeconds", "RedrivePolicy", "RedriveAllowPolicy",
            "FifoQueue", "ContentBasedDeduplication", "KmsMasterKeyId", "KmsDataKeyReusePeriodSeconds", "CreatedTimestamp", "LastModifiedTimestamp",
            "SqsManagedSseEnabled", "DeduplicationScope", "FifoThroughputLimit",
        ].into_iter().map(str::to_owned).collect()))]),
        exclude_map_keys: ["ApproximateNumberOfMessages", "ApproximateNumberOfMessagesNotVisible", "ApproximateNumberOfMessagesDelayed", "LastModifiedTimestamp"]
            .into_iter().map(|key| Ok(MapKeyExclusion { path: path("Attributes")?, key: key.to_owned() }))
            .collect::<Result<Vec<_>, NameError>>()?,
        ..empty()
    }),
("GetQueueUrl", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Lookup returns root QueueUrl and has no pagination, but required QueueName has no callable modeled producer. ListQueues exposes only QueueUrls list<string>; binding requires an explicit reviewed URL last-component extraction absent from the schema. A fixed queue name would silently narrow account inventory. QueueOwnerAWSAccountId is optional and must not introduce cross-account discovery.".to_owned()),
        ..empty()
    }),
("ListDeadLetterSourceQueues", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required QueueUrl is the dead-letter queue URL and can bind from the selected ListQueues scalar using $. Output queueUrls list<string> also supports scalar-self selection, but a source-to-dead-letter relation must retain the bound parent QueueUrl alongside each child. Canonical QueueArn identity and request-correlated parent provenance remain unavailable. Regional token pagination is NextToken to NextToken with MaxResults explicitly set in 1..1000; omitting MaxResults suppresses continuation. No modeled ARN member exists in each selected item.".to_owned()),
        ..empty()
    }),
("ListMessageMoveTasks", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Read-only task metadata requires SourceArn from the unavailable GetQueueAttributes Attributes[QueueArn] map-key binding. Results is a list of task structures; TaskHandle exists only while RUNNING, so it cannot identify completed, cancelled or failed tasks. SourceArn alone conflates repeated tasks; SourceArn plus StartedTimestamp needs a reviewed uniqueness guarantee not present in the model. MaxResults must be 10 for the complete modeled recent-task window (default 1); there is no continuation token and this is not complete historical enumeration. No message content is retrieved.".to_owned()),
        ..empty()
    }),
("ListQueueTags", TagEndpoint {
        parents: vec![TagEndpointParent { operation: name("GetQueueAttributes")?,
            request: TagEndpointRequest::Context { input: path("QueueUrl")?, parent: path("QueueUrl")? } }],
        selector: TagEndpointSelector::Map { path: path("Tags")? },
    }.operation(Scope::Region, Paginator::None)?),
("ListQueues", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        emission: Emission::DiscoveryOnly,
        items: Some(ItemSelector::Explicit { path: path("QueueUrls")?, kind: SelectionKind::List }),
        identity: Some(Identity::Key(vec![path("$")?])),
        page_size: Some(1000),
        paginator: Some(Paginator::Token(TokenPair { input: path("NextToken")?, output: path("NextToken")?, optional: false })),
        ..empty()
    }),
("PurgeQueue", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ReceiveMessage", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("RemovePermission", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("SendMessage", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("SendMessageBatch", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("SetQueueAttributes", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
