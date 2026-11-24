use super::*;

#[path = "../../../../tests/verify_cloud_control/store.rs"]
pub(super) mod store;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn given_engine_created_read_only_sqlite_when_reading_then_kind_and_file_bytes_are_preserved()
-> TestResult {
    let work = tempfile::tempdir()?;
    let database = work.path().join("inventory.sqlite");
    store::sqlite(
        &database,
        &[
            store::resource("123456789012", "us-east-1", "AWS::SNS::Topic", "topic")?,
            store::resource("123456789012", "us-east-1", "AWS::S3::Bucket", "bucket")?,
        ],
    )
    .await?;
    let before = std::fs::read(&database)?;
    let mut reader = Reader::connect(&format!("sqlite://{}", database.display())).await?;
    assert_eq!(
        reader
            .identities("AWS::SNS::Topic", "us-east-1", None)
            .await?,
        BTreeSet::from(["topic".to_owned()])
    );
    assert_eq!(
        reader
            .identities("AWS::EC2::VPC", "us-east-1", None)
            .await?,
        BTreeSet::new()
    );
    let Reader::Sqlite(connection) = &mut reader else {
        return Err("expected SQLite reader".into());
    };
    let error =
        sqlx::query("INSERT INTO resources(kind,id) VALUES ('AWS::SNS::Topic','unexpected')")
            .execute(connection)
            .await
            .err()
            .ok_or("expected a read-only write error")?;
    let error = error
        .as_database_error()
        .ok_or("expected a database error")?;
    assert_eq!(error.code().as_deref(), Some("8"));
    reader.close().await?;
    assert_eq!(std::fs::read(&database)?, before);
    Ok(())
}

#[tokio::test]
async fn given_unknown_database_scheme_when_opened_then_rejected() -> TestResult {
    let error = Reader::connect("mysql://localhost/inventory")
        .await
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control database must be a PostgreSQL or SQLite URL"
    );
    Ok(())
}

#[tokio::test]
async fn given_inventory_bound_when_reading_then_exact_limit_passes_and_next_identity_fails()
-> TestResult {
    let work = tempfile::tempdir()?;
    let database = work.path().join("inventory.sqlite");
    store::sqlite(&database, &bounded_rows()?).await?;
    let url = format!("sqlite://{}", database.display());
    let mut reader = Reader::connect(&url).await?;
    let expected: BTreeSet<_> = (1..=100_000).map(|id| id.to_string()).collect();
    assert_eq!(
        reader
            .identities("AWS::SNS::Topic", "us-east-1", None)
            .await?,
        expected
    );
    let error = reader
        .identities("AWS::S3::Bucket", "us-east-1", None)
        .await
        .err()
        .ok_or("expected a bound error")?;
    assert_eq!(
        error.to_string(),
        "Cloud Control inventory identity bound exceeded"
    );
    reader.close().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_postgres_reader_when_connected_then_session_is_read_only() -> TestResult {
    use crate::postgres::{TestDatabase, admin_url};
    let database = TestDatabase::create(&admin_url()?).await?;
    let result = async {
        store::postgres(database.url(), &[]).await?;
        let mut reader = Reader::connect(database.url()).await?;
        let Reader::Postgres(connection) = &mut reader else {
            return Err("expected PostgreSQL reader".into());
        };
        let setting = sqlx::query_scalar::<_, String>("SHOW transaction_read_only")
            .fetch_one(connection)
            .await?;
        reader.close().await?;
        assert_eq!(setting, "on");
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    let cleanup = database.finish().await;
    result?;
    cleanup?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_postgres_identity_bound_when_reading_then_exact_limit_passes_and_next_row_fails()
-> TestResult {
    use crate::postgres::{TestDatabase, admin_url};
    let database = TestDatabase::create(&admin_url()?).await?;
    let result = async {
        store::postgres(database.url(), &bounded_rows()?).await?;
        let mut reader = Reader::connect(database.url()).await?;
        let accepted = reader
            .identities("AWS::SNS::Topic", "us-east-1", None)
            .await;
        let rejected = reader
            .identities("AWS::S3::Bucket", "us-east-1", None)
            .await;
        reader.close().await?;
        let expected: BTreeSet<_> = (1..=100_000).map(|id| id.to_string()).collect();
        assert_eq!(accepted?, expected);
        let error = rejected.err().ok_or("expected a bound error")?;
        assert_eq!(
            error.to_string(),
            "Cloud Control inventory identity bound exceeded"
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    }
    .await;
    let cleanup = database.finish().await;
    result?;
    cleanup?;
    Ok(())
}

fn bounded_rows() -> Result<Vec<inventory_core::inventory::ResourceKey>, Box<dyn std::error::Error>>
{
    [
        ("AWS::SNS::Topic", "us-east-1", 100_000),
        ("AWS::S3::Bucket", "global", 100_001),
    ]
    .into_iter()
    .flat_map(|(kind, region, limit)| {
        (1..=limit).map(move |id| store::resource("123456789012", region, kind, &id.to_string()))
    })
    .collect()
}
