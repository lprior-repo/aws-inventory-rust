use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "CreateHttpNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreatePrivateDnsNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "CreatePublicDnsNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("CreateService", Decision::Denied(DenyReason::Mutating)),
    ("DeleteNamespace", Decision::Denied(DenyReason::Mutating)),
    ("DeleteService", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteServiceAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeregisterInstance", Decision::Denied(DenyReason::Mutating)),
    ("DiscoverInstances", Decision::Denied(DenyReason::Content)),
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
    ("RegisterInstance", Decision::Denied(DenyReason::Mutating)),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateHttpNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdateInstanceCustomHealthStatus",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdatePrivateDnsNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "UpdatePublicDnsNamespace",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("UpdateService", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateServiceAttributes",
        Decision::Denied(DenyReason::Mutating),
    ),
];
