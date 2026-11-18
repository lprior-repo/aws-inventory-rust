// Pinned source: .local/aws-models/models/aiops/service/2018-05-10/aiops-2018-05-10.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "CreateInvestigationGroup",
            deferred(
                "Model requires name, roleArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an investigation group in your account. Creating an investigation group is a one-time setup task for each Region in your account. It is a necessary task to be able to perform investigations. Settings in the investigation…",
            ),
        ),
        (
            "DeleteInvestigationGroup",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Deletes the specified investigation group from your account. You can currently have one investigation group per Region in your account. After you delete an investigation group, you can later create a new investigation group in…",
            ),
        ),
        (
            "DeleteInvestigationGroupPolicy",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes the IAM resource policy from being associated with the investigation group that you specify.",
            ),
        ),
        // Model: Returns the configuration information for the specified investigation group.
        (
            "GetInvestigationGroup",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("identifier")?,
                    Binding::From {
                        operation: name("ListInvestigationGroups")?,
                        path: path("arn")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("arn")?],
                ..read(
                    "$",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        (
            "GetInvestigationGroupPolicy",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Returns the JSON of the IAM resource policy associated with the specified investigation group in a string. For example,…",
            ),
        ),
        // Model: Returns the ARN and name of each investigation group in the account.
        (
            "ListInvestigationGroups",
            OperationOverlay {
                queryable: vec![path("arn")?],
                ..read(
                    "investigationGroups",
                    Identity::Arn(path("arn")?),
                    Scope::Region,
                    token("nextToken", "nextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires resourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Displays the tags associated with a CloudWatch investigations resource. Currently, investigation groups support tagging.",
            ),
        ),
        (
            "PutInvestigationGroupPolicy",
            deferred(
                "Model requires identifier, policy. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Creates an IAM resource policy and assigns it to the specified investigation group. If you create your investigation group with CreateInvestigationGroup and you want to enable CloudWatch alarms to create investigations and add…",
            ),
        ),
        (
            "TagResource",
            deferred(
                "Model requires resourceArn, tags. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Assigns one or more tags (key-value pairs) to the specified resource. Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access or change…",
            ),
        ),
        (
            "UntagResource",
            deferred(
                "Model requires resourceArn, tagKeys. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Removes one or more tags from the specified resource.",
            ),
        ),
        (
            "UpdateInvestigationGroup",
            deferred(
                "Model requires identifier. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: Updates the configuration of the specified investigation group.",
            ),
        ),
    ])
}
