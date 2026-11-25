use super::*;
mod authority;

fn response(
    plan: &CallablePlan<'_>,
    lease: &WorkLease,
    wire: &http::Request<bytes::Bytes>,
    requests: &mut Vec<(String, Option<String>)>,
) -> TestResult<Json> {
    let body: Json = serde_json::from_slice(wire.body())?;
    Ok(match plan.operation_name() {
        "ListChannels" if lease.token().is_none() => {
            assert_wire_token(wire, None, "nextToken")?;
            json!({"channels":[],"nextToken":"rootpage"})
        }
        "ListChannels" => {
            assert_wire_token(wire, Some("rootpage"), "nextToken")?;
            json!({"channels":[{"arn":"arn:aws:ivs:us-east-1:123456789012:channel/one","name":"one"},{"arn":"arn:aws:ivs:us-east-1:123456789012:channel/two","name":"two"}]})
        }
        "ListStreamKeys" => {
            let channel = text(&body, "channelArn")?;
            assert_eq!(
                Some(channel),
                lease.scope().parent().map(|parent| parent.id.as_str())
            );
            let name = channel.rsplit('/').next().ok_or("channel name absent")?;
            requests.push((
                name.to_owned(),
                body.get("nextToken")
                    .and_then(Json::as_str)
                    .map(str::to_owned),
            ));
            let (row, next) = match body.get("nextToken").and_then(Json::as_str) {
                None => (None, Some("keypage")),
                Some("keypage") => (Some("first"), Some("lastpage")),
                Some("lastpage") => (Some("second"), None),
                _ => return Err("unexpected stream-key cursor".into()),
            };
            let rows = row.map_or_else(Vec::new, |suffix| vec![json!({"arn":format!("arn:aws:ivs:us-east-1:123456789012:stream-key/{name}-{suffix}"),"channelArn":channel,"tags":{"env":"test"}})]);
            page("streamKeys", rows, next, "nextToken")
        }
        _ => return Err("forbidden stream detail/secret call scheduled".into()),
    })
}

#[tokio::test]
async fn given_channels_when_empty_and_second_stream_key_pages_resume_then_metadata_keeps_bound_channel_and_never_reads_secrets()
-> TestResult {
    let catalog = catalog()?;
    let mut scenario = Scenario::new(&catalog, "stream-key-metadata-only").await?;
    let root = fixture::plan(&catalog, "ivs", "ListChannels")?;
    let keys = fixture::plan(&catalog, "ivs", "ListStreamKeys")?;
    assert_eq!(keys.kind().as_str(), "AWS::IVS::StreamKey");
    scenario.seed(root).await?;
    let mut requests = Vec::new();
    for _ in 0..8 {
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10000)
            .await?
            .ok_or("expected channel/key page absent")?;
        let plan = fixture::plan(&catalog, "ivs", lease.scope().operation().as_str())?;
        let (request, wire) = scenario.prepared_request(plan, &lease)?;
        let response = response(plan, &lease, &wire, &mut requests)?;
        scenario
            .apply(&catalog, plan, lease, &request, response)
            .await?;
        scenario = reopen(scenario).await?;
    }
    assert!(
        scenario
            .store
            .claim(&scenario.sweep, 1001, 10000)
            .await?
            .is_none()
    );
    requests.sort();
    assert_eq!(
        requests,
        ["one", "two"]
            .into_iter()
            .flat_map(|name| [None, Some("keypage"), Some("lastpage")]
                .map(|token| (name.to_owned(), token.map(str::to_owned))))
            .collect::<Vec<_>>()
    );
    let saved_keys = saved(&scenario, keys.kind().as_str()).await?;
    assert_eq!(
        saved_keys
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        [
            "arn:aws:ivs:us-east-1:123456789012:stream-key/one-first",
            "arn:aws:ivs:us-east-1:123456789012:stream-key/one-second",
            "arn:aws:ivs:us-east-1:123456789012:stream-key/two-first",
            "arn:aws:ivs:us-east-1:123456789012:stream-key/two-second",
        ]
    );
    let edges = scenario.store.projection_edges(None, 100).await?;
    assert_eq!(edges.edges.len(), 4);
    for edge in edges.edges {
        assert_eq!(edge.relation.as_str(), "IN_CHANNEL");
        let channel = edge
            .to
            .id
            .as_str()
            .rsplit('/')
            .next()
            .ok_or("edge channel absent")?;
        assert!(
            edge.from
                .id
                .as_str()
                .contains(&format!("stream-key/{channel}-"))
        );
    }
    Ok(())
}
