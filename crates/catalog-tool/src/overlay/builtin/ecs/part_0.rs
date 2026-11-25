use super::super::{empty, name, path};
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, TokenPair,
};

pub(super) fn operations() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
("ListAccountSettings", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("settings")?),
        page_size: Some(10),
        identity: Some(Identity::Key(vec![path("principalArn")?, path("name")?])),
        fixed: [(path("effectiveSettings")?, FixedValue::Boolean(true))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListDaemonTaskDefinitions", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("daemonTaskDefinitions")?),
        identity: Some(Identity::Arn(path("arn")?)),
        fixed: [(path("status")?, FixedValue::String("ALL".to_owned())), (path("maxResults")?, FixedValue::Integer(100))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListGatewayClusterAssociations", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("gatewayClusterAssociations")?),
        page_size: Some(100),
        identity: Some(Identity::Key(vec![path("gatewayArn")?, path("clusterArn")?])),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListGatewayRoutes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("gatewayRoutes")?),
        page_size: Some(100),
        identity: Some(Identity::Arn(path("gatewayRouteArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("ListGateways", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("gateways")?),
        page_size: Some(100),
        identity: Some(Identity::Arn(path("gatewayArn")?)),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    }),
("DescribeGateway", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("gateway")?),
        tags: Some(path("tags")?),
        exclude_from_hash: vec![path("updatedAt")?],
        identity: Some(Identity::Arn(path("gatewayArn")?)),
        bind: [(path("gateway")?, Binding::From { operation: name("ListGateways")?, path: path("gatewayArn")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeGatewayRoute", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("gatewayRoute")?),
        identity: Some(Identity::Arn(path("gatewayRouteArn")?)),
        bind: [(path("gatewayRouteArn")?, Binding::From { operation: name("ListGatewayRoutes")?, path: path("gatewayRouteArn")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeCapacityProviders", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Paginated capacityProviders have capacityProviderArn/name identities and TAGS inclusion, but the unfiltered API mixes customer providers with built-in FARGATE/FARGATE_SPOT providers. No reviewed owned-provider filter exists; cluster binding alone does not prove that built-in providers are excluded. Do not invent ARNs or collect reference data as owned resources.".to_owned()),
        ..empty()
    }),
("DescribeClusters", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("clusters[].clusterArn is stable and include supports TAGS/SETTINGS/CONFIGURATIONS/ATTACHMENTS. ListClusters scalar discovery is callable, but required execution needs clusters as lists of at most 100, not scalar ARNs. Runtime child bindings copy selected values verbatim and do not serialize batch metadata; need target-aware singleton-list boxing or bounded batching before enabling. Omitting clusters silently describes only default.".to_owned()),
        ..empty()
    }),
("DescribeContainerInstances", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("containerInstances[].containerInstanceArn is stable, with ec2InstanceId references and TAGS inclusion. ListContainerInstances is callable, but detail requests need both containerInstances lists of at most 100 and that exact parent's cluster. Generic child bindings cannot correlate the scalar instance row with an ancestor cluster.".to_owned()),
        ..empty()
    }),
("DescribeDaemon", OperationOverlay {
        class: Some(OverlayClass::Expand),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("daemon")?),
        exclude_from_hash: vec![path("updatedAt")?],
        identity: Some(Identity::Arn(path("daemonArn")?)),
        bind: [(path("daemonArn")?, Binding::From { operation: name("ListDaemons")?, path: path("daemonArn")? })].into_iter().collect(),
        paginator: Some(Paginator::None),
        ..empty()
    }),
("DescribeDaemonDeployments", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("daemonDeployments[].daemonDeploymentArn is stable and ListDaemonDeployments is callable, but required daemonDeploymentArns needs a list, not the scalar parent ARN. Generic child execution copies the selected scalar verbatim and does not serialize batch metadata; needs target-aware singleton-list binding or enforced batches of at most 20.".to_owned()),
        ..empty()
    }),
("DescribeDaemonRevisions", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("daemonRevisions[].daemonRevisionArn is stable, but required daemonRevisionArns needs all current and historical revision references, including daemon.currentRevisions[].arn and deployment source/target revision ARNs, split into lists of at most 20. Generic nested-list extraction and bounded list splitting are not encoded by current runtime child bindings.".to_owned()),
        ..empty()
    }),
("DescribeGatewayResources", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("Required gateway can bind ListGateways.gatewayArn and nextToken/maxResults paginate, but resource groups live under gatewayResources.resources.hostSharedAlb, crossing a modeled union that member-path validation cannot traverse. Gateway-root identity would overwrite different infrastructure pages; need union-aware list selection keyed by loadBalancerArn, not a fabricated gateway resource row.".to_owned()),
        ..empty()
    }),
("DescribeServiceDeployments", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("serviceDeployments[].serviceDeploymentArn is stable, but required serviceDeploymentArns needs batches of at most 20 from ListServiceDeployments. That parent lacks complete callable service/cluster discovery; retain cluster and service references when enabling the chain.".to_owned()),
        ..empty()
    }),
("DescribeServiceRevisions", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("serviceRevisions[].serviceRevisionArn is stable, but required serviceRevisionArns needs batches of at most 20 from deployment revision references. No callable ListServiceDeployments/DescribeServiceDeployments source exists; service, cluster, task definition and network relationships require shared reviewed mappings.".to_owned()),
        ..empty()
    }),
("DescribeServices", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("services[].serviceArn is stable and include supports TAGS. ListServices is callable, but required services needs a list of at most 10 and the exact originating cluster; generic child bindings cannot correlate a scalar service row with an ancestor cluster. Existing subnet/security-group/task-definition/load-balancer relationships must survive generic mapping.".to_owned()),
        ..empty()
    }),
("DescribeTaskSets", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("taskSets[].taskSetArn is stable and include supports TAGS, but required cluster and service must bind the same discovered service. No callable DescribeServices parent exists; independent parent collections cannot substitute a correlated cluster/service pair.".to_owned()),
        ..empty()
    }),
("GetTaskProtection", OperationOverlay {
        class: Some(OverlayClass::Defer),
        defer_reason: Some("protectedTasks[].taskArn is stable and cluster can bind ListClusters, but the pinned model does not specify whether omitting optional tasks enumerates every relevant task. Explicit tasks needs a complete RUNNING/STOPPED discovery source, same-cluster context and lists of at most 100. Use a distinct protection metadata kind rather than overwriting task configuration.".to_owned()),
        ..empty()
    }),
("ListAttributes", OperationOverlay {
        class: Some(OverlayClass::Enumerate),
        scope: Some(Scope::Region),
        items: Some(ItemSelector::parse("attributes")?),
        page_size: Some(100),
        identity: Some(Identity::Key(vec![path("targetId")?, path("name")?])),
        bind: [(path("cluster")?, Binding::From { operation: name("ListClusters")?, path: path("$")? })].into_iter().collect(),
        fixed: [(path("targetType")?, FixedValue::String("container-instance".to_owned()))].into_iter().collect(),
        paginator: Some(Paginator::Token(TokenPair { input: path("nextToken")?, output: path("nextToken")?, optional: false })),
        ..empty()
    })
    ])
}
