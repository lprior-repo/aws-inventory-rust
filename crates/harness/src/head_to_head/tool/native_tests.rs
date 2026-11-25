use super::*;
use crate::postgres::{TestDatabase, admin_url};
use sqlx::{Connection, Executor, postgres::PgConnection};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn given_10001_postgres_kinds_when_counting_then_truncation_is_rejected() -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let result = async {
        let mut connection = PgConnection::connect(database.url()).await?;
        connection.execute("CREATE SCHEMA inventory").await?;
        connection.execute("CREATE TABLE inventory.resources (kind text NOT NULL)").await?;
        sqlx::query("INSERT INTO inventory.resources (kind) SELECT 'kind-' || n::text FROM generate_series(1, 10001) AS n")
            .execute(&mut connection).await?;
        connection.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(
            read_counts(database.url(), Duration::from_secs(60)).await
                .err().ok_or("expected an error")?.to_string()
        )
    }.await;
    database.finish().await?;
    assert_eq!(result?, "counts exceed 10000 kinds");
    Ok(())
}
