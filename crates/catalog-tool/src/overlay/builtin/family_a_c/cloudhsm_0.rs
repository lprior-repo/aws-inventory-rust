// Pinned source: .local/aws-models/models/cloudhsm/service/2014-05-30/cloudhsm-2014-05-30.json
// Explicit reviewed decisions; document text is provenance, never an executable classifier.
use super::support::*;

pub(super) fn part_0() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        (
            "AddTagsToResource",
            deferred(
                "Model requires ResourceArn, TagList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "CreateHapg",
            deferred(
                "Model requires Label. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "CreateHsm",
            deferred(
                "Model requires SubnetId, SshKey, IamRoleArn, SubscriptionType. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "CreateLunaClient",
            deferred(
                "Model requires Certificate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "DeleteHapg",
            deferred(
                "Model requires HapgArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "DeleteHsm",
            deferred(
                "Model requires HsmArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "DeleteLunaClient",
            deferred(
                "Model requires ClientArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        // Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For…
        (
            "DescribeHapg",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("HapgArn")?,
                    Binding::From {
                        operation: name("ListHapgs")?,
                        path: path("$")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("HapgArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("HapgArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        // Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For…
        (
            "DescribeHsm",
            OperationOverlay {
                class: Some(OverlayClass::Expand),
                bind: [(
                    path("HsmArn")?,
                    Binding::From {
                        operation: name("ListHsms")?,
                        path: path("$")?,
                    },
                )]
                .into_iter()
                .collect(),
                queryable: vec![path("HsmArn")?],
                ..read(
                    "$",
                    Identity::Arn(path("HsmArn")?),
                    Scope::Region,
                    Paginator::None,
                )?
            },
        ),
        // The pinned DescribeLunaClient response includes the complete Certificate body.
        // Safe ListLunaClients ARN summaries do not authorize reading that content.
        ("DescribeLunaClient", denied(DenyReason::Content)),
        (
            "GetConfig",
            deferred(
                "Model requires ClientArn, ClientVersion, HapgList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "ListAvailableZones",
            deferred(
                "No safe resource collection plan is approved for this modeled operation. Its output is AZList; settings, reference data, histories, content, and state-changing calls are not promoted by a read-like name. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        // Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For…
        (
            "ListHapgs",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "HapgList",
                    Identity::Arn(path("$")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        // Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For…
        (
            "ListHsms",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "HsmList",
                    Identity::Arn(path("$")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
    ])
}

pub(super) fn part_1() -> Result<Vec<(&'static str, OperationOverlay)>, NameError> {
    Ok(vec![
        // Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For…
        (
            "ListLunaClients",
            OperationOverlay {
                queryable: vec![path("$")?],
                ..read(
                    "ClientList",
                    Identity::Arn(path("$")?),
                    Scope::Region,
                    token("NextToken", "NextToken")?,
                )?
            },
        ),
        (
            "ListTagsForResource",
            deferred(
                "Model requires ResourceArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "ModifyHapg",
            deferred(
                "Model requires HapgArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "ModifyHsm",
            deferred(
                "Model requires HsmArn. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "ModifyLunaClient",
            deferred(
                "Model requires ClientArn, Certificate. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
        (
            "RemoveTagsFromResource",
            deferred(
                "Model requires ResourceArn, TagKeyList. No complete safe request/identity binding is approved for this operation; arbitrary identifiers, credentials, payloads, or a single fixed namespace are not supplied. Model: This is documentation for AWS CloudHSM Classic . For more information, see AWS CloudHSM Classic FAQs , the AWS CloudHSM Classic User Guide , and the AWS CloudHSM Classic API Reference . For information about the current version…",
            ),
        ),
    ])
}
