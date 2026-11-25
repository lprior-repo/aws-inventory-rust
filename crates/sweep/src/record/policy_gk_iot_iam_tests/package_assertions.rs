use super::*;

#[tokio::test]
async fn given_bound_package_when_native_duplicate_owner_is_equal_absent_or_hostile_then_page_commit_is_atomic_after_reopen()
-> TestResult {
    let catalog = fixture::catalog()?;
    let root = fixture::plan(&catalog, "iot", "ListPackages")?;
    let child = fixture::plan(&catalog, "iot", "ListPackageVersions")?;
    for (label, duplicate, accepted) in [
        ("equal", Some(json!("owner")), true),
        ("absent", None, true),
        ("foreign", Some(json!("foreign")), false),
        ("null", Some(Json::Null), true),
        ("wrong-type", Some(json!(7)), false),
    ] {
        let mut scenario = Scenario::new(&catalog, label).await?;
        scenario.seed(root).await?;
        let lease = scenario.claim("ListPackages").await?;
        let (request, _) = scenario.request(root, &lease)?;
        scenario
            .apply(
                &catalog,
                root,
                lease,
                &request,
                br#"{"packageSummaries":[{"packageName":"owner"}]}"#,
            )
            .await?;
        let lease = scenario.claim("ListPackageVersions").await?;
        assert_eq!(
            lease.scope().parent().map(|parent| parent.id.as_str()),
            Some("owner")
        );
        let (request, wire) = scenario.request(child, &lease)?;
        request_contains(&wire, "owner")?;
        let mut second = json!({"versionName":"v2","status":"DRAFT"});
        if let Some(value) = duplicate {
            second
                .as_object_mut()
                .ok_or("summary is not an object")?
                .insert("packageName".to_owned(), value);
        }
        let body = serde_json::to_vec(&json!({"packageVersionSummaries":[
            {"packageName":"owner","versionName":"v1","status":"DRAFT"}, second
        ]}))?;
        let result = scenario
            .apply(&catalog, child, lease, &request, &body)
            .await;
        if accepted {
            result?;
        } else {
            assert!(
                result.is_err(),
                "accepted hostile native packageName: {label}"
            );
        }
        scenario = scenario.reopen().await?;
        scenario
            .assert_nodes(
                "AWS::IoT::SoftwarePackageVersion",
                if accepted { 2 } else { 0 },
            )
            .await?;
        scenario
            .assert_nodes("AWS::IoT::SoftwarePackage", 1)
            .await?;
        let reader = scenario.reader().await?;
        let edges: Vec<(String, String)> = sqlx::query_as("SELECT to_kind,to_id FROM edges WHERE from_kind='AWS::IoT::SoftwarePackageVersion' ORDER BY from_id")
            .fetch_all(&reader).await?;
        assert_eq!(edges.len(), if accepted { 2 } else { 0 }, "{label}");
        assert!(
            edges
                .iter()
                .all(|(kind, id)| kind == "AWS::IoT::SoftwarePackage" && id == "owner")
        );
        reader.close().await;
        let pending = scenario.store.claim(&scenario.sweep, 20000, 10000).await?;
        if accepted {
            assert!(pending.is_none(), "unexpected detail or credential work");
        } else {
            let pending = pending.ok_or("rejected page lost retryable durable child work")?;
            assert_eq!(pending.scope().operation().as_str(), "ListPackageVersions");
            assert!(
                pending.token().is_none(),
                "rejected page advanced its persisted cursor"
            );
        }
    }
    Ok(())
}
