use super::*;

async fn versions(
    root: &'static str,
    operation: &'static str,
    input: &'static str,
    parent_kind: &'static str,
    kind: &'static str,
    segment: &'static str,
    forbidden: &'static [&'static str],
) -> TestResult {
    let parents = ["parent-a", "parent-b"].map(|parent| {
        let arn =
            format!("arn:aws:greengrass:us-east-1:123456789012:/greengrass/{segment}/{parent}");
        Parent {
            binding: parent.to_owned(),
            id: arn.clone(),
            kind: parent_kind,
            row: json!({"Id":parent,"Arn":arn,"Name":"definition metadata"}),
        }
    });
    // VersionInformation.Id intentionally repeats its parent on both populated pages. Only Arn/Version changes.
    exercise(
        Case {
            service: "greengrass",
            date: "2017-06-07",
            root,
            root_items: if root == "ListGroups" {
                "Groups"
            } else {
                "Definitions"
            },
            operation,
            items: "Versions",
            input,
            token: "NextToken",
            kind,
            relation: "VERSION_OF",
            forbidden,
        },
        parents,
    )
    .await
}

#[tokio::test]
async fn given_connector_definitions_when_explicit_tokens_resume_then_distinct_version_arns_not_parent_ids_are_persisted()
-> TestResult {
    versions(
        "ListConnectorDefinitions",
        "ListConnectorDefinitionVersions",
        "ConnectorDefinitionId",
        "AWS::Greengrass::ConnectorDefinition",
        "AWS::Greengrass::ConnectorDefinitionVersion",
        "definition/connectors",
        &["GetConnectorDefinitionVersion"],
    )
    .await
}

#[tokio::test]
async fn given_core_definitions_when_explicit_tokens_resume_then_distinct_version_arns_not_parent_ids_are_persisted()
-> TestResult {
    versions(
        "ListCoreDefinitions",
        "ListCoreDefinitionVersions",
        "CoreDefinitionId",
        "AWS::Greengrass::CoreDefinition",
        "AWS::Greengrass::CoreDefinitionVersion",
        "definition/cores",
        &["GetCoreDefinitionVersion"],
    )
    .await
}

#[tokio::test]
async fn given_device_definitions_when_explicit_tokens_resume_then_distinct_version_arns_not_parent_ids_are_persisted()
-> TestResult {
    versions(
        "ListDeviceDefinitions",
        "ListDeviceDefinitionVersions",
        "DeviceDefinitionId",
        "AWS::Greengrass::DeviceDefinition",
        "AWS::Greengrass::DeviceDefinitionVersion",
        "definition/devices",
        &["GetDeviceDefinitionVersion"],
    )
    .await
}

#[tokio::test]
async fn given_function_definitions_when_explicit_tokens_resume_then_distinct_version_arns_not_parent_ids_are_persisted()
-> TestResult {
    versions(
        "ListFunctionDefinitions",
        "ListFunctionDefinitionVersions",
        "FunctionDefinitionId",
        "AWS::Greengrass::FunctionDefinition",
        "AWS::Greengrass::FunctionDefinitionVersion",
        "definition/functions",
        &["GetFunctionDefinitionVersion"],
    )
    .await
}

#[tokio::test]
async fn given_logger_definitions_when_explicit_tokens_resume_then_distinct_version_arns_not_parent_ids_are_persisted()
-> TestResult {
    versions(
        "ListLoggerDefinitions",
        "ListLoggerDefinitionVersions",
        "LoggerDefinitionId",
        "AWS::Greengrass::LoggerDefinition",
        "AWS::Greengrass::LoggerDefinitionVersion",
        "definition/loggers",
        &["GetLoggerDefinitionVersion"],
    )
    .await
}

#[tokio::test]
async fn given_resource_definitions_when_explicit_tokens_resume_then_distinct_version_arns_not_parent_ids_are_persisted()
-> TestResult {
    versions(
        "ListResourceDefinitions",
        "ListResourceDefinitionVersions",
        "ResourceDefinitionId",
        "AWS::Greengrass::ResourceDefinition",
        "AWS::Greengrass::ResourceDefinitionVersion",
        "definition/resources",
        &["GetResourceDefinitionVersion"],
    )
    .await
}

#[tokio::test]
async fn given_subscription_definitions_when_explicit_tokens_resume_then_distinct_version_arns_not_parent_ids_are_persisted()
-> TestResult {
    versions(
        "ListSubscriptionDefinitions",
        "ListSubscriptionDefinitionVersions",
        "SubscriptionDefinitionId",
        "AWS::Greengrass::SubscriptionDefinition",
        "AWS::Greengrass::SubscriptionDefinitionVersion",
        "definition/subscriptions",
        &["GetSubscriptionDefinitionVersion"],
    )
    .await
}

#[tokio::test]
async fn given_groups_when_explicit_tokens_resume_then_distinct_group_version_arns_not_group_ids_are_persisted()
-> TestResult {
    versions(
        "ListGroups",
        "ListGroupVersions",
        "GroupId",
        "AWS::Greengrass::Group",
        "AWS::Greengrass::GroupVersion",
        "groups",
        &["GetGroupVersion"],
    )
    .await
}
