use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateAppInstance",
            deferred(
                "Model requires Name, ClientRequestToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an Amazon Chime SDK messaging AppInstance under an AWS account. Only SDK messaging customers use this API. CreateAppInstance supports idempotency behavior as described in the AWS API Standard. identity",
            ),
        ),
        (
            "CreateAppInstanceAdmin",
            deferred(
                "Model requires AppInstanceAdminArn, AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Promotes an AppInstanceUser or AppInstanceBot to an AppInstanceAdmin . The promoted entity can perform the following actions. ChannelModerator actions across all channels in the AppInstance . DeleteChannelMessage actions. Only an…",
            ),
        ),
        (
            "CreateAppInstanceBot",
            deferred(
                "Model requires AppInstanceArn, ClientRequestToken, Configuration. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a bot under an Amazon Chime AppInstance . The request consists of a unique Configuration and Name for that bot.",
            ),
        ),
        (
            "CreateAppInstanceUser",
            deferred(
                "Model requires AppInstanceArn, AppInstanceUserId, Name, ClientRequestToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates a user under an Amazon Chime AppInstance . The request consists of a unique appInstanceUserId and Name for that user.",
            ),
        ),
        (
            "DeleteAppInstance",
            deferred(
                "Model requires AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an AppInstance and all associated data asynchronously.",
            ),
        ),
        (
            "DeleteAppInstanceAdmin",
            deferred(
                "Model requires AppInstanceAdminArn, AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Demotes an AppInstanceAdmin to an AppInstanceUser or AppInstanceBot . This action does not delete the user.",
            ),
        ),
        (
            "DeleteAppInstanceBot",
            deferred(
                "Model requires AppInstanceBotArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an AppInstanceBot .",
            ),
        ),
        (
            "DeleteAppInstanceUser",
            deferred(
                "Model requires AppInstanceUserArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes an AppInstanceUser .",
            ),
        ),
        (
            "DeregisterAppInstanceUserEndpoint",
            deferred(
                "Model requires AppInstanceUserArn, EndpointId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deregisters an AppInstanceUserEndpoint .",
            ),
        ),
        ("DescribeAppInstance", denied(DenyReason::Content)),
        (
            "DescribeAppInstanceAdmin",
            deferred(
                "Model requires AppInstanceAdminArn, AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the full details of an AppInstanceAdmin .",
            ),
        ),
        ("DescribeAppInstanceBot", denied(DenyReason::Content)),
        ("DescribeAppInstanceUser", denied(DenyReason::Content)),
        (
            "DescribeAppInstanceUserEndpoint",
            deferred(
                "Model requires AppInstanceUserArn, EndpointId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the full details of an AppInstanceUserEndpoint .",
            ),
        ),
        (
            "GetAppInstanceRetentionSettings",
            deferred(
                "Model requires AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the retention settings for an AppInstance .",
            ),
        ),
        (
            "ListAppInstanceAdmins",
            deferred(
                "Model requires AppInstanceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns a list of the administrators in the AppInstance .",
            ),
        ),
        ("ListAppInstanceBots", denied(DenyReason::Content)),
        (
            "ListAppInstanceUserEndpoints",
            deferred(
                "Model requires AppInstanceUserArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists all the AppInstanceUserEndpoints created under a single AppInstanceUser .",
            ),
        ),
        ("ListAppInstanceUsers", denied(DenyReason::Content)),
        ("ListAppInstances", denied(DenyReason::Content)),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceARN. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists the tags applied to an Amazon Chime SDK identity resource.",
            ),
        ),
        (
            "PutAppInstanceRetentionSettings",
            deferred(
                "Model requires AppInstanceArn, AppInstanceRetentionSettings. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the amount of time in days that a given AppInstance retains data.",
            ),
        ),
        (
            "PutAppInstanceUserExpirationSettings",
            deferred(
                "Model requires AppInstanceUserArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the number of days before the AppInstanceUser is automatically deleted. A background process deletes expired AppInstanceUsers within 6 hours of expiration. Actual deletion times may vary. Expired AppInstanceUsers that have…",
            ),
        ),
        (
            "RegisterAppInstanceUserEndpoint",
            deferred(
                "Model requires AppInstanceUserArn, Type, ResourceArn, EndpointAttributes, ClientRequestToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Registers an endpoint under an Amazon Chime AppInstanceUser . The endpoint receives messages for a user. For push notifications, the endpoint is a mobile device used to receive mobile push notifications for a user.",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires ResourceARN, Tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Applies the specified tags to the specified Amazon Chime SDK identity resource.",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires ResourceARN, TagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the specified tags from the specified Amazon Chime SDK identity resource.",
            ),
        ),
        (
            "UpdateAppInstance",
            deferred(
                "Model requires AppInstanceArn, Name, Metadata. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates AppInstance metadata.",
            ),
        ),
        (
            "UpdateAppInstanceBot",
            deferred(
                "Model requires AppInstanceBotArn, Name, Metadata. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the name and metadata of an AppInstanceBot .",
            ),
        ),
        (
            "UpdateAppInstanceUser",
            deferred(
                "Model requires AppInstanceUserArn, Name, Metadata. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the details of an AppInstanceUser . You can update names and metadata.",
            ),
        ),
        (
            "UpdateAppInstanceUserEndpoint",
            deferred(
                "Model requires AppInstanceUserArn, EndpointId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the details of an AppInstanceUserEndpoint . You can update the name and AllowMessage values.",
            ),
        ),
    ])
}
