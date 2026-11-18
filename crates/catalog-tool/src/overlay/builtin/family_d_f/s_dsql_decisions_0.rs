// Explicit dispositions from .local/aws-models/models/dsql/service/2018-05-10/dsql-2018-05-10.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "CreateCluster",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: The CreateCluster API allows you to create both single-Region clusters and multi-Region clusters.",
    ),
    (
        "CreateStream",
        "Requires clusterIdentifier, targetDefinition, ordering, format; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new change data capture (CDC) stream for a cluster.",
    ),
    (
        "DeleteCluster",
        "Requires identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a cluster in Amazon Aurora DSQL.",
    ),
    (
        "DeleteClusterPolicy",
        "Requires identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the resource-based policy attached to a cluster.",
    ),
    (
        "DeleteStream",
        "Requires clusterIdentifier, streamIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a stream from a cluster.",
    ),
    (
        "GetCluster",
        "Requires identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about a cluster.",
    ),
    (
        "GetClusterPolicy",
        "Requires identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the resource-based policy document attached to a cluster.",
    ),
    (
        "GetStream",
        "Requires clusterIdentifier, streamIdentifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves information about a stream.",
    ),
    (
        "GetVpcEndpointServiceName",
        "Requires identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Retrieves the VPC endpoint service name.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists all of the tags for a resource.",
    ),
    (
        "PutClusterPolicy",
        "Requires identifier, policy; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Attaches a resource-based policy to a cluster.",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Tags a resource with a map of key and value pairs.",
    ),
    (
        "UntagResource",
        "Requires resourceArn, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes a tag from a resource.",
    ),
    (
        "UpdateCluster",
        "Requires identifier; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: The UpdateCluster API allows you to modify both single-Region and multi-Region cluster configurations.",
    ),
];
