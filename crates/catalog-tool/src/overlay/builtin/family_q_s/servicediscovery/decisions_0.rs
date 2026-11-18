// Pinned operation purposes: .local/aws-models/models/servicediscovery/service/2017-03-14/servicediscovery-2017-03-14.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Creates an HTTP namespace. Service instances registered using an HTTP namespace can be discovered using a DiscoverInstances request but can't be discovered using DNS. For the current quota on the number of namespaces that you can create using the same Amazon Web Services account, see Cloud Map quotas in the Cloud Map Developer Guide .
    (
        "CreateHttpNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a private namespace based on DNS, which is visible only inside a specified Amazon VPC. The namespace defines your service naming scheme. For example, if you name your namespace example.com and name your service backend , the resulting DNS name for the service is backend.example.com . Service instances that are registered using a private DNS namespace can be discovered using either a Discov
    (
        "CreatePrivateDnsNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a public namespace based on DNS, which is visible on the internet. The namespace defines your service naming scheme. For example, if you name your namespace example.com and name your service backend , the resulting DNS name for the service is backend.example.com . You can discover instances that were registered with a public DNS namespace by using either a DiscoverInstances request or usin
    (
        "CreatePublicDnsNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a service. This action defines the configuration for the following entities: For public and private DNS namespaces, one of the following combinations of DNS records in Amazon Route 53: A AAAA A and AAAA SRV CNAME Optionally, a health check After you create the service, you can submit a RegisterInstance request, and Cloud Map uses the values in the configuration to create the specified enti
    ("CreateService", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a namespace from the current account. If the namespace still contains one or more services, the request fails.
    ("DeleteNamespace", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes a specified service and all associated service attributes. If the service still contains one or more registered instances, the request fails.
    ("DeleteService", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes specific attributes associated with a service.
    (
        "DeleteServiceAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes the Amazon Route 53 DNS records and health check, if any, that Cloud Map created for the specified instance.
    ("DeregisterInstance", Decision::Denied(DenyReason::Mutating)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Discovers registered instances for a specified namespace and service. You can use DiscoverInstances to discover instances for any type of namespace. DiscoverInstances returns a randomized list of instances allowing customers to distribute traffic evenly across instances. For public and private DNS namespaces, you can also use DNS queries to discover instances.
    ("DiscoverInstances", Decision::Denied(DenyReason::Content)),
    // Returns customer content/data-plane records, not configuration inventory. Pinned purpose: Discovers the increasing revision associated with an instance.
    (
        "DiscoverInstancesRevision",
        Decision::Denied(DenyReason::Content),
    ),
    (
        "GetInstance",
        Decision::Deferred(
            "Required selectors ServiceId, InstanceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceOwner, Instance. Pinned purpose: Gets information about a specified instance.",
        ),
    ),
    (
        "GetInstancesHealthStatus",
        Decision::Deferred(
            "Required selectors ServiceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Status, NextToken. Pinned purpose: Gets the current health status ( Healthy , Unhealthy , or Unknown ) of one or more instances that are associated with a specified service. There's a brief delay between when you register an instance and when the health status for the instance is avai",
        ),
    ),
    (
        "GetNamespace",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Namespace. Pinned purpose: Gets information about a namespace.",
        ),
    ),
    (
        "GetOperation",
        Decision::Deferred(
            "Required selectors OperationId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Operation. Pinned purpose: Gets information about any operation that returns an operation ID in the response, such as a CreateHttpNamespace request. To get a list of operations that match specified criteria, see ListOperations .",
        ),
    ),
    (
        "GetService",
        Decision::Deferred(
            "Required selectors Id are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members Service. Pinned purpose: Gets the settings for a specified service.",
        ),
    ),
    (
        "GetServiceAttributes",
        Decision::Deferred(
            "Required selectors ServiceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ServiceAttributes. Pinned purpose: Returns the attributes associated with a specified service.",
        ),
    ),
    (
        "ListInstances",
        Decision::Deferred(
            "Required selectors ServiceId are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members ResourceOwner, Instances, NextToken. Pinned purpose: Lists summary information about the instances that you registered by using a specified service.",
        ),
    ),
    (
        "ListNamespaces",
        Decision::Deferred(
            "Namespaces have Type HTTP, DNS_PUBLIC or DNS_PRIVATE, corresponding to distinct canonical CFN namespace kinds. A single static per-operation kind would misclassify at least two variants; selecting only one type would silently omit namespace inventory. Requires per-item kind dispatch or finite typed request union.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: ResourceARN. Pinned purpose: Lists tags for the specified resource.",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Creates or updates one or more records and, optionally, creates a health check based on the settings in a specified service. When you submit a RegisterInstance request, the following occurs: For each DNS record that you define in the service that's specified by ServiceId , a record is created or updated in the hosted zone that's associated with the corresponding namespace. If the service includes
    ("RegisterInstance", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Adds one or more tags to the specified resource.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes one or more tags from the specified resource.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an HTTP namespace.
    (
        "UpdateHttpNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Submits a request to change the health status of a custom health check to healthy or unhealthy. You can use UpdateInstanceCustomHealthStatus to change the status only for custom health checks, which you define using HealthCheckCustomConfig when you create a service. You can't use it to change the status for Route 53 health checks, which you define using HealthCheckConfig . For more information, se
    (
        "UpdateInstanceCustomHealthStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a private DNS namespace.
    (
        "UpdatePrivateDnsNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Updates a public DNS namespace.
    (
        "UpdatePublicDnsNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Submits a request to perform the following operations: Update the TTL setting for existing DnsRecords configurations Add, update, or delete HealthCheckConfig for a specified service You can't add, update, or delete a HealthCheckCustomConfig configuration. For public and private DNS namespaces, note the following: If you omit any existing DnsRecords or HealthCheckConfig configurations from an Updat
    ("UpdateService", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Submits a request to update a specified service to add service-level attributes.
    (
        "UpdateServiceAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
];
