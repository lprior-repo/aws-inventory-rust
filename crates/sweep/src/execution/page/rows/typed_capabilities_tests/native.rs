use super::fixture::{bytes, decode};
use super::*;
use catalog_tool::smithy;
use std::collections::BTreeSet;
#[path = "fixture/live.rs"]
mod live;
use live::{NativePage, Scenario};
#[path = "native/namespace_cases.rs"]
mod namespace_cases;
#[path = "native/security_lake.rs"]
mod security_lake;

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
