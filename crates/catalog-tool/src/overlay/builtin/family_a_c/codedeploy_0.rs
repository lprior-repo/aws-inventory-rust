// Pinned source: .local/aws-models/models/codedeploy/service/2014-10-06/codedeploy-2014-10-06.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;
use crate::overlay::IdentityField;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        ("AddTagsToOnPremisesInstances", deferred("Model requires tags, instanceNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds tags to on-premises instances.")),
        ("BatchGetApplicationRevisions", deferred("Model requires applicationName, revisions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about one or more application revisions. The maximum number of application revisions that can be returned is 25.")),
        ("BatchGetApplications", deferred("Model requires applicationNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about one or more applications. The maximum number of applications that can be returned is 100.")),
        ("BatchGetDeploymentGroups", deferred("Model requires applicationName, deploymentGroupNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about one or more deployment groups.")),
        ("BatchGetDeploymentInstances", deferred("Model requires deploymentId, instanceIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This method works, but is deprecated. Use BatchGetDeploymentTargets instead. Returns an array of one or more instances associated with a deployment. This method works with EC2/On-premises and Lambda compute platforms. The newer…")),
        ("BatchGetDeploymentTargets", deferred("Model requires deploymentId, targetIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an array of one or more targets associated with a deployment. This method works with all compute types and should be used instead of the deprecated BatchGetDeploymentInstances . The maximum number of targets that can be…")),
        ("BatchGetDeployments", deferred("Model requires deploymentIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about one or more deployments. The maximum number of deployments that can be returned is 25.")),
        ("BatchGetOnPremisesInstances", deferred("Model requires instanceNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about one or more on-premises instances. The maximum number of on-premises instances that can be returned is 25.")),
        ("ContinueDeployment", deferred("No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: For a blue/green deployment, starts the process of rerouting traffic from instances in the original environment to instances in the replacement environment without waiting for a specified wait time to elapse. (Traffic rerouting,…")),
        ("CreateApplication", deferred("Model requires applicationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an application.")),
        ("CreateDeployment", deferred("Model requires applicationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deploys an application revision through the specified deployment group.")),
        ("CreateDeploymentConfig", deferred("Model requires deploymentConfigName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a deployment configuration.")),
        ("CreateDeploymentGroup", deferred("Model requires applicationName, deploymentGroupName, serviceRoleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a deployment group to which application revisions are deployed.")),
        ("DeleteApplication", deferred("Model requires applicationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an application.")),
        ("DeleteDeploymentConfig", deferred("Model requires deploymentConfigName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a deployment configuration. A deployment configuration cannot be deleted if it is currently in use. Predefined configurations cannot be deleted.")),
        ("DeleteDeploymentGroup", deferred("Model requires applicationName, deploymentGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes a deployment group.")),
        ("DeleteGitHubAccountToken", deferred("No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Deletes a GitHub account connection.")),
        ("DeleteResourcesByExternalId", deferred("No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Deletes resources linked to an external ID. This action only applies if you have configured blue/green deployments through CloudFormation. It is not necessary to call this action directly. CloudFormation calls it on your behalf…")),
        ("DeregisterOnPremisesInstance", deferred("Model requires instanceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deregisters an on-premises instance.")),
        // Model: Gets information about an application.
        ("GetApplication", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("applicationName")?, Binding::From { operation: name("ListApplications")?, path: path("$")? })].into_iter().collect(),
            queryable: vec![path("applicationName")?],
            ..read("application", Identity::Key(vec![path("applicationName")?]), Scope::Region, Paginator::None)?
        }),
        ("GetApplicationRevision", deferred("Model requires applicationName, revision. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about an application revision.")),
        ("GetDeployment", deferred("Model requires deploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a deployment. The content property of the appSpecContent object in the returned revision is always null. Use GetApplicationRevision and the sha256 property of the returned appSpecContent object to get the…")),
        // Model: Gets information about a deployment configuration.
        ("GetDeploymentConfig", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("deploymentConfigName")?, Binding::From { operation: name("ListDeploymentConfigs")?, path: path("$")? })].into_iter().collect(),
            queryable: vec![path("deploymentConfigName")?],
            ..read("deploymentConfigInfo", Identity::Key(vec![path("deploymentConfigName")?]), Scope::Region, Paginator::None)?
        }),
        ("GetDeploymentGroup", deferred("Model requires applicationName, deploymentGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about a deployment group.")),
        ("GetDeploymentInstance", deferred("Model requires deploymentId, instanceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about an instance as part of a deployment.")),
        ("GetDeploymentTarget", deferred("Model requires deploymentId, targetId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns information about a deployment target.")),
        ("GetOnPremisesInstance", deferred("Model requires instanceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about an on-premises instance.")),
        ("ListApplicationRevisions", deferred("Model requires applicationName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists information about revisions for an application.")),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: Lists the applications registered with the user or Amazon Web Services account.
        ("ListApplications", OperationOverlay {
            queryable: vec![path("$")?],
            ..read("applications", Identity::Key(vec![path("$")?]), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        // Model: Lists the deployment configurations with the user or Amazon Web Services account.
        ("ListDeploymentConfigs", OperationOverlay {
            queryable: vec![path("$")?],
            ..read("deploymentConfigsList", Identity::Key(vec![path("$")?]), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        // Reviewed full response: application name, deployment-group names and continuation; no revision contents or credentials.
        ("ListDeploymentGroups", OperationOverlay {
            class: Some(OverlayClass::Expand),
            bind: [(path("applicationName")?, Binding::From { operation: name("ListApplications")?, path: path("$")? })].into_iter().collect(),
            queryable: vec![path("$")?],
            ..read("deploymentGroups", Identity::CorrelatedKey(vec![
                IdentityField::Request(path("applicationName")?),
                IdentityField::Selected(path("$")?),
            ]), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        ("ListDeploymentInstances", deferred("Model requires deploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: The newer BatchGetDeploymentTargets should be used instead because it works with all compute types. ListDeploymentInstances throws an exception if it is used with a compute platform other than EC2/On-premises or Lambda. Lists the…")),
        ("ListDeploymentTargets", deferred("Model requires deploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns an array of target IDs that are associated a deployment.")),
        // Model: Lists the deployments in a deployment group for an application registered with the user or Amazon Web Services account.
        ("ListDeployments", OperationOverlay {
            queryable: vec![path("$")?],
            ..read("deployments", Identity::Key(vec![path("$")?]), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        ("ListGitHubAccountTokenNames", deferred("No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Lists the names of stored connections to GitHub accounts.")),
        // Model: Gets a list of names for one or more on-premises instances. Unless otherwise specified, both registered and deregistered on-premises instance names are listed. To list only registered or deregistered…
        ("ListOnPremisesInstances", OperationOverlay {
            queryable: vec![path("$")?],
            ..read("instanceNames", Identity::Key(vec![path("$")?]), Scope::Region, token("nextToken", "nextToken")?)?
        }),
        ("ListTagsForResource", deferred("Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of tags for the resource identified by a specified Amazon Resource Name (ARN). Tags are used to organize and categorize your CodeDeploy resources.")),
        ("PutLifecycleEventHookExecutionStatus", deferred("No safe resource collection plan is approved for this modeled operation. Its output is lifecycleEventHookExecutionId; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Sets the result of a Lambda validation function. The function validates lifecycle hooks during a deployment that uses the Lambda or Amazon ECS compute platform. For Lambda deployments, the available lifecycle hooks are…")),
        ("RegisterApplicationRevision", deferred("Model requires applicationName, revision. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Registers with CodeDeploy a revision for the specified application.")),
        ("RegisterOnPremisesInstance", deferred("Model requires instanceName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Registers an on-premises instance. Only one IAM ARN (an IAM session ARN or IAM user ARN) is supported in the request. You cannot use both.")),
        ("RemoveTagsFromOnPremisesInstances", deferred("Model requires tags, instanceNames. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from one or more on-premises instances.")),
        ("SkipWaitTimeForInstanceTermination", deferred("No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: In a blue/green deployment, overrides any specified wait time and starts terminating instances immediately after the traffic routing is complete.")),
        ("StopDeployment", deferred("Model requires deploymentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Attempts to stop an ongoing deployment.")),
        ("TagResource", deferred("Model requires ResourceArn, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Associates the list of tags in the input Tags parameter with the resource identified by the ResourceArn input parameter.")),
        ("UntagResource", deferred("Model requires ResourceArn, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Disassociates a resource from a list of tags. The resource is identified by the ResourceArn input parameter. The tags are identified by the list of keys in the TagKeys input parameter.")),
        ("UpdateApplication", deferred("No safe resource collection plan is approved for this modeled operation. Its output is not an inventory record; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Changes the name of an application.")),
        ("UpdateDeploymentGroup", deferred("Model requires applicationName, currentDeploymentGroupName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes information about a deployment group.")),
    ])
}
