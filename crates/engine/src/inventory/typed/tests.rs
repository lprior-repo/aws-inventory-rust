use super::bind_column;
use inventory_core::inventory::{
    AccountId, Kind, Observation, Region, ResourceId, ResourceKey, ScalarType,
};
use inventory_core::record::{Exclusions, FieldId, Float, Key, Record, Time, Value};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool, sqlite::SqlitePoolOptions};
use std::collections::BTreeMap;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn observation(value: Option<Value>) -> TestResult<Observation> {
    let mut fields = BTreeMap::new();
    if let Some(value) = value {
        fields.insert(Key::Field(FieldId::new(1)?), value);
    }
    Ok(Observation::new(
        ResourceKey {
            account: AccountId::parse("123456789012")?,
            region: Region::parse("us-east-1")?,
            kind: Kind::parse("AWS::Example::Thing")?,
            id: ResourceId::parse("thing")?,
        },
        Record::encode(&Value::Map(fields))?,
        &Exclusions::none(),
        None,
    )?)
}

async fn pool_with(columns: &str) -> TestResult<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!("CREATE TABLE r({columns})")))
        .execute(&pool)
        .await?;
    Ok(pool)
}

async fn insert(
    pool: &SqlitePool,
    names: &str,
    scalar: ScalarType,
    value: Option<Value>,
) -> TestResult<String> {
    let observation = observation(value)?;
    let mut query = QueryBuilder::<Sqlite>::new(format!("INSERT INTO r({names}) VALUES("));
    bind_column(&mut query, &observation, &[FieldId::new(1)?], scalar)?;
    query.push(")");
    let sql = query.sql().as_str().to_owned();
    query.build().execute(pool).await?;
    Ok(sql)
}

#[tokio::test]
async fn given_present_real_when_bound_then_value_and_finite_flag_are_two_separated_placeholders()
-> TestResult {
    let pool = pool_with("a REAL,b INTEGER").await?;
    let sql = insert(
        &pool,
        "a,b",
        ScalarType::Real,
        Some(Value::Float(Float::new(1.5))),
    )
    .await?;
    assert_eq!(sql, "INSERT INTO r(a,b) VALUES(?,?)");
    let row = sqlx::query("SELECT a,b FROM r").fetch_one(&pool).await?;
    assert_eq!(
        (row.try_get::<f64, _>(0)?, row.try_get::<i64, _>(1)?),
        (1.5, 0)
    );
    Ok(())
}

#[tokio::test]
async fn given_absent_and_nonfinite_real_when_bound_then_nulls_and_flags_are_stored() -> TestResult
{
    let pool = pool_with("a REAL,b INTEGER").await?;
    insert(&pool, "a,b", ScalarType::Real, None).await?;
    insert(
        &pool,
        "a,b",
        ScalarType::Real,
        Some(Value::Float(Float::new(f64::INFINITY))),
    )
    .await?;
    let rows = sqlx::query("SELECT a,b FROM r ORDER BY rowid")
        .fetch_all(&pool)
        .await?;
    let stored: Vec<(Option<f64>, Option<i64>)> = rows
        .iter()
        .map(|row| Ok((row.try_get(0)?, row.try_get(1)?)))
        .collect::<Result<_, sqlx::Error>>()?;
    assert_eq!(stored, vec![(None, None), (None, Some(1))]);
    Ok(())
}

#[tokio::test]
async fn given_present_timestamp_when_bound_then_seconds_and_nanos_are_two_separated_placeholders()
-> TestResult {
    let pool = pool_with("s INTEGER,n INTEGER").await?;
    let sql = insert(
        &pool,
        "s,n",
        ScalarType::Timestamp,
        Some(Value::Time(Time::new(1_700_000_000, 123)?)),
    )
    .await?;
    assert_eq!(sql, "INSERT INTO r(s,n) VALUES(?,?)");
    let row = sqlx::query("SELECT s,n FROM r").fetch_one(&pool).await?;
    assert_eq!(
        (row.try_get::<i64, _>(0)?, row.try_get::<i64, _>(1)?),
        (1_700_000_000, 123)
    );
    Ok(())
}

#[tokio::test]
async fn given_absent_timestamp_when_bound_then_both_physical_columns_are_null() -> TestResult {
    let pool = pool_with("s INTEGER,n INTEGER").await?;
    insert(&pool, "s,n", ScalarType::Timestamp, None).await?;
    let row = sqlx::query("SELECT s,n FROM r").fetch_one(&pool).await?;
    assert_eq!(
        (
            row.try_get::<Option<i64>, _>(0)?,
            row.try_get::<Option<i64>, _>(1)?
        ),
        (None, None)
    );
    Ok(())
}
