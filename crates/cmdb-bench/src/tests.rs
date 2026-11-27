use super::{BenchDatabase, Command, parse_command};
use crate::generate::generate;
use crate::measure::measure;
use cmdb_view::{Database, PageRequest, ResourceFilter};
use inventory_core::catalog::MAX_CATALOG_BYTES;
use inventory_core::catalog::runtime::compiled_inputs_sha256;
use inventory_core::record::{FieldId, Key, MapRef, RecordError, Value, ValueRef, encode_container};
use inventory_harness::postgres::{TestDatabase, admin_url};
use sha2::{Digest, Sha256};
use sqlx::{Connection, PgConnection};
use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

type TestResult = Result<(), Box<dyn Error>>;

type FixtureResult<T = ()> = Result<T, Box<dyn Error>>;

fn text(value: &str) -> Value {
    Value::Text(value.to_owned())
}

fn map(fields: impl IntoIterator<Item = (u32, Value)>) -> FixtureResult<Value> {
    Ok(Value::Map(
        fields
            .into_iter()
            .map(|(key, value)| Ok((Key::Field(FieldId::new(key)?), value)))
            .collect::<Result<BTreeMap<_, _>, RecordError>>()?,
    ))
}

fn kind_schema(kind: &str, table: &str) -> FixtureResult<Value> {
    map([
        (1, text(kind)),
        (2, text(table)),
        (3, Value::List(vec![])),
        (4, Value::Null),
    ])
}

fn kind_coverage(cfn_type: &str) -> FixtureResult<Value> {
    map([
        (1, Value::Null),
        (2, text(cfn_type)),
        (3, Value::Bool(true)),
        (4, Value::Null),
        (5, Value::Bool(true)),
    ])
}

fn bench_metadata() -> FixtureResult<Value> {
    map([
        (1, Value::Int(4)),
        (2, Value::List(vec![])),
        (
            3,
            Value::List(vec![
                kind_schema("AWS::EC2::Instance", "ec2_instances")?,
                kind_schema("AWS::IAM::Role", "iam_roles")?,
            ]),
        ),
        (4, Value::List(vec![])),
        (
            5,
            Value::List(vec![
                kind_coverage("AWS::EC2::Instance")?,
                kind_coverage("AWS::IAM::Role")?,
            ]),
        ),
        (6, Value::List(vec![])),
    ])
}

fn bench_catalog_bytes() -> FixtureResult<Vec<u8>> {
    let base = include_bytes!("../../../catalog/aws-api.catalog");
    let Value::Map(mut root) =
        ValueRef::Map(MapRef::from_container_bytes(base, MAX_CATALOG_BYTES)?).to_value()?
    else {
        return Err("wire catalog is not a map".into());
    };
    let extension = Key::Field(FieldId::new(60_000)?);
    root.remove(&extension);
    let models = Sha256::digest(encode_container(
        &Value::Map(root.clone()),
        MAX_CATALOG_BYTES,
    )?)
    .to_vec();
    let mut metadata = bench_metadata()?;
    let encoded = encode_container(&metadata, MAX_CATALOG_BYTES)?;
    let reviewed =
        compiled_inputs_sha256(MapRef::from_container_bytes(&encoded, MAX_CATALOG_BYTES)?)?
            .to_vec();
    let fixture = Sha256::digest(include_bytes!("tests.rs")).to_vec();
    let receipt = map([
        (1, Value::Int(1)),
        (2, map([(1, Value::Int(1)), (2, Value::Bytes(models))])?),
        (3, Value::Int(1)),
        (4, Value::Bytes(fixture.clone())),
        (5, Value::Null),
        (6, Value::Bytes(fixture)),
        (7, Value::Bytes(reviewed)),
    ])?;
    let Value::Map(fields) = &mut metadata else {
        return Err("runtime extension is not a map".into());
    };
    fields.insert(Key::Field(FieldId::new(7)?), receipt);
    root.insert(extension, metadata);
    Ok(encode_container(&Value::Map(root), MAX_CATALOG_BYTES)?)
}

fn catalog_file(directory: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let path = directory.join("catalog.bin");
    std::fs::write(&path, bench_catalog_bytes()?)?;
    Ok(path)
}

async fn generated_fixture_round_trips(url: &str, catalog: PathBuf) -> TestResult {
    generate(BenchDatabase::parse(url)?, catalog, 2, 3).await?;
    let database = Database::open(url).await?;
    let counts = database.counts().await?;
    assert_eq!(counts.accounts, 2);
    assert_eq!(counts.regions, 3);
    assert_eq!(counts.services, 2);
    assert_eq!(counts.resources, 6);
    let page = database
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 6,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.resources.len(), 6);
    assert_eq!(page.total_matching, 6);
    assert!(page.next.is_none());
    assert_eq!(
        page.resources
            .first()
            .map(|resource| resource.key.kind.as_str()),
        Some("AWS::EC2::Instance")
    );
    measure(BenchDatabase::parse(url)?).await?;
    Ok(())
}

#[tokio::test]
async fn generated_database_round_trips_through_reader() -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let directory = tempdir()?;
    let catalog = catalog_file(directory.path())?;
    let outcome = generated_fixture_round_trips(database.url(), catalog).await;
    let finished = database.finish().await.map_err(|error| error.to_string());
    outcome?;
    finished?;
    Ok(())
}

async fn refuses_second_generation(url: &str, catalog: PathBuf) -> TestResult {
    generate(BenchDatabase::parse(url)?, catalog.clone(), 1, 3).await?;
    assert!(
        generate(BenchDatabase::parse(url)?, catalog, 1, 3)
            .await
            .is_err()
    );
    let counts = Database::open(url).await?.counts().await?;
    assert_eq!(counts.resources, 3);
    Ok(())
}

#[tokio::test]
async fn generator_refuses_occupied_postgres_database() -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let directory = tempdir()?;
    let catalog = catalog_file(directory.path())?;
    let outcome = refuses_second_generation(database.url(), catalog).await;
    let finished = database.finish().await.map_err(|error| error.to_string());
    outcome?;
    finished?;
    Ok(())
}

#[test]
fn command_parser_rejects_zero_and_extra_arguments() {
    assert!(
        parse_command(
            ["generate", "postgresql://localhost/bench", "catalog.bin", "0", "3"]
                .map(str::to_owned)
                .into_iter()
        )
        .is_err()
    );
    assert!(
        parse_command(
            ["measure", "postgresql://localhost/bench", "extra"]
                .map(str::to_owned)
                .into_iter()
        )
        .is_err()
    );
}

#[test]
fn command_parser_rejects_non_postgres_targets() {
    assert!(
        parse_command(
            ["measure", "benchmark.sqlite"]
                .map(str::to_owned)
                .into_iter()
        )
        .is_err()
    );
}

#[test]
fn parser_preserves_explicit_workload_parameters() -> TestResult {
    let command = parse_command(
        ["generate", "postgresql://localhost/bench", "catalog.bin", "4", "25"]
            .map(str::to_owned)
            .into_iter(),
    )?;
    assert!(matches!(
        command,
        Command::Generate {
            accounts: 4,
            per_account: 25,
            ..
        }
    ));
    Ok(())
}

#[test]
fn database_target_debug_redacts_credentials() -> TestResult {
    let target = BenchDatabase::parse("postgresql://user:secret@localhost/bench")?;
    assert!(!format!("{target:?}").contains("secret"));
    Ok(())
}

async fn inventory_schema_exists(url: &str) -> Result<bool, Box<dyn Error>> {
    let mut connection = PgConnection::connect(url).await?;
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pg_namespace WHERE nspname = 'inventory')",
    )
    .fetch_one(&mut connection)
    .await?;
    connection.close().await?;
    Ok(exists)
}

async fn assert_cli_generation_refused(url: &str, path: &Path) -> TestResult {
    let Command::Generate {
        target,
        catalog,
        accounts,
        per_account,
    } = parse_command(
        [
            "generate".to_owned(),
            url.to_owned(),
            path.display().to_string(),
            "1".to_owned(),
            "3".to_owned(),
        ]
        .into_iter(),
    )?
    else {
        return Err("generate command did not parse".into());
    };
    assert!(generate(target, catalog, accounts, per_account).await.is_err());
    assert!(!inventory_schema_exists(url).await?);
    Ok(())
}

#[tokio::test]
async fn generator_refuses_catalog_without_benchmark_kinds() -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let directory = tempdir()?;
    let path = directory.path().join("catalog.bin");
    let bytes = crate::pipeline_test_catalog::bytes().map_err(|error| error.to_string())?;
    std::fs::write(&path, bytes)?;
    let outcome = assert_cli_generation_refused(database.url(), &path).await;
    let finished = database.finish().await.map_err(|error| error.to_string());
    outcome?;
    finished?;
    Ok(())
}

#[tokio::test]
async fn generator_refuses_existing_catalog_file_and_preserves_bytes() -> TestResult {
    let database = TestDatabase::create(&admin_url()?).await?;
    let directory = tempdir()?;
    let path = directory.path().join("catalog.bin");
    std::fs::write(&path, b"preserve existing")?;
    let outcome = assert_cli_generation_refused(database.url(), &path).await;
    let preserved = std::fs::read(&path)?;
    let finished = database.finish().await.map_err(|error| error.to_string());
    outcome?;
    finished?;
    assert_eq!(preserved, b"preserve existing");
    Ok(())
}

#[test]
fn layout_command_parses_target_nodes() -> TestResult {
    let command = parse_command(["layout", "1000"].map(str::to_owned).into_iter())?;
    assert!(matches!(command, Command::Layout(_)));
    Ok(())
}

#[test]
fn layout_command_rejects_zero_and_extra_arguments() {
    assert!(parse_command(["layout", "0"].map(str::to_owned).into_iter()).is_err());
    assert!(parse_command(["layout", "10", "extra"].map(str::to_owned).into_iter()).is_err());
}
