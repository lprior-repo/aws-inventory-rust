pub type RelationshipSeed = (
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
);

#[rustfmt::skip]
pub const GROUPS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "EKS",
        "Cluster",
        "Name",
        "ClusterName",
        "Addon FargateProfile Nodegroup AccessEntry PodIdentityAssociation",
    ),
    (
        "ElasticLoadBalancingV2",
        "LoadBalancer",
        "LoadBalancerArn",
        "LoadBalancerArn",
        "Listener",
    ),
    (
        "ApiGateway",
        "RestApi",
        "RestApiId",
        "RestApiId",
        "Authorizer Deployment DocumentationVersion GatewayResponse Model RequestValidator Resource Stage",
    ),
    (
        "ApiGatewayV2",
        "Api",
        "ApiId",
        "ApiId",
        "Authorizer Deployment Integration Route Stage",
    ),
    (
        "Logs",
        "LogGroup",
        "LogGroupName",
        "LogGroupName",
        "LogStream SubscriptionFilter",
    ),
    (
        "AppConfig",
        "Application",
        "ApplicationId",
        "ApplicationId",
        "Environment ConfigurationProfile",
    ),
    (
        "AppConfig",
        "Application",
        "ApplicationId",
        "ApplicationIdentifier",
        "ExperimentDefinition",
    ),
];

#[rustfmt::skip]
pub const NATIVE_KINDS: &[(&str, &str)] = &[
    ("AWS::EKS::Nodegroup", "aws_eks_node_group"),
    (
        "AWS::ElasticLoadBalancingV2::Listener",
        "aws_elbv2_listener",
    ),
];

#[rustfmt::skip]
pub const SENSITIVE: &[(&str, &[&str])] = &[
    ("AWS::EKS::Addon", &["/ConfigurationValues"]),
    (
        "AWS::ApiGateway::Stage",
        &["/Variables", "/CanarySetting/StageVariableOverrides"],
    ),
    ("AWS::ApiGatewayV2::Stage", &["/StageVariables"]),
    (
        "AWS::ApiGateway::Deployment",
        &[
            "/StageDescription/Variables",
            "/StageDescription/CanarySetting/StageVariableOverrides",
        ],
    ),
    (
        "AWS::ApiGateway::GatewayResponse",
        &["/ResponseParameters", "/ResponseTemplates"],
    ),
    (
        "AWS::ApiGatewayV2::Integration",
        &[
            "/RequestParameters",
            "/RequestTemplates",
            "/ResponseParameters",
        ],
    ),
    (
        "AWS::AppConfig::ConfigurationProfile",
        &["/Validators/*/Content"],
    ),
];

#[rustfmt::skip]
pub const RELATIONSHIPS: &[(&str, &[RelationshipSeed])] = &[(
    "AWS::EKS::Nodegroup",
    &[("/NodeRole", "ASSUMES_ROLE", "aws_iam_role", None)],
)];
