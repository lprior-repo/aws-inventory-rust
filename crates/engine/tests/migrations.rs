use inventory_core::{CollectionPage, CollectionScope, Edge, Resource, ScanStatus, Source};
use inventory_engine::store::{Store, StoreError};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{AssertSqlSafe, Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use std::collections::BTreeMap;
use std::path::Path;

type TestError = Box<dyn std::error::Error>;
type TestResult = Result<(), TestError>;
const CURRENT_SCHEMA_VERSION: i64 = 6;

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

fn source_page(source: Source, ids: &[&str], time: i64) -> CollectionPage {
    let mut resources = Vec::new();
    let mut edges = Vec::new();
    for id in ids {
        let mut resource = fixture(
            id,
            "aws_ec2_instance",
            json!({"instance_id":id,"marker":source.as_str(),"literal":"line\n\tcloud ☁",
                "nested":{"nullable":null,"array":[1,true,"string"]},
                "cloudformation_type":"AWS::EC2::Instance","resource_identifier":id,
                "provider_properties":{"InstanceId":id}}),
            time,
        );
        resource.source = source;
        resource
            .tags
            .insert("custom-tag".into(), "quotes\" and spaces".into());
        resources.push(resource);
        edges.push(Edge {
            from_id: (*id).into(),
            to_id: "unobserved-vpc".into(),
            relation: "IN_VPC".into(),
            source,
            observed_at: time,
            properties: json!({"marker":source.as_str(),"evidence":[{"raw":"space\tline\n"}]}),
        });
    }
    let mut input = page(resources, edges);
    input.scope.collector = format!("fixture_{}", source.as_str());
    input.scope.source = source;
    input
}

/// Seed historical rows directly: opening the current Store would migrate them
/// before the test could establish the old priority and replay contract.
async fn insert_legacy_page(
    connection: &mut SqliteConnection,
    input: &CollectionPage,
) -> TestResult {
    let old_priority = match input.scope.source {
        Source::Direct => 3_i64,
        Source::AwsConfig => 2,
        Source::ResourceExplorer => 1,
        Source::CloudControl => return Err("Cloud Control is not a historical source".into()),
    };
    let key = input.scope.key()?;
    let fingerprint = Sha256::digest(serde_json::to_vec(input)?).to_vec();
    sqlx::query(
        "INSERT INTO scope_checkpoints VALUES('populated',?1,?2,?3,?4,?5,?6,?7,?8,?9,0)
        ON CONFLICT(scan_id,scope_key) DO UPDATE SET last_page=excluded.last_page,
            next_token=excluded.next_token,complete=excluded.complete",
    )
    .bind(&key)
    .bind(&input.scope.account_id)
    .bind(&input.scope.region)
    .bind(&input.scope.collector)
    .bind(&input.scope.parent_id)
    .bind(input.scope.source.as_str())
    .bind(i64::from(input.page_number))
    .bind(&input.next_token)
    .bind(input.next_token.is_none())
    .execute(&mut *connection)
    .await?;
    sqlx::query("INSERT INTO collection_pages VALUES('populated',?1,?2,?3,?4,?5,?6)")
        .bind(&key)
        .bind(i64::from(input.page_number))
        .bind(fingerprint)
        .bind(&input.next_token)
        .bind(i64::try_from(input.resources.len())?)
        .bind(i64::try_from(input.edges.len())?)
        .execute(&mut *connection)
        .await?;
    for resource in &input.resources {
        // Noncanonical whitespace makes an accidental deserialize/reserialize
        // detectable, while the page digest still uses the v1/v2 wire format.
        sqlx::query("INSERT INTO resource_observations VALUES('populated',?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)")
            .bind(&key).bind(i64::from(input.page_number)).bind(&resource.id).bind(&resource.account_id).bind(&resource.region)
            .bind(&resource.kind).bind(&resource.name).bind(resource.source.as_str())
            .bind(old_priority).bind(resource.observed_at).bind(serde_json::to_string_pretty(resource)?)
            .execute(&mut *connection).await?;
    }
    for edge in &input.edges {
        sqlx::query("INSERT INTO edge_observations VALUES('populated',?1,?2,?3,?4,?5,?6,?7,?8,?9)")
            .bind(&key)
            .bind(i64::from(input.page_number))
            .bind(&edge.from_id)
            .bind(&edge.to_id)
            .bind(&edge.relation)
            .bind(edge.source.as_str())
            .bind(old_priority)
            .bind(edge.observed_at)
            .bind(serde_json::to_string_pretty(edge)?)
            .execute(&mut *connection)
            .await?;
    }
    Ok(())
}

async fn populated_legacy(path: &Path, version: i64) -> Result<Vec<CollectionPage>, TestError> {
    let mut connection = create_legacy_schema(path, version).await?;
    sqlx::query("INSERT INTO scans VALUES('populated',1,40,'partial')")
        .execute(&mut connection)
        .await?;
    let mut direct = source_page(Source::Direct, &["preferred-direct"], 10);
    direct.next_token = Some("historical-next-page".into());
    let mut terminal = source_page(Source::Direct, &[], 11);
    terminal.page_number = 2;
    let mut pending = source_page(Source::Direct, &[], 35);
    pending.scope.collector = "pending_native_scope".into();
    pending.next_token = Some("saved-resume-token".into());
    let pages = vec![
        direct,
        terminal,
        source_page(
            Source::AwsConfig,
            &["preferred-direct", "preferred-config", "preferred-cloud"],
            20,
        ),
        source_page(
            Source::ResourceExplorer,
            &[
                "preferred-direct",
                "preferred-config",
                "preferred-cloud",
                "preferred-explorer",
            ],
            30,
        ),
        pending,
    ];
    for input in &pages {
        insert_legacy_page(&mut connection, input).await?;
    }
    sqlx::raw_sql("UPDATE scope_checkpoints SET failed=1 WHERE source='aws_config';
        INSERT INTO collection_failures SELECT scan_id,scope_key,'fixture_partial','preserve failure text'
            FROM scope_checkpoints WHERE source='aws_config';
        INSERT INTO graph_export_checkpoints VALUES('populated','falkordb','preserved-snapshot',4,4,1);")
        .execute(&mut connection).await?;
    connection.close().await?;
    Ok(pages)
}

type StoredResource = (
    String,
    i64,
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    i64,
    String,
);
type StoredEdge = (String, i64, String, String, String, String, i64, String);
type StoredPage = (String, i64, Vec<u8>, Option<String>, i64, i64);
type StoredScope = (
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    i64,
    Option<String>,
    i64,
    i64,
);
type SchemaObject = (String, String, String, Option<String>);

#[derive(Debug, PartialEq, Eq)]
struct InventorySnapshot {
    scan: (i64, Option<i64>, String),
    resources: Vec<StoredResource>,
    edges: Vec<StoredEdge>,
    pages: Vec<StoredPage>,
    scopes: Vec<StoredScope>,
    failures: Vec<(String, String, String)>,
    graph: Vec<(String, String, i64, i64, i64)>,
}

async fn inventory_snapshot(
    connection: &mut SqliteConnection,
) -> Result<InventorySnapshot, TestError> {
    Ok(InventorySnapshot {
        scan: sqlx::query_as("SELECT started_at,finished_at,status FROM scans WHERE scan_id='populated'")
            .fetch_one(&mut *connection).await?,
        resources: sqlx::query_as("SELECT scope_key,page_number,resource_id,account_id,region,kind,name,source,observed_at,resource_json
            FROM resource_observations WHERE scan_id='populated' ORDER BY scope_key,resource_id")
            .fetch_all(&mut *connection).await?,
        edges: sqlx::query_as("SELECT scope_key,page_number,from_id,to_id,relation,source,observed_at,edge_json
            FROM edge_observations WHERE scan_id='populated' ORDER BY scope_key,from_id,to_id,relation")
            .fetch_all(&mut *connection).await?,
        pages: sqlx::query_as("SELECT scope_key,page_number,fingerprint,next_token,resource_count,edge_count
            FROM collection_pages WHERE scan_id='populated' ORDER BY scope_key,page_number")
            .fetch_all(&mut *connection).await?,
        scopes: sqlx::query_as("SELECT scope_key,account_id,region,collector,parent_id,source,last_page,next_token,complete,failed
            FROM scope_checkpoints WHERE scan_id='populated' ORDER BY scope_key")
            .fetch_all(&mut *connection).await?,
        failures: sqlx::query_as("SELECT scope_key,category,message FROM collection_failures WHERE scan_id='populated'
            ORDER BY scope_key,category,message").fetch_all(&mut *connection).await?,
        graph: sqlx::query_as("SELECT target,snapshot_id,resources_exported,edges_exported,complete
            FROM graph_export_checkpoints WHERE scan_id='populated' ORDER BY target,snapshot_id")
            .fetch_all(&mut *connection).await?,
    })
}

async fn schema_snapshot(
    connection: &mut SqliteConnection,
) -> Result<Vec<SchemaObject>, sqlx::Error> {
    sqlx::query_as("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name")
        .fetch_all(connection)
        .await
}

#[tokio::test]
async fn populated_v1_and_v2_upgrade_preserves_raw_rows_json_fingerprints_and_replay() -> TestResult
{
    for version in [1_i64, 2] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("populated.sqlite");
        let pages = populated_legacy(&path, version).await?;
        let mut connection = connect(&path).await?;
        let before = inventory_snapshot(&mut connection).await?;
        connection.close().await?;

        let store = Store::open(&path).await?;
        assert_eq!(store.scan_status("populated").await?, ScanStatus::Partial);
        let summary = store.summary("populated").await?;
        assert_eq!(
            (
                summary.resources,
                summary.edges,
                summary.completed_scopes,
                summary.failed_scopes
            ),
            (4, 4, 2, 1)
        );
        assert_eq!(
            store
                .query("SELECT user_version AS version FROM pragma_user_version", 1)
                .await?,
            json!([{"version":CURRENT_SCHEMA_VERSION}])
        );
        for table in ["resource_observations", "edge_observations"] {
            assert_eq!(store.query(&format!("SELECT DISTINCT source,source_priority FROM {table} ORDER BY source_priority DESC"), 4).await?,
                json!([{"source":"direct","source_priority":4},{"source":"aws_config","source_priority":2},{"source":"resource_explorer","source_priority":1}]));
        }
        for input in &pages {
            store.apply_page("populated", input.clone()).await?;
        }
        let pending = pages.last().ok_or("missing pending legacy page")?;
        let checkpoint = store
            .checkpoint("populated", &pending.scope)
            .await?
            .ok_or("missing saved checkpoint")?;
        assert_eq!(checkpoint.next_token.as_deref(), Some("saved-resume-token"));
        assert!(!checkpoint.complete && !checkpoint.failed);
        assert_eq!(store.summary("populated").await?, summary);
        let mut changed = pages.first().ok_or("missing legacy replay page")?.clone();
        let first = changed
            .resources
            .first_mut()
            .ok_or("missing legacy resource")?;
        first.name = Some("changed replay must not overwrite".into());
        assert!(matches!(
            store.apply_page("populated", changed).await,
            Err(StoreError::Conflict(_))
        ));
        store.close().await;

        let mut connection = connect(&path).await?;
        assert_eq!(inventory_snapshot(&mut connection).await?, before);
        let violations: i64 = sqlx::query_scalar("SELECT count(*) FROM pragma_foreign_key_check")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(violations, 0);
        connection.close().await?;
        let reopened = Store::open(&path).await?;
        for input in &pages {
            reopened.apply_page("populated", input.clone()).await?;
        }
        assert_eq!(reopened.summary("populated").await?, summary);
        reopened.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn four_sources_preserve_provenance_and_order_in_latest_and_snapshot_views() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("sources.sqlite");
    populated_legacy(&path, 2).await?;
    let store = Store::open(&path).await?;
    store.begin_scan("new-sources", 50).await?;
    for input in [
        source_page(
            Source::CloudControl,
            &["preferred-direct", "preferred-cloud", "new-all"],
            100,
        ),
        source_page(Source::AwsConfig, &["preferred-cloud", "new-all"], 200),
        source_page(
            Source::ResourceExplorer,
            &["preferred-cloud", "new-all"],
            300,
        ),
        source_page(Source::Direct, &["new-all"], 60),
    ] {
        store.apply_page("new-sources", input).await?;
    }
    store
        .finish_scan("new-sources", ScanStatus::Complete, 400)
        .await?;
    let expected = json!([
        {"id":"new-all","source":"direct","source_priority":4,"observed_at":60},
        {"id":"preferred-cloud","source":"cloud_control","source_priority":3,"observed_at":100},
        {"id":"preferred-config","source":"aws_config","source_priority":2,"observed_at":20},
        {"id":"preferred-direct","source":"direct","source_priority":4,"observed_at":10},
        {"id":"preferred-explorer","source":"resource_explorer","source_priority":1,"observed_at":30}
    ]);
    assert_eq!(
        store
            .query(
                "SELECT id,source,source_priority,observed_at FROM latest_resources ORDER BY id",
                10
            )
            .await?,
        expected
    );
    assert_eq!(store.query("SELECT from_id AS id,source,source_priority,observed_at FROM latest_edges ORDER BY from_id", 10).await?, expected);
    let snapshot = json!([
        {"id":"new-all","source":"direct","source_priority":4},
        {"id":"preferred-cloud","source":"cloud_control","source_priority":3},
        {"id":"preferred-direct","source":"cloud_control","source_priority":3}
    ]);
    assert_eq!(store.query("SELECT id,source,source_priority FROM scan_resources WHERE scan_id='new-sources' ORDER BY id", 10).await?, snapshot);
    assert_eq!(store.query("SELECT from_id AS id,source,source_priority FROM scan_edges WHERE scan_id='new-sources' ORDER BY from_id", 10).await?, snapshot);
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM latest_resource_sources", 1)
            .await?,
        json!([{"total":14}])
    );
    assert_eq!(store.query("SELECT id,source,source_priority,cloudformation_type FROM aws_cloudcontrol_resource ORDER BY id", 5).await?, json!([
        {"id":"new-all","source":"cloud_control","source_priority":3,"cloudformation_type":"AWS::EC2::Instance"},
        {"id":"preferred-cloud","source":"cloud_control","source_priority":3,"cloudformation_type":"AWS::EC2::Instance"},
        {"id":"preferred-direct","source":"cloud_control","source_priority":3,"cloudformation_type":"AWS::EC2::Instance"}
    ]));
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn rebuild_preserves_custom_schema_data_and_restores_writer_foreign_keys() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("extensions.sqlite");
    legacy_database(&path).await?;
    let mut connection = connect(&path).await?;
    sqlx::raw_sql(include_str!("../sql/002_architecture.sql"))
        .execute(&mut connection)
        .await?;
    sqlx::raw_sql(r#"
        CREATE TABLE user_audit(kind TEXT NOT NULL, subject TEXT NOT NULL) STRICT;
        CREATE TABLE user_notes(id TEXT PRIMARY KEY, payload BLOB NOT NULL) STRICT;
        INSERT INTO user_notes VALUES('preserve-custom-data',x'0001ff');
        CREATE TABLE user_resource_reference(
            scan_id TEXT, scope_key TEXT, page_number INTEGER, resource_id TEXT,
            FOREIGN KEY(scan_id,scope_key,page_number,resource_id)
                REFERENCES resource_observations(scan_id,scope_key,page_number,resource_id) ON DELETE CASCADE,
            FOREIGN KEY(scan_id,scope_key) REFERENCES scope_checkpoints(scan_id,scope_key) ON DELETE CASCADE
        ) STRICT;
        INSERT INTO user_resource_reference VALUES('legacy','scope',1,'i-legacy');
        CREATE TABLE user_fk_probe(parent_scan TEXT NOT NULL REFERENCES scans(scan_id)) STRICT;
        CREATE INDEX user_scope_index ON scope_checkpoints(collector) WHERE complete=1;
        CREATE INDEX "user_""resource""_index" ON resource_observations(lower(name),source_priority DESC) WHERE source='direct';
        CREATE INDEX user_edge_index ON edge_observations(relation,from_id);
        CREATE VIEW user_raw_resources AS SELECT resource_id,name,source FROM resource_observations;
        CREATE VIEW "user_""nested""_view" AS SELECT resource_id FROM user_raw_resources;
        CREATE VIEW user_service_columns AS SELECT service_arn,task_definition FROM aws_ecs_service;
        CREATE TRIGGER "user_""resource""_audit" AFTER INSERT ON resource_observations
            BEGIN INSERT INTO user_audit VALUES('resource',NEW.resource_id); END;
        CREATE TRIGGER user_edge_audit AFTER INSERT ON edge_observations
            BEGIN INSERT INTO user_audit VALUES('edge',NEW.from_id); END;
        CREATE TRIGGER user_scope_audit AFTER UPDATE ON scope_checkpoints
            WHEN OLD.last_page != NEW.last_page
            BEGIN INSERT INTO user_audit VALUES('checkpoint',NEW.collector); END;
        CREATE TRIGGER user_notes_reference AFTER INSERT ON user_notes
            BEGIN INSERT INTO user_audit SELECT 'note',resource_id FROM resource_observations WHERE resource_id=NEW.id; END;
        CREATE TRIGGER user_foreign_key_probe AFTER INSERT ON scans WHEN NEW.scan_id='foreign-key-probe'
            BEGIN INSERT INTO user_fk_probe VALUES('this-parent-does-not-exist'); END;
    "#).execute(&mut connection).await?;
    let before: Vec<SchemaObject> = schema_snapshot(&mut connection)
        .await?
        .into_iter()
        .filter(|(_, name, _, _)| name.starts_with("user_"))
        .collect();
    connection.close().await?;

    let store = Store::open(&path).await?;
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM user_audit", 1)
            .await?,
        json!([{"total":0}])
    );
    assert_eq!(
        store
            .query("SELECT resource_id FROM user_resource_reference", 1)
            .await?,
        json!([{"resource_id":"i-legacy"}])
    );
    assert_eq!(
        store
            .query("SELECT hex(payload) AS bytes FROM user_notes", 1)
            .await?,
        json!([{"bytes":"0001FF"}])
    );
    assert_eq!(
        store
            .query("SELECT resource_id FROM \"user_\"\"nested\"\"_view\"", 1)
            .await?,
        json!([{"resource_id":"i-legacy"}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM user_service_columns", 1)
            .await?,
        json!([{"total":0}])
    );
    assert!(matches!(store.begin_scan("foreign-key-probe", 10).await,
        Err(StoreError::Sqlx(error)) if error.as_database_error().is_some_and(|error| error.is_foreign_key_violation())));
    assert_eq!(
        store
            .query(
                "SELECT count(*) AS total FROM scans WHERE scan_id='foreign-key-probe'",
                1
            )
            .await?,
        json!([{"total":0}])
    );
    store.begin_scan("after-migration", 10).await?;
    store
        .apply_page(
            "after-migration",
            source_page(Source::Direct, &["i-new"], 11),
        )
        .await?;
    store
        .finish_scan("after-migration", ScanStatus::Complete, 12)
        .await?;
    assert_eq!(
        store
            .query("SELECT kind,subject FROM user_audit ORDER BY kind", 5)
            .await?,
        json!([
            {"kind":"checkpoint","subject":"fixture_direct"},{"kind":"edge","subject":"i-new"},{"kind":"resource","subject":"i-new"}
        ])
    );
    store.close().await;

    let mut connection = connect(&path).await?;
    let after: Vec<SchemaObject> = schema_snapshot(&mut connection)
        .await?
        .into_iter()
        .filter(|(_, name, _, _)| name.starts_with("user_"))
        .collect();
    assert_eq!(after, before);
    sqlx::query("INSERT INTO user_notes VALUES('i-legacy',x'02')")
        .execute(&mut connection)
        .await?;
    let subject: String = sqlx::query_scalar("SELECT subject FROM user_audit WHERE kind='note'")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(subject, "i-legacy");
    let violations: i64 = sqlx::query_scalar("SELECT count(*) FROM pragma_foreign_key_check")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(violations, 0);
    let references: Vec<String> = sqlx::query_scalar("SELECT \"table\" FROM pragma_foreign_key_list('user_resource_reference') ORDER BY \"table\"")
        .fetch_all(&mut connection).await?;
    assert!(
        references
            .iter()
            .all(|name| name == "scope_checkpoints" || name == "resource_observations")
    );
    assert_eq!(references.len(), 6);
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn source_migration_conflicts_rollback_rebuilt_tables_objects_and_version() -> TestResult {
    for conflict in [
        "CREATE TABLE new_scope_checkpoints(owner_data TEXT); INSERT INTO new_scope_checkpoints VALUES('keep');",
        "CREATE TABLE aws_cloudcontrol_resource(owner_data TEXT); INSERT INTO aws_cloudcontrol_resource VALUES('keep');",
        "CREATE VIEW aws_ecs_task_definition AS SELECT owner_data FROM user_owned;",
    ] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("source-conflict.sqlite");
        populated_legacy(&path, 2).await?;
        let mut connection = connect(&path).await?;
        sqlx::raw_sql("CREATE TABLE user_owned(owner_data TEXT); INSERT INTO user_owned VALUES('preserve-me');
            CREATE INDEX user_index ON resource_observations(name);
            CREATE VIEW user_view AS SELECT resource_id FROM resource_observations;
            CREATE TRIGGER user_trigger AFTER INSERT ON user_owned BEGIN SELECT count(*) FROM edge_observations; END;")
            .execute(&mut connection).await?;
        sqlx::raw_sql(conflict).execute(&mut connection).await?;
        let data_before = inventory_snapshot(&mut connection).await?;
        let schema_before = schema_snapshot(&mut connection).await?;
        connection.close().await?;
        assert!(matches!(Store::open(&path).await, Err(StoreError::Sqlx(_))));
        let mut connection = connect(&path).await?;
        assert_eq!(inventory_snapshot(&mut connection).await?, data_before);
        assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
        let version: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(version, 2);
        let marker: String = sqlx::query_scalar("SELECT owner_data FROM user_owned")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(marker, "preserve-me");
        let priority: i64 = sqlx::query_scalar(
            "SELECT DISTINCT source_priority FROM resource_observations WHERE source='direct'",
        )
        .fetch_one(&mut connection)
        .await?;
        assert_eq!(priority, 3);
        connection.close().await?;
    }
    Ok(())
}

#[tokio::test]
async fn foreign_key_check_failure_rolls_back_source_migration() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("broken-reference.sqlite");
    populated_legacy(&path, 2).await?;
    let mut connection = connect(&path).await?;
    sqlx::raw_sql("CREATE TABLE user_orphan(parent TEXT REFERENCES scans(scan_id));
        PRAGMA foreign_keys=OFF; INSERT INTO user_orphan VALUES('missing-parent'); PRAGMA foreign_keys=ON;")
        .execute(&mut connection).await?;
    let data_before = inventory_snapshot(&mut connection).await?;
    let schema_before = schema_snapshot(&mut connection).await?;
    connection.close().await?;
    assert!(matches!(
        Store::open(&path).await,
        Err(StoreError::Conflict("migration foreign key check failed"))
    ));
    let mut connection = connect(&path).await?;
    assert_eq!(inventory_snapshot(&mut connection).await?, data_before);
    assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(version, 2);
    let orphans: i64 = sqlx::query_scalar("SELECT count(*) FROM pragma_foreign_key_check")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(orphans, 1);
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn schema_object_bound_rejects_before_losing_existing_objects() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("schema-bound.sqlite");
    populated_legacy(&path, 2).await?;
    let mut connection = connect(&path).await?;
    let statements: String = (0..1_024)
        .map(|index| format!("CREATE VIEW user_bound_{index} AS SELECT {index} AS n;\n"))
        .collect();
    // Only fixed SQL and bounded integer fixture indices are interpolated.
    sqlx::raw_sql(AssertSqlSafe(statements))
        .execute(&mut connection)
        .await?;
    let schema_before = schema_snapshot(&mut connection).await?;
    connection.close().await?;
    assert!(matches!(
        Store::open(&path).await,
        Err(StoreError::Limit("migration schema objects"))
    ));
    let mut connection = connect(&path).await?;
    assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(version, 2);
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM resource_observations")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!(rows, 8);
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn unsupported_managed_table_extensions_are_rejected_without_losing_values() -> TestResult {
    for table in ["resource_observations", "edge_observations"] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("extended-managed-table.sqlite");
        populated_legacy(&path, 2).await?;
        let mut connection = connect(&path).await?;
        // Table names come only from the two fixed identifiers above.
        sqlx::raw_sql(AssertSqlSafe(format!(
            "ALTER TABLE {table} ADD COLUMN user_note TEXT;
            UPDATE {table} SET user_note='preserve application-owned value';"
        )))
        .execute(&mut connection)
        .await?;
        let data_before = inventory_snapshot(&mut connection).await?;
        let schema_before = schema_snapshot(&mut connection).await?;
        let notes_before: Vec<String> = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT user_note FROM {table} ORDER BY user_note"
        )))
        .fetch_all(&mut connection)
        .await?;
        connection.close().await?;

        // The managed table rebuild has no safe mapping for unknown columns.
        // Refuse migration instead of successfully discarding their contents.
        assert!(Store::open(&path).await.is_err());
        let mut connection = connect(&path).await?;
        assert_eq!(inventory_snapshot(&mut connection).await?, data_before);
        assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
        let notes_after: Vec<String> = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT user_note FROM {table} ORDER BY user_note"
        )))
        .fetch_all(&mut connection)
        .await?;
        assert_eq!(notes_after, notes_before);
        let version: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&mut connection)
            .await?;
        assert_eq!(version, 2);
        connection.close().await?;
    }
    Ok(())
}

#[tokio::test]
async fn upgrades_version_one_without_changing_inventory_or_checkpoints() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("legacy.sqlite");
    legacy_database(&path).await?;
    let store = Store::open(&path).await?;
    assert_eq!(store.summary("legacy").await?.resources, 1);
    assert_eq!(store.scan_status("legacy").await?, ScanStatus::Complete);
    assert_eq!(
        store
            .query("SELECT user_version AS version FROM pragma_user_version", 1)
            .await?,
        json!([{"version":CURRENT_SCHEMA_VERSION}])
    );
    assert_eq!(
        store
            .query("SELECT instance_id FROM aws_ec2_instance", 2)
            .await?,
        json!([{"instance_id":"i-legacy"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT target,snapshot_id,complete FROM graph_export_checkpoints",
                2
            )
            .await?,
        json!([{"target":"falkordb","snapshot_id":"legacy-snapshot","complete":1}])
    );
    assert_eq!(
        store
            .query("SELECT count(*) AS total FROM aws_elbv2_listener", 1)
            .await?,
        json!([{"total":0}])
    );
    store.close().await;
    let reopened = Store::open(&path).await?;
    assert_eq!(reopened.summary("legacy").await?.resources, 1);
    assert_eq!(
        reopened
            .query("SELECT count(*) AS total FROM collection_pages", 1)
            .await?,
        json!([{"total":1}])
    );
    reopened.close().await;
    Ok(())
}

#[tokio::test]
async fn failed_migration_rolls_back_views_and_version_together() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("conflict.sqlite");
    legacy_database(&path).await?;
    let mut connection = connect(&path).await?;
    sqlx::query("CREATE TABLE aws_elbv2_target_group (owner_data TEXT)")
        .execute(&mut connection)
        .await?;
    sqlx::query("INSERT INTO aws_elbv2_target_group VALUES('preserve-me')")
        .execute(&mut connection)
        .await?;
    connection.close().await?;
    assert!(matches!(Store::open(&path).await, Err(StoreError::Sqlx(_))));
    let mut connection = connect(&path).await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    let views: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sqlite_schema WHERE name='aws_vpc_route_table'")
            .fetch_one(&mut connection)
            .await?;
    let marker: String = sqlx::query_scalar("SELECT owner_data FROM aws_elbv2_target_group")
        .fetch_one(&mut connection)
        .await?;
    let observations: i64 = sqlx::query_scalar("SELECT count(*) FROM resource_observations")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!((version, views, observations), (1, 0, 1));
    assert_eq!(marker, "preserve-me");
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn rejects_future_schema_without_mutating_it() -> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("future.sqlite");
    let mut connection = connect(&path).await?;
    sqlx::raw_sql("CREATE TABLE sentinel(value TEXT); INSERT INTO sentinel VALUES('future'); PRAGMA user_version=99;")
        .execute(&mut connection).await?;
    let schema_before = schema_snapshot(&mut connection).await?;
    connection.close().await?;
    assert!(matches!(
        Store::open(&path).await,
        Err(StoreError::SchemaVersion(99))
    ));
    let mut connection = connect(&path).await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut connection)
        .await?;
    let marker: String = sqlx::query_scalar("SELECT value FROM sentinel")
        .fetch_one(&mut connection)
        .await?;
    assert_eq!((version, marker.as_str()), (99, "future"));
    assert_eq!(schema_snapshot(&mut connection).await?, schema_before);
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn architecture_edges_and_routes_do_not_mix_snapshots() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("older", 1).await?;
    let older_route = fixture(
        "rtb",
        "aws_vpc_route_table",
        json!({"route_table_id":"rtb","vpc_id":"vpc",
        "routes":[{"destination_cidr_block":"0.0.0.0/0","gateway_id":"igw","state":"active","origin":"CreateRoute"},
                  {"destination_cidr_block":"10.0.0.0/16","gateway_id":"local","state":"active"}]}),
        2,
    );
    store
        .apply_page(
            "older",
            page(
                vec![
                    older_route,
                    fixture(
                        "igw",
                        "aws_vpc_internet_gateway",
                        json!({"internet_gateway_id":"igw"}),
                        2,
                    ),
                ],
                vec![Edge {
                    from_id: "rtb".into(),
                    to_id: "igw".into(),
                    relation: "ROUTES_TO".into(),
                    source: Source::Direct,
                    observed_at: 2,
                    properties: json!({"evidence":"route configuration"}),
                }],
            ),
        )
        .await?;
    store.finish_scan("older", ScanStatus::Complete, 3).await?;
    store.begin_scan("newer", 4).await?;
    let newer_route = fixture(
        "rtb",
        "aws_vpc_route_table",
        json!({"route_table_id":"rtb","vpc_id":"vpc",
        "routes":[{"destination_cidr_block":"0.0.0.0/0","gateway_id":"igw","state":"blackhole"}]}),
        5,
    );
    store
        .apply_page(
            "newer",
            page(
                vec![newer_route],
                vec![Edge {
                    from_id: "rtb".into(),
                    to_id: "igw".into(),
                    relation: "ROUTES_TO".into(),
                    source: Source::Direct,
                    observed_at: 5,
                    properties: json!({}),
                }],
            ),
        )
        .await?;
    store.finish_scan("newer", ScanStatus::Partial, 6).await?;
    assert_eq!(
        store
            .query("SELECT scan_id,state FROM aws_vpc_route", 10)
            .await?,
        json!([{"scan_id":"newer","state":"blackhole"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT count(*) AS total FROM scan_vpc_routes WHERE scan_id='older'",
                1
            )
            .await?,
        json!([{"total":2}])
    );
    assert_eq!(
        store
            .query(
                "SELECT scan_id,target_observed FROM scan_architecture_edges ORDER BY scan_id",
                10
            )
            .await?,
        json!([{"scan_id":"newer","target_observed":0},{"scan_id":"older","target_observed":1}])
    );
    store.close().await;
    Ok(())
}

#[tokio::test]
async fn load_balancer_aliases_preserve_type_and_target_identity_fields() -> TestResult {
    let store = Store::open(":memory:").await?;
    store.begin_scan("load-balancers", 1).await?;
    let resources = vec![
        fixture(
            "alb",
            "aws_elbv2_load_balancer",
            json!({"type":"application","load_balancer_name":"public-web","scheme":"internet-facing"}),
            2,
        ),
        fixture(
            "nlb",
            "aws_elbv2_load_balancer",
            json!({"type":"network","load_balancer_name":"internal-tcp","scheme":"internal"}),
            2,
        ),
        fixture(
            "gwlb",
            "aws_elbv2_load_balancer",
            json!({"type":"gateway","load_balancer_name":"inspection"}),
            2,
        ),
        fixture(
            "target-a",
            "aws_elbv2_target_health",
            json!({"target_group_arn":"tg","target_id":"i-one","target_port":443,
            "availability_zone":"us-east-1a","quic_server_id":"server-a","target_health_state":"healthy"}),
            2,
        ),
        fixture(
            "target-b",
            "aws_elbv2_target_health",
            json!({"target_group_arn":"tg","target_id":"i-one","target_port":8443,
            "availability_zone":"us-east-1a","quic_server_id":"server-b","target_health_state":"unhealthy"}),
            2,
        ),
    ];
    store
        .apply_page("load-balancers", page(resources, vec![]))
        .await?;
    store
        .finish_scan("load-balancers", ScanStatus::Complete, 3)
        .await?;
    assert_eq!(
        store
            .query(
                "SELECT load_balancer_name,scheme FROM aws_ec2_application_load_balancer",
                10
            )
            .await?,
        json!([{"load_balancer_name":"public-web","scheme":"internet-facing"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT load_balancer_type FROM aws_ec2_network_load_balancer",
                10
            )
            .await?,
        json!([{"load_balancer_type":"network"}])
    );
    assert_eq!(
        store
            .query(
                "SELECT load_balancer_type FROM aws_ec2_gateway_load_balancer",
                10
            )
            .await?,
        json!([{"load_balancer_type":"gateway"}])
    );
    assert_eq!(store.query("SELECT target_port,quic_server_id,target_health_state FROM aws_elbv2_target_health ORDER BY target_port",10).await?,
        json!([{"target_port":443,"quic_server_id":"server-a","target_health_state":"healthy"},
               {"target_port":8443,"quic_server_id":"server-b","target_health_state":"unhealthy"}]));
    store.close().await;
    Ok(())
}
