use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AssociateCustomDomain",
            deferred(
                "Model requires ServiceArn, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associate your own domain name with the App Runner subdomain URL of your App Runner service. After you call AssociateCustomDomain and receive a successful response, use the information in the CustomDomain record that's returned…",
            ),
        ),
        (
            "CreateAutoScalingConfiguration",
            deferred(
                "Model requires AutoScalingConfigurationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create an App Runner automatic scaling configuration resource. App Runner requires this resource when you create or update App Runner services and you require non-default auto scaling settings. You can share an auto scaling…",
            ),
        ),
        (
            "CreateConnection",
            deferred(
                "Model requires ConnectionName, ProviderType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create an App Runner connection resource. App Runner requires a connection resource when you create App Runner services that access private repositories from certain third-party providers. You can share a connection across…",
            ),
        ),
        (
            "CreateObservabilityConfiguration",
            deferred(
                "Model requires ObservabilityConfigurationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create an App Runner observability configuration resource. App Runner requires this resource when you create or update App Runner services and you want to enable non-default observability features. You can share an observability…",
            ),
        ),
        (
            "CreateService",
            deferred(
                "Model requires ServiceName, SourceConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create an App Runner service. After the service is created, the action also automatically starts a deployment. This is an asynchronous operation. On a successful call, you can use the returned OperationId and the ListOperations…",
            ),
        ),
        (
            "CreateVpcConnector",
            deferred(
                "Model requires VpcConnectorName, Subnets. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create an App Runner VPC connector resource. App Runner requires this resource when you want to associate your App Runner service to a custom Amazon Virtual Private Cloud (Amazon VPC).",
            ),
        ),
        (
            "CreateVpcIngressConnection",
            deferred(
                "Model requires ServiceArn, VpcIngressConnectionName, IngressVpcConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Create an App Runner VPC Ingress Connection resource. App Runner requires this resource when you want to associate your App Runner service with an Amazon VPC endpoint.",
            ),
        ),
        (
            "DeleteAutoScalingConfiguration",
            deferred(
                "Model requires AutoScalingConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete an App Runner automatic scaling configuration resource. You can delete a top level auto scaling configuration, a specific revision of one, or all revisions associated with the top level configuration. You can't delete the…",
            ),
        ),
        (
            "DeleteConnection",
            deferred(
                "Model requires ConnectionArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete an App Runner connection. You must first ensure that there are no running App Runner services that use this connection. If there are any, the DeleteConnection action fails.",
            ),
        ),
        (
            "DeleteObservabilityConfiguration",
            deferred(
                "Model requires ObservabilityConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete an App Runner observability configuration resource. You can delete a specific revision or the latest active revision. You can't delete a configuration that's used by one or more App Runner services.",
            ),
        ),
        (
            "DeleteService",
            deferred(
                "Model requires ServiceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete an App Runner service. This is an asynchronous operation. On a successful call, you can use the returned OperationId and the ListOperations call to track the operation's progress. Make sure that you don't have any active…",
            ),
        ),
        (
            "DeleteVpcConnector",
            deferred(
                "Model requires VpcConnectorArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete an App Runner VPC connector resource. You can't delete a connector that's used by one or more App Runner services.",
            ),
        ),
        (
            "DeleteVpcIngressConnection",
            deferred(
                "Model requires VpcIngressConnectionArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Delete an App Runner VPC Ingress Connection resource that's associated with an App Runner service. The VPC Ingress Connection must be in one of the following states to be deleted: AVAILABLE FAILED_CREATION FAILED_UPDATE…",
            ),
        ),
        (
            "DescribeAutoScalingConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("AutoScalingConfigurationArn")?,
                    Binding::From {
                        operation: name("ListAutoScalingConfigurations")?,
                        path: path("AutoScalingConfigurationArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("AutoScalingConfigurationArn")?],
                ..read(
                    "AutoScalingConfiguration",
                    Identity::Arn(path("AutoScalingConfigurationArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DescribeCustomDomains",
            deferred(
                "Model requires ServiceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Return a description of custom domain names that are associated with an App Runner service.",
            ),
        ),
        (
            "DescribeObservabilityConfiguration",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("ObservabilityConfigurationArn")?,
                    Binding::From {
                        operation: name("ListObservabilityConfigurations")?,
                        path: path("ObservabilityConfigurationArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("ObservabilityConfigurationArn")?],
                ..read(
                    "ObservabilityConfiguration",
                    Identity::Arn(path("ObservabilityConfigurationArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        ("DescribeService", denied(DenyReason::Secret)),
        (
            "DescribeVpcConnector",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("VpcConnectorArn")?,
                    Binding::From {
                        operation: name("ListVpcConnectors")?,
                        path: path("VpcConnectorArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("VpcConnectorArn")?],
                ..read(
                    "VpcConnector",
                    Identity::Arn(path("VpcConnectorArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "DescribeVpcIngressConnection",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("VpcIngressConnectionArn")?,
                    Binding::From {
                        operation: name("ListVpcIngressConnections")?,
                        path: path("VpcIngressConnectionArn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("VpcIngressConnectionArn")?],
                ..read(
                    "VpcIngressConnection",
                    Identity::Arn(path("VpcIngressConnectionArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "DisassociateCustomDomain",
            deferred(
                "Model requires ServiceArn, DomainName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociate a custom domain name from an App Runner service. Certificates tracking domain validity are associated with a custom domain and are stored in AWS Certificate Manager (ACM) . These certificates aren't deleted as part…",
            ),
        ),
        (
            "ListAutoScalingConfigurations",
            OperationOverlay {
                fixed: [(path("LatestOnly")?, FixedValue::Boolean(false))]
                    .into_iter()
                    .collect(),
                queryable: vec![path("AutoScalingConfigurationArn")?],
                ..read(
                    "AutoScalingConfigurationSummaryList",
                    Identity::Arn(path("AutoScalingConfigurationArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListConnections",
            OperationOverlay {
                queryable: vec![path("ConnectionArn")?],
                ..read(
                    "ConnectionSummaryList",
                    Identity::Arn(path("ConnectionArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListObservabilityConfigurations",
            OperationOverlay {
                fixed: [(path("LatestOnly")?, FixedValue::Boolean(false))]
                    .into_iter()
                    .collect(),
                queryable: vec![path("ObservabilityConfigurationArn")?],
                ..read(
                    "ObservabilityConfigurationSummaryList",
                    Identity::Arn(path("ObservabilityConfigurationArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListOperations",
            deferred(
                "Model requires ServiceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Return a list of operations that occurred on an App Runner service. The resulting list of OperationSummary objects is sorted in reverse chronological order. The first object on the list represents the last started operation.",
            ),
        ),
        (
            "ListServices",
            OperationOverlay {
                queryable: vec![path("ServiceArn")?],
                ..read(
                    "ServiceSummaryList",
                    Identity::Arn(path("ServiceArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListServicesForAutoScalingConfiguration",
            deferred(
                "Model requires AutoScalingConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the associated App Runner services using an auto scaling configuration.",
            ),
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: List tags that are associated with for an App Runner resource. The response contains a list of tag key-value pairs.",
            ),
        ),
        (
            "ListVpcConnectors",
            OperationOverlay {
                queryable: vec![path("VpcConnectorArn")?],
                ..read(
                    "VpcConnectors",
                    Identity::Arn(path("VpcConnectorArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_2() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "ListVpcIngressConnections",
            OperationOverlay {
                queryable: vec![path("VpcIngressConnectionArn")?],
                ..read(
                    "VpcIngressConnectionSummaryList",
                    Identity::Arn(path("VpcIngressConnectionArn")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "PauseService",
            deferred(
                "Model requires ServiceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Pause an active App Runner service. App Runner reduces compute capacity for the service to zero and loses state (for example, ephemeral storage is removed). This is an asynchronous operation. On a successful call, you can use the…",
            ),
        ),
        (
            "ResumeService",
            deferred(
                "Model requires ServiceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Resume an active App Runner service. App Runner provisions compute capacity for the service. This is an asynchronous operation. On a successful call, you can use the returned OperationId and the ListOperations call to track the…",
            ),
        ),
        (
            "StartDeployment",
            deferred(
                "Model requires ServiceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiate a manual deployment of the latest commit in a source code repository or the latest image in a source image repository to an App Runner service. For a source code repository, App Runner retrieves the commit and builds a…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Add tags to, or update the tag values of, an App Runner resource. A tag is a key-value pair.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Remove tags from an App Runner resource.",
            ),
        ),
        (
            "UpdateDefaultAutoScalingConfiguration",
            deferred(
                "Model requires AutoScalingConfigurationArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update an auto scaling configuration to be the default. The existing default auto scaling configuration will be set to non-default automatically.",
            ),
        ),
        (
            "UpdateService",
            deferred(
                "Model requires ServiceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update an App Runner service. You can update the source configuration and instance configuration of the service. You can also update the ARN of the auto scaling configuration resource that's associated with the service. However,…",
            ),
        ),
        (
            "UpdateVpcIngressConnection",
            deferred(
                "Model requires VpcIngressConnectionArn, IngressVpcConfiguration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Update an existing App Runner VPC Ingress Connection resource. The VPC Ingress Connection must be in one of the following states to be updated: AVAILABLE FAILED_CREATION FAILED_UPDATE",
            ),
        ),
    ])
}
