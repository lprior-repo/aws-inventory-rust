use inventory_core::{
    catalog::runtime::{BindingSource, BindingValue, RuntimeCatalog},
    inventory::{
        AccountId, Kind, Observation, OperationId, Region, ResourceId, ResourceKey, ScopeKey,
        ServiceId,
    },
    record::{Exclusions, Key, Record, Value},
};
use inventory_engine::inventory::{
    BinaryStore, DatabaseUrl, InventoryStore, PageCommit, PgStore, PgWriteConfig, SchemaRegistry,
    SweepId, WorkSpec,
};
use sha2::{Digest, Sha256};
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use std::{collections::BTreeMap, path::Path, sync::LazyLock};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn resource(account: &str, region: &str, kind: &str, id: &str) -> TestResult<ResourceKey> {
    Ok(ResourceKey {
        account: AccountId::parse(account)?,
        region: Region::parse(region)?,
        kind: Kind::parse(kind)?,
        id: ResourceId::parse(id)?,
    })
}

fn registry() -> TestResult<&'static (SchemaRegistry, Record)> {
    static CATALOG: LazyLock<Result<(SchemaRegistry, Record), String>> = LazyLock::new(|| {
        let result = (|| -> TestResult<(SchemaRegistry, Record)> {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../catalog/aws-api.catalog");
            let bytes = std::fs::read(path)?;
            let catalog = RuntimeCatalog::from_bytes(&bytes)?;
            let plan = catalog
                .plans()
                .iter()
                .find(|plan| plan.service_name() == "s3" && plan.operation_name() == "ListBuckets")
                .ok_or("missing S3 fixture operation")?;
            let [binding] = plan.bindings() else {
                return Err("expected one S3 fixture request binding".into());
            };
            let ([field], BindingSource::Fixed(BindingValue::Integer(value))) =
                (binding.path.as_ref(), &binding.source)
            else {
                return Err("expected a scalar integer S3 fixture request binding".into());
            };
            let request = Record::encode(&Value::Map(BTreeMap::from([(
                Key::Field(*field),
                Value::Int(*value),
            )])))?;
            Ok((
                SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?,
                request,
            ))
        })();
        result.map_err(|error| error.to_string())
    });
    CATALOG.as_ref().map_err(|error| error.clone().into())
}

pub async fn sqlite(path: &Path, rows: &[ResourceKey]) -> TestResult {
    let (registry, bucket_request) = registry()?;
    let store = BinaryStore::create(path, registry.clone()).await?;
    write(&store, rows, bucket_request).await?;
    drop(store);
    checkpoint(path).await
}

pub async fn append(path: &Path, rows: &[ResourceKey]) -> TestResult {
    let (_, bucket_request) = registry()?;
    let store = BinaryStore::open(path).await?;
    write(&store, rows, bucket_request).await?;
    drop(store);
    checkpoint(path).await
}

async fn checkpoint(path: &Path) -> TestResult {
    let options = SqliteConnectOptions::new().filename(path);
    let mut connection = SqliteConnection::connect_with(&options).await?;
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    Ok(())
}

pub async fn postgres(url: &str, rows: &[ResourceKey]) -> TestResult {
    let (registry, bucket_request) = registry()?;
    let store = PgStore::create(
        &DatabaseUrl::parse(url)?,
        registry.clone(),
        PgWriteConfig::default(),
    )
    .await?;
    let result = write(&store, rows, bucket_request).await;
    store.close().await;
    result
}

async fn write(
    store: &impl InventoryStore,
    rows: &[ResourceKey],
    bucket_request: &Record,
) -> TestResult {
    let mut groups = BTreeMap::new();
    for row in rows {
        groups
            .entry((&row.account, &row.region, &row.kind))
            .or_insert_with(Vec::new)
            .push(row);
    }
    for rows in groups.into_values() {
        write_scope(store, &rows, bucket_request).await?;
    }
    Ok(())
}

async fn write_scope(
    store: &impl InventoryStore,
    rows: &[&ResourceKey],
    bucket_request: &Record,
) -> TestResult {
    let first = rows.first().ok_or("empty inventory fixture page")?;
    let (service, operation) = match first.kind.as_str() {
        "AWS::SNS::Topic" => ("sns", "ListTopics"),
        "AWS::S3::Bucket" => ("s3", "ListBuckets"),
        "AWS::IAM::Role" => ("iam", "ListRoles"),
        kind => return Err(format!("unsupported inventory fixture kind {kind}").into()),
    };
    let scope = ScopeKey::new(
        first.account.clone(),
        first.region.clone(),
        ServiceId::parse(service)?,
        OperationId::parse(operation)?,
        None,
    )?;
    let sweep = SweepId::new(format!("cloud-control-fixture-{}", uuid::Uuid::new_v4()))?;
    let record = Record::encode(&Value::Map(BTreeMap::new()))?;
    let bindings = if first.kind.as_str() == "AWS::S3::Bucket" {
        bucket_request
    } else {
        &record
    };
    store
        .seed_works(&[WorkSpec::new(
            sweep.clone(),
            scope,
            None,
            bindings.as_bytes().to_vec(),
        )?])
        .await?;
    let pages = rows.chunks(100_000);
    let last = pages.len().saturating_sub(1);
    for (index, page) in pages.enumerate() {
        write_page(store, &sweep, page, &record, index == last).await?;
    }
    Ok(())
}

async fn write_page(
    store: &impl InventoryStore,
    sweep: &SweepId,
    rows: &[&ResourceKey],
    record: &Record,
    done: bool,
) -> TestResult {
    let lease = store
        .claim(sweep, 100, 10_000)
        .await?
        .ok_or("inventory fixture scope unclaimable")?;
    let observations = rows
        .iter()
        .map(|key| {
            Observation::new(
                (*key).clone(),
                record.clone(),
                &Exclusions::none(),
                Some(key.id.as_str().into()),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let next_token = if done {
        None
    } else {
        Some(
            rows.last()
                .ok_or("empty inventory fixture page")?
                .id
                .as_str()
                .as_bytes()
                .to_vec(),
        )
    };
    let continuation_digest = next_token
        .as_ref()
        .map(|token| Sha256::digest(token).into());
    store
        .commit_page(PageCommit {
            lease,
            observations,
            tags: Vec::new(),
            edges: Vec::new(),
            unresolved: Vec::new(),
            next_token,
            continuation_digest,
            children: Vec::new(),
            done,
            observed_at_ms: 100,
        })
        .await?;
    Ok(())
}
