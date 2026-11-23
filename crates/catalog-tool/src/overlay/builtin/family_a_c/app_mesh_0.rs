// Pinned source: .local/aws-models/models/app-mesh/service/2019-01-25/app-mesh-2019-01-25.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateGatewayRoute",
            deferred(
                "Model requires gatewayRouteName, meshName, virtualGatewayName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a gateway route. A gateway route is attached to a virtual gateway and routes traffic to an existing virtual service. If a route matches a request, it can distribute traffic to a target virtual service. For more…",
            ),
        ),
        (
            "CreateMesh",
            deferred(
                "Model requires meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a service mesh. A service mesh is a logical boundary for network traffic between services that are represented by resources within the mesh. After you create your service mesh, you can create virtual services, virtual…",
            ),
        ),
        (
            "CreateRoute",
            deferred(
                "Model requires routeName, meshName, virtualRouterName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a route that is associated with a virtual router. You can route several different protocols and define a retry policy for a route. Traffic can be routed to one or more virtual nodes. For more information about routes, see…",
            ),
        ),
        (
            "CreateVirtualGateway",
            deferred(
                "Model requires virtualGatewayName, meshName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a virtual gateway. A virtual gateway allows resources outside your mesh to communicate to resources that are inside your mesh. The virtual gateway represents an Envoy proxy running in an Amazon ECS task, in a Kubernetes…",
            ),
        ),
        (
            "CreateVirtualNode",
            deferred(
                "Model requires virtualNodeName, meshName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a virtual node within a service mesh. A virtual node acts as a logical pointer to a particular task group, such as an Amazon ECS service or a Kubernetes deployment. When you create a virtual node, you can specify the…",
            ),
        ),
        (
            "CreateVirtualRouter",
            deferred(
                "Model requires virtualRouterName, meshName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a virtual router within a service mesh. Specify a listener for any inbound traffic that your virtual router receives. Create a virtual router for each protocol and port that you need to route. Virtual routers handle…",
            ),
        ),
        (
            "CreateVirtualService",
            deferred(
                "Model requires virtualServiceName, meshName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a virtual service within a service mesh. A virtual service is an abstraction of a real service that is provided by a virtual node directly or indirectly by means of a virtual router. Dependent services call your virtual…",
            ),
        ),
        (
            "DeleteGatewayRoute",
            deferred(
                "Model requires gatewayRouteName, meshName, virtualGatewayName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing gateway route.",
            ),
        ),
        (
            "DeleteMesh",
            deferred(
                "Model requires meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing service mesh. You must delete all resources (virtual services, routes, virtual routers, and virtual nodes) in the service mesh before you can delete the mesh itself.",
            ),
        ),
        (
            "DeleteRoute",
            deferred(
                "Model requires routeName, meshName, virtualRouterName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing route.",
            ),
        ),
        (
            "DeleteVirtualGateway",
            deferred(
                "Model requires virtualGatewayName, meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing virtual gateway. You cannot delete a virtual gateway if any gateway routes are associated to it.",
            ),
        ),
        (
            "DeleteVirtualNode",
            deferred(
                "Model requires virtualNodeName, meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing virtual node. You must delete any virtual services that list a virtual node as a service provider before you can delete the virtual node itself.",
            ),
        ),
        (
            "DeleteVirtualRouter",
            deferred(
                "Model requires virtualRouterName, meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing virtual router. You must delete any routes associated with the virtual router before you can delete the router itself.",
            ),
        ),
        (
            "DeleteVirtualService",
            deferred(
                "Model requires virtualServiceName, meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing virtual service.",
            ),
        ),
        (
            "DescribeGatewayRoute",
            deferred(
                "Model requires gatewayRouteName, meshName, virtualGatewayName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an existing gateway route.",
            ),
        ),
        // Model: Describes an existing service mesh.
        (
            "DescribeMesh",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("meshName")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshName")?,
                    },
                )]
                .into_iter()
                .chain([(
                    path("meshOwner")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshOwner")?,
                    },
                )])
                .collect(),
                queryable: vec![path("metadata.arn")?],
                ..read(
                    "mesh",
                    Identity::Arn(path("metadata.arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeRoute",
            deferred(
                "Model requires routeName, meshName, virtualRouterName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an existing route.",
            ),
        ),
        (
            "DescribeVirtualGateway",
            deferred(
                "Model requires virtualGatewayName, meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an existing virtual gateway.",
            ),
        ),
        (
            "DescribeVirtualNode",
            deferred(
                "Model requires virtualNodeName, meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an existing virtual node.",
            ),
        ),
        (
            "DescribeVirtualRouter",
            deferred(
                "Model requires virtualRouterName, meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an existing virtual router.",
            ),
        ),
        (
            "DescribeVirtualService",
            deferred(
                "Model requires virtualServiceName, meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an existing virtual service.",
            ),
        ),
        // Reviewed full response: route references contain identifiers/ownership/version/times, not route specifications.
        (
            "ListGatewayRoutes",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [
                    (
                        path("meshName")?,
                        Binding::From {
                            operation: name("ListVirtualGateways")?,
                            path: path("meshName")?,
                        },
                    ),
                    (
                        path("virtualGatewayName")?,
                        Binding::From {
                            operation: name("ListVirtualGateways")?,
                            path: path("virtualGatewayName")?,
                        },
                    ),
                    (
                        path("meshOwner")?,
                        Binding::From {
                            operation: name("ListVirtualGateways")?,
                            path: path("meshOwner")?,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "gatewayRoutes",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of existing service meshes.
        (
            "ListMeshes",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "meshes",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Bind every routing key from the same reference, never independent mesh/router roots.
        (
            "ListRoutes",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [
                    (
                        path("meshName")?,
                        Binding::From {
                            operation: name("ListVirtualRouters")?,
                            path: path("meshName")?,
                        },
                    ),
                    (
                        path("virtualRouterName")?,
                        Binding::From {
                            operation: name("ListVirtualRouters")?,
                            path: path("virtualRouterName")?,
                        },
                    ),
                    (
                        path("meshOwner")?,
                        Binding::From {
                            operation: name("ListVirtualRouters")?,
                            path: path("meshOwner")?,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "routes",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the tags for an App Mesh resource.",
            ),
        ),
        // Model: Returns a list of existing virtual gateways in a service mesh.
        (
            "ListVirtualGateways",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("meshName")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshName")?,
                    },
                )]
                .into_iter()
                .chain([(
                    path("meshOwner")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshOwner")?,
                    },
                )])
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "virtualGateways",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Returns a list of existing virtual nodes.
        (
            "ListVirtualNodes",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("meshName")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshName")?,
                    },
                )]
                .into_iter()
                .chain([(
                    path("meshOwner")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshOwner")?,
                    },
                )])
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "virtualNodes",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of existing virtual routers in a service mesh.
        (
            "ListVirtualRouters",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("meshName")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshName")?,
                    },
                )]
                .into_iter()
                .chain([(
                    path("meshOwner")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshOwner")?,
                    },
                )])
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "virtualRouters",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        // Model: Returns a list of existing virtual services in a service mesh.
        (
            "ListVirtualServices",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("meshName")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshName")?,
                    },
                )]
                .into_iter()
                .chain([(
                    path("meshOwner")?,
                    Binding::From {
                        operation: name("ListMeshes")?,
                        path: path("meshOwner")?,
                    },
                )])
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "virtualServices",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the specified tags to a resource with the specified resourceArn . If existing tags on a resource aren't specified in the request parameters, they aren't changed. When a resource is deleted, the tags associated with…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes specified tags from a resource.",
            ),
        ),
        (
            "UpdateGatewayRoute",
            deferred(
                "Model requires gatewayRouteName, meshName, virtualGatewayName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing gateway route that is associated to a specified virtual gateway in a service mesh.",
            ),
        ),
        (
            "UpdateMesh",
            deferred(
                "Model requires meshName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing service mesh.",
            ),
        ),
        (
            "UpdateRoute",
            deferred(
                "Model requires routeName, meshName, virtualRouterName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing route for a specified service mesh and virtual router.",
            ),
        ),
        (
            "UpdateVirtualGateway",
            deferred(
                "Model requires virtualGatewayName, meshName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing virtual gateway in a specified service mesh.",
            ),
        ),
        (
            "UpdateVirtualNode",
            deferred(
                "Model requires virtualNodeName, meshName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing virtual node in a specified service mesh.",
            ),
        ),
        (
            "UpdateVirtualRouter",
            deferred(
                "Model requires virtualRouterName, meshName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing virtual router in a specified service mesh.",
            ),
        ),
        (
            "UpdateVirtualService",
            deferred(
                "Model requires virtualServiceName, meshName, spec. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing virtual service in a specified service mesh.",
            ),
        ),
    ])
}
