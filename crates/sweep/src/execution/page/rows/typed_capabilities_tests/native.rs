use super::fixture::{bytes, decode};
use super::*;
use catalog_tool::smithy;
use std::collections::BTreeSet;
#[path = "fixture/live.rs"]
mod live;
use live::{NativePage, Scenario};

const VLAN: &str = "AWS::ApiOnly::EVSVlan";
const VERSION: &str = "AWS::Wisdom::AIPromptVersion";
const NAMESPACE: &str = "AWS::S3Tables::Namespace";
const ENV_ONE: &str = "env-0000000001";
const ENV_TWO: &str = "env-0000000002";
const ASSISTANT: &str = "11111111-1111-4111-8111-111111111111";
const PROMPT_ONE: &str = "22222222-2222-4222-8222-222222222222";
const PROMPT_TWO: &str = "33333333-3333-4333-8333-333333333333";
fn environment(id: &str) -> String {
    format!("arn:aws:evs:us-east-1:123456789012:environment/{id}")
}
fn prompt(id: &str) -> String {
    format!("arn:aws:wisdom:us-east-1:123456789012:ai-prompt/{ASSISTANT}/{id}")
}
fn bucket(id: &str) -> String {
    format!("arn:aws:s3tables:us-east-1:123456789012:bucket/{id}")
}
fn correlated(parent: &str, component: &str) -> String {
    format!("{}:{parent}|{}:{component}", parent.len(), component.len())
}

fn child(page: &NativePage<'_, '_>, parent_kind: &str, parent_id: &str) -> TestResult {
    let parent = page
        .lease
        .scope()
        .parent()
        .ok_or("child lacks discovered parent")?;
    assert_eq!(parent.account.as_str(), "123456789012");
    assert_eq!(parent.region.as_str(), "us-east-1");
    assert_eq!(parent.kind.as_str(), parent_kind);
    assert_eq!(parent.id.as_str(), parent_id);
    assert!(page.lease.scope().context().is_some());
    Ok(())
}

fn evs(
    page: NativePage<'_, '_>,
    invalid: Option<&serde_json::Value>,
    seen: &mut BTreeSet<(String, Option<String>)>,
) -> TestResult<serde_json::Value> {
    if page.plan.operation_name() == "ListEnvironments" {
        return match page.text("nextToken")? {
            None => Ok(serde_json::json!({"environmentSummaries":[],"nextToken":"root-next"})),
            Some(token) => {
                assert_eq!(token, "root-next");
                Ok(serde_json::json!({"environmentSummaries":[
                {"environmentId":ENV_ONE,"environmentArn":environment(ENV_ONE)},
                {"environmentId":ENV_TWO,"environmentArn":environment(ENV_TWO)}]}))
            }
        };
    }
    assert_eq!(page.plan.operation_name(), "ListEnvironmentVlans");
    let id = page
        .text("environmentId")?
        .ok_or("environment selector missing")?;
    child(&page, "AWS::EVS::Environment", &environment(&id))?;
    assert_eq!(
        page.body_text("environmentId")?.as_deref(),
        Some(id.as_str())
    );
    let token = page.text("nextToken")?;
    assert_eq!(page.body_text("nextToken")?, token);
    assert!(
        seen.insert((id.clone(), token.clone())),
        "a parent cursor replayed"
    );
    if let Some(row) = invalid {
        return Ok(serde_json::json!({"environmentVlans":[row]}));
    }
    match token {
        None => Ok(
            serde_json::json!({"environmentVlans":[{"vlanId":i32::MIN}],"nextToken":format!("for-{id}")}),
        ),
        Some(token) => {
            assert_eq!(token, format!("for-{id}"));
            Ok(if id == ENV_ONE {
                serde_json::json!({"environmentVlans":[{"vlanId":i32::MAX}]})
            } else {
                serde_json::json!({"environmentVlans":[]})
            })
        }
    }
}

#[tokio::test]
async fn given_evs_integral_vlans_on_two_parents_and_pages_when_persisted_then_ids_keep_parent_and_signed_integer()
-> TestResult {
    let scenario = Scenario::new("evs", "ListEnvironments").await?;
    let mut seen = BTreeSet::new();
    scenario.run(|page| evs(page, None, &mut seen)).await?;
    assert_eq!(
        seen,
        [ENV_ONE, ENV_TWO]
            .into_iter()
            .flat_map(|id| [(id.into(), None), (id.into(), Some(format!("for-{id}")))])
            .collect()
    );
    let ids = [
        correlated(ENV_ONE, &i32::MIN.to_string()),
        correlated(ENV_ONE, &i32::MAX.to_string()),
        correlated(ENV_TWO, &i32::MIN.to_string()),
    ];
    let [id0, id1, id2] = &ids;
    assert_eq!(scenario.keys(VLAN).await?, live::keys(VLAN, &ids));
    let pairs = vec![
        (id0.clone(), environment(ENV_ONE)),
        (id1.clone(), environment(ENV_ONE)),
        (id2.clone(), environment(ENV_TWO)),
    ];
    assert_eq!(
        scenario.edges(VLAN).await?,
        live::edges(VLAN, "AWS::EVS::Environment", "IN_ENVIRONMENT", &pairs)
    );
    Ok(())
}

fn qconnect(
    page: NativePage<'_, '_>,
    invalid: Option<&serde_json::Value>,
    seen: &mut BTreeSet<(String, Option<String>)>,
) -> TestResult<serde_json::Value> {
    match page.plan.operation_name() {
        "ListAssistants" => {
            return Ok(
                serde_json::json!({"assistantSummaries":[{"assistantId":ASSISTANT,"assistantArn":format!("arn:aws:wisdom:us-east-1:123456789012:assistant/{ASSISTANT}")}]}),
            );
        }
        "ListAIPrompts" => {
            assert_eq!(page.text("assistantId")?.as_deref(), Some(ASSISTANT));
            return Ok(serde_json::json!({"aiPromptSummaries":[
                {"assistantId":ASSISTANT,"aiPromptId":PROMPT_ONE,"aiPromptArn":prompt(PROMPT_ONE),"tags":{"owner":"one"}},
                {"assistantId":ASSISTANT,"aiPromptId":PROMPT_TWO,"aiPromptArn":prompt(PROMPT_TWO),"tags":{"owner":"two"}}]}));
        }
        "ListAIPromptVersions" => {}
        _ => return Err("unexpected prompt scenario operation".into()),
    }
    let id = page.text("aiPromptId")?.ok_or("prompt selector missing")?;
    assert_eq!(page.text("assistantId")?.as_deref(), Some(ASSISTANT));
    assert_eq!(page.text("origin")?.as_deref(), Some("CUSTOMER"));
    child(&page, "AWS::Wisdom::AIPrompt", &prompt(&id))?;
    assert!(page.wire.uri().path().contains(&id));
    let token = page.text("nextToken")?;
    assert!(
        seen.insert((id.clone(), token.clone())),
        "a prompt cursor replayed"
    );
    if let Some(row) = invalid {
        return Ok(serde_json::json!({"aiPromptVersionSummaries":[row]}));
    }
    match token {
        None => Ok(
            serde_json::json!({"aiPromptVersionSummaries":[{"aiPromptSummary":{"aiPromptArn":prompt(&id)},"versionNumber":1}],"nextToken":format!("for-{id}")}),
        ),
        Some(token) => {
            assert_eq!(token, format!("for-{id}"));
            assert!(
                page.wire
                    .uri()
                    .query()
                    .is_some_and(|query| query.contains(&token))
            );
            Ok(if id == PROMPT_ONE {
                serde_json::json!({"aiPromptVersionSummaries":[{"aiPromptSummary":{"aiPromptArn":prompt(&id)},"versionNumber":i64::MAX}]})
            } else {
                serde_json::json!({"aiPromptVersionSummaries":[]})
            })
        }
    }
}

#[tokio::test]
async fn given_qconnect_long_versions_when_decoded_then_exact_max_and_prompt_isolation_persist_without_coercion()
-> TestResult {
    let scenario = Scenario::new("qconnect", "ListAssistants").await?;
    let mut seen = BTreeSet::new();
    scenario.run(|page| qconnect(page, None, &mut seen)).await?;
    assert_eq!(
        seen,
        [PROMPT_ONE, PROMPT_TWO]
            .into_iter()
            .flat_map(|id| [(id.into(), None), (id.into(), Some(format!("for-{id}")))])
            .collect()
    );
    let one = prompt(PROMPT_ONE);
    let two = prompt(PROMPT_TWO);
    let ids = [
        correlated(&one, "1"),
        correlated(&one, &i64::MAX.to_string()),
        correlated(&two, "1"),
    ];
    let [id0, id1, id2] = &ids;
    assert_eq!(scenario.keys(VERSION).await?, live::keys(VERSION, &ids));
    assert_eq!(
        scenario.edges(VERSION).await?,
        live::edges(
            VERSION,
            "AWS::Wisdom::AIPrompt",
            "VERSION_OF",
            &[
                (id0.clone(), one.clone()),
                (id1.clone(), one),
                (id2.clone(), two)
            ]
        )
    );
    Ok(())
}

fn namespaces(
    page: NativePage<'_, '_>,
    replacement: Option<&serde_json::Value>,
    seen: &mut BTreeSet<(String, Option<String>)>,
) -> TestResult<serde_json::Value> {
    if page.plan.operation_name() == "ListTableBuckets" {
        return Ok(serde_json::json!({"tableBuckets":[
            {"arn":bucket("table-one"),"name":"table-one","ownerAccountId":"123456789012","createdAt":"1970-01-01T00:00:00Z"},
            {"arn":bucket("table-two"),"name":"table-two","ownerAccountId":"123456789012","createdAt":"1970-01-01T00:00:00Z"}]}));
    }
    assert_eq!(page.plan.operation_name(), "ListNamespaces");
    let arn = page
        .text("tableBucketARN")?
        .ok_or("bucket selector missing")?;
    child(&page, "AWS::S3Tables::TableBucket", &arn)?;
    let id = arn.rsplit('/').next().ok_or("bucket name missing")?;
    assert!(page.wire.uri().to_string().contains(id));
    let token = page.text("continuationToken")?;
    assert!(
        seen.insert((arn.clone(), token.clone())),
        "a bucket cursor replayed"
    );
    if let Some(value) = replacement {
        return Ok(serde_json::json!({"namespaces":[{"namespace":value}]}));
    }
    match token {
        None => Ok(
            serde_json::json!({"namespaces":[{"namespace":["ab","c"]}],"continuationToken":format!("for-{id}")}),
        ),
        Some(token) => {
            assert_eq!(token, format!("for-{id}"));
            assert!(
                page.wire
                    .uri()
                    .query()
                    .is_some_and(|query| query.contains(&token))
            );
            Ok(if id == "table-one" {
                serde_json::json!({"namespaces":[{"namespace":["a","bc"]}]})
            } else {
                serde_json::json!({"namespaces":[]})
            })
        }
    }
}

#[tokio::test]
async fn given_modeled_namespace_components_when_persisted_then_lists_and_bucket_parents_stay_distinct()
-> TestResult {
    let scenario = Scenario::new("s3tables", "ListTableBuckets").await?;
    let mut seen = BTreeSet::new();
    scenario
        .run(|page| namespaces(page, None, &mut seen))
        .await?;
    assert_eq!(
        seen,
        ["table-one", "table-two"]
            .into_iter()
            .flat_map(|id| [(bucket(id), None), (bucket(id), Some(format!("for-{id}")))])
            .collect()
    );
    let one = bucket("table-one");
    let two = bucket("table-two");
    let ids = [
        correlated(&one, "list:2|2:ab|1:c"),
        correlated(&one, "list:2|1:a|2:bc"),
        correlated(&two, "list:2|2:ab|1:c"),
    ];
    let [id0, id1, id2] = &ids;
    assert_eq!(scenario.keys(NAMESPACE).await?, live::keys(NAMESPACE, &ids));
    assert_eq!(
        scenario.edges(NAMESPACE).await?,
        live::edges(
            NAMESPACE,
            "AWS::S3Tables::TableBucket",
            "IN_TABLE_BUCKET",
            &[
                (id0.clone(), one.clone()),
                (id1.clone(), one),
                (id2.clone(), two)
            ]
        )
    );
    Ok(())
}

#[tokio::test]
async fn given_malformed_native_vlans_when_a_discovered_child_runs_then_no_invalid_identity_commits()
-> TestResult {
    for invalid in [
        serde_json::json!({}),
        serde_json::json!({"vlanId":"1"}),
        serde_json::json!({"vlanId":2147483648_i64}),
        serde_json::json!({"vlanId":-2147483649_i64}),
    ] {
        let scenario = Scenario::new("evs", "ListEnvironments").await?;
        assert!(
            scenario
                .run(|page| evs(page, Some(&invalid), &mut BTreeSet::new()))
                .await
                .is_err()
        );
        assert!(scenario.keys(VLAN).await?.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn given_malformed_native_versions_or_spoofed_arn_when_a_discovered_child_runs_then_no_invalid_identity_commits()
-> TestResult {
    let mut invalid=[serde_json::json!(0),serde_json::json!("1"),serde_json::json!(true),serde_json::json!(null),serde_json::json!(9223372036854775808_u64)]
        .into_iter().map(|version|serde_json::json!({"aiPromptSummary":{"aiPromptArn":prompt(PROMPT_ONE)},"versionNumber":version})).collect::<Vec<_>>();
    invalid.push(serde_json::json!({"aiPromptSummary":{"aiPromptArn":prompt(PROMPT_ONE).replace("123456789012","999999999999")},"versionNumber":1}));
    for row in invalid {
        let scenario = Scenario::new("qconnect", "ListAssistants").await?;
        assert!(
            scenario
                .run(|page| qconnect(page, Some(&row), &mut BTreeSet::new()))
                .await
                .is_err()
        );
        assert!(scenario.keys(VERSION).await?.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn given_native_namespace_limits_when_discovered_children_run_then_exact_limits_commit_and_invalid_values_do_not()
-> TestResult {
    for valid in [
        serde_json::json!([]),
        serde_json::json!(["x".repeat(255)]),
        serde_json::json!(vec!["a"; 64]),
    ] {
        let scenario = Scenario::new("s3tables", "ListTableBuckets").await?;
        scenario
            .run(|page| namespaces(page, Some(&valid), &mut BTreeSet::new()))
            .await?;
        let component = match valid
            .as_array()
            .ok_or("valid namespace is not a list")?
            .len()
        {
            0 => "list:0".into(),
            1 => format!("list:1|255:{}", "x".repeat(255)),
            _ => format!("list:64{}", "|1:a".repeat(64)),
        };
        let ids = vec![
            correlated(&bucket("table-one"), &component),
            correlated(&bucket("table-two"), &component),
        ];
        assert_eq!(scenario.keys(NAMESPACE).await?, live::keys(NAMESPACE, &ids));
    }
    for invalid in [
        serde_json::json!([""]),
        serde_json::json!([1]),
        serde_json::json!("namespace"),
        serde_json::json!(["x".repeat(256)]),
        serde_json::json!(vec!["a"; 65]),
    ] {
        let scenario = Scenario::new("s3tables", "ListTableBuckets").await?;
        assert!(
            scenario
                .run(|page| namespaces(page, Some(&invalid), &mut BTreeSet::new()))
                .await
                .is_err()
        );
        assert!(scenario.keys(NAMESPACE).await?.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn given_namespace_members_outside_the_pinned_pattern_when_a_mixed_native_page_is_assembled_then_no_children_commit()
-> TestResult {
    for invalid in ["a:b", "a-b", "Upper", "a b", "é"] {
        let scenario = Scenario::new("s3tables", "ListTableBuckets").await?;
        let malformed = serde_json::json!([invalid]);
        let mut seen = BTreeSet::new();
        let result = scenario
            .run(|page| {
                let child = page.plan.operation_name() == "ListNamespaces";
                let mut response = namespaces(page, Some(&malformed), &mut seen)?;
                if child {
                    response
                        .get_mut("namespaces")
                        .and_then(serde_json::Value::as_array_mut)
                        .ok_or("namespace fixture response is not a list")?
                        .insert(0, serde_json::json!({"namespace":["valid_name"]}));
                }
                Ok(response)
            })
            .await;
        let error = result
            .err()
            .ok_or("a malformed modeled namespace was committed")?;
        assert!(
            matches!(
                error.downcast_ref::<SweepError>(),
                Some(SweepError::Operation(
                    "modeled identity text operand violates source constraints"
                ))
            ),
            "{error}"
        );
        assert_eq!(scenario.keys(NAMESPACE).await?, live::keys(NAMESPACE, &[]));
        assert_eq!(
            scenario.edges(NAMESPACE).await?,
            live::edges(
                NAMESPACE,
                "AWS::S3Tables::TableBucket",
                "IN_TABLE_BUCKET",
                &[]
            )
        );
    }
    Ok(())
}

#[tokio::test]
async fn given_security_lake_regional_roots_when_encoded_and_committed_then_exact_routed_region_is_the_only_query_selector()
-> TestResult {
    use crate::input;
    let kind = "AWS::SecurityLake::DataLake";
    for region in ["us-east-1", "us-west-2"] {
        let scenario = Scenario::new_region("securitylake", "ListDataLakes", region).await?;
        let arn = format!("arn:aws:securitylake:{region}:123456789012:data-lake/default");
        scenario
            .run(|page| {
                assert_eq!(page.plan.operation_name(), "ListDataLakes");
                assert!(!input::requires_parent(page.plan));
                assert!(page.plan.pagination().tokens.is_empty());
                assert!(page.lease.scope().parent().is_none());
                assert_eq!(page.lease.scope().region().as_str(), region);
                assert_eq!(
                    page.wire.uri().query(),
                    Some(format!("regions={region}").as_str())
                );
                assert!(page.wire.body().is_empty());
                Ok(serde_json::json!({"dataLakes":[{"dataLakeArn":arn,"region":region}]}))
            })
            .await?;
        assert_eq!(
            scenario.keys(kind).await?,
            vec![("123456789012".into(), region.into(), kind.into(), arn)]
        );
        let plan = scenario
            .catalog
            .lookup("securitylake", "ListDataLakes")
            .ok_or("data lake root absent")?;
        assert!(matches!(
            input::root_bindings(plan, Some(&AccountId::parse("123456789012")?), None),
            Err(SweepError::Operation(
                "region list binding requires the routed request region"
            ))
        ));
    }
    let scenario = Scenario::new("securitylake", "ListDataLakes").await?;
    assert!(scenario.run(|_| Ok(serde_json::json!({"dataLakes":[{
        "dataLakeArn":"arn:aws:securitylake:us-west-2:123456789012:data-lake/default","region":"us-west-2"}]}))).await.is_err());
    assert!(scenario.keys(kind).await?.is_empty());
    Ok(())
}
