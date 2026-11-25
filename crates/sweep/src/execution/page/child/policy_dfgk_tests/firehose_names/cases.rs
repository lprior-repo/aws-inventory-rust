use super::*;
#[tokio::test]
async fn given_native_firehose_names_when_last_name_continues_and_nonempty_final_page_stops_then_literal_scoped_arns_survive_reopen()
-> TestResult {
    let catalog = catalog()?;
    assert!(
        catalog
            .lookup("firehose", "DescribeDeliveryStream")
            .is_none()
    );
    assert_eq!(MAX_NAME.len(), 64);
    let plan = fixture::plan(&catalog, "firehose", "ListDeliveryStreams")?;
    let scenario = Scenario::new(&catalog, "native-firehose-last-name").await?;
    scenario.seed(plan).await?;
    first_page(&scenario, &catalog, plan).await?;
    let lease = claim(&scenario, 1001).await?;
    let request = native_request(
        &scenario,
        plan,
        &lease,
        json!({"ExclusiveStartDeliveryStreamName":"stream.Middle-09_"}),
    )?;
    scenario
        .apply(
            &catalog,
            plan,
            lease,
            &request,
            json!({"DeliveryStreamNames":[MAX_NAME],"HasMoreDeliveryStreams":false}),
        )
        .await?;
    assert_persisted(scenario, &[FIRST_ARN, LAST_ARN, MAX_ARN], "complete").await
}

#[tokio::test]
async fn given_a_durable_firehose_marker_when_true_page_has_empty_or_missing_names_then_whole_page_rejects_and_resume_keeps_prior_data()
-> TestResult {
    for invalid in [
        json!({"DeliveryStreamNames":[],"HasMoreDeliveryStreams":true}),
        json!({"HasMoreDeliveryStreams":true}),
    ] {
        let catalog = catalog()?;
        let plan = fixture::plan(&catalog, "firehose", "ListDeliveryStreams")?;
        let scenario = Scenario::new(&catalog, "native-firehose-invalid-next").await?;
        scenario.seed(plan).await?;
        first_page(&scenario, &catalog, plan).await?;
        let lease = claim(&scenario, 1001).await?;
        let saved_cursor = lease.token().ok_or("prior native cursor absent")?.to_vec();
        let request = native_request(
            &scenario,
            plan,
            &lease,
            json!({"ExclusiveStartDeliveryStreamName":"stream.Middle-09_"}),
        )?;
        let reader = scenario.reader().await?;
        let before = state(&reader).await?;
        let error = match scenario
            .apply(&catalog, plan, lease, &request, invalid)
            .await
        {
            Ok(()) => return Err("malformed Firehose continuation was accepted".into()),
            Err(error) => error,
        };
        let Some(SweepError::Operation(message)) = error.downcast_ref::<SweepError>() else {
            return Err(format!("wrong native continuation error: {error}").into());
        };
        assert_eq!(
            *message,
            "pagination response omitted required continuation"
        );
        assert_eq!(state(&reader).await?, before);
        reader.close().await;
        let resumed = reopen(scenario).await?;
        let reader = resumed.reader().await?;
        assert_eq!(state(&reader).await?, before);
        let lease = claim(&resumed, 12_000).await?;
        assert_eq!(lease.token(), Some(saved_cursor.as_slice()));
        assert_eq!(
            native_request(
                &resumed,
                plan,
                &lease,
                json!({"ExclusiveStartDeliveryStreamName":"stream.Middle-09_"})
            )?
            .as_bytes(),
            request.as_bytes()
        );
        let actual: Vec<ScopedKey> = sqlx::query_as(
            "SELECT account,region,kind,id FROM resources ORDER BY account,region,kind,id",
        )
        .fetch_all(&reader)
        .await?;
        assert_eq!(
            actual,
            vec![
                (
                    "123456789012".to_owned(),
                    "us-east-1".to_owned(),
                    KIND.to_owned(),
                    FIRST_ARN.to_owned()
                ),
                (
                    "123456789012".to_owned(),
                    "us-east-1".to_owned(),
                    KIND.to_owned(),
                    LAST_ARN.to_owned()
                )
            ]
        );
        reader.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_a_reopened_native_firehose_cursor_when_last_marker_repeats_then_new_page_data_persists_as_partial_without_more_work()
-> TestResult {
    let catalog = catalog()?;
    let plan = fixture::plan(&catalog, "firehose", "ListDeliveryStreams")?;
    let scenario = Scenario::new(&catalog, "native-firehose-repeated-marker").await?;
    scenario.seed(plan).await?;
    first_page(&scenario, &catalog, plan).await?;
    let scenario = reopen(scenario).await?;
    let lease = claim(&scenario, 1001).await?;
    let request = native_request(
        &scenario,
        plan,
        &lease,
        json!({"ExclusiveStartDeliveryStreamName":"stream.Middle-09_"}),
    )?;
    scenario
        .apply(
            &catalog,
            plan,
            lease,
            &request,
            json!({"DeliveryStreamNames":["b",LAST_NAME],"HasMoreDeliveryStreams":true}),
        )
        .await?;
    assert_persisted(scenario, &[FIRST_ARN, NEW_ARN, LAST_ARN], "partial").await
}
