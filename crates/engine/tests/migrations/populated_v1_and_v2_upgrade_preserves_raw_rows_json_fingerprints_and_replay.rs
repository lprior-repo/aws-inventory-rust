//! Behavior: populated_v1_and_v2_upgrade_preserves_raw_rows_json_fingerprints_and_replay.
use super::*;

#[tokio::test]
pub(super) async fn populated_v1_and_v2_upgrade_preserves_raw_rows_json_fingerprints_and_replay()
-> TestResult {
    for version in [1_i64, 2] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("populated.sqlite");
        let pages = populated_legacy(&path, version).await?;
        let mut connection = connect(&path).await?;
        let before = inventory_snapshot(&mut connection).await?;
        connection.close().await?;

        let store = Store::open(&path).await?;
        assert_eq!(store.scan_status("populated").await?, ScanStatus::Partial);
        let summary = store.summary("populated").await?;
        assert_eq!(
            (
                summary.resources,
                summary.edges,
                summary.completed_scopes,
                summary.failed_scopes
            ),
            (4, 4, 2, 1)
        );
        assert_eq!(
            store
                .query("SELECT user_version AS version FROM pragma_user_version", 1)
                .await?,
            json!([{"version":CURRENT_SCHEMA_VERSION}])
        );
        for table in ["resource_observations", "edge_observations"] {
            assert_eq!(store.query(&format!("SELECT DISTINCT source,source_priority FROM {table} ORDER BY source_priority DESC"), 4).await?,
                json!([{"source":"direct","source_priority":4},{"source":"aws_config","source_priority":2},{"source":"resource_explorer","source_priority":1}]));
        }
        for input in &pages {
            store.apply_page("populated", input.clone()).await?;
        }
        let pending = pages.last().ok_or("missing pending legacy page")?;
        let checkpoint = store
            .checkpoint("populated", &pending.scope)
            .await?
            .ok_or("missing saved checkpoint")?;
        assert_eq!(checkpoint.next_token.as_deref(), Some("saved-resume-token"));
        assert!(!checkpoint.complete && !checkpoint.failed);
        assert_eq!(store.summary("populated").await?, summary);
        let mut changed = pages.first().ok_or("missing legacy replay page")?.clone();
        let first = changed
            .resources
            .first_mut()
            .ok_or("missing legacy resource")?;
        first.name = Some("changed replay must not overwrite".into());
        assert!(matches!(
            store.apply_page("populated", changed).await,
            Err(StoreError::Conflict(_))
        ));
        store.close().await;

        let mut connection = connect(&path).await?;
        assert_eq!(inventory_snapshot(&mut connection).await?, before);
        let violations: i64 = sqlx::query_scalar("SELECT count(*) FROM pragma_foreign_key_check")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(violations, 0);
        connection.close().await?;
        let reopened = Store::open(&path).await?;
        for input in &pages {
            reopened.apply_page("populated", input.clone()).await?;
        }
        assert_eq!(reopened.summary("populated").await?, summary);
        reopened.close().await;
    }
    Ok(())
}
