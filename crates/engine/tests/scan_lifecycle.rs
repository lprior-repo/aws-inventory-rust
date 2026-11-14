use inventory_core::{CollectionPage, CollectionScope, ScanStatus, Source};
use inventory_engine::store::{Store, StoreError};
use sqlx::{Connection, SqliteConnection};

type TestResult = Result<(), Box<dyn std::error::Error>>;

async fn inconsistent_store(
    status: &str,
    finished_at: Option<i64>,
) -> Result<(tempfile::TempDir, Store), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let store = Store::open(&path).await?;
    store.begin_scan("corrupt", 10).await?;
    store.close().await;
    let options = sqlx::sqlite::SqliteConnectOptions::new().filename(&path);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    sqlx::query("UPDATE scans SET status=?1,finished_at=?2 WHERE scan_id='corrupt'")
        .bind(status)
        .bind(finished_at)
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    Ok((directory, Store::open(path).await?))
}

fn page() -> CollectionPage {
    CollectionPage {
        scope: CollectionScope {
            account_id: "123456789012".into(),
            region: "us-east-1".into(),
            collector: "ec2".into(),
            parent_id: None,
            source: Source::Direct,
        },
        page_number: 1,
        next_token: None,
        resources: Vec::new(),
        edges: Vec::new(),
    }
}

fn rejects_lifecycle<T>(result: Result<T, StoreError>) {
    assert!(matches!(
        result,
        Err(StoreError::Conflict("invalid persisted scan lifecycle"))
    ));
}

#[tokio::test]
async fn given_running_row_with_finish_time_when_read_or_written_then_rejected() -> TestResult {
    let (_directory, store) = inconsistent_store("running", Some(20)).await?;
    rejects_lifecycle(store.scan_status("corrupt").await);
    rejects_lifecycle(store.summary("corrupt").await);
    rejects_lifecycle(store.begin_scan("corrupt", 10).await);
    rejects_lifecycle(store.apply_page("corrupt", page()).await);
    rejects_lifecycle(
        store
            .finish_scan("corrupt", ScanStatus::Cancelled, 20)
            .await,
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS n FROM collection_pages", 1)
            .await?,
        serde_json::json!([{"n": 0}])
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn given_terminal_row_without_finish_time_when_read_or_finished_then_rejected() -> TestResult
{
    for status in ["complete", "partial", "cancelled"] {
        let (_directory, store) = inconsistent_store(status, None).await?;
        rejects_lifecycle(store.scan_status("corrupt").await);
        rejects_lifecycle(store.summary("corrupt").await);
        rejects_lifecycle(
            store
                .finish_scan("corrupt", ScanStatus::Cancelled, 20)
                .await,
        );
        store.close().await;
    }
    Ok(())
}
