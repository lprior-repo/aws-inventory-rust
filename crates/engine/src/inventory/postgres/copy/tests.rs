use super::{Field, field, load, row};
use inventory_core::record::Time;
use sqlx::{Connection, PgConnection, Row};

const TYPES: &str = "CREATE TEMP TABLE copy_types (id bigint, t text, i bigint, f double precision, b boolean, bytes bytea, ts timestamptz, j jsonb)";
const COPY_TYPES: &str = "COPY pg_temp.copy_types FROM STDIN (FORMAT binary)";

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn connect() -> TestResult<PgConnection> {
    Ok(PgConnection::connect(&inventory_harness::postgres::admin_url()?).await?)
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn binary_copy_round_trips_each_postgres_type_and_null() -> TestResult {
    // Given a row containing every field type and another containing NULLs.
    let mut connection = connect().await?;
    let mut tx = connection.begin().await?;
    let timestamp = Time::new(-1, 876_543_210)?;
    let fields = [
        Field::Text("café🦀"),
        Field::Bigint(i64::MIN),
        Field::Double(-42.125),
        Field::Boolean(true),
        Field::Bytes(&[0, 255, 10]),
        Field::Timestamp(timestamp),
        Field::Jsonb(Some(r#"{"a":[true,null]}"#)),
    ];
    // When PostgreSQL reads the binary stream.
    let count = load(&mut tx, TYPES, COPY_TYPES, &[1, 2], |buffer, id| {
        row(buffer, 8)?;
        field(buffer, Field::Bigint(*id))?;
        fields
            .into_iter()
            .try_for_each(|value| field(buffer, if *id == 1 { value } else { Field::Null }))
    })
    .await?;
    // Then PostgreSQL decoded every value, including the pre-Unix timestamp and NULLs.
    assert_eq!(count, 2);
    let values = sqlx::query(
        "SELECT *, ts = timestamptz '1969-12-31 23:59:59.876543+00' AS time_matches, j = '{\"a\":[true,null]}'::jsonb AS json_matches FROM copy_types WHERE id = 1",
    )
    .fetch_one(&mut *tx)
    .await?;
    assert_eq!(values.try_get::<&str, _>("t")?, "café🦀");
    assert_eq!(values.try_get::<i64, _>("i")?, i64::MIN);
    assert_eq!(values.try_get::<f64, _>("f")?, -42.125);
    assert!(values.try_get::<bool, _>("b")?);
    assert_eq!(values.try_get::<Vec<u8>, _>("bytes")?, [0, 255, 10]);
    assert!(values.try_get::<bool, _>("time_matches")?);
    assert!(values.try_get::<bool, _>("json_matches")?);
    let all_null: bool = sqlx::query_scalar("SELECT t IS NULL AND i IS NULL AND f IS NULL AND b IS NULL AND bytes IS NULL AND ts IS NULL AND j IS NULL FROM copy_types WHERE id = 2")
        .fetch_one(&mut *tx).await?;
    assert!(all_null);
    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires INVENTORY_POSTGRES_URL"]
async fn staging_copies_100000_rows_and_clears_at_commit_on_the_same_connection() -> TestResult {
    // Given a staging table on a single connection and 100,000 distinct rows.
    let mut connection = connect().await?;
    let rows: Vec<i64> = (1..=100_000).collect();
    let mut tx = connection.begin().await?;
    // When COPY sends many chunks and the transaction commits.
    let copied = load(
        &mut tx,
        "CREATE TEMPORARY TABLE IF NOT EXISTS stg_copy_test (id bigint NOT NULL) ON COMMIT DELETE ROWS",
        "COPY pg_temp.stg_copy_test FROM STDIN (FORMAT binary)",
        &rows,
        |buffer, id| {
            row(buffer, 1)?;
            field(buffer, Field::Bigint(*id))
        },
    )
    .await?;
    let staged: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT count(*),count(DISTINCT id),min(id),max(id) FROM pg_temp.stg_copy_test",
    )
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    // Then all rows arrived before commit, and none remain afterward.
    assert_eq!(copied, 100_000);
    assert_eq!(staged, (100_000, 100_000, 1, 100_000));
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_temp.stg_copy_test")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(remaining, 0);
    Ok(())
}
