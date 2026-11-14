use super::*;

#[tokio::test]
pub(super) async fn reopen_preserves_atomic_page_and_resume_token() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("inventory.sqlite");
    let mut input = page(
        "ec2",
        Source::Direct,
        vec![resource("i-1", Source::Direct, 10)],
    );
    input.edges.push(relationship());
    input.next_token = Some("continuation".to_owned());
    {
        let store = Store::open(&path).await?;
        store.begin_scan("one", 1).await?;
        store.apply_page("one", input.clone()).await?;
        store.close().await;
    }
    let reopened = Store::open(&path).await?;
    assert_eq!(reopened.resources("one").await?, input.resources);
    assert_eq!(reopened.edges("one").await?, input.edges);
    let saved = reopened
        .checkpoint("one", &input.scope)
        .await?
        .ok_or(StoreError::Conflict("missing persisted checkpoint"))?;
    assert_eq!(saved.last_page, 1);
    assert_eq!(saved.next_token.as_deref(), Some("continuation"));
    assert!(!saved.complete);
    reopened.apply_page("one", input.clone()).await?;
    let mut connection = <sqlx::SqliteConnection as sqlx::Connection>::connect_with(
        &sqlx::sqlite::SqliteConnectOptions::new().filename(&path),
    )
    .await?;
    let mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(mode, "wal");
    sqlx::Connection::close(connection).await?;
    reopened.close().await;
    Ok(())
}
