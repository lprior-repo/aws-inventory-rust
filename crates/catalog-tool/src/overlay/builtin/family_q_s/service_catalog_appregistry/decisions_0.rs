use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    (
        "AssociateAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("AssociateResource", Decision::Denied(DenyReason::Mutating)),
    ("CreateApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "CreateAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    ("DeleteApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "DeleteAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "DisassociateResource",
        Decision::Denied(DenyReason::Mutating),
    ),
    (
        "GetApplication",
        Decision::Deferred(
            "Required selectors application are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members id, arn, name, description, creationTime, lastUpdateTime, associatedResourceCount, tags, integrations, applicationTag. Pinned purpose: Retrieves metadata information about one of your applications. The application can be specified by its ARN, ID, or name (which is unique within one account in one region at a given point in time). Specify by ARN or ID in automated workflows if you wa",
        ),
    ),
    (
        "GetAssociatedResource",
        Decision::Deferred(
            "Required selectors application, resourceType, resource are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resource, options, applicationTagResult. Pinned purpose: Gets the resource associated with the application.",
        ),
    ),
    (
        "GetAttributeGroup",
        Decision::Deferred(
            "Required selectors attributeGroup are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members id, arn, name, description, attributes, creationTime, lastUpdateTime, tags, createdBy. Pinned purpose: Retrieves an attribute group by its ARN, ID, or name. The attribute group can be specified by its ARN, ID, or name.",
        ),
    ),
    (
        "GetConfiguration",
        Decision::Deferred(
            "The optional-input response does not establish an immutable locally scoped owned-resource identity. Response members configuration. Pinned request/ownership contract: Retrieves a TagKey configuration from an account.",
        ),
    ),
    (
        "ListAssociatedAttributeGroups",
        Decision::Deferred(
            "Required selectors application are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members attributeGroups, nextToken. Pinned purpose: Lists all attribute groups that are associated with specified application. Results are paginated.",
        ),
    ),
    (
        "ListAssociatedResources",
        Decision::Deferred(
            "Required selectors application are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members resources, nextToken. Pinned purpose: Lists all of the resources that are associated with the specified application. Results are paginated. If you share an application, and a consumer account associates a tag query to the application, all of the users who can access the application can a",
        ),
    ),
    (
        "ListAttributeGroupsForApplication",
        Decision::Deferred(
            "Required selectors application are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members attributeGroupsDetails, nextToken. Pinned purpose: Lists the details of all attribute groups associated with a specific application. The results display in pages.",
        ),
    ),
    (
        "ListTagsForResource",
        Decision::Deferred(
            "Tag-only observation must join the originating resource identity with complete-snapshot semantics, not be persisted as a separate resource. Required request members: resourceArn. Pinned purpose: Lists all of the tags on the resource.",
        ),
    ),
    ("PutConfiguration", Decision::Denied(DenyReason::Mutating)),
    (
        "SyncResource",
        Decision::Deferred(
            "Required selectors resourceType, resource are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members applicationArn, resourceArn, actionTaken. Pinned purpose: Syncs the resource with current AppRegistry records. Specifically, the resource’s AppRegistry system tags sync with its associated application. We remove the resource's AppRegistry system tags if it does not associate with the application. The caller",
        ),
    ),
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
    (
        "UpdateAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
];
