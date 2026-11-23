use super::*;
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
