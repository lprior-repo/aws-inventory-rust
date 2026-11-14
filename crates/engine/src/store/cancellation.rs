use super::*;

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    use std::collections::BTreeMap;
    use tokio::sync::Notify;

    #[tokio::test]
    pub(super) async fn cancelling_after_first_sqlite_observation_rolls_back_whole_page()
    -> Result<(), StoreError> {
        let store = Store::open(":memory:").await?;
        store.begin_scan("cancel", 1).await?;
        let first_write = Arc::new(Notify::new());
        let notification = Arc::clone(&first_write);
        {
            let mut writer = store.writer.acquire().await?;
            writer.lock_handle().await?.set_update_hook(move |event| {
                if event.table == "resource_observations" {
                    notification.notify_one();
                }
            });
        }
        let scope = CollectionScope {
            account_id: "111111111111".to_owned(),
            region: "us-east-1".to_owned(),
            collector: "ec2".to_owned(),
            parent_id: None,
            source: Source::Direct,
        };
        let resources = (0..1_000_u32)
            .map(|index| Resource {
                id: format!("i-{index}"),
                account_id: scope.account_id.clone(),
                region: scope.region.clone(),
                kind: "aws_ec2_instance".to_owned(),
                name: None,
                properties: serde_json::json!({}),
                tags: BTreeMap::new(),
                source: Source::Direct,
                observed_at: 10,
            })
            .collect();
        let page = CollectionPage {
            scope: scope.clone(),
            page_number: 1,
            next_token: Some("continue".to_owned()),
            resources,
            edges: Vec::new(),
        };
        {
            let ingestion = store.apply_page("cancel", page);
            tokio::pin!(ingestion);
            tokio::select! {
                biased;
                () = first_write.notified() => {}
                result = &mut ingestion => {
                    result?;
                    return Err(StoreError::Conflict("page completed before cancellation injection"));
                }
            }
        }

        {
            let mut writer = store.writer.acquire().await?;
            writer.lock_handle().await?.remove_update_hook();
        }
        assert_eq!(store.summary("cancel").await?.resources, 0);
        assert!(store.checkpoint("cancel", &scope).await?.is_none());
        assert_eq!(
            store
                .query("SELECT count(*) AS n FROM collection_pages", 1)
                .await?,
            serde_json::json!([{"n":0}])
        );
        store
            .apply_page(
                "cancel",
                CollectionPage {
                    scope,
                    page_number: 1,
                    next_token: None,
                    resources: Vec::new(),
                    edges: Vec::new(),
                },
            )
            .await?;
        assert_eq!(
            store
                .finish_scan("cancel", ScanStatus::Complete, 20)
                .await?
                .completed_scopes,
            1
        );
        store.close().await;
        Ok(())
    }
}
