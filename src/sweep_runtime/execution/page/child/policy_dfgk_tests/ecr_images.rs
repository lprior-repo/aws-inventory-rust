use super::*;

#[tokio::test]
async fn given_many_unrelated_plans_when_indexed_repository_pages_resume_then_only_three_image_lifecycle_branches_persist() -> TestResult {
    let catalog = fixture::catalog()?;
    let unrelated = catalog.plans().iter().filter(|plan| plan.service_name() != "ecr").count();
    assert!(unrelated >= 20, "fixture must retain many unrelated producer plans");
    let scenario = Scenario::new(&catalog, "ecr-image-lifecycle-branches").await?;
    let root = fixture::plan(&catalog, "ecr", "DescribeRepositories")?;
    scenario.seed(root).await?;
    let lease = scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.ok_or("repository work absent")?;
    let (request, _) = scenario.request(root, &lease)?;
    let repositories = ["alpha", "beta"].map(|name| json!({
        "registryId":"123456789012", "repositoryName":name,
        "repositoryArn":format!("arn:aws:ecr:us-east-1:123456789012:repository/{name}")
    }));
    scenario.apply(&catalog, root, lease, &request, json!({"repositories":repositories})).await?;
    let mut requested = BTreeSet::new();
    for _ in 0..12 {
        let lease = scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.ok_or("repository/status pagination chain was lost")?;
        assert_eq!(lease.scope().operation().as_str(), "DescribeImages");
        assert_eq!(lease.scope().service().as_str(), "ecr");
        let plan = fixture::plan(&catalog, "ecr", "DescribeImages")?;
        let (request, wire) = scenario.request(plan, &lease)?;
        let repository = text(&wire, "repositoryName")?.to_owned();
        let status = wire.get("filter").and_then(|value| value.get("imageStatus"))
            .and_then(Json::as_str).ok_or("image lifecycle filter absent")?.to_owned();
        let second_page = lease.token().is_some();
        assert_eq!(lease.scope().branch(), Some(status.as_str()));
        assert_eq!(wire.get("nextToken").and_then(Json::as_str), second_page.then_some("same-next-token"));
        assert_eq!(lease.scope().parent().ok_or("image work lost repository")?.id.as_str(),
            format!("arn:aws:ecr:us-east-1:123456789012:repository/{repository}"));
        assert!(requested.insert((repository.clone(), status.clone(), second_page)));
        let mut response = json!({"imageDetails":[{"registryId":"123456789012",
            "repositoryName":repository, "imageDigest":format!("sha256:{status}-{second_page}"),
            "imageStatus":status}]});
        if !second_page {
            response.as_object_mut().ok_or("response is not an object")?
                .insert("nextToken".to_owned(), json!("same-next-token"));
        }
        scenario.apply(&catalog, plan, lease, &request, response).await?;
    }
    let expected = ["alpha", "beta"].into_iter().flat_map(|repository| {
        ["ACTIVE", "ARCHIVED", "ACTIVATING"].into_iter().flat_map(move |status| {
            [false, true].into_iter().map(move |page| (repository.to_owned(), status.to_owned(), page))
        })
    }).collect::<BTreeSet<_>>();
    assert_eq!(requested, expected);
    assert!(scenario.store.claim(&scenario.sweep, 1001, 10_000).await?.is_none());
    let reader = scenario.reader().await?;
    let images: Vec<String> = sqlx::query_scalar("SELECT id FROM resource_facets WHERE kind='AWS::ApiOnly::ECRImage' ORDER BY id")
        .fetch_all(&reader).await?;
    assert_eq!(images.len(), 12);
    assert_eq!(images.iter().collect::<BTreeSet<_>>().len(), 12);
    let repositories: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resource_facets WHERE kind='AWS::ECR::Repository'")
        .fetch_one(&reader).await?;
    assert_eq!(repositories, 2);
    reader.close().await;
    Ok(())
}
