use super::*;
use std::cell::Cell;

const ARN: &str = "arn:aws:timestream-influxdb:us-east-1:123456789012:db-parameter-group/abc123";

#[tokio::test]
async fn given_empty_and_valid_sources_when_fake_transport_sweeps_then_only_valid_tag_child_runs()
-> TestResult {
    let compiled = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled)?;
    let scenario = seeded(&catalog).await?;
    let (scenario, seen) = scenario
        .drain(&catalog, "timestream-influxdb", |plan, _, wire, _| {
            respond(plan, wire, ARN)
        })
        .await?;
    assert_eq!(seen.get("ListDbParameterGroups"), Some(&1));
    assert_eq!(seen.get("ListTagsForResource"), Some(&1));
    scenario
        .assert_ids(
            "AWS::ApiOnly::TimestreamInfluxDBParameterGroup",
            vec!["InfluxDBV3Core".into(), "abc123".into()],
        )
        .await?;
    let summary = scenario.store.sweep_summary(&scenario.sweep).await?;
    assert_eq!((summary.complete_scopes, summary.failed_scopes), (2, 0));
    Ok(())
}

#[tokio::test]
async fn given_nonempty_invalid_arns_when_fake_transport_rejects_then_error_is_not_silently_skipped()
-> TestResult {
    let compiled = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled)?;
    for invalid in ["not-an-arn", " ", " arn:invalid "] {
        let scenario = seeded(&catalog).await?;
        let calls = Cell::new(0_u64);
        let result = scenario
            .drain(&catalog, "timestream-influxdb", |plan, _, wire, _| {
                if plan.operation_name() == "ListTagsForResource" {
                    assert_source(wire, invalid)?;
                    calls.set(calls.get().saturating_add(1));
                    return Err("ValidationException: invalid resource ARN".into());
                }
                respond(plan, wire, invalid)
            })
            .await;
        assert_eq!(calls.get(), 1);
        let error = result.err().ok_or("invalid ARN error was swallowed")?;
        assert!(error.to_string().contains("ValidationException"));
    }
    Ok(())
}

fn compiled() -> TestResult<Vec<u8>> {
    Ok(crate::consumer_fixture::compile(
        "timestream-influxdb",
        "2023-01-27",
        &[
            "ListDbBackups",
            "ListDbClusters",
            "ListDbInstances",
            "ListDbParameterGroups",
            "ListTagsForResource",
        ],
    )?
    .bytes)
}

async fn seeded<'a, 'catalog>(
    catalog: &'a RuntimeCatalog<'catalog>,
) -> TestResult<Scenario<'a, 'catalog>> {
    let scenario = Scenario::new(catalog, "influx-tag-source").await?;
    scenario
        .seed(catalog, "timestream-influxdb", &["ListDbParameterGroups"])
        .await?;
    Ok(scenario)
}

fn respond(
    plan: &CallablePlan<'_>,
    wire: &http::Request<bytes::Bytes>,
    source: &str,
) -> TestResult<Json> {
    match plan.operation_name() {
        "ListDbParameterGroups" => Ok(json!({"items":[
            {"id":"InfluxDBV3Core","name":"InfluxDBV3Core","arn":""},
            {"id":"abc123","name":"customer","arn":source}
        ]})),
        "ListTagsForResource" => {
            assert_source(wire, source)?;
            Ok(json!({"tags":{"source":"fake-transport"}}))
        }
        _ => Err("unexpected parameter-group operation".into()),
    }
}

fn assert_source(wire: &http::Request<bytes::Bytes>, source: &str) -> TestResult {
    let input: Json = serde_json::from_slice(wire.body())?;
    assert_eq!(input.get("resourceArn").and_then(Json::as_str), Some(source));
    Ok(())
}
