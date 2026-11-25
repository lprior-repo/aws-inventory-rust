use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "BulkPublish",
            deferred(
                "Model requires IdentityPoolId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Initiates a bulk publish of all existing datasets for an Identity Pool to the configured stream. Customers are limited to one successful bulk publish per 24 hours. Bulk publish is an asynchronous request, customers can see the…",
            ),
        ),
        (
            "DeleteDataset",
            deferred(
                "Model requires IdentityPoolId, IdentityId, DatasetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specific dataset. The dataset will be deleted permanently, and the action can't be undone. Datasets that this dataset was merged with will no longer report the merge. Any subsequent operation on this dataset will…",
            ),
        ),
        (
            "DescribeDataset",
            deferred(
                "Model requires IdentityPoolId, IdentityId, DatasetName. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets meta data about a dataset by identity and dataset name. With Amazon Cognito Sync, each identity has access only to its own data. Thus, the credentials used to make this API call need to have access to the identity data. This…",
            ),
        ),
        (
            "DescribeIdentityPoolUsage",
            deferred(
                "Model requires IdentityPoolId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets usage details (for example, data storage) about a particular identity pool. This API can only be called with developer credentials. You cannot call this API with the temporary user credentials provided by Cognito Identity.…",
            ),
        ),
        (
            "DescribeIdentityUsage",
            deferred(
                "Model requires IdentityPoolId, IdentityId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets usage information for an identity, including number of datasets and data usage. This API can be called with temporary user credentials provided by Cognito Identity or with developer credentials. DescribeIdentityUsage The…",
            ),
        ),
        (
            "GetBulkPublishDetails",
            deferred(
                "Model requires IdentityPoolId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Get the status of the last BulkPublish operation for an identity pool. This API can only be called with developer credentials. You cannot call this API with the temporary user credentials provided by Cognito Identity.",
            ),
        ),
        (
            "GetCognitoEvents",
            deferred(
                "Model requires IdentityPoolId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the events and the corresponding Lambda functions associated with an identity pool. This API can only be called with developer credentials. You cannot call this API with the temporary user credentials provided by Cognito…",
            ),
        ),
        (
            "GetIdentityPoolConfiguration",
            deferred(
                "Model requires IdentityPoolId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Gets the configuration settings of an identity pool. This API can only be called with developer credentials. You cannot call this API with the temporary user credentials provided by Cognito Identity. GetIdentityPoolConfiguration…",
            ),
        ),
        (
            "ListDatasets",
            deferred(
                "Model requires IdentityPoolId, IdentityId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Lists datasets for an identity. With Amazon Cognito Sync, each identity has access only to its own data. Thus, the credentials used to make this API call need to have access to the identity data. ListDatasets can be called with…",
            ),
        ),
        (
            "ListIdentityPoolUsage",
            OperationOverlay {
                queryable: vec![path("IdentityPoolId")?],
                ..read(
                    "IdentityPoolUsages",
                    Identity::Key(vec![path("IdentityPoolId")?]),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        ("ListRecords", denied(DenyReason::Content)),
        (
            "RegisterDevice",
            deferred(
                "Model requires IdentityPoolId, IdentityId, Platform, Token. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Registers a device to receive push sync notifications. This API can only be called with temporary credentials provided by Cognito Identity. You cannot call this API with developer credentials. RegisterDevice The following…",
            ),
        ),
        (
            "SetCognitoEvents",
            deferred(
                "Model requires IdentityPoolId, Events. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the AWS Lambda function for a given event type for an identity pool. This request only updates the key/value pair specified. Other key/values pairs are not updated. To remove a key value pair, pass a empty value for the…",
            ),
        ),
        (
            "SetIdentityPoolConfiguration",
            deferred(
                "Model requires IdentityPoolId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Sets the necessary configuration for push sync. This API can only be called with developer credentials. You cannot call this API with the temporary user credentials provided by Cognito Identity. SetIdentityPoolConfiguration The…",
            ),
        ),
        (
            "SubscribeToDataset",
            deferred(
                "Model requires IdentityPoolId, IdentityId, DatasetName, DeviceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Subscribes to receive notifications when a dataset is modified by another device. This API can only be called with temporary credentials provided by Cognito Identity. You cannot call this API with developer credentials.…",
            ),
        ),
        (
            "UnsubscribeFromDataset",
            deferred(
                "Model requires IdentityPoolId, IdentityId, DatasetName, DeviceId. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Unsubscribes from receiving notifications when a dataset is modified by another device. This API can only be called with temporary credentials provided by Cognito Identity. You cannot call this API with developer credentials.…",
            ),
        ),
        (
            "UpdateRecords",
            deferred(
                "Model requires IdentityPoolId, IdentityId, DatasetName, SyncSessionToken. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Posts updates to records and adds and deletes records for a dataset and user. The sync count in the record patch is your last known sync count for that record. The server will reject an UpdateRecords request with a…",
            ),
        ),
    ])
}
