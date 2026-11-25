use super::*;
#[tokio::test]
async fn given_security_lake_regional_roots_when_encoded_and_committed_then_exact_routed_region_is_the_only_query_selector()
-> TestResult {
    use crate::input;
    let kind = "AWS::SecurityLake::DataLake";
    for region in ["us-east-1", "us-west-2"] {
        let scenario = Scenario::new_region("securitylake", "ListDataLakes", region).await?;
        let arn = format!("arn:aws:securitylake:{region}:123456789012:data-lake/default");
        scenario
            .run(|page| {
                assert_eq!(page.plan.operation_name(), "ListDataLakes");
                assert!(!input::requires_parent(page.plan));
                assert!(page.plan.pagination().tokens.is_empty());
                assert!(page.lease.scope().parent().is_none());
                assert_eq!(page.lease.scope().region().as_str(), region);
                assert_eq!(
                    page.wire.uri().query(),
                    Some(format!("regions={region}").as_str())
                );
                assert!(page.wire.body().is_empty());
                Ok(serde_json::json!({"dataLakes":[{"dataLakeArn":arn,"region":region}]}))
            })
            .await?;
        assert_eq!(
            scenario.keys(kind).await?,
            vec![("123456789012".into(), region.into(), kind.into(), arn)]
        );
        let plan = scenario
            .catalog
            .lookup("securitylake", "ListDataLakes")
            .ok_or("data lake root absent")?;
        assert!(matches!(
            input::root_bindings(plan, Some(&AccountId::parse("123456789012")?), None),
            Err(SweepError::Operation(
                "region list binding requires the routed request region"
            ))
        ));
    }
    let scenario = Scenario::new("securitylake", "ListDataLakes").await?;
    assert!(scenario.run(|_| Ok(serde_json::json!({"dataLakes":[{
        "dataLakeArn":"arn:aws:securitylake:us-west-2:123456789012:data-lake/default","region":"us-west-2"}]}))).await.is_err());
    assert!(scenario.keys(kind).await?.is_empty());
    Ok(())
}
