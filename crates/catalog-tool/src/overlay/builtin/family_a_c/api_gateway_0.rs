// Pinned source: .local/aws-models/models/api-gateway/service/2015-07-09/api-gateway-2015-07-09.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;
use crate::overlay::IdentityField;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateApiKey",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is id, value, name, customerId, description, enabled, createdDate, lastUpdatedDate, stageKeys, tags; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Create an ApiKey resource.",
            ),
        ),
        (
            "CreateAuthorizer",
            deferred(
                "Model requires restApiId, name, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a new Authorizer resource to an existing RestApi resource.",
            ),
        ),
        (
            "CreateBasePathMapping",
            deferred(
                "Model requires domainName, restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new BasePathMapping resource.",
            ),
        ),
        (
            "CreateDeployment",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Deployment resource, which makes a specified RestApi callable over the internet.",
            ),
        ),
        (
            "CreateDocumentationPart",
            deferred(
                "Model requires restApiId, location, properties. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a documentation part.",
            ),
        ),
        (
            "CreateDocumentationVersion",
            deferred(
                "Model requires restApiId, documentationVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a documentation version",
            ),
        ),
        (
            "CreateDomainName",
            deferred(
                "Model requires domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new domain name.",
            ),
        ),
        (
            "CreateDomainNameAccessAssociation",
            deferred(
                "Model requires domainNameArn, accessAssociationSourceType, accessAssociationSource. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a domain name access association resource between an access association source and a private custom domain name.",
            ),
        ),
        (
            "CreateModel",
            deferred(
                "Model requires restApiId, name, contentType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a new Model resource to an existing RestApi resource.",
            ),
        ),
        (
            "CreateRequestValidator",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a RequestValidator of a given RestApi.",
            ),
        ),
        (
            "CreateResource",
            deferred(
                "Model requires restApiId, parentId, pathPart. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Resource resource.",
            ),
        ),
        (
            "CreateRestApi",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new RestApi resource.",
            ),
        ),
        (
            "CreateStage",
            deferred(
                "Model requires restApiId, stageName, deploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Stage resource that references a pre-existing Deployment for the API.",
            ),
        ),
        (
            "CreateUsagePlan",
            deferred(
                "Model requires name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a usage plan with the throttle and quota limits, as well as the associated API stages, specified in the payload.",
            ),
        ),
        (
            "CreateUsagePlanKey",
            deferred(
                "Model requires usagePlanId, keyId, keyType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a usage plan key for adding an existing API key to a usage plan.",
            ),
        ),
        (
            "CreateVpcLink",
            deferred(
                "Model requires name, targetArns. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a VPC link, under the caller's account in a selected region, in an asynchronous operation that typically takes 2-4 minutes to complete and become operational. The caller must have permissions to create and update VPC…",
            ),
        ),
        (
            "DeleteApiKey",
            deferred(
                "Model requires apiKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the ApiKey resource.",
            ),
        ),
        (
            "DeleteAuthorizer",
            deferred(
                "Model requires restApiId, authorizerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing Authorizer resource.",
            ),
        ),
        (
            "DeleteBasePathMapping",
            deferred(
                "Model requires domainName, basePath. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the BasePathMapping resource.",
            ),
        ),
        (
            "DeleteClientCertificate",
            deferred(
                "Model requires clientCertificateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the ClientCertificate resource.",
            ),
        ),
        (
            "DeleteDeployment",
            deferred(
                "Model requires restApiId, deploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Deployment resource. Deleting a deployment will only succeed if there are no Stage resources associated with it.",
            ),
        ),
        (
            "DeleteDocumentationPart",
            deferred(
                "Model requires restApiId, documentationPartId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a documentation part",
            ),
        ),
        (
            "DeleteDocumentationVersion",
            deferred(
                "Model requires restApiId, documentationVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a documentation version.",
            ),
        ),
        (
            "DeleteDomainName",
            deferred(
                "Model requires domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the DomainName resource.",
            ),
        ),
        (
            "DeleteDomainNameAccessAssociation",
            deferred(
                "Model requires domainNameAccessAssociationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the DomainNameAccessAssociation resource. Only the AWS account that created the DomainNameAccessAssociation resource can delete it. To stop an access association source in another AWS account from accessing your private…",
            ),
        ),
        (
            "DeleteGatewayResponse",
            deferred(
                "Model requires restApiId, responseType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Clears any customization of a GatewayResponse of a specified response type on the given RestApi and resets it with the default settings.",
            ),
        ),
        (
            "DeleteIntegration",
            deferred(
                "Model requires restApiId, resourceId, httpMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents a delete integration.",
            ),
        ),
        (
            "DeleteIntegrationResponse",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, statusCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents a delete integration response.",
            ),
        ),
        (
            "DeleteMethod",
            deferred(
                "Model requires restApiId, resourceId, httpMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing Method resource.",
            ),
        ),
        (
            "DeleteMethodResponse",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, statusCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing MethodResponse resource.",
            ),
        ),
        (
            "DeleteModel",
            deferred(
                "Model requires restApiId, modelName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a model.",
            ),
        ),
        (
            "DeleteRequestValidator",
            deferred(
                "Model requires restApiId, requestValidatorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a RequestValidator of a given RestApi.",
            ),
        ),
        (
            "DeleteResource",
            deferred(
                "Model requires restApiId, resourceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Resource resource.",
            ),
        ),
        (
            "DeleteRestApi",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified API.",
            ),
        ),
        (
            "DeleteStage",
            deferred(
                "Model requires restApiId, stageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Stage resource.",
            ),
        ),
        (
            "DeleteUsagePlan",
            deferred(
                "Model requires usagePlanId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a usage plan of a given plan Id.",
            ),
        ),
        (
            "DeleteUsagePlanKey",
            deferred(
                "Model requires usagePlanId, keyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a usage plan key and remove the underlying API key from the associated usage plan.",
            ),
        ),
        (
            "DeleteVpcLink",
            deferred(
                "Model requires vpcLinkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an existing VpcLink of a specified identifier.",
            ),
        ),
        (
            "FlushStageAuthorizersCache",
            deferred(
                "Model requires restApiId, stageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Flushes all authorizer cache entries on a stage.",
            ),
        ),
        (
            "FlushStageCache",
            deferred(
                "Model requires restApiId, stageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Flushes a stage's cache.",
            ),
        ),
        (
            "GenerateClientCertificate",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is clientCertificateId, description, pemEncodedCertificate, createdDate, expirationDate, tags; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Generates a ClientCertificate resource.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Gets information about the current Account resource.
        (
            "GetAccount",
            OperationOverlay {
                ..read(
                    "$",
                    Identity::Template(
                        "apigateway-account:{partition}:{account}:{region}".to_owned(),
                    ),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetApiKey",
            deferred(
                "Model requires apiKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about the current ApiKey resource.",
            ),
        ),
        // Model: Gets information about the current ApiKeys resource.
        (
            "GetApiKeys",
            OperationOverlay {
                fixed: [(path("includeValues")?, FixedValue::Boolean(false))]
                    .into_iter()
                    .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("id")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        (
            "GetAuthorizer",
            deferred(
                "Model requires restApiId, authorizerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describe an existing Authorizer resource.",
            ),
        ),
        // Credentials are IAM-role references; this summary never contains an authorizer secret.
        (
            "GetAuthorizers",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("restApiId")?,
                    Binding::From {
                        operation: name("GetRestApis")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                ..read(
                    "items",
                    Identity::CorrelatedKey(vec![
                        IdentityField::Request(path("restApiId")?),
                        IdentityField::Selected(path("id")?),
                    ]),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        (
            "GetBasePathMapping",
            deferred(
                "Model requires domainName, basePath. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describe a BasePathMapping resource.",
            ),
        ),
        (
            "GetBasePathMappings",
            deferred(
                "Model requires domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents a collection of BasePathMapping resources.",
            ),
        ),
        (
            "GetClientCertificate",
            deferred(
                "Model requires clientCertificateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about the current ClientCertificate resource.",
            ),
        ),
        // Model: Gets a collection of ClientCertificate resources.
        (
            "GetClientCertificates",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("clientCertificateId")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("clientCertificateId")?]),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        (
            "GetDeployment",
            deferred(
                "Model requires restApiId, deploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a Deployment resource.",
            ),
        ),
        (
            "GetDeployments",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a Deployments collection.",
            ),
        ),
        (
            "GetDocumentationPart",
            deferred(
                "Model requires restApiId, documentationPartId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a documentation part.",
            ),
        ),
        (
            "GetDocumentationParts",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets documentation parts.",
            ),
        ),
        (
            "GetDocumentationVersion",
            deferred(
                "Model requires restApiId, documentationVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a documentation version.",
            ),
        ),
        (
            "GetDocumentationVersions",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets documentation versions.",
            ),
        ),
        // Model: Represents a domain name that is contained in a simpler, more intuitive URL that can be called.
        (
            "GetDomainName",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("domainName")?,
                    Binding::From {
                        operation: name("GetDomainNames")?,
                        path: path("domainName")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("domainNameArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("domainNameArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Represents a collection on DomainNameAccessAssociations resources.
        (
            "GetDomainNameAccessAssociations",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("domainNameAccessAssociationArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("domainNameAccessAssociationArn")?),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        // Model: Represents a collection of DomainName resources.
        (
            "GetDomainNames",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("domainNameArn")?],
                ..read(
                    "items",
                    Identity::Arn(path("domainNameArn")?),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        (
            "GetExport",
            deferred(
                "Model requires restApiId, stageName, exportType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Exports a deployed version of a RestApi in a specified format.",
            ),
        ),
        (
            "GetGatewayResponse",
            deferred(
                "Model requires restApiId, responseType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a GatewayResponse of a specified response type on the given RestApi.",
            ),
        ),
        (
            "GetGatewayResponses",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the GatewayResponses collection on the given RestApi. If an API developer has not added any definitions for gateway responses, the result will be the API Gateway-generated default GatewayResponses collection for the…",
            ),
        ),
        (
            "GetIntegration",
            deferred(
                "Model requires restApiId, resourceId, httpMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the integration settings.",
            ),
        ),
        (
            "GetIntegrationResponse",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, statusCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents a get integration response.",
            ),
        ),
        (
            "GetMethod",
            deferred(
                "Model requires restApiId, resourceId, httpMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describe an existing Method resource.",
            ),
        ),
        (
            "GetMethodResponse",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, statusCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes a MethodResponse resource.",
            ),
        ),
        (
            "GetModel",
            deferred(
                "Model requires restApiId, modelName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes an existing model defined for a RestApi resource.",
            ),
        ),
        (
            "GetModelTemplate",
            deferred(
                "Model requires restApiId, modelName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Generates a sample mapping template that can be used to transform a payload into the structure of a model.",
            ),
        ),
        (
            "GetModels",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Describes existing Models defined for a RestApi resource.",
            ),
        ),
        (
            "GetRequestValidator",
            deferred(
                "Model requires restApiId, requestValidatorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a RequestValidator of a given RestApi.",
            ),
        ),
        (
            "GetRequestValidators",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("restApiId")?,
                    Binding::From {
                        operation: name("GetRestApis")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                ..read(
                    "items",
                    Identity::CorrelatedKey(vec![
                        IdentityField::Request(path("restApiId")?),
                        IdentityField::Selected(path("id")?),
                    ]),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        (
            "GetResource",
            deferred(
                "Model requires restApiId, resourceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists information about a resource.",
            ),
        ),
        // Deliberately leave embed unset: methods and integration/template payloads are opt-in.
        (
            "GetResources",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("restApiId")?,
                    Binding::From {
                        operation: name("GetRestApis")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                ..read(
                    "items",
                    Identity::CorrelatedKey(vec![
                        IdentityField::Request(path("restApiId")?),
                        IdentityField::Selected(path("id")?),
                    ]),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        // Model: Lists the RestApi resource in the collection.
        (
            "GetRestApi",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("restApiId")?,
                    Binding::From {
                        operation: name("GetRestApis")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("id")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        // Model: Lists the RestApis resources for your collection.
        (
            "GetRestApis",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("id")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        (
            "GetSdk",
            deferred(
                "Model requires restApiId, stageName, sdkType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Generates a client SDK for a RestApi and Stage.",
            ),
        ),
        (
            "GetSdkType",
            deferred(
                "Model requires id. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an SDK type.",
            ),
        ),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "GetSdkTypes",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is items; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets SDK types",
            ),
        ),
        (
            "GetStage",
            deferred(
                "Model requires restApiId, stageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a Stage resource.",
            ),
        ),
        (
            "GetStages",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about one or more Stage resources.",
            ),
        ),
        (
            "GetTags",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the Tags collection for a given resource.",
            ),
        ),
        (
            "GetUsage",
            deferred(
                "Model requires usagePlanId, startDate, endDate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the usage data of a usage plan in a specified time interval.",
            ),
        ),
        // Model: Gets a usage plan of a given plan identifier.
        (
            "GetUsagePlan",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("usagePlanId")?,
                    Binding::From {
                        operation: name("GetUsagePlans")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("id")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetUsagePlanKey",
            deferred(
                "Model requires usagePlanId, keyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a usage plan key of a given key identifier.",
            ),
        ),
        (
            "GetUsagePlanKeys",
            deferred(
                "Model requires usagePlanId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets all the usage plan keys representing the API keys added to a specified usage plan.",
            ),
        ),
        // Model: Gets all the usage plans of the caller's account.
        (
            "GetUsagePlans",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("id")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        // Model: Gets a specified VPC link under the caller's account in a region.
        (
            "GetVpcLink",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("vpcLinkId")?,
                    Binding::From {
                        operation: name("GetVpcLinks")?,
                        path: path("id")?,
                    },
                )]
                .into_iter()
                .collect(),
                tags: Some(path("tags")?),
                queryable: vec![path("id")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        // Model: Gets the VpcLinks collection under the caller's account in a selected region.
        (
            "GetVpcLinks",
            OperationOverlay {
                tags: Some(path("tags")?),
                queryable: vec![path("id")?],
                ..read(
                    "items",
                    Identity::Key(vec![path("id")?]),
                    Scope::Region,
                    token("position", "position")?,
                )?
            },
        ),
        (
            "ImportApiKeys",
            deferred(
                "Model requires body, format. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Import API keys from an external source, such as a CSV-formatted file.",
            ),
        ),
        (
            "ImportDocumentationParts",
            deferred(
                "Model requires restApiId, body. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Imports documentation parts",
            ),
        ),
        (
            "ImportRestApi",
            deferred(
                "Model requires body. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: A feature of the API Gateway control service for creating a new API from an external API definition file.",
            ),
        ),
        (
            "PutGatewayResponse",
            deferred(
                "Model requires restApiId, responseType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a customization of a GatewayResponse of a specified response type and status code on the given RestApi.",
            ),
        ),
        (
            "PutIntegration",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, type. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets up a method's integration.",
            ),
        ),
        (
            "PutIntegrationResponse",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, statusCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents a put integration.",
            ),
        ),
        (
            "PutMethod",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, authorizationType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Add a method to an existing Resource resource.",
            ),
        ),
    ])
}

pub(super) fn part_4() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "PutMethodResponse",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, statusCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds a MethodResponse to an existing Method resource.",
            ),
        ),
        (
            "PutRestApi",
            deferred(
                "Model requires restApiId, body. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: A feature of the API Gateway control service for updating an existing API with an input of external API definitions. The update can take the form of merging the supplied definition into the existing API or overwriting the…",
            ),
        ),
        (
            "RejectDomainNameAccessAssociation",
            deferred(
                "Model requires domainNameAccessAssociationArn, domainNameArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Rejects a domain name access association with a private custom domain name. To reject a domain name access association with an access association source in another AWS account, use this operation. To remove a domain name access…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds or updates a tag on a given resource.",
            ),
        ),
        (
            "TestInvokeAuthorizer",
            deferred(
                "Model requires restApiId, authorizerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Simulate the execution of an Authorizer in your RestApi with headers, parameters, and an incoming request body.",
            ),
        ),
        (
            "TestInvokeMethod",
            deferred(
                "Model requires restApiId, resourceId, httpMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Simulate the invocation of a Method in your RestApi with headers, parameters, and an incoming request body.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes a tag from a given resource.",
            ),
        ),
        (
            "UpdateAccount",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is cloudwatchRoleArn, throttleSettings, features, apiKeyVersion; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Changes information about the current Account resource.",
            ),
        ),
        (
            "UpdateApiKey",
            deferred(
                "Model requires apiKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about an ApiKey resource.",
            ),
        ),
        (
            "UpdateAuthorizer",
            deferred(
                "Model requires restApiId, authorizerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Authorizer resource.",
            ),
        ),
        (
            "UpdateBasePathMapping",
            deferred(
                "Model requires domainName, basePath. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about the BasePathMapping resource.",
            ),
        ),
        (
            "UpdateClientCertificate",
            deferred(
                "Model requires clientCertificateId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about an ClientCertificate resource.",
            ),
        ),
        (
            "UpdateDeployment",
            deferred(
                "Model requires restApiId, deploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about a Deployment resource.",
            ),
        ),
        (
            "UpdateDocumentationPart",
            deferred(
                "Model requires restApiId, documentationPartId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a documentation part.",
            ),
        ),
        (
            "UpdateDocumentationVersion",
            deferred(
                "Model requires restApiId, documentationVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a documentation version.",
            ),
        ),
        (
            "UpdateDomainName",
            deferred(
                "Model requires domainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about the DomainName resource.",
            ),
        ),
        (
            "UpdateGatewayResponse",
            deferred(
                "Model requires restApiId, responseType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a GatewayResponse of a specified response type on the given RestApi.",
            ),
        ),
        (
            "UpdateIntegration",
            deferred(
                "Model requires restApiId, resourceId, httpMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents an update integration.",
            ),
        ),
        (
            "UpdateIntegrationResponse",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, statusCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Represents an update integration response.",
            ),
        ),
        (
            "UpdateMethod",
            deferred(
                "Model requires restApiId, resourceId, httpMethod. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing Method resource.",
            ),
        ),
        (
            "UpdateMethodResponse",
            deferred(
                "Model requires restApiId, resourceId, httpMethod, statusCode. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing MethodResponse resource.",
            ),
        ),
        (
            "UpdateModel",
            deferred(
                "Model requires restApiId, modelName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about a model. The maximum size of the model is 400 KB.",
            ),
        ),
        (
            "UpdateRequestValidator",
            deferred(
                "Model requires restApiId, requestValidatorId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a RequestValidator of a given RestApi.",
            ),
        ),
        (
            "UpdateResource",
            deferred(
                "Model requires restApiId, resourceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about a Resource resource.",
            ),
        ),
        (
            "UpdateRestApi",
            deferred(
                "Model requires restApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about the specified API.",
            ),
        ),
        (
            "UpdateStage",
            deferred(
                "Model requires restApiId, stageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about a Stage resource.",
            ),
        ),
        (
            "UpdateUsage",
            deferred(
                "Model requires usagePlanId, keyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Grants a temporary extension to the remaining quota of a usage plan associated with a specified API key.",
            ),
        ),
        (
            "UpdateUsagePlan",
            deferred(
                "Model requires usagePlanId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a usage plan of a given plan Id.",
            ),
        ),
        (
            "UpdateVpcLink",
            deferred(
                "Model requires vpcLinkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an existing VpcLink of a specified identifier.",
            ),
        ),
    ])
}
