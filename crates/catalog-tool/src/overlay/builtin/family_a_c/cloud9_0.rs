use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateEnvironmentEC2",
            deferred(
                "Model requires name, instanceType, imageId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Cloud9 development environment, launches an Amazon Elastic Compute Cloud (Amazon EC2) instance, and then connects from the instance to the environment. Cloud9 is no longer available to new customers. Existing customers…",
            ),
        ),
        (
            "CreateEnvironmentMembership",
            deferred(
                "Model requires environmentId, userArn, permissions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds an environment member to an Cloud9 development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
        (
            "DeleteEnvironment",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an Cloud9 development environment. If an Amazon EC2 instance is connected to the environment, also terminates the instance. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use…",
            ),
        ),
        (
            "DeleteEnvironmentMembership",
            deferred(
                "Model requires environmentId, userArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an environment member from a development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
        (
            "DescribeEnvironmentMemberships",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is memberships; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: Gets information about environment members for an Cloud9 development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
        (
            "DescribeEnvironmentStatus",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets status information for an Cloud9 development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
        (
            "DescribeEnvironments",
            deferred(
                "Model requires environmentIds. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets information about Cloud9 development environments. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
        (
            "ListEnvironments",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "environmentIds",
                    Identity::Key(vec![path("$")?]),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets a list of the tags associated with an Cloud9 development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Adds tags to an Cloud9 development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\" Tags that you add to an Cloud9 environment by…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes tags from an Cloud9 development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
        (
            "UpdateEnvironment",
            deferred(
                "Model requires environmentId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes the settings of an existing Cloud9 development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
        (
            "UpdateEnvironmentMembership",
            deferred(
                "Model requires environmentId, userArn, permissions. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Changes the settings of an existing environment member for an Cloud9 development environment. Cloud9 is no longer available to new customers. Existing customers of Cloud9 can continue to use the service as normal. Learn more\"",
            ),
        ),
    ])
}
