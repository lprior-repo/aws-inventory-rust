use super::*;
use aws_wire::{sign::Credentials, transport::Config};
use inventory_engine::limiter::ExecutorConfig;
use inventory_sweep::CredentialFailure;
use std::time::Duration;

#[path = "../../../../catalog-tool/tests/support/bindings.rs"]
mod fixture;
#[path = "../../../tests/probe/server.rs"]
mod server;
use server::{Server, TestResult};

struct TestCredentials;
impl CredentialSource for TestCredentials {
    fn region(&self) -> &str {
        "us-east-1"
    }
    async fn resolve(&self) -> Result<Credentials, CredentialFailure> {
        Credentials::new("TEST", "TEST", None).map_err(|error| CredentialFailure {
            retryable: false,
            message: error.to_string(),
        })
    }
}

async fn compare(list: &str, describe: &str) -> TestResult<(String, String)> {
    let bytes = fixture::compiled(1, |_| {})?;
    let (report, overlay, requests) = compare_catalog(&bytes, &[list, describe]).await?;
    let request = requests.get(1).ok_or("describe request missing")?;
    assert!(request.contains("\"EntityId\":\"one\""));
    assert!(request.contains("\"OrganizationId\":\"org-0\""));
    Ok((report, overlay))
}

async fn compare_catalog(
    bytes: &[u8],
    bodies: &[&str],
) -> TestResult<(String, String, Vec<String>)> {
    let catalog = RuntimeCatalog::from_bytes(bytes)?;
    let server = Server::new().await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..Config::default()
    })?;
    let executor = ApiExecutor::new(ExecutorConfig {
        max_attempts: 1,
        ..ExecutorConfig::default()
    })?;
    let region = RegionId::parse("us-east-1")?;
    let endpoint = server.endpoint()?;
    let account = AccountId::parse("123456789012")?;
    let probe = Probe {
        catalog: &catalog,
        credentials: &TestCredentials,
        client: &client,
        executor: &executor,
        region: &region,
        endpoint: &endpoint,
        services: &[],
    };
    let (completed, completion) = tokio::sync::oneshot::channel();
    let ((report, evidence), requests) = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(
            async {
                let result = probe
                    .compare_pair("bindings/ListParents0/ListChildren", &account)
                    .await;
                completed.send(()).map_err(|_| "comparison server closed")?;
                Ok::<_, Box<dyn std::error::Error>>(result?)
            },
            server.serve_until(bodies, completion)
        )
    })
    .await??;
    Ok((
        report,
        overlays(evidence.into_iter().collect(), "2026-11-24T00:00:00Z")?,
        requests,
    ))
}

pub(super) fn extra_member_catalog() -> TestResult<Vec<u8>> {
    fixture::compiled_with_model(
        1,
        |_| {},
        |_| {},
        |shapes| {
            shapes.insert(
                "n#ParentItem".into(),
                serde_json::json!({
                    "type":"structure","members":{"Id":{"target":"smithy.api#String"}}
                }),
            );
            shapes.insert(
                "n#ParentItems".into(),
                serde_json::json!({
                    "type":"list","member":{"target":"n#ParentItem"}
                }),
            );
            shapes.insert("n#ParentOutput".into(), serde_json::json!({
            "type":"structure","members":{"Items":{"target":"n#ParentItems"},"NextToken":{"target":"smithy.api#String"}}
        }));
            shapes.insert("n#ListParents0".into(), serde_json::json!({
            "type":"operation","input":{"target":"n#Input"},"output":{"target":"n#ParentOutput"}
        }));
        },
    )
}

#[tokio::test]
async fn given_equal_raw_items_when_probed_then_overlay_contains_resource_evidence() -> TestResult {
    let body = r#"{"Items":[{"Id":"one","Count":7}]}"#;
    let (report, overlay) = compare(body, body).await?;
    assert_eq!(report, "bindings/ListParents0/ListChildren\tequal\t1\n");
    assert!(overlay.contains("skip = [\"ListChildren\"]"));
    assert!(overlay.contains("resources = 1"));
    assert!(
        overlay
            .contains("[service.\"bindings\".operation.ListParents0.skip_evidence.ListChildren]")
    );
    Ok(())
}

#[tokio::test]
async fn given_raw_null_when_describe_fills_it_then_probe_names_member_and_prints_no_skip()
-> TestResult {
    let (report, overlay) = compare(
        r#"{"Items":[{"Id":"one","Count":null}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
    )
    .await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tdifferent\t$.Count\n"
    );
    assert_eq!(overlay, "");
    Ok(())
}

#[tokio::test]
async fn given_extra_describe_value_when_probed_then_missing_member_prevents_skip() -> TestResult {
    let (report, overlay) = compare(
        r#"{"Items":[{"Id":"one"}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
    )
    .await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tdifferent\t$.Count\n"
    );
    assert_eq!(overlay, "");
    Ok(())
}

#[tokio::test]
async fn given_missing_null_and_value_on_either_side_when_compared_then_no_skip_is_printed()
-> TestResult {
    let absent = r#"{"Items":[{"Id":"one"}]}"#;
    let null = r#"{"Items":[{"Id":"one","Count":null}]}"#;
    let filled = r#"{"Items":[{"Id":"one","Count":7}]}"#;
    for (list, describe) in [
        (absent, null),
        (null, absent),
        (filled, absent),
        (filled, null),
    ] {
        let (report, overlay) = compare(list, describe).await?;
        assert_eq!(
            report,
            "bindings/ListParents0/ListChildren\tdifferent\t$.Count\n"
        );
        assert_eq!(overlay, "");
    }
    Ok(())
}

#[tokio::test]
async fn given_extra_describe_model_member_omitted_on_both_sides_when_compared_then_skip_is_refused()
-> TestResult {
    let bytes = extra_member_catalog()?;
    let body = r#"{"Items":[{"Id":"one"}]}"#;
    let (report, overlay, requests) = compare_catalog(&bytes, &[body, body]).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tdifferent\t$.Count\n"
    );
    assert_eq!(overlay, "");
    assert_eq!(requests.len(), 0);
    Ok(())
}

#[tokio::test]
async fn given_difference_on_second_list_page_when_compared_then_all_pages_prevent_skip()
-> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let bodies = [
        r#"{"Items":[{"Id":"one","Count":7}],"NextToken":"second"}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
        r#"{"Items":[{"Id":"two","Count":null}]}"#,
        r#"{"Items":[{"Id":"two","Count":8}]}"#,
    ];
    let (report, overlay, requests) = compare_catalog(&bytes, &bodies).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tdifferent\t$.Count\n"
    );
    assert_eq!(overlay, "");
    assert_eq!(requests.len(), 4);
    assert!(
        requests
            .get(2)
            .ok_or("second page request missing")?
            .contains("\"NextToken\":\"second\"")
    );
    assert!(
        requests
            .get(3)
            .ok_or("second describe request missing")?
            .contains("\"EntityId\":\"two\"")
    );
    Ok(())
}

#[tokio::test]
async fn given_no_seeded_resources_when_compared_then_pair_is_unverified_without_overlay()
-> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let (report, overlay, requests) = compare_catalog(&bytes, &[r#"{"Items":[]}"#]).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tunverified\tno seeded resources\n"
    );
    assert_eq!(overlay, "");
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[tokio::test]
async fn given_one_seeded_and_one_empty_bound_branch_when_compared_then_count_is_unverified()
-> TestResult {
    use catalog_tool::overlay::{ExecutionBranch, FixedValue, MemberPath, Scope};
    let organization = MemberPath::new("OrganizationId")?;
    let branches = ["seeded", "empty"].map(|name| ExecutionBranch {
        discriminator: name.into(),
        fixed: BTreeMap::from([(organization.clone(), FixedValue::String(name.into()))]),
        bind: BTreeMap::new(),
        emission: None,
        row_policy: None,
        scope: Scope::Region,
        request_region: None,
    });
    let bytes = fixture::compiled_with_parents(
        1,
        |parent| {
            parent.branches = branches.to_vec();
        },
        |_| {},
    )?;
    let row = r#"{"Items":[{"Id":"one","Count":7}]}"#;
    let (report, overlay, requests) =
        compare_catalog(&bytes, &[row, row, r#"{"Items":[]}"#]).await?;
    assert_eq!(requests.len(), 3);
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tunverified\tcompared 1 before an empty branch variant\n"
    );
    assert!(overlay.is_empty());
    Ok(())
}

#[tokio::test]
async fn given_unbound_child_when_compared_then_pair_is_unverified_without_requests() -> TestResult
{
    let organization = catalog_tool::overlay::MemberPath::new("OrganizationId")?;
    let bytes = fixture::compiled(1, |child| {
        child.bind.retain(|path, _| path.to_string() == "Regions");
        child.fixed.insert(
            organization,
            catalog_tool::overlay::FixedValue::String("org".into()),
        );
    })?;
    let (report, overlay, requests) = compare_catalog(&bytes, &[]).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tunverified\tchild binding is not exclusive to the parent branch\n"
    );
    assert_eq!(overlay, "");
    assert!(requests.is_empty());
    Ok(())
}

#[tokio::test]
async fn given_child_with_two_producers_when_compared_then_no_skip_receipt_is_emitted() -> TestResult
{
    let bytes = mixed_producer_catalog(false)?;
    let (report, overlay, requests) = compare_catalog(&bytes, &[]).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tunverified\tchild binding is not exclusive to the parent branch\n"
    );
    assert!(overlay.is_empty());
    assert!(requests.is_empty());
    Ok(())
}

#[tokio::test]
async fn given_seeded_then_foreign_bound_child_branch_when_compared_then_count_is_unverified()
-> TestResult {
    let bytes = mixed_producer_catalog(true)?;
    let row = r#"{"Items":[{"Id":"one","Count":7}]}"#;
    let (report, overlay, requests) = compare_catalog(&bytes, &[row, row]).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tunverified\tcompared 1 before a child branch not bound exclusively to the parent\n"
    );
    assert!(overlay.is_empty());
    assert_eq!(requests.len(), 2);
    Ok(())
}

fn mixed_producer_catalog(branched: bool) -> TestResult<Vec<u8>> {
    use catalog_tool::overlay::{Binding, ExecutionBranch, MemberPath, OperationName, Scope};
    let organization = MemberPath::new("OrganizationId")?;
    let entity = MemberPath::new("EntityId")?;
    let binding = |operation| -> TestResult<Binding> {
        Ok(Binding::From {
            operation: OperationName::new(operation)?,
            path: MemberPath::new("Id")?,
        })
    };
    let local = binding("ListParents0")?;
    let foreign = binding("ListParents1")?;
    fixture::compiled(2, |child| {
        if branched {
            child.branches = vec![
                ExecutionBranch {
                    discriminator: "bound".into(),
                    fixed: BTreeMap::new(),
                    bind: BTreeMap::from([(organization.clone(), local.clone())]),
                    emission: None,
                    row_policy: None,
                    scope: Scope::Region,
                    request_region: None,
                },
                ExecutionBranch {
                    discriminator: "mixed".into(),
                    fixed: BTreeMap::new(),
                    bind: BTreeMap::new(),
                    emission: None,
                    row_policy: None,
                    scope: Scope::Region,
                    request_region: None,
                },
            ];
        }
        child.bind.insert(entity, local);
        child.bind.insert(organization, foreign);
    })
}

const PAIR: &str = "bindings/ListParents0/ListChildren";

async fn compare_command(
    when: &str,
    pairs: &[String],
) -> TestResult<Result<(String, String), ProbeError>> {
    let bytes = fixture::compiled(1, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let server = Server::new().await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..Config::default()
    })?;
    let executor = ApiExecutor::new(ExecutorConfig {
        max_attempts: 1,
        ..ExecutorConfig::default()
    })?;
    let region = RegionId::parse("us-east-1")?;
    let endpoint = server.endpoint()?;
    let probe = Probe {
        catalog: &catalog,
        credentials: &TestCredentials,
        client: &client,
        executor: &executor,
        region: &region,
        endpoint: &endpoint,
        services: &[],
    };
    let (completed, completion) = tokio::sync::oneshot::channel();
    let (result, requests) = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(
            async {
                let result = probe.compare(pairs, when).await;
                completed.send(()).map_err(|_| "comparison server closed")?;
                Ok::<_, Box<dyn std::error::Error>>(result)
            },
            server.serve_until(&[], completion)
        )
    })
    .await??;
    assert_eq!(requests.len(), 0);
    Ok(result)
}

#[tokio::test]
async fn given_empty_comparison_date_when_command_runs_then_date_is_refused() -> TestResult {
    assert!(matches!(
        compare_command("", &[PAIR.into()]).await?,
        Err(ProbeError::Invalid(
            "comparison date must be bounded nonempty text"
        ))
    ));
    Ok(())
}

#[tokio::test]
async fn given_no_account_identity_when_command_runs_then_comparison_is_refused() -> TestResult {
    assert!(matches!(
        compare_command("2026-11-24", &[PAIR.into()]).await?,
        Err(ProbeError::Invalid(
            "comparison requires successful account identity probe"
        ))
    ));
    Ok(())
}

#[tokio::test]
async fn given_duplicate_pair_when_compare_runs_then_rejected_before_any_request() -> TestResult {
    assert!(matches!(
        compare_command("2026-11-24", &[PAIR.into(), PAIR.into()]).await?,
        Err(ProbeError::DuplicatePair(_))
    ));
    Ok(())
}

#[tokio::test]
async fn given_existing_root_probe_when_run_then_first_page_contract_is_unchanged() -> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let server = Server::new().await?;
    let client = Client::new(Config {
        extra_ca_bundle: Some(server.ca.clone()),
        ..Config::default()
    })?;
    let executor = ApiExecutor::new(ExecutorConfig {
        max_attempts: 1,
        ..ExecutorConfig::default()
    })?;
    let region = RegionId::parse("us-east-1")?;
    let endpoint = server.endpoint()?;
    let (report, requests) = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::try_join!(
            async {
                Ok::<_, Box<dyn std::error::Error>>(
                    crate::probe::run(
                        &catalog,
                        &TestCredentials,
                        &client,
                        &executor,
                        &region,
                        &endpoint,
                        &[],
                    )
                    .await?,
                )
            },
            server.serve(1)
        )
    })
    .await??;
    assert!(report.contains("bindings ListParents0 awsJson1_1 items 2"));
    assert_eq!(requests.len(), 1);
    Ok(())
}

#[test]
fn given_unicode_and_quotes_in_receipt_when_rendered_then_toml_round_trips() -> TestResult {
    let compared = "members\u{2028}\"equal\"\\checked";
    let when = "2026-11-24\u{2028}reviewed";
    let overlay = overlays(
        vec![Evidence {
            service: "bindings".into(),
            parent: "ListParents0".into(),
            child: "ListChildren".into(),
            inputs_sha256: "a".repeat(64),
            compared: compared.into(),
            resources: 3,
        }],
        when,
    )?;
    let parsed: toml::Value = toml::from_str(&overlay)?;
    let parent = parsed
        .get("service")
        .and_then(|value| value.get("bindings"))
        .and_then(|value| value.get("operation"))
        .and_then(|value| value.get("ListParents0"))
        .ok_or("parent missing")?;
    assert_eq!(
        parent.get("skip").and_then(toml::Value::as_array),
        Some(&vec![toml::Value::String("ListChildren".into())])
    );
    let evidence = parent
        .get("skip_evidence")
        .and_then(|value| value.get("ListChildren"))
        .ok_or("receipt missing")?;
    assert_eq!(
        evidence.get("compared").and_then(toml::Value::as_str),
        Some(compared)
    );
    assert_eq!(
        evidence.get("when").and_then(toml::Value::as_str),
        Some(when)
    );
    assert_eq!(
        evidence.get("resources").and_then(toml::Value::as_integer),
        Some(3)
    );
    Ok(())
}

#[test]
fn given_two_services_with_same_parent_when_rendered_then_both_receipts_parse() -> TestResult {
    let evidence = [("first-service", 2), ("second-service", 3)]
        .into_iter()
        .map(|(service, resources)| Evidence {
            service: service.into(),
            parent: "ListThings".into(),
            child: "DescribeThing".into(),
            inputs_sha256: "a".repeat(64),
            compared: "equal seeded values".into(),
            resources,
        })
        .collect();
    let receipt: toml::Value = toml::from_str(&overlays(evidence, "2026-11-24")?)?;
    for (service, resources) in [("first-service", 2), ("second-service", 3)] {
        let stored_count = receipt
            .get("service")
            .and_then(|value| value.get(service))
            .and_then(|value| value.get("operation"))
            .and_then(|value| value.get("ListThings"))
            .and_then(|value| value.get("skip_evidence"))
            .and_then(|value| value.get("DescribeThing"))
            .and_then(|value| value.get("resources"))
            .and_then(toml::Value::as_integer);
        assert_eq!(stored_count, Some(resources));
    }
    Ok(())
}

#[tokio::test]
async fn given_newline_comparison_date_when_command_runs_then_exact_date_error_is_returned()
-> TestResult {
    assert!(matches!(
        compare_command("2026-11-24\nreviewed", &[PAIR.into()]).await?,
        Err(ProbeError::Invalid(
            "comparison date must be bounded nonempty text"
        ))
    ));
    Ok(())
}

#[tokio::test]
async fn given_difference_only_in_second_row_when_compared_then_every_resource_prevents_skip()
-> TestResult {
    let bytes = fixture::compiled(1, |_| {})?;
    let bodies = [
        r#"{"Items":[{"Id":"one","Count":7},{"Id":"two","Count":8}]}"#,
        r#"{"Items":[{"Id":"one","Count":7}]}"#,
        r#"{"Items":[{"Id":"two","Count":9}]}"#,
    ];
    let (report, overlay, requests) = compare_catalog(&bytes, &bodies).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tdifferent\t$.Count\n"
    );
    assert_eq!(overlay, "");
    assert_eq!(requests.len(), 3);
    assert!(
        requests
            .get(2)
            .ok_or("second describe request missing")?
            .contains("\"EntityId\":\"two\"")
    );
    Ok(())
}

fn nested_member_catalog(container: &str) -> TestResult<Vec<u8>> {
    fixture::compiled_with_model(
        1,
        |_| {},
        |_| {},
        |shapes| {
            shapes.insert(
                "n#ParentNestedItem".into(),
                serde_json::json!({
                    "type":"structure","members":{"Id":{"target":"smithy.api#String"}}
                }),
            );
            shapes.insert("n#ChildNestedItem".into(), serde_json::json!({
            "type":"structure","members":{"Id":{"target":"smithy.api#String"},"Count":{"target":"smithy.api#Long"}}
        }));
            for prefix in ["Parent", "Child"] {
                let target = format!("n#{prefix}NestedItem");
                let shape = if container == "list" {
                    serde_json::json!({"type":"list","member":{"target":target}})
                } else {
                    serde_json::json!({"type":"map","key":{"target":"smithy.api#String"},"value":{"target":target}})
                };
                shapes.insert(format!("n#{prefix}Nested"), shape);
            }
            shapes.insert("n#ParentItem".into(), serde_json::json!({
            "type":"structure","members":{"Id":{"target":"smithy.api#String"},"Items":{"target":"n#ParentNested"}}
        }));
            shapes.insert("n#Item".into(), serde_json::json!({
            "type":"structure","members":{"Id":{"target":"smithy.api#String"},"Items":{"target":"n#ChildNested"}}
        }));
            shapes.insert(
                "n#ParentItems".into(),
                serde_json::json!({
                    "type":"list","member":{"target":"n#ParentItem"}
                }),
            );
            shapes.insert("n#ParentOutput".into(), serde_json::json!({
            "type":"structure","members":{"Items":{"target":"n#ParentItems"},"NextToken":{"target":"smithy.api#String"}}
        }));
            shapes.insert("n#ListParents0".into(), serde_json::json!({
            "type":"operation","input":{"target":"n#Input"},"output":{"target":"n#ParentOutput"}
        }));
        },
    )
}

#[tokio::test]
async fn given_extra_describe_list_element_member_when_raw_values_match_then_skip_is_refused()
-> TestResult {
    let bytes = nested_member_catalog("list")?;
    let body = r#"{"Items":[{"Id":"one","Items":[]}]}"#;
    let (report, overlay, requests) = compare_catalog(&bytes, &[body, body]).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tdifferent\t$.Items[].Count\n"
    );
    assert_eq!(overlay, "");
    assert_eq!(requests.len(), 0);
    Ok(())
}

#[tokio::test]
async fn given_extra_describe_map_value_member_when_raw_values_match_then_skip_is_refused()
-> TestResult {
    let bytes = nested_member_catalog("map")?;
    let body = r#"{"Items":[{"Id":"one","Items":{}}]}"#;
    let (report, overlay, requests) = compare_catalog(&bytes, &[body, body]).await?;
    assert_eq!(
        report,
        "bindings/ListParents0/ListChildren\tdifferent\t$.Items[].Count\n"
    );
    assert_eq!(overlay, "");
    assert_eq!(requests.len(), 0);
    Ok(())
}
