use super::{
    TestResult,
    compare::{self, Expected},
    native::NativeGraph,
    runtime::NativeRuntime,
};
use inventory_core::record::{Key, Record, Value};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub(super) async fn retain(
    database: &Path,
    native: &NativeRuntime,
    expected: &Expected,
    graph: &NativeGraph,
) -> TestResult {
    let Some(root) = std::env::var_os("INVENTORY_NATIVE_GRAPH_ARTIFACT_DIR") else {
        return Ok(());
    };
    let root = PathBuf::from(root);
    tokio::fs::create_dir_all(&root).await?;
    let owned = tempfile::Builder::new()
        .prefix("native-graph-")
        .tempdir_in(&root)?;
    let output = owned.path().join("inventory.sqlite");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(SqliteConnectOptions::new().filename(database))
        .await?;
    sqlx::query("VACUUM INTO ?1")
        .bind(output.to_str().ok_or("non-UTF8 artifact path")?)
        .execute(&pool)
        .await?;
    pool.close().await;
    let resources = expected
        .resources
        .iter()
        .map(|key| strings(compare::key_fields(key)))
        .collect();
    let edges = expected.edges.iter().cloned().map(strings).collect();
    let nodes = graph
        .rows(inventory_graph::cypher::diagnostics::NODE_HASHES)
        .await?;
    let native_edges = graph
        .rows(inventory_graph::cypher::diagnostics::RESOURCE_EDGES)
        .await?;
    let tags = graph
        .rows(inventory_graph::cypher::diagnostics::SCOPED_AFFILIATIONS)
        .await?;
    let record = Record::encode(&Value::Map(BTreeMap::from([
        (Key::Name("format".into()), Value::Int(1)),
        (
            Key::Name("expected_resources".into()),
            Value::List(resources),
        ),
        (Key::Name("expected_edges".into()), Value::List(edges)),
        (
            Key::Name("expected_safe_fields".into()),
            Value::List(
                expected
                    .safe_records
                    .iter()
                    .map(|(key, fields)| {
                        Value::Map(BTreeMap::from([
                            (
                                Key::Name("resource_key".into()),
                                strings(compare::key_fields(key)),
                            ),
                            (
                                Key::Name("fields".into()),
                                Value::Map(
                                    fields
                                        .iter()
                                        .map(|(name, value)| {
                                            (Key::Name(name.clone()), value.clone())
                                        })
                                        .collect(),
                                ),
                            ),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            Key::Name("native_resources".into()),
            Value::List(nodes.into_iter().map(strings).collect()),
        ),
        (
            Key::Name("native_edges".into()),
            Value::List(native_edges.into_iter().map(strings).collect()),
        ),
        (
            Key::Name("native_tag_affiliations".into()),
            Value::List(tags.into_iter().map(strings).collect()),
        ),
    ])))?;
    tokio::fs::write(
        owned.path().join("native-evidence.record"),
        record.as_bytes(),
    )
    .await?;
    for name in [
        "collector-initial.stdout",
        "collector-initial.stderr",
        "collector-updated.stdout",
        "collector-updated.stderr",
        "collector-miss-one.stdout",
        "collector-miss-one.stderr",
        "collector-denied-between-misses.stdout",
        "collector-denied-between-misses.stderr",
        "collector-miss-two.stdout",
        "collector-miss-two.stderr",
        "falkor.log",
    ] {
        tokio::fs::copy(native.work().join(name), owned.path().join(name)).await?;
    }
    let retained = owned.keep();
    eprintln!(
        "native graph evidence retained at {}; parent Makepad fixture database: {}",
        retained.display(),
        output.display()
    );
    Ok(())
}

fn strings(values: Vec<String>) -> Value {
    Value::List(values.into_iter().map(Value::Text).collect())
}
