use super::policy;
use crate::overlay::{NameError, ServiceOverlay};

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    let approved = vec![
        (
            "ListConnectors",
            policy::query(
                policy::enumerate(
                    "connectors",
                    policy::arn("connectorArn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["connectorArn"],
            )?,
        ),
        (
            "ListCustomPlugins",
            policy::query(
                policy::enumerate(
                    "customPlugins",
                    policy::arn("customPluginArn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["customPluginArn"],
            )?,
        ),
        (
            "ListWorkerConfigurations",
            policy::query(
                policy::enumerate(
                    "workerConfigurations",
                    policy::arn("workerConfigurationArn")?,
                    Some(("nextToken", "nextToken")),
                )?,
                &["workerConfigurationArn"],
            )?,
        ),
        (
            "DescribeCustomPlugin",
            policy::query(
                policy::bind(
                    policy::enumerate("$", policy::arn("customPluginArn")?, None)?,
                    "customPluginArn",
                    "ListCustomPlugins",
                    "customPluginArn",
                )?,
                &["customPluginArn"],
            )?,
        ),
        (
            "ListConnectorOperations",
            policy::query(
                policy::bind(
                    policy::enumerate(
                        "connectorOperations",
                        policy::arn("connectorOperationArn")?,
                        Some(("nextToken", "nextToken")),
                    )?,
                    "connectorArn",
                    "ListConnectors",
                    "connectorArn",
                )?,
                &["connectorOperationArn"],
            )?,
        ),
    ];
    policy::finish(
        "kafkaconnect",
        approved,
        DEFERRED,
        "Connector/plugin/worker-configuration summaries and plugin metadata detail are approved. Worker detail returns sensitive propertiesFileContent and is denied, not projected away. Other detail/operation paths remain unapproved.",
        &[
            (policy::DenyReason::Mutating, MUTATING),
            (policy::DenyReason::Secret, SECRET),
        ],
    )
}

const DEFERRED: &[&str] = &[
    "DescribeConnector",
    "DescribeConnectorOperation",
    "ListTagsForResource",
];

const MUTATING: &[&str] = &[
    "CreateConnector",
    "CreateCustomPlugin",
    "CreateWorkerConfiguration",
    "DeleteConnector",
    "DeleteCustomPlugin",
    "DeleteWorkerConfiguration",
    "PauseConnector",
    "RestartConnector",
    "ResumeConnector",
    "TagResource",
    "UntagResource",
    "UpdateConnector",
];

const SECRET: &[&str] = &["DescribeWorkerConfiguration"];
