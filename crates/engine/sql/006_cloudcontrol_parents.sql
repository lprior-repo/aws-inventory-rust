CREATE VIEW aws_cc_api_gateway_authorizer AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AuthType') AS "auth_type",
       json_extract(r.provider_properties, '$.AuthorizerCredentials') AS "authorizer_credentials",
       json_extract(r.provider_properties, '$.AuthorizerId') AS "authorizer_id",
       json_extract(r.provider_properties, '$.AuthorizerResultTtlInSeconds') AS "authorizer_result_ttl_in_seconds",
       json_extract(r.provider_properties, '$.AuthorizerUri') AS "authorizer_uri",
       json_extract(r.provider_properties, '$.IdentitySource') AS "identity_source",
       json_extract(r.provider_properties, '$.IdentityValidationExpression') AS "identity_validation_expression",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.ProviderARNs') AS "provider_ar_ns",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id",
       json_extract(r.provider_properties, '$.Type') AS "type"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::Authorizer';

CREATE VIEW aws_cc_api_gateway_deployment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.DeploymentId') AS "deployment_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::Deployment';

CREATE VIEW aws_cc_api_gateway_documentation_version AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DocumentationVersion') AS "documentation_version",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::DocumentationVersion';

CREATE VIEW aws_cc_api_gateway_gateway_response AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.ResponseType') AS "response_type",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id",
       json_extract(r.provider_properties, '$.StatusCode') AS "status_code"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::GatewayResponse';

CREATE VIEW aws_cc_api_gateway_model AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ContentType') AS "content_type",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id",
       json_extract(r.provider_properties, '$.Schema') AS "schema"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::Model';

CREATE VIEW aws_cc_api_gateway_request_validator AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RequestValidatorId') AS "request_validator_id",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id",
       json_extract(r.provider_properties, '$.ValidateRequestBody') AS "validate_request_body",
       json_extract(r.provider_properties, '$.ValidateRequestParameters') AS "validate_request_parameters"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::RequestValidator';

CREATE VIEW aws_cc_api_gateway_resource AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ParentId') AS "parent_id",
       json_extract(r.provider_properties, '$.PathPart') AS "path_part",
       json_extract(r.provider_properties, '$.ResourceId') AS "resource_id",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::Resource';

CREATE VIEW aws_cc_api_gateway_stage AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessLogSetting') AS "access_log_setting",
       json_extract(r.provider_properties, '$.CacheClusterEnabled') AS "cache_cluster_enabled",
       json_extract(r.provider_properties, '$.CacheClusterSize') AS "cache_cluster_size",
       json_extract(r.provider_properties, '$.CanarySetting') AS "canary_setting",
       json_extract(r.provider_properties, '$.ClientCertificateId') AS "client_certificate_id",
       json_extract(r.provider_properties, '$.DeploymentId') AS "deployment_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.DocumentationVersion') AS "documentation_version",
       json_extract(r.provider_properties, '$.MethodSettings') AS "method_settings",
       json_extract(r.provider_properties, '$.RestApiId') AS "rest_api_id",
       json_extract(r.provider_properties, '$.StageName') AS "stage_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.TracingEnabled') AS "tracing_enabled"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGateway::Stage';

CREATE VIEW aws_cc_api_gateway_v2_authorizer AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApiId') AS "api_id",
       json_extract(r.provider_properties, '$.AuthorizerId') AS "authorizer_id",
       json_extract(r.provider_properties, '$.AuthorizerType') AS "authorizer_type",
       json_extract(r.provider_properties, '$.IdentitySource') AS "identity_source",
       json_extract(r.provider_properties, '$.IdentityValidationExpression') AS "identity_validation_expression",
       json_extract(r.provider_properties, '$.JwtConfiguration') AS "jwt_configuration",
       json_extract(r.provider_properties, '$.Name') AS "provider_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGatewayV2::Authorizer';

CREATE VIEW aws_cc_api_gateway_v2_deployment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApiId') AS "api_id",
       json_extract(r.provider_properties, '$.DeploymentId') AS "deployment_id",
       json_extract(r.provider_properties, '$.Description') AS "description"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGatewayV2::Deployment';

CREATE VIEW aws_cc_api_gateway_v2_integration AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApiId') AS "api_id",
       json_extract(r.provider_properties, '$.ConnectionId') AS "connection_id",
       json_extract(r.provider_properties, '$.ConnectionType') AS "connection_type",
       json_extract(r.provider_properties, '$.ContentHandlingStrategy') AS "content_handling_strategy",
       json_extract(r.provider_properties, '$.CredentialsArn') AS "credentials_arn",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.IntegrationId') AS "integration_id",
       json_extract(r.provider_properties, '$.IntegrationMethod') AS "integration_method",
       json_extract(r.provider_properties, '$.IntegrationSubtype') AS "integration_subtype",
       json_extract(r.provider_properties, '$.IntegrationType') AS "integration_type",
       json_extract(r.provider_properties, '$.IntegrationUri') AS "integration_uri",
       json_extract(r.provider_properties, '$.PassthroughBehavior') AS "passthrough_behavior",
       json_extract(r.provider_properties, '$.PayloadFormatVersion') AS "payload_format_version",
       json_extract(r.provider_properties, '$.TemplateSelectionExpression') AS "template_selection_expression",
       json_extract(r.provider_properties, '$.TimeoutInMillis') AS "timeout_in_millis",
       json_extract(r.provider_properties, '$.TlsConfig') AS "tls_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGatewayV2::Integration';

CREATE VIEW aws_cc_api_gateway_v2_route AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApiId') AS "api_id",
       json_extract(r.provider_properties, '$.ApiKeyRequired') AS "api_key_required",
       json_extract(r.provider_properties, '$.AuthorizationScopes') AS "authorization_scopes",
       json_extract(r.provider_properties, '$.AuthorizationType') AS "authorization_type",
       json_extract(r.provider_properties, '$.ModelSelectionExpression') AS "model_selection_expression",
       json_extract(r.provider_properties, '$.OperationName') AS "operation_name",
       json_extract(r.provider_properties, '$.RequestModels') AS "request_models",
       json_extract(r.provider_properties, '$.RouteId') AS "route_id",
       json_extract(r.provider_properties, '$.RouteKey') AS "route_key",
       json_extract(r.provider_properties, '$.RouteResponseSelectionExpression') AS "route_response_selection_expression",
       json_extract(r.provider_properties, '$.Target') AS "target"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGatewayV2::Route';

CREATE VIEW aws_cc_api_gateway_v2_stage AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AccessLogSettings') AS "access_log_settings",
       json_extract(r.provider_properties, '$.ApiId') AS "api_id",
       json_extract(r.provider_properties, '$.AutoDeploy') AS "auto_deploy",
       json_extract(r.provider_properties, '$.ClientCertificateId') AS "client_certificate_id",
       json_extract(r.provider_properties, '$.DefaultRouteSettings') AS "default_route_settings",
       json_extract(r.provider_properties, '$.DeploymentId') AS "deployment_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.RouteSettings') AS "route_settings",
       json_extract(r.provider_properties, '$.StageName') AS "stage_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ApiGatewayV2::Stage';

CREATE VIEW aws_cc_app_config_configuration_profile AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationId') AS "application_id",
       json_extract(r.provider_properties, '$.ConfigurationProfileId') AS "configuration_profile_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.KmsKeyArn') AS "kms_key_arn",
       json_extract(r.provider_properties, '$.KmsKeyIdentifier') AS "kms_key_identifier",
       json_extract(r.provider_properties, '$.LocationUri') AS "location_uri",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.RetrievalRoleArn') AS "retrieval_role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Type') AS "type",
       json_extract(r.provider_properties, '$.Validators') AS "validators"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppConfig::ConfigurationProfile';

CREATE VIEW aws_cc_app_config_environment AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplicationId') AS "application_id",
       json_extract(r.provider_properties, '$.Description') AS "description",
       json_extract(r.provider_properties, '$.EnvironmentId') AS "environment_id",
       json_extract(r.provider_properties, '$.Monitors') AS "monitors",
       json_extract(r.provider_properties, '$.Name') AS "provider_name",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::AppConfig::Environment';

CREATE VIEW aws_cc_eks_addon AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AddonName') AS "addon_name",
       json_extract(r.provider_properties, '$.AddonVersion') AS "addon_version",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.NamespaceConfig') AS "namespace_config",
       json_extract(r.provider_properties, '$.ServiceAccountRoleArn') AS "service_account_role_arn",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EKS::Addon';

CREATE VIEW aws_cc_eks_fargate_profile AS
SELECT r.*,
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.FargateProfileName') AS "fargate_profile_name",
       json_extract(r.provider_properties, '$.PodExecutionRoleArn') AS "pod_execution_role_arn",
       json_extract(r.provider_properties, '$.Selectors') AS "selectors",
       json_extract(r.provider_properties, '$.Subnets') AS "subnets",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EKS::FargateProfile';

CREATE VIEW aws_cc_eks_nodegroup AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AmiType') AS "ami_type",
       json_extract(r.provider_properties, '$.Arn') AS "provider_arn",
       json_extract(r.provider_properties, '$.CapacityType') AS "capacity_type",
       json_extract(r.provider_properties, '$.ClusterName') AS "cluster_name",
       json_extract(r.provider_properties, '$.DiskSize') AS "disk_size",
       json_extract(r.provider_properties, '$.Id') AS "provider_id",
       json_extract(r.provider_properties, '$.InstanceTypes') AS "instance_types",
       json_extract(r.provider_properties, '$.Labels') AS "labels",
       json_extract(r.provider_properties, '$.LaunchTemplate') AS "launch_template",
       json_extract(r.provider_properties, '$.NodeRepairConfig') AS "node_repair_config",
       json_extract(r.provider_properties, '$.NodeRole') AS "node_role",
       json_extract(r.provider_properties, '$.NodegroupName') AS "nodegroup_name",
       json_extract(r.provider_properties, '$.ReleaseVersion') AS "release_version",
       json_extract(r.provider_properties, '$.RemoteAccess') AS "remote_access",
       json_extract(r.provider_properties, '$.ScalingConfig') AS "scaling_config",
       json_extract(r.provider_properties, '$.Subnets') AS "subnets",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags",
       json_extract(r.provider_properties, '$.Taints') AS "taints",
       json_extract(r.provider_properties, '$.UpdateConfig') AS "update_config",
       json_extract(r.provider_properties, '$.Version') AS "version",
       json_extract(r.provider_properties, '$.WarmPoolConfig') AS "warm_pool_config"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::EKS::Nodegroup';

CREATE VIEW aws_cc_elastic_load_balancing_v2_listener AS
SELECT r.*,
       json_extract(r.provider_properties, '$.AlpnPolicy') AS "alpn_policy",
       json_extract(r.provider_properties, '$.Certificates') AS "certificates",
       json_extract(r.provider_properties, '$.DefaultActions') AS "default_actions",
       json_extract(r.provider_properties, '$.ListenerArn') AS "listener_arn",
       json_extract(r.provider_properties, '$.ListenerAttributes') AS "listener_attributes",
       json_extract(r.provider_properties, '$.LoadBalancerArn') AS "load_balancer_arn",
       json_extract(r.provider_properties, '$.MutualAuthentication') AS "mutual_authentication",
       json_extract(r.provider_properties, '$.Port') AS "port",
       json_extract(r.provider_properties, '$.Protocol') AS "protocol",
       json_extract(r.provider_properties, '$.SslPolicy') AS "ssl_policy",
       json_extract(r.provider_properties, '$.Tags') AS "provider_tags"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::ElasticLoadBalancingV2::Listener';

CREATE VIEW aws_cc_logs_log_stream AS
SELECT r.*,
       json_extract(r.provider_properties, '$.LogGroupName') AS "log_group_name",
       json_extract(r.provider_properties, '$.LogStreamName') AS "log_stream_name"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::LogStream';

CREATE VIEW aws_cc_logs_subscription_filter AS
SELECT r.*,
       json_extract(r.provider_properties, '$.ApplyOnTransformedLogs') AS "apply_on_transformed_logs",
       json_extract(r.provider_properties, '$.DestinationArn') AS "destination_arn",
       json_extract(r.provider_properties, '$.Distribution') AS "distribution",
       json_extract(r.provider_properties, '$.EmitSystemFields') AS "emit_system_fields",
       json_extract(r.provider_properties, '$.FieldSelectionCriteria') AS "field_selection_criteria",
       json_extract(r.provider_properties, '$.FilterName') AS "filter_name",
       json_extract(r.provider_properties, '$.FilterPattern') AS "filter_pattern",
       json_extract(r.provider_properties, '$.LogGroupName') AS "log_group_name",
       json_extract(r.provider_properties, '$.RoleArn') AS "role_arn"
FROM aws_cloudcontrol_resource r WHERE cloudformation_type = 'AWS::Logs::SubscriptionFilter';

PRAGMA user_version = 6;
