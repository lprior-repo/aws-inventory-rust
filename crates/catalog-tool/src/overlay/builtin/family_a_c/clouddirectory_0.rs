use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AddFacetToObject",
            deferred(
                "Model requires DirectoryArn, SchemaFacet, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a new Facet to an object. An object can have more than one facet applied on it.",
            ),
        ),
        (
            "ApplySchema",
            deferred(
                "Model requires PublishedSchemaArn, DirectoryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Copies the input published schema, at the specified version, into the Directory with the same name and version as that of the published schema.",
            ),
        ),
        (
            "AttachObject",
            deferred(
                "Model requires DirectoryArn, ParentReference, ChildReference, LinkName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Attaches an existing object to another object. An object can be accessed in two ways: Using the path Using ObjectIdentifier",
            ),
        ),
        (
            "AttachPolicy",
            deferred(
                "Model requires DirectoryArn, PolicyReference, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Attaches a policy object to a regular object. An object can have a limited number of attached policies.",
            ),
        ),
        (
            "AttachToIndex",
            deferred(
                "Model requires DirectoryArn, IndexReference, TargetReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Attaches the specified object to the specified index.",
            ),
        ),
        (
            "AttachTypedLink",
            deferred(
                "Model requires DirectoryArn, SourceObjectReference, TargetObjectReference, TypedLinkFacet, Attributes. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Attaches a typed link to a specified source and target object. For more information, see Typed Links .",
            ),
        ),
        (
            "BatchRead",
            deferred(
                "Model requires DirectoryArn, Operations. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Performs all the read operations in a batch.",
            ),
        ),
        (
            "BatchWrite",
            deferred(
                "Model requires DirectoryArn, Operations. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Performs all the write operations in a batch. Either all the operations succeed or none.",
            ),
        ),
        (
            "CreateDirectory",
            deferred(
                "Model requires Name, SchemaArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Directory by copying the published schema into the directory. A directory cannot be created without a schema. You can also quickly create a directory using a managed schema, called the QuickStartSchema . For more…",
            ),
        ),
        (
            "CreateFacet",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Facet in a schema. Facet creation is allowed only in development or applied schemas.",
            ),
        ),
        (
            "CreateIndex",
            deferred(
                "Model requires DirectoryArn, OrderedIndexedAttributeList, IsUnique. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an index object. See Indexing and search for more information.",
            ),
        ),
        (
            "CreateObject",
            deferred(
                "Model requires DirectoryArn, SchemaFacets. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an object in a Directory . Additionally attaches the object to a parent, if a parent reference and LinkName is specified. An object is simply a collection of Facet attributes. You can also use this API call to create a…",
            ),
        ),
        (
            "CreateSchema",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new schema in a development state. A schema can exist in three phases: Development: This is a mutable phase of the schema. All new schemas are in the development phase. Once the schema is finalized, it can be published.…",
            ),
        ),
        (
            "CreateTypedLinkFacet",
            deferred(
                "Model requires SchemaArn, Facet. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a TypedLinkFacet . For more information, see Typed Links .",
            ),
        ),
        (
            "DeleteDirectory",
            deferred(
                "Model requires DirectoryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a directory. Only disabled directories can be deleted. A deleted directory cannot be undone. Exercise extreme caution when deleting directories.",
            ),
        ),
        (
            "DeleteFacet",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a given Facet . All attributes and Rule s that are associated with the facet will be deleted. Only development schema facets are allowed deletion.",
            ),
        ),
        (
            "DeleteObject",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an object and its associated attributes. Only objects with no children and no parents can be deleted. The maximum number of attributes that can be deleted during an object deletion is 30. For more information, see Amazon…",
            ),
        ),
        (
            "DeleteSchema",
            deferred(
                "Model requires SchemaArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a given schema. Schemas in a development and published state can only be deleted.",
            ),
        ),
        (
            "DeleteTypedLinkFacet",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a TypedLinkFacet . For more information, see Typed Links .",
            ),
        ),
        (
            "DetachFromIndex",
            deferred(
                "Model requires DirectoryArn, IndexReference, TargetReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Detaches the specified object from the specified index.",
            ),
        ),
        (
            "DetachObject",
            deferred(
                "Model requires DirectoryArn, ParentReference, LinkName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Detaches a given object from the parent object. The object that is to be detached from the parent is specified by the link name.",
            ),
        ),
        (
            "DetachPolicy",
            deferred(
                "Model requires DirectoryArn, PolicyReference, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Detaches a policy from an object.",
            ),
        ),
        (
            "DetachTypedLink",
            deferred(
                "Model requires DirectoryArn, TypedLinkSpecifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Detaches a typed link from a specified source and target object. For more information, see Typed Links .",
            ),
        ),
        (
            "DisableDirectory",
            deferred(
                "Model requires DirectoryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disables the specified directory. Disabled directories cannot be read or written to. Only enabled directories can be disabled. Disabled directories may be reenabled.",
            ),
        ),
        (
            "EnableDirectory",
            deferred(
                "Model requires DirectoryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Enables the specified directory. Only disabled directories can be enabled. Once enabled, the directory can then be read and written to.",
            ),
        ),
        (
            "GetAppliedSchemaVersion",
            deferred(
                "Model requires SchemaArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns current applied schema version ARN, including the minor version in use.",
            ),
        ),
        (
            "GetDirectory",
            deferred(
                "Model requires DirectoryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves metadata about a directory.",
            ),
        ),
        (
            "GetFacet",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets details of the Facet , such as facet name, attributes, Rule s, or ObjectType . You can call this on all kinds of schema facets -- published, development, or applied.",
            ),
        ),
        (
            "GetLinkAttributes",
            deferred(
                "Model requires DirectoryArn, TypedLinkSpecifier, AttributeNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves attributes that are associated with a typed link.",
            ),
        ),
        (
            "GetObjectAttributes",
            deferred(
                "Model requires DirectoryArn, ObjectReference, SchemaFacet, AttributeNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves attributes within a facet that are associated with an object.",
            ),
        ),
        (
            "GetObjectInformation",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves metadata about an object.",
            ),
        ),
        (
            "GetSchemaAsJson",
            deferred(
                "Model requires SchemaArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a JSON representation of the schema. See JSON Schema Format for more information.",
            ),
        ),
        (
            "GetTypedLinkFacetInformation",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the identity attribute order for a specific TypedLinkFacet . For more information, see Typed Links .",
            ),
        ),
        (
            "ListAppliedSchemaArns",
            deferred(
                "Model requires DirectoryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists schema major versions applied to a directory. If SchemaArn is provided, lists the minor version.",
            ),
        ),
        (
            "ListAttachedIndices",
            deferred(
                "Model requires DirectoryArn, TargetReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists indices attached to the specified object.",
            ),
        ),
        (
            "ListDevelopmentSchemaArns",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "SchemaArns",
                    Identity::Arn(path("$")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListDirectories",
            OperationOverlay {
                queryable: vec![path("DirectoryArn")?],
                ..read(
                    "Directories",
                    Identity::Arn(path("DirectoryArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListFacetAttributes",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves attributes attached to the facet.",
            ),
        ),
        (
            "ListFacetNames",
            deferred(
                "Model requires SchemaArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the names of facets that exist in a schema.",
            ),
        ),
        (
            "ListIncomingTypedLinks",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a paginated list of all the incoming TypedLinkSpecifier information for an object. It also supports filtering by typed link facet and identity attributes. For more information, see Typed Links .",
            ),
        ),
        (
            "ListIndex",
            deferred(
                "Model requires DirectoryArn, IndexReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists objects attached to the specified index.",
            ),
        ),
        (
            "ListManagedSchemaArns",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is SchemaArns; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the major version families of each managed schema. If a major version ARN is provided as SchemaArn, the minor version revisions in that family are listed instead.",
            ),
        ),
        ("ListObjectAttributes", denied(DenyReason::Content)),
        (
            "ListObjectChildren",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a paginated list of child objects that are associated with a given object.",
            ),
        ),
        (
            "ListObjectParentPaths",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves all available parent paths for any object type such as node, leaf node, policy node, and index node objects. For more information about objects, see Directory Structure . Use this API to evaluate all parents for an…",
            ),
        ),
        (
            "ListObjectParents",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists parent objects that are associated with a given object in pagination fashion.",
            ),
        ),
        (
            "ListObjectPolicies",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns policies attached to an object in pagination fashion.",
            ),
        ),
        (
            "ListOutgoingTypedLinks",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a paginated list of all the outgoing TypedLinkSpecifier information for an object. It also supports filtering by typed link facet and identity attributes. For more information, see Typed Links .",
            ),
        ),
        (
            "ListPolicyAttachments",
            deferred(
                "Model requires DirectoryArn, PolicyReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns all of the ObjectIdentifiers to which a given policy is attached.",
            ),
        ),
        (
            "ListPublishedSchemaArns",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "SchemaArns",
                    Identity::Arn(path("$")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns tags for a resource. Tagging is currently supported only for directories with a limit of 50 tags per directory. All 50 tags are returned for a given directory with this API call.",
            ),
        ),
        (
            "ListTypedLinkFacetAttributes",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a paginated list of all attribute definitions for a particular TypedLinkFacet . For more information, see Typed Links .",
            ),
        ),
        (
            "ListTypedLinkFacetNames",
            deferred(
                "Model requires SchemaArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a paginated list of TypedLink facet names for a particular schema. For more information, see Typed Links .",
            ),
        ),
        (
            "LookupPolicy",
            deferred(
                "Model requires DirectoryArn, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all policies from the root of the Directory to the object specified. If there are no policies present, an empty list is returned. If policies are present, and if some objects don't have the policies attached, it returns the…",
            ),
        ),
        (
            "PublishSchema",
            deferred(
                "Model requires DevelopmentSchemaArn, Version. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Publishes a development schema with a major version and a recommended minor version.",
            ),
        ),
        (
            "PutSchemaFromJson",
            deferred(
                "Model requires SchemaArn, Document. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Allows a schema to be updated using JSON upload. Only available for development schemas. See JSON Schema Format for more information.",
            ),
        ),
        (
            "RemoveFacetFromObject",
            deferred(
                "Model requires DirectoryArn, SchemaFacet, ObjectReference. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified facet from the specified object.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: An API operation for adding tags to a resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: An API operation for removing tags from a resource.",
            ),
        ),
        (
            "UpdateFacet",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Does the following: Adds new Attributes , Rules , or ObjectTypes . Updates existing Attributes , Rules , or ObjectTypes . Deletes existing Attributes , Rules , or ObjectTypes .",
            ),
        ),
        (
            "UpdateLinkAttributes",
            deferred(
                "Model requires DirectoryArn, TypedLinkSpecifier, AttributeUpdates. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a given typed link’s attributes. Attributes to be updated must not contribute to the typed link’s identity, as defined by its IdentityAttributeOrder .",
            ),
        ),
        (
            "UpdateObjectAttributes",
            deferred(
                "Model requires DirectoryArn, ObjectReference, AttributeUpdates. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a given object's attributes.",
            ),
        ),
        (
            "UpdateSchema",
            deferred(
                "Model requires SchemaArn, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the schema name with a new name. Only development schema names can be updated.",
            ),
        ),
        (
            "UpdateTypedLinkFacet",
            deferred(
                "Model requires SchemaArn, Name, AttributeUpdates, IdentityAttributeOrder. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a TypedLinkFacet . For more information, see Typed Links .",
            ),
        ),
        (
            "UpgradeAppliedSchema",
            deferred(
                "Model requires PublishedSchemaArn, DirectoryArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Upgrades a single directory in-place using the PublishedSchemaArn with schema updates found in MinorVersion . Backwards-compatible minor version upgrades are instantaneously available for readers on all objects in the directory.…",
            ),
        ),
        (
            "UpgradePublishedSchema",
            deferred(
                "Model requires DevelopmentSchemaArn, PublishedSchemaArn, MinorVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Upgrades a published schema under a new minor version revision using the current contents of DevelopmentSchemaArn .",
            ),
        ),
    ])
}
