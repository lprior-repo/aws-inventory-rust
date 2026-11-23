use super::*;

const KIND: &str = "AWS::ApiOnly::TimestreamInfluxDBParameterGroup";

#[tokio::test]
async fn given_influx_defaults_with_empty_arns_when_fake_transport_sweeps_then_all_ids_persist()
-> TestResult {
    let compiled = compiled()?;
    let catalog = RuntimeCatalog::from_bytes(&compiled)?;
    let scenario = Scenario::new(&catalog, "mixed-influx-groups").await?;
    scenario
        .seed(&catalog, "timestream-influxdb", &["ListDbParameterGroups"])
        .await?;
    let (scenario, seen) = scenario
        .drain(&catalog, "timestream-influxdb", |plan, _, wire, resumed| {
            assert!(!resumed);
            response(plan.operation_name(), wire)
        })
        .await?;
    assert_eq!(seen.get("ListDbParameterGroups"), Some(&1));
    assert_eq!(seen.get("GetDbParameterGroup"), Some(&2));
    scenario
        .assert_ids(KIND, vec!["InfluxDBV3Core".to_owned(), "abc123".to_owned()])
        .await?;
    assert_summary(&scenario).await?;
    Ok(())
}

fn mixed_page() -> Json {
    json!({"items":[
        {"id":"InfluxDBV3Core","name":"InfluxDBV3Core","arn":""},
        {"id":"abc123","name":"customer","arn":"arn:aws:timestream-influxdb:us-east-1:123456789012:db-parameter-group/abc123"}
    ]})
}

fn detail(wire: &http::Request<bytes::Bytes>) -> TestResult<Json> {
    let input: Json = serde_json::from_slice(wire.body())?;
    let id = input
        .get("identifier")
        .and_then(Json::as_str)
        .ok_or("identifier missing")?;
    let arn = if id == "abc123" {
        "arn:aws:timestream-influxdb:us-east-1:123456789012:db-parameter-group/abc123"
    } else {
        ""
    };
    Ok(json!({"id":id,"name":id,"arn":arn}))
}

fn response(operation: &str, wire: &http::Request<bytes::Bytes>) -> TestResult<Json> {
    assert_eq!(wire.method(), http::Method::POST);
    match operation {
        "ListDbParameterGroups" => Ok(mixed_page()),
        "GetDbParameterGroup" => detail(wire),
        _ => Err("unexpected parameter-group operation".into()),
    }
}

async fn assert_summary(scenario: &Scenario<'_, '_>) -> TestResult {
    let summary = scenario.store.sweep_summary(&scenario.sweep).await?;
    assert_eq!(summary.complete_scopes, 3);
    assert_eq!(summary.failed_scopes, 0);
    Ok(())
}

fn compiled() -> TestResult<Vec<u8>> {
    Ok(crate::consumer_fixture::compile(
        "timestream-influxdb",
        "2023-01-27",
        &["ListDbParameterGroups", "GetDbParameterGroup"],
    )?
    .bytes)
}
