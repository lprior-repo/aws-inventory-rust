// Explicit dispositions from .local/aws-models/models/emr-containers/service/2020-10-01/emr-containers-2020-10-01.json; never a runtime read-name approval rule.
pub(super) const ENTRIES: &[(&str, &str)] = &[
    (
        "CancelJobRun",
        "Requires id, virtualClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Cancels a job run.",
    ),
    (
        "CreateJobTemplate",
        "Requires name, clientToken, jobTemplateData; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a job template.",
    ),
    (
        "CreateManagedEndpoint",
        "Requires name, virtualClusterId, type, releaseLabel, executionRoleArn, clientToken; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a managed endpoint.",
    ),
    (
        "CreateSecurityConfiguration",
        "Requires clientToken, name, securityConfigurationData; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a security configuration.",
    ),
    (
        "CreateVirtualCluster",
        "Requires name, containerProvider, clientToken; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Creates a virtual cluster.",
    ),
    (
        "DeleteJobTemplate",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a job template.",
    ),
    (
        "DeleteManagedEndpoint",
        "Requires id, virtualClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a managed endpoint.",
    ),
    (
        "DeleteSecurityConfiguration",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a security configuration.",
    ),
    (
        "DeleteVirtualCluster",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Deletes a virtual cluster.",
    ),
    (
        "DescribeJobRun",
        "Requires id, virtualClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Displays detailed information about a job run.",
    ),
    (
        "DescribeJobTemplate",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Displays detailed information about a specified job template.",
    ),
    (
        "DescribeManagedEndpoint",
        "Requires id, virtualClusterId; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Displays detailed information about a managed endpoint.",
    ),
    (
        "DescribeSecurityConfiguration",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Displays detailed information about a specified security configuration.",
    ),
    (
        "DescribeVirtualCluster",
        "Requires id; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Displays detailed information about a specified virtual cluster.",
    ),
    (
        "GetManagedEndpointSessionCredentials",
        "Requires endpointIdentifier, virtualClusterIdentifier, executionRoleArn, credentialType; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Generate a session token to connect to a managed endpoint.",
    ),
    (
        "ListTagsForResource",
        "Requires resourceArn; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Lists the tags assigned to the resources.",
    ),
    (
        "StartJobRun",
        "Requires virtualClusterId, clientToken; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Starts a job run.",
    ),
    (
        "TagResource",
        "Requires resourceArn, tags; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Assigns tags to resources.",
    ),
    (
        "UntagResource",
        "Requires resourceArn, tagKeys; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Removes tags from resources.",
    ),
    (
        "UpdateVirtualCluster",
        "Requires id, clientToken; no reviewed complete parent binding and selected-item resource identity is authorized for this operation. Pinned operation: Updates a virtual cluster.",
    ),
];
