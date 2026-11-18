use super::super::{empty, name, path};
use crate::classes::DenyReason;
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

// Exact typed migration of the reviewed pinned-model policy; not emulator evidence.
pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListClusters", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("clusterArns")?),
        page_size: Some(100),
        identity: Some(Identity::Arn(path("$")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListContainerInstances", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("containerInstanceArns")?),
        page_size: Some(100),
        identity: Some(Identity::Arn(path("$")?)),
        bind: [(path("cluster")?, Binding::From { operation: name("ListClusters")?, path: path("$")? })].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListDaemonDeployments", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("daemonDeployments")?),
        identity: Some(Identity::Arn(path("daemonDeploymentArn")?)),
        bind: [(path("daemonArn")?, Binding::From { operation: name("ListDaemons")?, path: path("daemonArn")? })].into_iter().collect(),
        fixed: [(path("maxResults")?, FixedValue::Integer(100))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListDaemons", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("daemonSummariesList")?),
        identity: Some(Identity::Arn(path("daemonArn")?)),
        bind: [(path("clusterArn")?, Binding::From { operation: name("ListClusters")?, path: path("$")? })].into_iter().collect(),
        fixed: [(path("maxResults")?, FixedValue::Integer(100))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListServiceDeployments", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Paginated serviceDeployments[].serviceDeploymentArn is stable, but required service and optional cluster must come from the same discovered service; omitted cluster defaults to default. No callable cluster-preserving ListServices/DescribeServices discovery chain exists.".to_owned()),
        ..empty()
    }),
("ListServices", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("serviceArns")?),
        page_size: Some(100),
        identity: Some(Identity::Arn(path("$")?)),
        bind: [(path("cluster")?, Binding::From { operation: name("ListClusters")?, path: path("$")? })].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListServicesByNamespace", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required namespace needs reviewed Cloud Map namespace discovery across services, but bindings name only same-service operations. Scalar serviceArns is now representable; namespace results can span clusters, and detail requests still need the matching cluster rather than a default or independent cluster collection.".to_owned()),
        ..empty()
    }),
("ListTagsForResource", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required resourceArn needs a reviewed heterogeneous resource source and output tags contains only key/value, without the resource ARN. Need parent-context identity/tag enrichment rather than treating tag keys as independent region-wide resources or overwriting the parent resource.".to_owned()),
        ..empty()
    }),
("ListTaskDefinitionFamilies", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("families")?),
        page_size: Some(100),
        identity: Some(Identity::Key(vec![path("$")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListTaskDefinitions", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Scalar taskDefinitionArns is now representable, but complete enumeration needs ACTIVE/INACTIVE/DELETE_IN_PROGRESS status fanout instead of default ACTIVE only. Current fixed inputs cannot express a status-product plan. DescribeTaskDefinition remains content-denied because it returns plaintext environment and container command content.".to_owned()),
        ..empty()
    }),
("ListTasks", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Scalar taskArns is now representable, but complete discovery needs RUNNING/STOPPED desiredStatus fanout and retained cluster context. Current fixed inputs cannot express a status-product plan; PENDING is modeled but never a desired status. DescribeTasks remains content-denied because overrides returns plaintext environment/command content.".to_owned()),
        ..empty()
    }),
("DescribeDaemonTaskDefinition", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeExpressGatewayService", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeTaskDefinition", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("DescribeTasks", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Content)),
        ..empty()
    }),
("AssociateGatewayWithCluster", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("ContinueServiceDeployment", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateCapacityProvider", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateCluster", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    }),
("CreateDaemon", OperationOverlay {
        class: Some(OverlayClass::Deny(DenyReason::Mutating)),
        ..empty()
    })
    ])
}
