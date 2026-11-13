use inventory_core::{CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source};
use inventory_engine::store::{Store, StoreError};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{AssertSqlSafe, Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use std::collections::BTreeMap;
use std::path::Path;

type TestError = Box<dyn std::error::Error>;
type TestResult = Result<(), TestError>;
const CURRENT_SCHEMA_VERSION: i64 = inventory_engine::store::SCHEMA_VERSION;

fn fixture(id: &str, kind: &str, properties: serde_json::Value, time: i64) -> Resource {
    Resource {
        id: id.into(),
        account_id: "111111111111".into(),
        region: "us-east-1".into(),
        kind: kind.into(),
        name: Some(id.into()),
        properties,
        tags: BTreeMap::new(),
        source: Source::Direct,
        observed_at: time,
    }
}

fn page(resources: Vec<Resource>, edges: Vec<Edge>) -> CollectionPage {
    CollectionPage {
        scope: CollectionScope {
            account_id: "111111111111".into(),
            region: "us-east-1".into(),
            collector: "architecture_fixture".into(),
            parent_id: None,
            source: Source::Direct,
        },
        page_number: 1,
        next_token: None,
        resources,
        edges,
    }
}

async fn connect(path: &Path) -> Result<SqliteConnection, sqlx::Error> {
    SqliteConnection::connect_with(
        &SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true),
    )
    .await
}

async fn create_legacy_schema(path: &Path, version: i64) -> Result<SqliteConnection, TestError> {
    let mut connection = connect(path).await?;
    sqlx::raw_sql(include_str!("../sql/001_inventory.sql"))
        .execute(&mut connection)
        .await?;
    if version == 2 {
        sqlx::raw_sql(include_str!("../sql/002_architecture.sql"))
            .execute(&mut connection)
            .await?;
    } else if version != 1 {
        return Err("fixture only supports historical schemas 1 and 2".into());
    }
    Ok(connection)
}

async fn legacy_database(path: &Path) -> TestResult {
    let mut connection = create_legacy_schema(path, 1).await?;
    sqlx::raw_sql("INSERT INTO scans VALUES('legacy',1,3,'complete');
        INSERT INTO scope_checkpoints VALUES('legacy','scope','111111111111','us-east-1','ec2',NULL,'direct',1,NULL,1,0);
        INSERT INTO collection_pages VALUES('legacy','scope',1,zeroblob(32),NULL,1,0);
        INSERT INTO graph_export_checkpoints VALUES('legacy','falkordb','legacy-snapshot',1,0,1);")
        .execute(&mut connection).await?;
    let resource = fixture(
        "i-legacy",
        "aws_ec2_instance",
        json!({"instance_id":"i-legacy"}),
        2,
    );
    sqlx::query(
        "INSERT INTO resource_observations VALUES('legacy','scope',1,'i-legacy',
        '111111111111','us-east-1','aws_ec2_instance','i-legacy','direct',3,2,?)",
    )
    .bind(serde_json::to_string(&resource)?)
    .execute(&mut connection)
    .await?;
    connection.close().await?;
    Ok(())
}

#[path = "migrations/architecture_edges_and_routes_do_not_mix_snapshots.rs"]
mod architecture_edges_and_routes_do_not_mix_snapshots;
#[path = "migrations/coverage_registry_upgrades_version_seven.rs"]
mod coverage_registry_upgrades_version_seven;
#[path = "migrations/failed_migration_rolls_back_views_and_version_together.rs"]
mod failed_migration_rolls_back_views_and_version_together;
#[path = "migrations/foreign_key_check_failure_rolls_back_source_migration.rs"]
mod foreign_key_check_failure_rolls_back_source_migration;
#[path = "migrations/four_sources_preserve_provenance_and_order_in_latest_and_snapshot_views.rs"]
mod four_sources_preserve_provenance_and_order_in_latest_and_snapshot_views;
#[path = "migrations/freshness_view_rebuild_upgrades_version_six.rs"]
mod freshness_view_rebuild_upgrades_version_six;
#[path = "migrations/legacy_population.rs"]
mod legacy_population;
#[path = "migrations/load_balancer_aliases_preserve_type_and_target_identity_fields.rs"]
mod load_balancer_aliases_preserve_type_and_target_identity_fields;
#[path = "migrations/populated_v1_and_v2_upgrade_preserves_raw_rows_json_fingerprints_and_replay.rs"]
mod populated_v1_and_v2_upgrade_preserves_raw_rows_json_fingerprints_and_replay;
#[path = "migrations/rebuild_preserves_custom_schema_data_and_restores_writer_foreign_keys.rs"]
mod rebuild_preserves_custom_schema_data_and_restores_writer_foreign_keys;
#[path = "migrations/rejects_future_schema_without_mutating_it.rs"]
mod rejects_future_schema_without_mutating_it;
#[path = "migrations/schema_object_bound_rejects_before_losing_existing_objects.rs"]
mod schema_object_bound_rejects_before_losing_existing_objects;
#[path = "migrations/source_migration_conflicts_rollback_rebuilt_tables_objects_and_version.rs"]
mod source_migration_conflicts_rollback_rebuilt_tables_objects_and_version;
#[path = "migrations/unsupported_managed_table_extensions_are_rejected_without_losing_values.rs"]
mod unsupported_managed_table_extensions_are_rejected_without_losing_values;
#[path = "migrations/upgrades_version_one_without_changing_inventory_or_checkpoints.rs"]
mod upgrades_version_one_without_changing_inventory_or_checkpoints;
use legacy_population::*;
#[path = "migrations/snapshot.rs"]
mod snapshot;
use snapshot::*;
