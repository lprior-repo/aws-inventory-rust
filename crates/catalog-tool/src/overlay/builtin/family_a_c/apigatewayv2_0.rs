// Pinned source: .local/aws-models/models/apigatewayv2/service/2018-11-29/apigatewayv2-2018-11-29.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;
use crate::overlay::IdentityField;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("CreateApi", deferred("Model requires Name, ProtocolType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Api resource.")),
        ("CreateApiMapping", deferred("Model requires ApiId, DomainName, Stage. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an API mapping.")),
        ("CreateAuthorizer", deferred("Model requires ApiId, AuthorizerType, IdentitySource, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Authorizer for an API.")),
        ("CreateDeployment", deferred("Model requires ApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Deployment for an API.")),
        ("CreateDomainName", deferred("Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a domain name.")),
        ("CreateIntegration", deferred("Model requires ApiId, IntegrationType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Integration.")),
        ("CreateIntegrationResponse", deferred("Model requires ApiId, IntegrationId, IntegrationResponseKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an IntegrationResponses.")),
        ("CreateModel", deferred("Model requires ApiId, Name, Schema. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Model for an API.")),
        ("CreatePortal", deferred("Model requires Authorization, EndpointConfiguration, PortalContent. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a portal.")),
        ("CreatePortalProduct", deferred("Model requires DisplayName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new portal product.")),
        ("CreateProductPage", deferred("Model requires DisplayContent, PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new product page for a portal product.")),
        ("CreateProductRestEndpointPage", deferred("Model requires PortalProductId, RestEndpointIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a product REST endpoint page for a portal product.")),
        ("CreateRoute", deferred("Model requires ApiId, RouteKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Route for an API.")),
        ("CreateRouteResponse", deferred("Model requires ApiId, RouteId, RouteResponseKey. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a RouteResponse for a Route.")),
        ("CreateRoutingRule", deferred("Model requires Actions, Conditions, DomainName, Priority. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a RoutingRule.")),
        ("CreateStage", deferred("Model requires ApiId, StageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a Stage for an API.")),
        ("CreateVpcLink", deferred("Model requires Name, SubnetIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a VPC link.")),
        ("DeleteAccessLogSettings", deferred("Model requires ApiId, StageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the AccessLogSettings for a Stage. To disable access logging for a Stage, delete its AccessLogSettings.")),
        ("DeleteApi", deferred("Model requires ApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Api resource.")),
        ("DeleteApiMapping", deferred("Model requires ApiMappingId, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an API mapping.")),
        ("DeleteAuthorizer", deferred("Model requires ApiId, AuthorizerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Authorizer.")),
        ("DeleteCorsConfiguration", deferred("Model requires ApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a CORS configuration.")),
        ("DeleteDeployment", deferred("Model requires ApiId, DeploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Deployment.")),
        ("DeleteDomainName", deferred("Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a domain name.")),
        ("DeleteIntegration", deferred("Model requires ApiId, IntegrationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Integration.")),
        ("DeleteIntegrationResponse", deferred("Model requires ApiId, IntegrationId, IntegrationResponseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an IntegrationResponses.")),
        ("DeleteModel", deferred("Model requires ApiId, ModelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Model.")),
        ("DeletePortal", deferred("Model requires PortalId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a portal.")),
        ("DeletePortalProduct", deferred("Model requires PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a portal product.")),
        ("DeletePortalProductSharingPolicy", deferred("Model requires PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the sharing policy for a portal product.")),
        ("DeleteProductPage", deferred("Model requires PortalProductId, ProductPageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a product page of a portal product.")),
        ("DeleteProductRestEndpointPage", deferred("Model requires PortalProductId, ProductRestEndpointPageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a product REST endpoint page.")),
        ("DeleteRoute", deferred("Model requires ApiId, RouteId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Route.")),
        ("DeleteRouteRequestParameter", deferred("Model requires ApiId, RequestParameterKey, RouteId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a route request parameter. Supported only for WebSocket APIs.")),
        ("DeleteRouteResponse", deferred("Model requires ApiId, RouteId, RouteResponseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a RouteResponse.")),
        ("DeleteRouteSettings", deferred("Model requires ApiId, RouteKey, StageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the RouteSettings for a stage.")),
        ("DeleteRoutingRule", deferred("Model requires DomainName, RoutingRuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a routing rule.")),
        ("DeleteStage", deferred("Model requires ApiId, StageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Stage.")),
        ("DeleteVpcLink", deferred("Model requires VpcLinkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a VPC link.")),
        ("DisablePortal", deferred("Model requires PortalId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the publication of a portal portal.")),
        ("ExportApi", deferred("Model requires ApiId, OutputType, Specification. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: ")),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Gets an Api resource.
        ("GetApi", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("ApiId")?, Binding::From { operation: name("GetApis")?, path: path("ApiId")? })].into_iter().collect(),
            tags: Some(path("Tags")?),
            queryable: vec![path("ApiId")?],
            ..read("$", Identity::Key(vec![path("ApiId")?]), Scope::Region, Paginator::None)?
        }),
        ("GetApiMapping", deferred("Model requires ApiMappingId, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an API mapping.")),
        ("GetApiMappings", deferred("Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets API mappings.")),
        // Model: Gets a collection of Api resources.
        ("GetApis", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("ApiId")?],
            ..read("Items", Identity::Key(vec![path("ApiId")?]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("GetAuthorizer", deferred("Model requires ApiId, AuthorizerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an Authorizer.")),
        ("GetAuthorizers", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("ApiId")?, Binding::From { operation: name("GetApis")?, path: path("ApiId")? })].into_iter().collect(),
            ..read("Items", Identity::CorrelatedKey(vec![
                IdentityField::Request(path("ApiId")?), IdentityField::Selected(path("AuthorizerId")?),
            ]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("GetDeployment", deferred("Model requires ApiId, DeploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a Deployment.")),
        ("GetDeployments", deferred("Model requires ApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the Deployments for an API.")),
        // Model: Gets a domain name.
        ("GetDomainName", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("DomainName")?, Binding::From { operation: name("GetDomainNames")?, path: path("DomainName")? })].into_iter().collect(),
            tags: Some(path("Tags")?),
            queryable: vec![path("DomainNameArn")?],
            ..read("$", Identity::Arn(path("DomainNameArn")?), Scope::Region, Paginator::None)?
        }),
        // Model: Gets the domain names for an AWS account.
        ("GetDomainNames", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("DomainName")?, path("DomainNameArn")?],
            ..read("Items", Identity::Arn(path("DomainNameArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("GetIntegration", deferred("Model requires ApiId, IntegrationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an Integration.")),
        ("GetIntegrationResponse", deferred("Model requires ApiId, IntegrationId, IntegrationResponseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets an IntegrationResponses.")),
        ("GetIntegrationResponses", deferred("Model requires ApiId, IntegrationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the IntegrationResponses for an Integration.")),
        ("GetIntegrations", deferred("Model requires ApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the Integrations for an API.")),
        ("GetModel", deferred("Model requires ApiId, ModelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a Model.")),
        ("GetModelTemplate", deferred("Model requires ApiId, ModelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a model template.")),
        ("GetModels", deferred("Model requires ApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the Models for an API.")),
        ("GetPortal", deferred("Model requires PortalId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a portal.")),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("GetPortalProduct", deferred("Model requires PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a portal product.")),
        ("GetPortalProductSharingPolicy", deferred("Model requires PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the sharing policy for a portal product.")),
        ("GetProductPage", deferred("Model requires PortalProductId, ProductPageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a product page of a portal product.")),
        ("GetProductRestEndpointPage", deferred("Model requires PortalProductId, ProductRestEndpointPageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a product REST endpoint page.")),
        ("GetRoute", deferred("Model requires ApiId, RouteId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a Route.")),
        ("GetRouteResponse", deferred("Model requires ApiId, RouteId, RouteResponseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a RouteResponse.")),
        ("GetRouteResponses", deferred("Model requires ApiId, RouteId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the RouteResponses for a Route.")),
        ("GetRoutes", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("ApiId")?, Binding::From { operation: name("GetApis")?, path: path("ApiId")? })].into_iter().collect(),
            ..read("Items", Identity::CorrelatedKey(vec![
                IdentityField::Request(path("ApiId")?), IdentityField::Selected(path("RouteId")?),
            ]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("GetRoutingRule", deferred("Model requires DomainName, RoutingRuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a routing rule.")),
        ("GetStage", deferred("Model requires ApiId, StageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a Stage.")),
        ("GetStages", deferred("Model requires ApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the Stages for an API.")),
        ("GetTags", deferred("Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a collection of Tag resources.")),
        // Model: Gets a VPC link.
        ("GetVpcLink", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("VpcLinkId")?, Binding::From { operation: name("GetVpcLinks")?, path: path("VpcLinkId")? })].into_iter().collect(),
            tags: Some(path("Tags")?),
            queryable: vec![path("VpcLinkId")?],
            ..read("$", Identity::Key(vec![path("VpcLinkId")?]), Scope::Region, Paginator::None)?
        }),
        // Model: Gets a collection of VPC links.
        ("GetVpcLinks", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("VpcLinkId")?],
            ..read("Items", Identity::Key(vec![path("VpcLinkId")?]), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("ImportApi", deferred("Model requires Body. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Imports an API.")),
        // Model: Lists portal products.
        ("ListPortalProducts", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("PortalProductArn")?],
            ..read("Items", Identity::Arn(path("PortalProductArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        // Portal summaries contain configuration metadata and inline tags, not portal document payloads.
        ("ListPortals", OperationOverlay {
            tags: Some(path("Tags")?),
            queryable: vec![path("PortalArn")?, path("PortalId")?],
            ..read("Items", Identity::Arn(path("PortalArn")?), Scope::Region, token("NextToken", "NextToken")?)?
        }),
        ("ListProductPages", deferred("Model requires PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the product pages for a portal product.")),
        ("ListProductRestEndpointPages", deferred("Model requires PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the product REST endpoint pages of a portal product.")),
        ("ListRoutingRules", deferred("Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists routing rules.")),
        ("PreviewPortal", deferred("Model requires PortalId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a portal preview.")),
        ("PublishPortal", deferred("Model requires PortalId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Publishes a portal.")),
        ("PutPortalProductSharingPolicy", deferred("Model requires PolicyDocument, PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the sharing policy for a portal product.")),
        ("PutRoutingRule", deferred("Model requires Actions, Conditions, DomainName, Priority, RoutingRuleId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a routing rule.")),
        ("ReimportApi", deferred("Model requires ApiId, Body. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Puts an Api resource.")),
    ])
}

pub(super) fn part_3() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("ResetAuthorizersCache", deferred("Model requires ApiId, StageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Resets all authorizer cache entries on a stage. Supported only for HTTP APIs.")),
        ("TagResource", deferred("Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new Tag resource to represent a tag.")),
        ("UntagResource", deferred("Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a Tag.")),
        ("UpdateApi", deferred("Model requires ApiId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an Api resource.")),
        ("UpdateApiMapping", deferred("Model requires ApiId, ApiMappingId, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The API mapping.")),
        ("UpdateAuthorizer", deferred("Model requires ApiId, AuthorizerId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an Authorizer.")),
        ("UpdateDeployment", deferred("Model requires ApiId, DeploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Deployment.")),
        ("UpdateDomainName", deferred("Model requires DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a domain name.")),
        ("UpdateIntegration", deferred("Model requires ApiId, IntegrationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an Integration.")),
        ("UpdateIntegrationResponse", deferred("Model requires ApiId, IntegrationId, IntegrationResponseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an IntegrationResponses.")),
        ("UpdateModel", deferred("Model requires ApiId, ModelId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Model.")),
        ("UpdatePortal", deferred("Model requires PortalId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a portal.")),
        ("UpdatePortalProduct", deferred("Model requires PortalProductId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the portal product.")),
        ("UpdateProductPage", deferred("Model requires PortalProductId, ProductPageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a product page of a portal product.")),
        ("UpdateProductRestEndpointPage", deferred("Model requires PortalProductId, ProductRestEndpointPageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a product REST endpoint page.")),
        ("UpdateRoute", deferred("Model requires ApiId, RouteId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Route.")),
        ("UpdateRouteResponse", deferred("Model requires ApiId, RouteId, RouteResponseId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a RouteResponse.")),
        ("UpdateStage", deferred("Model requires ApiId, StageName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a Stage.")),
        ("UpdateVpcLink", deferred("Model requires VpcLinkId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a VPC link.")),
    ])
}
