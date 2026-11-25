pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "CreateCluster",
        "Requires ClusterName, NodeType, ReplicationFactor, IamRoleArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a DAX cluster.",
    ),
    (
        "CreateParameterGroup",
        "Requires ParameterGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new parameter group.",
    ),
    (
        "CreateSubnetGroup",
        "Requires SubnetGroupName, SubnetIds; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a new subnet group.",
    ),
    (
        "DecreaseReplicationFactor",
        "Requires ClusterName, NewReplicationFactor; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes one or more nodes from a DAX cluster.",
    ),
    (
        "DeleteCluster",
        "Requires ClusterName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a previously provisioned DAX cluster.",
    ),
    (
        "DeleteParameterGroup",
        "Requires ParameterGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes the specified parameter group.",
    ),
    (
        "DeleteSubnetGroup",
        "Requires SubnetGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a subnet group.",
    ),
    (
        "DescribeDefaultParameters",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns the default system parameter information for the DAX caching software.",
    ),
    (
        "DescribeEvents",
        "Not a reviewed resource-enumeration policy; the operation remains noncallable rather than implying complete collection. Pinned operation: Returns events related to DAX clusters and parameter groups.",
    ),
    (
        "DescribeParameters",
        "Parameters lacks ParameterGroupName; parameter names repeat across groups. Requires parent-input identity, not name-only identity. Pinned operation: Returns the detailed parameter list for a particular parameter group.",
    ),
    (
        "IncreaseReplicationFactor",
        "Requires ClusterName, NewReplicationFactor; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Adds one or more nodes to a DAX cluster.",
    ),
    (
        "ListTags",
        "Requires ResourceName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: List all of the tags for a DAX cluster.",
    ),
    (
        "RebootNode",
        "Requires ClusterName, NodeId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Reboots a single node of a DAX cluster.",
    ),
    (
        "TagResource",
        "Requires ResourceName, Tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Associates a set of tags with a DAX resource.",
    ),
    (
        "UntagResource",
        "Requires ResourceName, TagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes the association of tags from a DAX resource.",
    ),
    (
        "UpdateCluster",
        "Requires ClusterName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the settings for a DAX cluster.",
    ),
    (
        "UpdateParameterGroup",
        "Requires ParameterGroupName, ParameterNameValues; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies the parameters of a parameter group.",
    ),
    (
        "UpdateSubnetGroup",
        "Requires SubnetGroupName; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Modifies an existing subnet group.",
    ),
];
