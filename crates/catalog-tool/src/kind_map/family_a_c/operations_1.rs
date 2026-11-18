use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping {
        service: "amplifybackend",
        operation: "ListS3Buckets",
        kind: "AWS::S3::Bucket",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetAccount",
        kind: "AWS::ApiGateway::Account",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetApiKeys",
        kind: "AWS::ApiGateway::ApiKey",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetClientCertificates",
        kind: "AWS::ApiGateway::ClientCertificate",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetDomainName",
        kind: "AWS::ApiGateway::DomainName",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetDomainNameAccessAssociations",
        kind: "AWS::ApiGateway::DomainNameAccessAssociation",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetDomainNames",
        kind: "AWS::ApiGateway::DomainName",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetRestApi",
        kind: "AWS::ApiGateway::RestApi",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetRestApis",
        kind: "AWS::ApiGateway::RestApi",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetUsagePlan",
        kind: "AWS::ApiGateway::UsagePlan",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetUsagePlans",
        kind: "AWS::ApiGateway::UsagePlan",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetVpcLink",
        kind: "AWS::ApiGateway::VpcLink",
    },
    OperationMapping {
        service: "api-gateway",
        operation: "GetVpcLinks",
        kind: "AWS::ApiGateway::VpcLink",
    },
    OperationMapping {
        service: "apigatewayv2",
        operation: "GetApi",
        kind: "AWS::ApiGatewayV2::Api",
    },
    OperationMapping {
        service: "apigatewayv2",
        operation: "GetApis",
        kind: "AWS::ApiGatewayV2::Api",
    },
    OperationMapping {
        service: "apigatewayv2",
        operation: "GetDomainName",
        kind: "AWS::ApiGatewayV2::DomainName",
    },
    OperationMapping {
        service: "apigatewayv2",
        operation: "GetDomainNames",
        kind: "AWS::ApiGatewayV2::DomainName",
    },
    OperationMapping {
        service: "apigatewayv2",
        operation: "GetVpcLink",
        kind: "AWS::ApiGatewayV2::VpcLink",
    },
    OperationMapping {
        service: "apigatewayv2",
        operation: "GetVpcLinks",
        kind: "AWS::ApiGatewayV2::VpcLink",
    },
    OperationMapping {
        service: "apigatewayv2",
        operation: "ListPortalProducts",
        kind: "AWS::ApiOnly::ApiGatewayV2PortalProduct",
    },
];
