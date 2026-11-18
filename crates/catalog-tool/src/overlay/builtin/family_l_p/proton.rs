// Pinned Smithy source: .local/aws-models/models/proton/service/2020-07-20/proton-2020-07-20.json
use super::policy::{Group, Id, Resource, compile};
use crate::overlay::Scope;
use crate::{
    classes::DenyReason,
    overlay::{NameError, OverlayClass, ServiceOverlay},
};

const GROUPS: &[Group] = &[
    Group {
        operations: &[
            "AcceptEnvironmentAccountConnection",
            "CancelComponentDeployment",
            "CancelEnvironmentDeployment",
            "CancelServiceInstanceDeployment",
            "CancelServicePipelineDeployment",
            "CreateComponent",
            "CreateEnvironment",
            "CreateEnvironmentAccountConnection",
            "CreateEnvironmentTemplate",
            "CreateEnvironmentTemplateVersion",
            "CreateRepository",
            "CreateService",
            "CreateServiceInstance",
            "CreateServiceSyncConfig",
            "CreateServiceTemplate",
            "CreateServiceTemplateVersion",
            "CreateTemplateSyncConfig",
            "DeleteComponent",
            "DeleteDeployment",
            "DeleteEnvironment",
            "DeleteEnvironmentAccountConnection",
            "DeleteEnvironmentTemplate",
            "DeleteEnvironmentTemplateVersion",
            "DeleteRepository",
            "DeleteService",
            "DeleteServiceSyncConfig",
            "DeleteServiceTemplate",
            "DeleteServiceTemplateVersion",
            "DeleteTemplateSyncConfig",
            "NotifyResourceDeploymentStatusChange",
            "RejectEnvironmentAccountConnection",
            "TagResource",
            "UntagResource",
            "UpdateAccountSettings",
            "UpdateComponent",
            "UpdateEnvironment",
            "UpdateEnvironmentAccountConnection",
            "UpdateEnvironmentTemplate",
            "UpdateEnvironmentTemplateVersion",
            "UpdateService",
            "UpdateServiceInstance",
            "UpdateServicePipeline",
            "UpdateServiceSyncBlocker",
            "UpdateServiceSyncConfig",
            "UpdateServiceTemplate",
            "UpdateServiceTemplateVersion",
            "UpdateTemplateSyncConfig",
        ],
        class: OverlayClass::Deny(DenyReason::Mutating),
        reason: None,
    },
    Group {
        operations: &[
            "GetAccountSettings",
            "GetResourcesSummary",
            "ListDeployments",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "The authorized metadata summaries are collected separately; this operation has no complete resource identity, safety and discovery contract.",
        ),
    },
    Group {
        operations: &[
            "GetComponent",
            "GetEnvironment",
            "GetEnvironmentTemplate",
            "GetService",
            "GetServiceTemplate",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (name); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetDeployment", "GetEnvironmentAccountConnection"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (id); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetEnvironmentTemplateVersion", "GetServiceTemplateVersion"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (templateName, majorVersion, minorVersion); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetRepository"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (provider, name); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetRepositorySyncStatus"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (repositoryName, repositoryProvider, branch, syncType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetServiceInstance"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (name, serviceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetServiceInstanceSyncStatus",
            "ListServiceInstanceProvisionedResources",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (serviceName, serviceInstanceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "GetServiceSyncBlockerSummary",
            "GetServiceSyncConfig",
            "ListServicePipelineOutputs",
            "ListServicePipelineProvisionedResources",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (serviceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTemplateSyncConfig"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (templateName, templateType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["GetTemplateSyncStatus"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (templateName, templateType, templateVersion); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListComponentOutputs", "ListComponentProvisionedResources"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (componentName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &[
            "ListEnvironmentOutputs",
            "ListEnvironmentProvisionedResources",
        ],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (environmentName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListRepositorySyncDefinitions"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (repositoryName, repositoryProvider, syncType); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListServiceInstanceOutputs"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (serviceInstanceName, serviceName); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
    Group {
        operations: &["ListTagsForResource"],
        class: OverlayClass::Defer,
        reason: Some(
            "Expansion requires request context (resourceArn); this operation has no complete metadata-only dependency/output contract in the embedded planner.",
        ),
    },
];

const RESOURCES: &[Resource] = &[
    // List components with summary data. You can filter the result list by environment, service, or a single service instance. For more information about components, see Proton component
    Resource {
        operation: "ListComponents",
        items: "components",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "name",
            "environmentName",
            "serviceName",
            "serviceInstanceName",
            "deploymentStatus",
            "lastAttemptedDeploymentId",
            "lastSucceededDeploymentId",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List environment templates.
    Resource {
        operation: "ListEnvironmentTemplates",
        items: "templates",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "displayName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List environments with detail data summaries.
    Resource {
        operation: "ListEnvironments",
        items: "environments",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "name",
            "templateName",
            "deploymentStatus",
            "protonServiceRoleArn",
            "environmentAccountConnectionId",
            "environmentAccountId",
            "componentRoleArn",
            "lastAttemptedDeploymentId",
            "lastSucceededDeploymentId",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List linked repositories with detail data.
    Resource {
        operation: "ListRepositories",
        items: "repositories",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "connectionArn"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List service instances with summary data. This action lists service instances of all services in the Amazon Web Services account.
    Resource {
        operation: "ListServiceInstances",
        items: "serviceInstances",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "name",
            "serviceName",
            "environmentName",
            "templateName",
            "deploymentStatus",
            "lastAttemptedDeploymentId",
            "lastSucceededDeploymentId",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List service templates with detail data.
    Resource {
        operation: "ListServiceTemplates",
        items: "templates",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "displayName"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // List services with summaries of detail data.
    Resource {
        operation: "ListServices",
        items: "services",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &["arn", "name", "templateName", "status"],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    Resource {
        operation: "ListEnvironmentAccountConnections",
        items: "environmentAccountConnections",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "id",
            "managementAccountId",
            "environmentAccountId",
            "environmentName",
            "status",
        ],
        tags: None,
        bindings: &[],
        fixed: &[],
    },
    // Major discovery feeds a finite minor stage; every returned minor uses its native version ARN.
    Resource {
        operation: "ListEnvironmentTemplateVersions",
        items: "templateVersions",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "templateName",
            "majorVersion",
            "minorVersion",
            "status",
        ],
        tags: None,
        bindings: &[("templateName", "ListEnvironmentTemplates", "name")],
        fixed: &[],
    },
    Resource {
        operation: "ListServiceTemplateVersions",
        items: "templateVersions",
        id: Id::Arn("arn"),
        scope: Scope::Region,
        tokens: Some(("nextToken", "nextToken")),
        query: &[
            "arn",
            "templateName",
            "majorVersion",
            "minorVersion",
            "status",
        ],
        tags: None,
        bindings: &[("templateName", "ListServiceTemplates", "name")],
        fixed: &[],
    },
];

pub(super) fn overlay() -> Result<ServiceOverlay, NameError> {
    use super::super::{name, path};
    use crate::overlay::{Binding, ContextSource, Emission, ExecutionBranch, FixedValue};
    let mut service = compile("proton", GROUPS, RESOURCES)?;
    if let Some(plan) = service
        .operations
        .get_mut(&name("ListEnvironmentAccountConnections")?)
    {
        plan.branches = ["MANAGEMENT_ACCOUNT", "ENVIRONMENT_ACCOUNT"]
            .into_iter()
            .map(|role| {
                Ok(ExecutionBranch {
                    discriminator: role.to_owned(),
                    fixed: [(path("requestedBy")?, FixedValue::String(role.to_owned()))]
                        .into_iter()
                        .collect(),
                    bind: Default::default(),
                    emission: None,
                    row_policy: None,
                    scope: Scope::Region,
                    request_region: None,
                })
            })
            .collect::<Result<_, NameError>>()?;
    }
    for (operation, templates) in [
        (
            "ListEnvironmentTemplateVersions",
            "ListEnvironmentTemplates",
        ),
        ("ListServiceTemplateVersions", "ListServiceTemplates"),
    ] {
        if let Some(plan) = service.operations.get_mut(&name(operation)?) {
            plan.bind.clear();
            plan.branches = vec![
                ExecutionBranch {
                    discriminator: "majors".to_owned(),
                    fixed: Default::default(),
                    bind: [(
                        path("templateName")?,
                        Binding::From {
                            operation: name(templates)?,
                            path: path("name")?,
                        },
                    )]
                    .into_iter()
                    .collect(),
                    emission: Some(Emission::DiscoveryOnly),
                    row_policy: None,
                    scope: Scope::Region,
                    request_region: None,
                },
                ExecutionBranch {
                    discriminator: "minors".to_owned(),
                    fixed: Default::default(),
                    bind: [
                        (
                            path("templateName")?,
                            Binding::ContextFromBranch {
                                operation: name(operation)?,
                                branch: "majors".to_owned(),
                                source: ContextSource::Request,
                                path: path("templateName")?,
                            },
                        ),
                        (
                            path("majorVersion")?,
                            Binding::FromBranch {
                                operation: name(operation)?,
                                branch: "majors".to_owned(),
                                path: path("majorVersion")?,
                            },
                        ),
                    ]
                    .into_iter()
                    .collect(),
                    emission: None,
                    row_policy: None,
                    scope: Scope::Region,
                    request_region: None,
                },
            ];
        }
    }
    Ok(service)
}
