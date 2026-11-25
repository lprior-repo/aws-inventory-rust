use super::*;
use std::collections::BTreeMap;

fn shape<'a>(model: &'a Json, name: &str) -> TestResult<&'a Json> {
    model
        .get("shapes")
        .and_then(|shapes| shapes.get(format!("com.amazonaws.ivs#{name}")))
        .ok_or_else(|| format!("pinned IVS shape absent: {name}").into())
}

fn structure(model: &Json, name: &str, expected: &[(&str, &str)]) -> TestResult {
    let value = shape(model, name)?;
    assert_eq!(value.get("type"), Some(&json!("structure")));
    let actual = value
        .get("members")
        .and_then(Json::as_object)
        .ok_or("structure members absent")?
        .iter()
        .map(|(name, member)| {
            let target = member
                .get("target")
                .and_then(Json::as_str)
                .ok_or("member target absent")?;
            Ok((name.clone(), target.to_owned()))
        })
        .collect::<TestResult<BTreeMap<_, _>>>()?;
    let expected = expected
        .iter()
        .map(|(name, target)| (name.to_string(), format!("com.amazonaws.ivs#{target}")))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        actual, expected,
        "unreviewed descendants appeared in {name}"
    );
    Ok(())
}

#[test]
fn given_pinned_ivs_model_when_reviewing_complete_root_outputs_then_every_descendant_is_metadata_not_media_or_credentials()
-> TestResult {
    let model: Json = serde_json::from_slice(&std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.local/aws-models/models/ivs/service/2020-07-14/ivs-2020-07-14.json"),
    )?)?;
    structure(
        &model,
        "ListAdConfigurationsResponse",
        &[
            ("adConfigurations", "AdConfigurationList"),
            ("nextToken", "PaginationToken"),
        ],
    )?;
    structure(
        &model,
        "ListFailoverSlateConfigurationsResponse",
        &[
            (
                "failoverSlateConfigurations",
                "FailoverSlateConfigurationList",
            ),
            ("nextToken", "PaginationToken"),
        ],
    )?;
    structure(
        &model,
        "AdConfigurationSummary",
        &[
            ("arn", "AdConfigurationArn"),
            ("name", "AdConfigurationName"),
            (
                "mediaTailorPlaybackConfigurations",
                "MediaTailorPlaybackConfigurationsList",
            ),
            ("postRollConfiguration", "PostRollConfiguration"),
            ("tags", "Tags"),
        ],
    )?;
    structure(
        &model,
        "MediaTailorPlaybackConfiguration",
        &[(
            "playbackConfigurationArn",
            "MediaTailorPlaybackConfigurationArn",
        )],
    )?;
    structure(
        &model,
        "PostRollConfiguration",
        &[
            ("durationSeconds", "AdDurationSeconds"),
            ("enabled", "Boolean"),
        ],
    )?;
    structure(
        &model,
        "FailoverSlateConfigurationSummary",
        &[
            ("arn", "FailoverSlateConfigurationArn"),
            ("name", "FailoverSlateConfigurationName"),
            (
                "activationThresholdSeconds",
                "FailoverSlateActivationThresholdSeconds",
            ),
            ("minSlateDisplaySeconds", "FailoverSlateMinDisplaySeconds"),
            ("maxSlateDisplaySeconds", "FailoverSlateMaxDisplaySeconds"),
            ("state", "FailoverSlateConfigurationState"),
            ("tags", "Tags"),
        ],
    )?;
    for (name, member) in [
        ("AdConfigurationList", "AdConfigurationSummary"),
        (
            "FailoverSlateConfigurationList",
            "FailoverSlateConfigurationSummary",
        ),
        (
            "MediaTailorPlaybackConfigurationsList",
            "MediaTailorPlaybackConfiguration",
        ),
    ] {
        let value = shape(&model, name)?;
        assert_eq!(value.get("type"), Some(&json!("list")));
        assert_eq!(
            value.pointer("/member/target"),
            Some(&json!(format!("com.amazonaws.ivs#{member}")))
        );
    }
    let tags = shape(&model, "Tags")?;
    assert_eq!(tags.get("type"), Some(&json!("map")));
    assert_eq!(
        tags.pointer("/key/target"),
        Some(&json!("com.amazonaws.ivs#TagKey"))
    );
    assert_eq!(
        tags.pointer("/value/target"),
        Some(&json!("com.amazonaws.ivs#TagValue"))
    );
    for (name, kind) in [
        ("AdConfigurationArn", "string"),
        ("AdConfigurationName", "string"),
        ("FailoverSlateConfigurationArn", "string"),
        ("FailoverSlateConfigurationName", "string"),
        ("FailoverSlateConfigurationState", "string"),
        ("MediaTailorPlaybackConfigurationArn", "string"),
        ("AdDurationSeconds", "integer"),
        ("Boolean", "boolean"),
        ("FailoverSlateActivationThresholdSeconds", "integer"),
        ("FailoverSlateMinDisplaySeconds", "integer"),
        ("FailoverSlateMaxDisplaySeconds", "integer"),
        ("TagKey", "string"),
        ("TagValue", "string"),
        ("PaginationToken", "string"),
    ] {
        assert_eq!(
            shape(&model, name)?.get("type"),
            Some(&json!(kind)),
            "non-metadata leaf {name}"
        );
    }
    let state = shape(&model, "FailoverSlateConfigurationState")?
        .pointer("/traits/smithy.api#enum")
        .and_then(Json::as_array)
        .ok_or("failover state enum absent")?;
    assert_eq!(
        state
            .iter()
            .map(|entry| entry.get("value").and_then(Json::as_str))
            .collect::<Vec<_>>(),
        [
            Some("CREATING"),
            Some("CREATE_FAILED"),
            Some("CREATED"),
            Some("UPDATING")
        ]
    );
    Ok(())
}

#[test]
fn given_account_owned_unfiltered_root_docs_when_reviewing_inputs_and_registry_then_kinds_and_reference_scope_are_exact()
-> TestResult {
    let model: Json = serde_json::from_slice(&std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.local/aws-models/models/ivs/service/2020-07-14/ivs-2020-07-14.json"),
    )?)?;
    for case in cases::CASES {
        structure(
            &model,
            &format!("{}Request", case.operation),
            &[
                ("nextToken", "PaginationToken"),
                (
                    "maxResults",
                    if case.operation == "ListAdConfigurations" {
                        "MaxAdConfigurationResults"
                    } else {
                        "MaxFailoverSlateConfigurationResults"
                    },
                ),
            ],
        )?;
        let operation = shape(&model, case.operation)?;
        let docs = operation
            .pointer("/traits/smithy.api#documentation")
            .and_then(Json::as_str)
            .ok_or("root docs absent")?;
        assert!(docs.contains("in your account"));
        assert!(docs.contains("region where the API request is processed"));
    }
    let docs = shape(&model, "MediaTailorPlaybackConfiguration")?
        .pointer("/members/playbackConfigurationArn/traits/smithy.api#documentation")
        .and_then(Json::as_str)
        .ok_or("MediaTailor reference docs absent")?;
    assert!(docs.contains("customer-created") && docs.contains("same region and account"));
    let registry: Json = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../catalog/cloudcontrol-universal.json"
    )))?;
    let kinds = registry
        .get("resource_types")
        .and_then(Json::as_array)
        .ok_or("CFN resource registry absent")?;
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| kind.get("type_name").and_then(Json::as_str)
                == Some("AWS::IVS::AdConfiguration"))
            .count(),
        1
    );
    assert_eq!(
        kinds
            .iter()
            .filter(|kind| kind.get("type_name").and_then(Json::as_str)
                == Some("AWS::IVS::FailoverSlateConfiguration"))
            .count(),
        0
    );
    Ok(())
}
