// Pinned source: .local/aws-models/models/appconfig/service/2019-10-09/appconfig-2019-10-09.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateApplication",
            deferred(
                "Model requires Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an application. In AppConfig, an application is simply an organizational construct like a folder. This organizational construct has a relationship with some unit of executable code. For example, you could create an…",
            ),
        ),
        (
            "CreateConfigurationProfile",
            deferred(
                "Model requires ApplicationId, Name, LocationUri. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a configuration profile, which is information that enables AppConfig to access the configuration source. Valid configuration sources include the following: Configuration data in YAML, JSON, and other formats stored in the…",
            ),
        ),
        (
            "CreateDeploymentStrategy",
            deferred(
                "Model requires Name, DeploymentDurationInMinutes, GrowthFactor. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a deployment strategy that defines important criteria for rolling out your configuration to the designated targets. A deployment strategy includes the overall duration required, a percentage of targets to receive the…",
            ),
        ),
        (
            "CreateEnvironment",
            deferred(
                "Model requires ApplicationId, Name. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an environment. For each application, you define one or more environments. An environment is a deployment group of AppConfig targets, such as applications in a Beta or Production environment. You can also define…",
            ),
        ),
        (
            "CreateExperimentDefinition",
            deferred(
                "Model requires ApplicationIdentifier, Name, ConfigurationProfileIdentifier, EnvironmentIdentifier, FlagKey, Treatments, Control, AudienceRule. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an experiment definition in AppConfig. An experiment definition describes the purpose, scope, and operational configuration of an experiment, including the target audience, feature flag, and treatment configurations.",
            ),
        ),
        (
            "CreateExtension",
            deferred(
                "Model requires Name, Actions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an AppConfig extension. An extension augments your ability to inject logic or behavior at different points during the AppConfig workflow of creating or deploying a configuration. You can create your own extensions or use…",
            ),
        ),
        (
            "CreateExtensionAssociation",
            deferred(
                "Model requires ExtensionIdentifier, ResourceIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: When you create an extension or configure an Amazon Web Services authored extension, you associate the extension with an AppConfig application, environment, or configuration profile. For example, you can choose to run the…",
            ),
        ),
        (
            "CreateHostedConfigurationVersion",
            deferred(
                "Model requires ApplicationId, ConfigurationProfileId, Content, ContentType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a new configuration in the AppConfig hosted configuration store. If you're creating a feature flag, we recommend you familiarize yourself with the JSON schema for feature flag data. For more information, see Type…",
            ),
        ),
        (
            "DeleteApplication",
            deferred(
                "Model requires ApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an application.",
            ),
        ),
        (
            "DeleteConfigurationProfile",
            deferred(
                "Model requires ApplicationId, ConfigurationProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a configuration profile. To prevent users from unintentionally deleting actively-used configuration profiles, enable deletion protection .",
            ),
        ),
        (
            "DeleteDeploymentStrategy",
            deferred(
                "Model requires DeploymentStrategyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a deployment strategy.",
            ),
        ),
        (
            "DeleteEnvironment",
            deferred(
                "Model requires EnvironmentId, ApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an environment. To prevent users from unintentionally deleting actively-used environments, enable deletion protection .",
            ),
        ),
        (
            "DeleteExperimentDefinition",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an experiment definition. You can archive the definition to hide it from the active list while preserving it for future reference, or permanently delete it along with all associated run history.",
            ),
        ),
        (
            "DeleteExtension",
            deferred(
                "Model requires ExtensionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an AppConfig extension. You must delete all associations to an extension before you delete the extension.",
            ),
        ),
        (
            "DeleteExtensionAssociation",
            deferred(
                "Model requires ExtensionAssociationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an extension association. This action doesn't delete extensions defined in the association.",
            ),
        ),
        (
            "DeleteHostedConfigurationVersion",
            deferred(
                "Model requires ApplicationId, ConfigurationProfileId, VersionNumber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a version of a configuration from the AppConfig hosted configuration store.",
            ),
        ),
        (
            "GetAccountSettings",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is DeletionProtection, VendedMetrics; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Returns information about the status of the DeletionProtection parameter.",
            ),
        ),
        // Model: Retrieves information about an application.
        (
            "GetApplication",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("ApplicationId")?,
                    Binding::From {
                        operation: name("ListApplications")?,
                        path: path("Id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("Id")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("Id")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetConfiguration",
            deferred(
                "Model requires Application, Environment, Configuration, ClientId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: (Deprecated) Retrieves the latest deployed configuration. Note the following important information. This API action is deprecated. Calls to receive configuration data should use the StartConfigurationSession and…",
            ),
        ),
        (
            "GetConfigurationProfile",
            deferred(
                "Model requires ApplicationId, ConfigurationProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a configuration profile.",
            ),
        ),
        (
            "GetDeployment",
            deferred(
                "Model requires ApplicationId, EnvironmentId, DeploymentNumber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a configuration deployment.",
            ),
        ),
        // Model: Retrieves information about a deployment strategy. A deployment strategy defines important criteria for rolling out your configuration to the designated targets. A deployment strategy includes the…
        (
            "GetDeploymentStrategy",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("DeploymentStrategyId")?,
                    Binding::From {
                        operation: name("ListDeploymentStrategies")?,
                        path: path("Id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("Id")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("Id")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetEnvironment",
            deferred(
                "Model requires ApplicationId, EnvironmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about an environment. An environment is a deployment group of AppConfig applications, such as applications in a Production environment or in an EU_Region environment. Each configuration deployment targets an…",
            ),
        ),
        (
            "GetExperimentDefinition",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about an experiment definition.",
            ),
        ),
        (
            "GetExperimentRun",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier, Run. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about an experiment run, including its status, start time, and exposure settings.",
            ),
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Returns information about an AppConfig extension.
        (
            "GetExtension",
            deferred(
                "ListExtensions includes AWS-authored reference extensions without an account-owner request filter; its child describe cannot run safely until an owned-only selector exists.",
            ),
        ),
        // Model: Returns information about an AppConfig extension association. For more information about extensions and associations, see Extending workflows in the AppConfig User Guide .
        (
            "GetExtensionAssociation",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("ExtensionAssociationId")?,
                    Binding::From {
                        operation: name("ListExtensionAssociations")?,
                        path: path("Id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("Id")?, path("Arn")?],
                ..read(
                    "$",
                    Identity::Key(vec![path("Id")?]),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetHostedConfigurationVersion",
            deferred(
                "Model requires ApplicationId, ConfigurationProfileId, VersionNumber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves information about a specific configuration version.",
            ),
        ),
        // Model: Lists all applications in your Amazon Web Services account.
        (
            "ListApplications",
            OperationOverlay {
                queryable: vec![path("Id")?],
                ..read(
                    "Items",
                    Identity::Key(vec![path("Id")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        // Model: Lists the configuration profiles for an application.
        (
            "ListConfigurationProfiles",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("ApplicationId")?,
                    Binding::From {
                        operation: name("ListApplications")?,
                        path: path("Id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("ApplicationId")?, path("Id")?],
                ..read(
                    "Items",
                    Identity::Key(vec![path("ApplicationId")?, path("Id")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        // Model: Lists deployment strategies.
        (
            "ListDeploymentStrategies",
            OperationOverlay {
                queryable: vec![path("Id")?],
                ..read(
                    "Items",
                    Identity::Key(vec![path("Id")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListDeployments",
            deferred(
                "Model requires ApplicationId, EnvironmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the deployments for an environment in descending deployment number order.",
            ),
        ),
        // Model: Lists the environments for an application.
        (
            "ListEnvironments",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("ApplicationId")?,
                    Binding::From {
                        operation: name("ListApplications")?,
                        path: path("Id")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("ApplicationId")?, path("Id")?],
                ..read(
                    "Items",
                    Identity::Key(vec![path("ApplicationId")?, path("Id")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists the experiment definitions for an account. You can filter results by application, configuration profile, environment, or status.
        (
            "ListExperimentDefinitions",
            OperationOverlay {
                queryable: vec![path("Id")?],
                ..read(
                    "Items",
                    Identity::Key(vec![path("Id")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListExperimentRunEvents",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier, Run. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the events for a specified experiment run. Events provide a timeline of actions and state changes that occurred during the run.",
            ),
        ),
        (
            "ListExperimentRuns",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the experiment runs for a specified experiment definition. You can filter by status.",
            ),
        ),
        // Model: Lists all AppConfig extension associations in the account. For more information about extensions and associations, see Extending workflows in the AppConfig User Guide .
        (
            "ListExtensionAssociations",
            OperationOverlay {
                queryable: vec![path("Id")?],
                ..read(
                    "Items",
                    Identity::Key(vec![path("Id")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        // Model: Lists all custom and Amazon Web Services authored AppConfig extensions in the account. For more information about extensions, see Extending workflows in the AppConfig User Guide .
        (
            "ListExtensions",
            deferred(
                "The modeled collection mixes custom and AWS-authored extensions and offers no account-owner request filter.",
            ),
        ),
        (
            "ListHostedConfigurationVersions",
            deferred(
                "Model requires ApplicationId, ConfigurationProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists configurations stored in the AppConfig hosted configuration store by version.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Retrieves the list of key-value tags assigned to the resource.",
            ),
        ),
        (
            "StartDeployment",
            deferred(
                "Model requires ApplicationId, EnvironmentId, DeploymentStrategyId, ConfigurationProfileId, ConfigurationVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts a deployment. AppConfig Agent supports deploying feature flag or free-form configuration data to specific segments or individual users during a gradual rollout. Entity-based gradual deployments ensure that once a user or…",
            ),
        ),
        (
            "StartExperimentRun",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Starts an experiment run for the specified experiment definition. An experiment run delivers treatments to the target audience and collects metrics. You can start multiple experiment runs from the same experiment definition.…",
            ),
        ),
        (
            "StopDeployment",
            deferred(
                "Model requires ApplicationId, EnvironmentId, DeploymentNumber. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a deployment. This API action works only on deployments that have a status of DEPLOYING , unless an AllowRevert parameter is supplied. If the AllowRevert parameter is supplied, the status of an in-progress deployment will…",
            ),
        ),
        (
            "StopExperimentRun",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier, Run. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Stops a running experiment. Stopping an experiment run ends audience exposure and returns users to the currently deployed feature flag configuration.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns metadata to an AppConfig resource. Tags help organize and categorize your AppConfig resources. Each tag consists of a key and an optional value, both of which you define. You can specify a maximum of 50 tags for a…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a tag key and value from an AppConfig resource.",
            ),
        ),
        (
            "UpdateAccountSettings",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is DeletionProtection, VendedMetrics; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Updates the value of the DeletionProtection parameter.",
            ),
        ),
        (
            "UpdateApplication",
            deferred(
                "Model requires ApplicationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an application.",
            ),
        ),
        (
            "UpdateConfigurationProfile",
            deferred(
                "Model requires ApplicationId, ConfigurationProfileId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a configuration profile.",
            ),
        ),
        (
            "UpdateDeploymentStrategy",
            deferred(
                "Model requires DeploymentStrategyId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a deployment strategy.",
            ),
        ),
        (
            "UpdateEnvironment",
            deferred(
                "Model requires ApplicationId, EnvironmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an environment.",
            ),
        ),
        (
            "UpdateExperimentDefinition",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an experiment definition. You can update treatments, the control, audience rules, and other properties. You cannot update an experiment definition while an experiment run is active.",
            ),
        ),
        (
            "UpdateExperimentRun",
            deferred(
                "Model requires ApplicationIdentifier, ExperimentDefinitionIdentifier, Run. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates a running experiment. Use this operation to increase audience exposure, modify treatment assignment overrides, or update the description of an active experiment run. Audience exposure can only be increased, not decreased.",
            ),
        ),
        (
            "UpdateExtension",
            deferred(
                "Model requires ExtensionIdentifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an AppConfig extension. For more information about extensions, see Extending workflows in the AppConfig User Guide .",
            ),
        ),
        (
            "UpdateExtensionAssociation",
            deferred(
                "Model requires ExtensionAssociationId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates an association. For more information about extensions and associations, see Extending workflows in the AppConfig User Guide .",
            ),
        ),
        (
            "ValidateConfiguration",
            deferred(
                "Model requires ApplicationId, ConfigurationProfileId, ConfigurationVersion. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Uses the validators in a configuration profile to validate a configuration.",
            ),
        ),
    ])
}
