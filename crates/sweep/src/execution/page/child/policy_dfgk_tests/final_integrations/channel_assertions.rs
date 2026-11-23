use super::*;

#[tokio::test]
async fn given_bound_channel_when_summary_parent_is_equal_absent_or_hostile_then_stream_key_pages_commit_atomically()
-> TestResult {
    let catalog = catalog()?;
    let root = fixture::plan(&catalog, "ivs", "ListChannels")?;
    let child = fixture::plan(&catalog, "ivs", "ListStreamKeys")?;
    let owner = "arn:aws:ivs:us-east-1:123456789012:channel/owner";
    for (label, duplicate, accepted) in [
        ("equal", Some(json!(owner)), true),
        ("absent", None, true),
        (
            "other-channel",
            Some(json!("arn:aws:ivs:us-east-1:123456789012:channel/other")),
            false,
        ),
        (
            "foreign-account",
            Some(json!("arn:aws:ivs:us-east-1:999999999999:channel/owner")),
            false,
        ),
        (
            "foreign-region",
            Some(json!("arn:aws:ivs:eu-west-1:123456789012:channel/owner")),
            false,
        ),
        (
            "foreign-partition",
            Some(json!("arn:aws-cn:ivs:us-east-1:123456789012:channel/owner")),
            false,
        ),
        ("null", Some(Json::Null), true),
        ("wrong-type", Some(json!(7)), false),
    ] {
        // Given persisted channel metadata and its real request-bound stream-key child lease.
        let scenario = Scenario::new(&catalog, label).await?;
        scenario.seed(root).await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10000)
            .await?
            .ok_or("channel lease absent")?;
        let (request, _) = scenario.prepared_request(root, &lease)?;
        scenario
            .apply(
                &catalog,
                root,
                lease,
                &request,
                json!({"channels":[{"arn":owner}]}),
            )
            .await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1001, 10000)
            .await?
            .ok_or("stream-key child absent")?;
        assert_eq!(lease.scope().operation().as_str(), "ListStreamKeys");
        assert_eq!(
            lease.scope().parent().map(|parent| parent.id.as_str()),
            Some(owner)
        );
        let (request, wire) = scenario.prepared_request(child, &lease)?;
        let body: Json = serde_json::from_slice(wire.body())?;
        assert_eq!(wire.method(), http::Method::POST);
        assert_eq!(wire.uri().path(), "/ListStreamKeys");
        assert_eq!(body.get("channelArn").and_then(Json::as_str), Some(owner));
        let mut second = json!({"arn":"arn:aws:ivs:us-east-1:123456789012:stream-key/second"});
        if let Some(value) = duplicate {
            second
                .as_object_mut()
                .ok_or("summary is not an object")?
                .insert("channelArn".to_owned(), value);
        }
        // When one native page contains a valid first row and a potentially hostile duplicate owner.
        let result = scenario.apply(&catalog, child, lease, &request, json!({"streamKeys":[
            {"arn":"arn:aws:ivs:us-east-1:123456789012:stream-key/first","channelArn":owner}, second
        ]})).await;
        if accepted {
            result?;
        } else {
            assert!(
                result.is_err(),
                "accepted inconsistent channel duplicate: {label}"
            );
        }
        // Then valid/absent duplicates preserve both metadata rows and request-origin edges;
        // mismatches reject the whole page rather than silently committing the valid first row.
        let keys = saved(&scenario, child.kind().as_str()).await?;
        assert_eq!(keys.len(), if accepted { 2 } else { 0 }, "{label}");
        assert_eq!(saved(&scenario, root.kind().as_str()).await?.len(), 1);
        let edges = scenario.store.projection_edges(None, 100).await?;
        assert_eq!(edges.edges.len(), if accepted { 2 } else { 0 }, "{label}");
        for edge in edges.edges {
            assert_eq!(edge.to.id.as_str(), owner);
            assert_eq!(edge.from.kind, *child.kind());
            assert_eq!(edge.relation.as_str(), "IN_CHANNEL");
        }
    }
    Ok(())
}
