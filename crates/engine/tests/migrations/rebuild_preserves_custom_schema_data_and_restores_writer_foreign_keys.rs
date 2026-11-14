use super::*;

#[tokio::test]
pub(super) async fn rebuild_preserves_custom_schema_data_and_restores_writer_foreign_keys()
-> TestResult {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("extensions.sqlite");
    legacy_database(&path).await?;
    let mut connection = connect(&path).await?;
    sqlx::raw_sql(include_str!("../../sql/002_architecture.sql"))
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
