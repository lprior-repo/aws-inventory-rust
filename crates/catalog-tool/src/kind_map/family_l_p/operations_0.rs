use crate::kind_map::OperationMapping;

pub(super) const ENTRIES: &[OperationMapping<'static>] = &[
    OperationMapping { service: "lakeformation", operation: "ListResources", kind: "AWS::LakeFormation::Resource" },
    OperationMapping { service: "lakeformation", operation: "ListLFTags", kind: "AWS::LakeFormation::Tag" },
    OperationMapping { service: "lakeformation", operation: "ListLFTagExpressions", kind: "AWS::ApiOnly::LakeFormationTagExpression" },
    OperationMapping { service: "lambda-core", operation: "ListNetworkConnectors", kind: "AWS::Lambda::NetworkConnector" },
    OperationMapping { service: "lambda-microvms", operation: "ListMicrovms", kind: "AWS::ApiOnly::LambdaMicrovm" },
    OperationMapping { service: "lambda-microvms", operation: "ListMicrovmImages", kind: "AWS::Lambda::MicrovmImage" },
    OperationMapping { service: "lambda-web", operation: "ListWebFunctions", kind: "AWS::Lambda::WebFunction" },
    OperationMapping { service: "launch-wizard", operation: "ListDeployments", kind: "AWS::LaunchWizard::Deployment" },
    OperationMapping { service: "lex-model-building-service", operation: "GetBots", kind: "AWS::ApiOnly::LexV1Bot" },
    OperationMapping { service: "lex-models-v2", operation: "ListBots", kind: "AWS::Lex::Bot" },
    OperationMapping { service: "license-manager-linux-subscriptions", operation: "ListLinuxSubscriptionInstances", kind: "AWS::ApiOnly::LinuxSubscriptionInstance" },
    OperationMapping { service: "license-manager-linux-subscriptions", operation: "ListLinuxSubscriptions", kind: "AWS::ApiOnly::LinuxSubscription" },
    OperationMapping { service: "license-manager-linux-subscriptions", operation: "ListRegisteredSubscriptionProviders", kind: "AWS::ApiOnly::LinuxSubscriptionProvider" },
    OperationMapping { service: "license-manager-user-subscriptions", operation: "ListIdentityProviders", kind: "AWS::ApiOnly::UserSubscriptionIdentityProvider" },
    OperationMapping { service: "license-manager-user-subscriptions", operation: "ListInstances", kind: "AWS::ApiOnly::UserSubscriptionInstance" },
    OperationMapping { service: "license-manager-user-subscriptions", operation: "ListLicenseServerEndpoints", kind: "AWS::ApiOnly::UserSubscriptionLicenseServerEndpoint" },
    OperationMapping { service: "license-manager", operation: "ListLicenseConfigurations", kind: "AWS::LicenseManager::LicenseConfiguration" },
    OperationMapping { service: "license-manager", operation: "ListLicenses", kind: "AWS::LicenseManager::License" },
    OperationMapping { service: "license-manager", operation: "ListLicenseManagerReportGenerators", kind: "AWS::LicenseManager::ReportGenerator" },
    OperationMapping { service: "license-manager", operation: "ListLicenseAssetGroups", kind: "AWS::LicenseManager::LicenseAssetGroup" },
];
