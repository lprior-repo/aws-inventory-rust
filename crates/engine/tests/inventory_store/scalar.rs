use super::support::*;
use inventory_core::{
    catalog::runtime::RuntimeCatalog,
    inventory::{Observation, ScalarType},
    record::{Exclusions, Record, Value, ValueRef, scalar_item, scalar_record},
};
use inventory_engine::inventory::{BinaryStore, SchemaRegistry, SweepId};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

#[test]
fn given_native_scalar_catalogs_when_checked_then_self_identity_and_typed_root_column_are_preserved()
-> TestResult {
    use inventory_core::catalog::runtime::{IdentityPlan, RelationSource};
    for scalar in [ScalarType::Text, ScalarType::Integer, ScalarType::Bool] {
        let bytes = scalar_catalog_bytes(scalar)?;
        let runtime = RuntimeCatalog::from_bytes(&bytes)?;
        let plan = runtime
            .lookup("example", "ListThings")
            .ok_or("scalar plan")?;
        let IdentityPlan::Key(paths) = plan.identity() else {
            return Err("expected scalar key identity".into());
        };
        assert_eq!(paths.len(), 1);
        assert!(paths.first().ok_or("scalar self path")?.is_empty());
        let column = runtime
            .schemas()
            .first()
            .ok_or("scalar schema")?
            .columns()
            .first()
            .ok_or("scalar column")?;
        assert!(column.path().is_empty());
        assert_eq!(column.scalar(), scalar);
        if scalar == ScalarType::Text {
            assert_eq!(runtime.relationships().len(), 2);
            assert!(
                runtime
                    .relationships()
                    .iter()
                    .all(|relation| relation.source == RelationSource::Request
                        && relation.target_path.is_empty())
            );
        }
    }
    Ok(())
}

async fn given_store(scalar: ScalarType) -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let catalog = scalar_catalog_bytes(scalar)?;
    let runtime = RuntimeCatalog::from_bytes(&catalog)?;
    let registry = SchemaRegistry::new(&catalog, runtime.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store))
}

async fn when_committed(store: &BinaryStore, value: ValueRef<'_>) -> TestResult {
    let observation = Observation::new(
        resource("scalar")?,
        scalar_record(value)?,
        &Exclusions::none(),
        None,
    )?;
    store.seed_work(work("scalar-sweep", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("scalar-sweep".to_owned())?, 100, 20)
        .await?
        .ok_or("missing scalar lease")?;
    store
        .commit_page(page(lease, vec![observation], true, None, 101))
        .await?;
    Ok(())
}

async fn then_reopened(
    directory: &tempfile::TempDir,
    expected: ValueRef<'_>,
    sql_value: &str,
    sql_type: &str,
) -> TestResult {
    let database = directory.path().join("inventory.sqlite");
    let reopened = BinaryStore::open(&database).await?;
    let projected = reopened
        .projection_resource(&resource("scalar")?)
        .await?
        .ok_or("reopened scalar resource missing")?;
    assert_eq!(projected.key.id.as_str(), "scalar");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(database))
        .await?;
    let (text, kind, bytes): (String, String, Vec<u8>) = sqlx::query_as(
        "SELECT CAST(value AS TEXT),typeof(value),record FROM thing_table WHERE id='scalar'",
    )
    .fetch_one(&pool)
    .await?;
    let record = Record::from_bytes(bytes)?;
    assert_eq!((text.as_str(), kind.as_str()), (sql_value, sql_type));
    assert_eq!(
        scalar_item(record.root())?
            .ok_or("scalar missing")?
            .to_value()?,
        expected.to_value()?
    );
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_long_scalar_when_stored_and_reopened_then_typed_column_and_cbor_are_exact()
-> TestResult {
    let (given, store) = given_store(ScalarType::Integer).await?;
    when_committed(&store, ValueRef::Int(i64::MAX)).await?;
    drop(store);
    then_reopened(
        &given,
        ValueRef::Int(i64::MAX),
        "9223372036854775807",
        "integer",
    )
    .await
}

#[tokio::test]
async fn given_text_scalar_when_stored_and_reopened_then_typed_column_and_cbor_are_exact()
-> TestResult {
    let (given, store) = given_store(ScalarType::Text).await?;
    when_committed(&store, ValueRef::Text("exact-parent")).await?;
    drop(store);
    then_reopened(
        &given,
        ValueRef::Text("exact-parent"),
        "exact-parent",
        "text",
    )
    .await
}

#[tokio::test]
async fn given_boolean_scalar_when_stored_and_reopened_then_typed_column_and_cbor_are_exact()
-> TestResult {
    let (given, store) = given_store(ScalarType::Bool).await?;
    when_committed(&store, ValueRef::Bool(false)).await?;
    drop(store);
    then_reopened(&given, ValueRef::Bool(false), "0", "integer").await
}

fn many_observations() -> TestResult<Vec<Observation>> {
    (0_i64..1001)
        .map(|value| {
            Ok(Observation::new(
                resource(&value.to_string())?,
                scalar_record(ValueRef::Int(value))?,
                &Exclusions::none(),
                None,
            )?)
        })
        .collect()
}

type ScalarRow = (String, i64, Vec<u8>);

async fn persisted_rows(directory: &tempfile::TempDir) -> TestResult<Vec<ScalarRow>> {
    let database = directory.path().join("inventory.sqlite");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(database))
        .await?;
    let rows =
        sqlx::query_as::<_, ScalarRow>("SELECT id,value,record FROM thing_table ORDER BY value")
            .fetch_all(&pool)
            .await?;
    pool.close().await;
    Ok(rows)
}

fn assert_many_rows(rows: Vec<ScalarRow>) -> TestResult {
    assert_eq!(rows.len(), 1001);
    rows.into_iter()
        .enumerate()
        .try_for_each(|(index, (id, value, bytes))| {
            let expected = i64::try_from(index)?;
            assert_eq!(id, expected.to_string());
            assert_eq!(value, expected);
            let record = Record::from_bytes(bytes)?;
            assert_eq!(
                scalar_item(record.root())?
                    .ok_or("scalar missing")?
                    .to_value()?,
                Value::Int(expected)
            );
            Ok(())
        })
}

async fn then_many_reopened(directory: &tempfile::TempDir) -> TestResult {
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    let last = reopened
        .projection_resource(&resource("1000")?)
        .await?
        .ok_or("last scalar resource missing after reopen")?;
    assert_eq!(last.key.id.as_str(), "1000");
    assert_many_rows(persisted_rows(directory).await?)
}

async fn when_many_committed(store: &BinaryStore) -> TestResult {
    store.seed_work(work("scalar-page", "us-east-1")?).await?;
    let lease = store
        .claim(&SweepId::new("scalar-page".to_owned())?, 100, 20)
        .await?
        .ok_or("missing scalar page lease")?;
    store
        .commit_page(page(lease, many_observations()?, true, None, 101))
        .await?;
    Ok(())
}

#[tokio::test]
async fn given_1001_scalar_items_when_committed_and_reopened_then_every_typed_row_is_retained()
-> TestResult {
    let (given, store) = given_store(ScalarType::Integer).await?;
    when_many_committed(&store).await?;
    drop(store);
    then_many_reopened(&given).await
}
