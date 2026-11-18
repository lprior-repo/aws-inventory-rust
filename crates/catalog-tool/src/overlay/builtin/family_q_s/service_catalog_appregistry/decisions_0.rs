// Pinned operation purposes: .local/aws-models/models/service-catalog-appregistry/service/2020-06-24/service-catalog-appregistry-2020-06-24.json
use super::super::construction::Decision;
use crate::classes::DenyReason;

pub(super) const DECISIONS: &[(&str, Decision)] = &[
    // Changes service state rather than reading inventory. Pinned purpose: Associates an attribute group with an application to augment the application's metadata with the group's attributes. This feature enables applications to be described with user-defined details that are machine-readable, such as third-party integrations.
    (
        "AssociateAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Associates a resource with an application. The resource can be specified by its ARN or name. The application can be specified by ARN, ID, or name. Minimum permissions You must have the following permissions to associate a resource using the OPTIONS parameter set to APPLY_APPLICATION_TAG . tag:GetResources tag:TagResources You must also have these additional permissions if you don't use the AWSServ
    ("AssociateResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new application that is the top-level node in a hierarchy of related cloud resource abstractions.
    ("CreateApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Creates a new attribute group as a container for user-defined attributes. This feature enables users to have full control over their cloud application's metadata in a rich machine-readable format to facilitate integration with automated workflows and third-party tools.
    (
        "CreateAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an application that is specified either by its application ID, name, or ARN. All associated attribute groups and resources must be disassociated from it before deleting an application.
    ("DeleteApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Deletes an attribute group, specified either by its attribute group ID, name, or ARN.
    (
        "DeleteAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates an attribute group from an application to remove the extra attributes contained in the attribute group from the application's metadata. This operation reverts AssociateAttributeGroup .
    (
        "DisassociateAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Disassociates a resource from application. Both the resource and the application can be specified either by ID or name. Minimum permissions You must have the following permissions to remove a resource that's been associated with an application using the APPLY_APPLICATION_TAG option for AssociateResource . tag:GetResources tag:UntagResources You must also have the following permissions if you don't
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
    // Changes service state rather than reading inventory. Pinned purpose: Associates a TagKey configuration to an account.
    ("PutConfiguration", Decision::Denied(DenyReason::Mutating)),
    (
        "SyncResource",
        Decision::Deferred(
            "Required selectors resourceType, resource are not caller account/Region context. Must bind the exact documented owning parent and preserve fan-out before enabling; no guessed identifier or partial filtered request is permitted. Response members applicationArn, resourceArn, actionTaken. Pinned purpose: Syncs the resource with current AppRegistry records. Specifically, the resource’s AppRegistry system tags sync with its associated application. We remove the resource's AppRegistry system tags if it does not associate with the application. The caller",
        ),
    ),
    // Changes service state rather than reading inventory. Pinned purpose: Assigns one or more tags (key-value pairs) to the specified resource. Each tag consists of a key and an optional value. If a tag with the same key is already associated with the resource, this action updates its value. This operation returns an empty response if the call was successful.
    ("TagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Removes tags from a resource. This operation returns an empty response if the call was successful.
    ("UntagResource", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing application with new attributes.
    ("UpdateApplication", Decision::Denied(DenyReason::Mutating)),
    // Changes service state rather than reading inventory. Pinned purpose: Updates an existing attribute group with new details.
    (
        "UpdateAttributeGroup",
        Decision::Denied(DenyReason::Mutating),
    ),
];
