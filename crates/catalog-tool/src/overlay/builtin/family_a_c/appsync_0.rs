use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateApi",
            deferred(
                "Model requires domainName, apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Maps an endpoint to your custom domain.",
            ),
        ),
        (
            "AssociateMergedGraphqlApi",
            deferred(
                "Model requires sourceApiIdentifier, mergedApiIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an association between a Merged API and source API using the source API's identifier.",
            ),
        ),
        (
            "AssociateSourceGraphqlApi",
            deferred(
                "Model requires mergedApiIdentifier, sourceApiIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an association between a Merged API and source API using the Merged API's identifier.",
            ),
        ),
        (
            "CreateApi",
            deferred(
                "Model requires name, eventConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Api object. Use this operation to create an AppSync API with your preferred configuration, such as an Event API that provides real-time message publishing and message subscriptions over WebSockets.",
            ),
        ),
        (
            "CreateApiCache",
            deferred(
                "Model requires apiId, ttl, apiCachingBehavior, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a cache for the GraphQL API.",
            ),
        ),
        (
            "CreateApiKey",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a unique key that you can distribute to clients who invoke your API.",
            ),
        ),
        (
            "CreateChannelNamespace",
            deferred(
                "Model requires apiId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a ChannelNamespace for an Api .",
            ),
        ),
        (
            "CreateDataSource",
            deferred(
                "Model requires apiId, name, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a DataSource object.",
            ),
        ),
        (
            "CreateDomainName",
            deferred(
                "Model requires domainName, certificateArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a custom DomainName object.",
            ),
        ),
        (
            "CreateFunction",
            deferred(
                "Model requires apiId, name, dataSourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Function object. A function is a reusable entity. You can use multiple functions to compose the resolver logic.",
            ),
        ),
        (
            "CreateGraphqlApi",
            deferred(
                "Model requires name, authenticationType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a GraphqlApi object.",
            ),
        ),
        (
            "CreateResolver",
            deferred(
                "Model requires apiId, typeName, fieldName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Resolver object. A resolver converts incoming requests into a format that a data source can understand, and converts the data source's responses into GraphQL.",
            ),
        ),
        (
            "CreateType",
            deferred(
                "Model requires apiId, definition, format. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Type object.",
            ),
        ),
        (
            "DeleteApi",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Api object",
            ),
        ),
        (
            "DeleteApiCache",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an ApiCache object.",
            ),
        ),
        (
            "DeleteApiKey",
            deferred(
                "Model requires apiId, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an API key.",
            ),
        ),
        (
            "DeleteChannelNamespace",
            deferred(
                "Model requires apiId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a ChannelNamespace .",
            ),
        ),
        (
            "DeleteDataSource",
            deferred(
                "Model requires apiId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a DataSource object.",
            ),
        ),
        (
            "DeleteDomainName",
            deferred(
                "Model requires domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a custom DomainName object.",
            ),
        ),
        (
            "DeleteFunction",
            deferred(
                "Model requires apiId, functionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Function .",
            ),
        ),
        (
            "DeleteGraphqlApi",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a GraphqlApi object.",
            ),
        ),
        (
            "DeleteResolver",
            deferred(
                "Model requires apiId, typeName, fieldName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Resolver object.",
            ),
        ),
        (
            "DeleteType",
            deferred(
                "Model requires apiId, typeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Type object.",
            ),
        ),
        (
            "DisassociateApi",
            deferred(
                "Model requires domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes an ApiAssociation object from a custom domain.",
            ),
        ),
        (
            "DisassociateMergedGraphqlApi",
            deferred(
                "Model requires sourceApiIdentifier, associationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an association between a Merged API and source API using the source API's identifier and the association ID.",
            ),
        ),
        (
            "DisassociateSourceGraphqlApi",
            deferred(
                "Model requires mergedApiIdentifier, associationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an association between a Merged API and source API using the Merged API's identifier and the association ID.",
            ),
        ),
        (
            "EvaluateCode",
            deferred(
                "Model requires runtime, code, context. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Evaluates the given code and returns the response. The code definition requirements depend on the specified runtime. For APPSYNC_JS runtimes, the code defines the request and response functions. The request function takes the…",
            ),
        ),
        (
            "EvaluateMappingTemplate",
            deferred(
                "Model requires template, context. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Evaluates a given template and returns the response. The mapping template can be a request or response template. Request templates take the incoming request after a GraphQL operation is parsed and convert it into a request…",
            ),
        ),
        (
            "FlushApiCache",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Flushes an ApiCache object.",
            ),
        ),
        (
            "GetApi",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("apiId")?,
                    Binding::From {
                        operation: name("ListApis")?,
                        path: path("apiId")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("apiArn")?],
                ..read(
                    "api",
                    Identity::Arn(path("apiArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetApiAssociation",
            deferred(
                "Model requires domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an ApiAssociation object.",
            ),
        ),
        (
            "GetApiCache",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves an ApiCache object.",
            ),
        ),
        (
            "GetChannelNamespace",
            deferred(
                "Model requires apiId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the channel namespace for a specified Api .",
            ),
        ),
        (
            "GetDataSource",
            deferred(
                "Model requires apiId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a DataSource object.",
            ),
        ),
        (
            "GetDataSourceIntrospection",
            deferred(
                "Model requires introspectionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the record of an existing introspection. If the retrieval is successful, the result of the instrospection will also be returned. If the retrieval fails the operation, an error message will be returned instead.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetDomainName",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("domainName")?,
                    Binding::From {
                        operation: name("ListDomainNames")?,
                        path: path("domainName")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("domainName")?],
                ..read(
                    "domainNameConfig",
                    Identity::Key(vec![path("domainName")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        ("GetFunction", denied(DenyReason::Content)),
        (
            "GetGraphqlApi",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("apiId")?,
                    Binding::From {
                        operation: name("ListGraphqlApis")?,
                        path: path("apiId")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                ..read(
                    "graphqlApi",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetGraphqlApiEnvironmentVariables",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the list of environmental variable key-value pairs associated with an API by its ID value.",
            ),
        ),
        (
            "GetIntrospectionSchema",
            deferred(
                "Model requires apiId, format. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the introspection schema for a GraphQL API.",
            ),
        ),
        (
            "GetResolver",
            deferred(
                "Model requires apiId, typeName, fieldName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a Resolver object.",
            ),
        ),
        (
            "GetSchemaCreationStatus",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the current status of a schema creation operation.",
            ),
        ),
        (
            "GetSourceApiAssociation",
            deferred(
                "Model requires mergedApiIdentifier, associationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a SourceApiAssociation object.",
            ),
        ),
        (
            "GetType",
            deferred(
                "Model requires apiId, typeName, format. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves a Type object.",
            ),
        ),
        (
            "ListApiKeys",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the API keys for a given API. API keys are deleted automatically 60 days after they expire. However, they may still be included in the response until they have actually been deleted. You can safely call DeleteApiKey to…",
            ),
        ),
        (
            "ListApis",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("apiArn")?],
                ..read(
                    "apis",
                    Identity::Arn(path("apiArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListChannelNamespaces",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("apiId")?,
                    Binding::From {
                        operation: name("ListApis")?,
                        path: path("apiId")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("channelNamespaceArn")?],
                ..read(
                    "channelNamespaces",
                    Identity::Arn(path("channelNamespaceArn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListDataSources",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the data sources for a given API.",
            ),
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListDomainNames",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("domainName")?],
                ..read(
                    "domainNameConfigs",
                    Identity::Key(vec![path("domainName")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListFunctions",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List multiple functions.",
            ),
        ),
        (
            "ListGraphqlApis",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("arn")?],
                fixed: [(
                    path("owner")?,
                    FixedValue::String("CURRENT_ACCOUNT".to_owned()),
                )]
                .into_iter()
                .collect(),
                ..read(
                    "graphqlApis",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListResolvers",
            deferred(
                "Model requires apiId, typeName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the resolvers for a given API and type.",
            ),
        ),
        (
            "ListResolversByFunction",
            deferred(
                "Model requires apiId, functionId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List the resolvers that are associated with a specific function.",
            ),
        ),
        (
            "ListSourceApiAssociations",
            deferred(
                "Model requires apiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the SourceApiAssociationSummary data.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags for a resource.",
            ),
        ),
        (
            "ListTypes",
            deferred(
                "Model requires apiId, format. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the types for a given API.",
            ),
        ),
        (
            "ListTypesByAssociation",
            deferred(
                "Model requires mergedApiIdentifier, associationId, format. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists Type objects by the source API association ID.",
            ),
        ),
        (
            "PutGraphqlApiEnvironmentVariables",
            deferred(
                "Model requires apiId, environmentVariables. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a list of environmental variables in an API by its ID value. When creating an environmental variable, it must follow the constraints below: Both JavaScript and VTL templates support environmental variables. Environmental…",
            ),
        ),
        (
            "StartDataSourceIntrospection",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is introspectionId, introspectionStatus, introspectionStatusDetail; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Creates a new introspection. Returns the introspectionId of the new introspection after its creation.",
            ),
        ),
        (
            "StartSchemaCreation",
            deferred(
                "Model requires apiId, definition. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a new schema to your GraphQL API. This operation is asynchronous. Use to determine when it has completed.",
            ),
        ),
        (
            "StartSchemaMerge",
            deferred(
                "Model requires associationId, mergedApiIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a merge operation. Returns a status that shows the result of the merge operation.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Tags a resource with user-supplied tags.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Untags a resource.",
            ),
        ),
        (
            "UpdateApi",
            deferred(
                "Model requires apiId, name, eventConfig. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an Api .",
            ),
        ),
        (
            "UpdateApiCache",
            deferred(
                "Model requires apiId, ttl, apiCachingBehavior, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the cache for the GraphQL API.",
            ),
        ),
        (
            "UpdateApiKey",
            deferred(
                "Model requires apiId, id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an API key. You can update the key as long as it's not deleted.",
            ),
        ),
        (
            "UpdateChannelNamespace",
            deferred(
                "Model requires apiId, name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a ChannelNamespace associated with an Api .",
            ),
        ),
        (
            "UpdateDataSource",
            deferred(
                "Model requires apiId, name, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a DataSource object.",
            ),
        ),
        (
            "UpdateDomainName",
            deferred(
                "Model requires domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a custom DomainName object.",
            ),
        ),
        (
            "UpdateFunction",
            deferred(
                "Model requires apiId, name, functionId, dataSourceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Function object.",
            ),
        ),
        (
            "UpdateGraphqlApi",
            deferred(
                "Model requires apiId, name, authenticationType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a GraphqlApi object.",
            ),
        ),
        (
            "UpdateResolver",
            deferred(
                "Model requires apiId, typeName, fieldName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Resolver object.",
            ),
        ),
        (
            "UpdateSourceApiAssociation",
            deferred(
                "Model requires associationId, mergedApiIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates some of the configuration choices of a particular source API association.",
            ),
        ),
        (
            "UpdateType",
            deferred(
                "Model requires apiId, typeName, format. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Type object.",
            ),
        ),
    ])
}
