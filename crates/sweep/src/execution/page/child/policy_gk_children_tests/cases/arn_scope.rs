use super::*;

#[tokio::test]
async fn given_bound_definition_work_when_a_version_arn_has_wrong_scope_or_is_malformed_then_no_page_prefix_is_persisted()
-> TestResult {
    // Given: real pinned immutable-version policies, durable parent fanout and production request encoding.
    let compiled = compile(
        "greengrass",
        "2017-06-07",
        &[
            "ListCoreDefinitions",
            "ListCoreDefinitionVersions",
            "GetCoreDefinitionVersion",
        ],
    )?;
    let catalog = RuntimeCatalog::from_bytes(&compiled.bytes)?;
    let root = catalog
        .lookup("greengrass", "ListCoreDefinitions")
        .ok_or("definition root absent")?;
    let child = catalog
        .lookup("greengrass", "ListCoreDefinitionVersions")
        .ok_or("version plan absent")?;
    assert!(
        catalog
            .lookup("greengrass", "GetCoreDefinitionVersion")
            .is_none()
    );
    let parents = ["parent-a", "parent-b"].map(|id| json!({"Id":id,
        "Arn":format!("arn:aws:greengrass:us-east-1:123456789012:/greengrass/definition/cores/{id}")}));
    let invalid = [
        (
            "arn:aws-cn:greengrass:us-east-1:123456789012:/greengrass/definition/cores/parent-a/versions/bad",
            "modeled ARN does not match trusted request scope",
        ),
        (
            "arn:aws:greengrass:us-east-1:999999999999:/greengrass/definition/cores/parent-a/versions/bad",
            "modeled ARN does not match trusted request scope",
        ),
        ("not-an-arn", "modeled ARN is malformed"),
    ];
    for (arn, expected) in invalid {
        let scenario = Scenario::new(&catalog).await?;
        scenario.seed(root).await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1000, 10000)
            .await?
            .ok_or("root lease absent")?;
        let (request, _) = scenario.request(root, &lease)?;
        scenario
            .apply(
                &catalog,
                root,
                lease,
                &request,
                json!({"Definitions":parents}),
                2,
            )
            .await?;
        let lease = scenario
            .store
            .claim(&scenario.sweep, 1000, 10000)
            .await?
            .ok_or("bound version lease absent")?;
        let (request, _) = scenario.request(child, &lease)?;
        let field = compiled
            .fields
            .id("CoreDefinitionId")
            .ok_or("bound parent field missing")?;
        let ValueRef::Text(parent) =
            record::path_value(request.root(), &[field])?.ok_or("bound parent absent")?
        else {
            return Err("bound parent is not text".into());
        };
        let invalid_arn = arn.replace("parent-a", parent);
        // When: a valid row precedes an invalid ARN in the same decoded response page.
        let response = json!({"Versions":[
            {"Arn":format!("arn:aws:greengrass:us-east-1:123456789012:/greengrass/definition/cores/{parent}/versions/good"),"Id":parent,"Version":"good"},
            {"Arn":invalid_arn,"Id":parent,"Version":"bad"}
        ]});
        let error = match scenario
            .apply(&catalog, child, lease, &request, response, 0)
            .await
        {
            Err(error) => error,
            Ok(()) => return Err("invalid version ARN was committed".into()),
        };
        assert!(
            matches!(error.downcast_ref::<crate::SweepError>(),
            Some(crate::SweepError::Operation(message)) if *message == expected),
            "wrong rejection: {error}"
        );
        // Then: rejection is atomic, including the valid prefix, and no definition body was requested.
        let reader = scenario.reader().await?;
        let nodes: Vec<(String, String)> =
            sqlx::query_as("SELECT kind,id FROM resources ORDER BY id")
                .fetch_all(&reader)
                .await?;
        assert_eq!(nodes, ["parent-a", "parent-b"].map(|id| ("AWS::Greengrass::CoreDefinition".to_owned(),
            format!("arn:aws:greengrass:us-east-1:123456789012:/greengrass/definition/cores/{id}"))).to_vec());
        reader.close().await;
    }
    Ok(())
}
