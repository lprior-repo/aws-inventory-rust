use super::{SchemaRegistry, StoreError};
use inventory_core::inventory::{KindSchema, ScalarType};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;

const TAG_OBSERVATIONS_SQL: &str = "CREATE TABLE tag_observations(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,coverage TEXT NOT NULL CHECK(coverage IN('complete','partial','failed','unobserved')),observed_at INTEGER NOT NULL CHECK(observed_at>=0),last_complete_at INTEGER CHECK(last_complete_at>=0),PRIMARY KEY(account,region,kind,id),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID";
const SCOPE_FRESHNESS_SQL: &str = include_str!("../../sql/inventory_scope_freshness.sql");

pub(super) async fn initialize(
    pool: &SqlitePool,
    registry: &SchemaRegistry,
) -> Result<(), StoreError> {
    let mut tx = pool.begin().await?;
    let version: i64 = sqlx::query("PRAGMA user_version")
        .fetch_one(&mut *tx)
        .await?
        .try_get(0)?;
    if version != 0 {
        return Err(StoreError::SchemaVersion { found: version });
    }
    create_shared_tables(&mut tx).await?;
    super::legacy::create(&mut tx).await?;
    super::relation_permissions::initialize_provenance(&mut tx).await?;
    sqlx::query("INSERT INTO metadata(key,value) VALUES('catalog_digest',?1),('projection_rebuild',?2),('projection_revision',0),('projection_pending',0),('canonical_facet_dependencies_v3',X'01')")
        .bind(registry.digest().as_slice()).bind(vec![0_u8])
        .execute(&mut *tx).await?;
    super::authority::mark(&mut tx).await?;
    for (service, operation) in registry.operations() {
        sqlx::query("INSERT INTO catalog_operations(service,operation) VALUES(?1,?2)")
            .bind(service)
            .bind(operation)
            .execute(&mut *tx)
            .await?;
    }
    for schema in registry.schemas() {
        create_kind(&mut tx, schema).await?;
    }
    super::relation_permissions::persist(&mut tx, registry).await?;
    create_scope_freshness_view(&mut tx).await?;
    sqlx::query("PRAGMA user_version=16")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn migrate_inventory(pool: &SqlitePool) -> Result<(), StoreError> {
    let mut tx = pool.begin().await?;
    let version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(&mut *tx)
        .await?;
    if !matches!(version, 14..=16) {
        return Err(StoreError::SchemaVersion { found: version });
    }
    sqlx::query("DROP TRIGGER IF EXISTS delete_resource_children")
        .execute(&mut *tx)
        .await?;
    if version == 14 {
        sqlx::query(TAG_OBSERVATIONS_SQL).execute(&mut *tx).await?;
    }
    if version < 16 {
        sqlx::query(super::facets::CREATE_SQL)
            .execute(&mut *tx)
            .await?;
        sqlx::query("ALTER TABLE work_queue ADD COLUMN context BLOB NOT NULL DEFAULT X''")
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "ALTER TABLE unresolved_refs ADD COLUMN target_kinds BLOB NOT NULL DEFAULT X''",
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query("ALTER TABLE unresolved_refs ADD COLUMN resolution TEXT NOT NULL DEFAULT 'unresolved' CHECK(resolution IN('unresolved','resolved','ambiguous'))")
            .execute(&mut *tx).await?;
        sqlx::query("PRAGMA user_version=16")
            .execute(&mut *tx)
            .await?;
    }
    super::legacy::create(&mut tx).await?;
    let work_columns = sqlx::query("PRAGMA table_info(work_queue)")
        .fetch_all(&mut *tx)
        .await?;
    let has_epoch = work_columns.iter().try_fold(false, |found, row| {
        Ok::<_, sqlx::Error>(found || row.try_get::<&str, _>("name")? == "authority_generation")
    })?;
    if !has_epoch {
        sqlx::query("ALTER TABLE work_queue ADD COLUMN authority_generation INTEGER NOT NULL DEFAULT 0 CHECK(authority_generation>=0)").execute(&mut *tx).await?;
    }
    let has_prefix = work_columns.iter().try_fold(false, |found, row| {
        Ok::<_, sqlx::Error>(found || row.try_get::<&str, _>("name")? == "authority_complete")
    })?;
    if !has_prefix {
        sqlx::query("ALTER TABLE work_queue ADD COLUMN authority_complete INTEGER NOT NULL DEFAULT 0 CHECK(authority_complete IN(0,1))").execute(&mut *tx).await?;
    }
    super::relation_permissions::initialize_provenance(&mut tx).await?;
    super::authority::migrate(&mut tx).await?;
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='sweeps')",
    )
    .fetch_one(&mut *tx)
    .await?;
    if !table_exists {
        sqlx::query(
            "CREATE TABLE sweeps(sweep_id TEXT PRIMARY KEY,account TEXT NOT NULL,partition TEXT NOT NULL DEFAULT 'aws',started_at INTEGER NOT NULL,ended_at INTEGER NOT NULL DEFAULT 0,completed INTEGER NOT NULL DEFAULT 0) WITHOUT ROWID",
        )
        .execute(&mut *tx)
        .await?;
    }
    let facet_columns = sqlx::query("PRAGMA table_info(resource_facets)")
        .fetch_all(&mut *tx)
        .await?;
    let attributed = facet_columns.iter().try_fold(false, |found, row| {
        Ok::<_, sqlx::Error>(found || row.try_get::<String, _>("name")? == "source_key")
    })?;
    let source_authority = facet_columns.iter().try_fold(false, |found, row| {
        Ok::<_, sqlx::Error>(found || row.try_get::<&str, _>("name")? == "source_authority")
    })?;
    let checked: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM metadata WHERE key='canonical_facet_dependencies_v3' AND value=X'01')").fetch_one(&mut *tx).await?;
    if version < 16 || !attributed || !source_authority || !has_epoch || !has_prefix || !checked {
        // Catalog equality cannot certify a dependency written under the old
        // stage/input proof domain. Preserve its data, never promote its proof.
        super::legacy::demote_facets(&mut tx, attributed).await?;
        super::relation_permissions::invalidate_prior_authority(&mut tx).await?;
        sqlx::query("DROP TABLE resource_facets")
            .execute(&mut *tx)
            .await?;
        sqlx::query(super::facets::CREATE_SQL)
            .execute(&mut *tx)
            .await?;
    } else if !facet_columns.iter().any(|row| {
        row.try_get::<&str, _>("name")
            .is_ok_and(|name| name == "bound_request")
    }) {
        // Existing V3 facets remain readable; absent validated requests cannot authorize bound-context targets.
        sqlx::query("ALTER TABLE resource_facets ADD COLUMN bound_request BLOB")
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT INTO metadata(key,value) VALUES('canonical_facet_dependencies_v3',X'01') ON CONFLICT(key) DO UPDATE SET value=excluded.value").execute(&mut *tx).await?;
    for statement in [
        "CREATE INDEX IF NOT EXISTS work_queue_ready ON work_queue(sweep_id,state,scope_key)",
        "CREATE INDEX IF NOT EXISTS scope_outcomes_complete ON scope_outcomes(scope_key,outcome,observed_at)",
        "CREATE INDEX IF NOT EXISTS unresolved_refs_target ON unresolved_refs(target_scope,target_kind)",
    ] {
        sqlx::query(statement).execute(&mut *tx).await?;
    }
    create_scope_freshness_view(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}

/// The scope freshness view is derived data: it is replaced on every open so a
/// definition change reaches existing databases without a schema version bump.
async fn create_scope_freshness_view(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(), StoreError> {
    sqlx::query("DROP VIEW IF EXISTS scope_freshness")
        .execute(&mut **tx)
        .await?;
    sqlx::query(SCOPE_FRESHNESS_SQL).execute(&mut **tx).await?;
    Ok(())
}

async fn create_shared_tables(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(), StoreError> {
    let indexes = [
        "CREATE INDEX IF NOT EXISTS work_queue_ready ON work_queue(sweep_id,state,scope_key)",
        "CREATE INDEX IF NOT EXISTS scope_outcomes_complete ON scope_outcomes(scope_key,outcome,observed_at)",
        "CREATE INDEX IF NOT EXISTS unresolved_refs_target ON unresolved_refs(target_scope,target_kind)",
        "CREATE INDEX IF NOT EXISTS resource_scopes_key ON resource_scopes(scope_key)",
    ];
    for statement in [
        "CREATE TABLE metadata(key TEXT PRIMARY KEY,value BLOB NOT NULL) WITHOUT ROWID",
        "CREATE TABLE kinds(kind TEXT PRIMARY KEY,table_name TEXT NOT NULL UNIQUE) WITHOUT ROWID",
        "CREATE TABLE kind_schemas(kind TEXT PRIMARY KEY,table_name TEXT NOT NULL,definition BLOB NOT NULL) WITHOUT ROWID",
        "CREATE TABLE resources(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,name TEXT,content_hash BLOB NOT NULL,first_seen INTEGER NOT NULL,last_changed INTEGER NOT NULL,PRIMARY KEY(account,region,kind,id)) WITHOUT ROWID",
        "CREATE TABLE sweeps(sweep_id TEXT PRIMARY KEY,account TEXT NOT NULL,partition TEXT NOT NULL DEFAULT 'aws',started_at INTEGER NOT NULL,ended_at INTEGER NOT NULL DEFAULT 0,completed INTEGER NOT NULL DEFAULT 0) WITHOUT ROWID",
        super::facets::CREATE_SQL,
        "CREATE TABLE resource_scopes(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,scope_key BLOB NOT NULL,last_seen INTEGER NOT NULL,missing_complete INTEGER NOT NULL DEFAULT 0,observed_sweep TEXT NOT NULL,completed_sweep TEXT NOT NULL DEFAULT '',PRIMARY KEY(account,region,kind,id,scope_key),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE tags(account TEXT NOT NULL,region TEXT NOT NULL,kind TEXT NOT NULL,id TEXT NOT NULL,tag_key TEXT NOT NULL,tag_value TEXT NOT NULL,PRIMARY KEY(account,region,kind,id,tag_key),FOREIGN KEY(account,region,kind,id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        TAG_OBSERVATIONS_SQL,
        "CREATE TABLE edges(from_account TEXT NOT NULL,from_region TEXT NOT NULL,from_kind TEXT NOT NULL,from_id TEXT NOT NULL,to_account TEXT NOT NULL,to_region TEXT NOT NULL,to_kind TEXT NOT NULL,to_id TEXT NOT NULL,relation TEXT NOT NULL,source TEXT NOT NULL,PRIMARY KEY(from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source),FOREIGN KEY(from_account,from_region,from_kind,from_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE,FOREIGN KEY(to_account,to_region,to_kind,to_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE unresolved_refs(from_account TEXT NOT NULL,from_region TEXT NOT NULL,from_kind TEXT NOT NULL,from_id TEXT NOT NULL,target_scope BLOB NOT NULL,target_kind TEXT NOT NULL,target_path BLOB NOT NULL,target_value TEXT NOT NULL,relation TEXT NOT NULL,source TEXT NOT NULL,target_kinds BLOB NOT NULL DEFAULT X'',resolution TEXT NOT NULL DEFAULT 'unresolved' CHECK(resolution IN('unresolved','resolved','ambiguous')),PRIMARY KEY(from_account,from_region,from_kind,from_id,target_scope,target_kind,target_path,target_value,relation,source),FOREIGN KEY(from_account,from_region,from_kind,from_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE resource_parents(parent_account TEXT NOT NULL,parent_region TEXT NOT NULL,parent_kind TEXT NOT NULL,parent_id TEXT NOT NULL,child_account TEXT NOT NULL,child_region TEXT NOT NULL,child_kind TEXT NOT NULL,child_id TEXT NOT NULL,scope_key BLOB NOT NULL,PRIMARY KEY(parent_account,parent_region,parent_kind,parent_id,child_account,child_region,child_kind,child_id,scope_key),FOREIGN KEY(parent_account,parent_region,parent_kind,parent_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE,FOREIGN KEY(child_account,child_region,child_kind,child_id) REFERENCES resources(account,region,kind,id) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE scope_outcomes(sweep_id TEXT NOT NULL,scope_key BLOB NOT NULL,outcome TEXT NOT NULL,observed_at INTEGER NOT NULL,PRIMARY KEY(sweep_id,scope_key)) WITHOUT ROWID",
        "CREATE TABLE work_queue(sweep_id TEXT NOT NULL,scope_key BLOB NOT NULL,token BLOB,bindings BLOB NOT NULL,context BLOB NOT NULL DEFAULT X'',state INTEGER NOT NULL,generation INTEGER NOT NULL,authority_generation INTEGER NOT NULL DEFAULT 0 CHECK(authority_generation>=0),authority_complete INTEGER NOT NULL DEFAULT 0 CHECK(authority_complete IN(0,1)),lease_until INTEGER NOT NULL,PRIMARY KEY(sweep_id,scope_key)) WITHOUT ROWID",
        "CREATE TABLE work_tokens(sweep_id TEXT NOT NULL,scope_key BLOB NOT NULL,digest BLOB NOT NULL CHECK(length(digest)=32),PRIMARY KEY(sweep_id,scope_key,digest),FOREIGN KEY(sweep_id,scope_key) REFERENCES work_queue(sweep_id,scope_key) ON DELETE CASCADE) WITHOUT ROWID",
        "CREATE TABLE catalog_operations(service TEXT NOT NULL,operation TEXT NOT NULL,PRIMARY KEY(service,operation)) WITHOUT ROWID",
        "CREATE TABLE projection_outbox(resource_key BLOB PRIMARY KEY,revision INTEGER NOT NULL,deleted INTEGER NOT NULL,full_rebuild INTEGER NOT NULL DEFAULT 0) WITHOUT ROWID",
        "CREATE TABLE learned_rates(service TEXT NOT NULL,operation TEXT NOT NULL,rate REAL NOT NULL,sample_count INTEGER NOT NULL,updated_at INTEGER NOT NULL,PRIMARY KEY(service,operation)) WITHOUT ROWID",
    ] {
        sqlx::query(statement).execute(&mut **tx).await?;
    }
    for statement in indexes {
        sqlx::query(statement).execute(&mut **tx).await?;
    }
    Ok(())
}

async fn create_kind(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    schema: &Arc<KindSchema>,
) -> Result<(), StoreError> {
    let table = quote(schema.table().as_str());
    let mut fields = vec![
        "account TEXT NOT NULL".to_owned(),
        "region TEXT NOT NULL".to_owned(),
        "id TEXT NOT NULL".to_owned(),
        "content_hash BLOB NOT NULL".to_owned(),
        "first_seen INTEGER NOT NULL".to_owned(),
        "last_changed INTEGER NOT NULL".to_owned(),
        "record BLOB NOT NULL".to_owned(),
    ];
    schema
        .columns()
        .iter()
        .for_each(|column| append_column(&mut fields, column.name(), column.scalar()));
    fields.push("PRIMARY KEY(account,region,id)".to_owned());
    let statement = format!("CREATE TABLE {table}({}) WITHOUT ROWID", fields.join(","));
    sqlx::QueryBuilder::<sqlx::Sqlite>::new(statement)
        .build()
        .execute(&mut **tx)
        .await?;
    sqlx::query("INSERT INTO kinds(kind,table_name) VALUES(?1,?2)")
        .bind(schema.kind().as_str())
        .bind(schema.table().as_str())
        .execute(&mut **tx)
        .await?;
    sqlx::query("INSERT INTO kind_schemas(kind,table_name,definition) VALUES(?1,?2,?3)")
        .bind(schema.kind().as_str())
        .bind(schema.table().as_str())
        .bind(encode_schema(schema)?)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn append_column(fields: &mut Vec<String>, name: &str, scalar: ScalarType) {
    let q = quote(name);
    match scalar {
        ScalarType::Bool | ScalarType::Integer => fields.push(format!("{q} INTEGER")),
        ScalarType::Real => {
            fields.push(format!("{q} REAL"));
            fields.push(format!("{} INTEGER", quote(&format!("{name}__nonfinite"))));
        }
        ScalarType::Text => fields.push(format!("{q} TEXT")),
        ScalarType::Blob => fields.push(format!("{q} BLOB")),
        ScalarType::Timestamp => {
            fields.push(format!("{} INTEGER", quote(&format!("{name}__seconds"))));
            fields.push(format!("{} INTEGER", quote(&format!("{name}__nanos"))));
        }
    }
}

fn quote(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
pub(super) async fn load_registry(pool: &SqlitePool) -> Result<SchemaRegistry, StoreError> {
    let digest: Vec<u8> = sqlx::query("SELECT value FROM metadata WHERE key='catalog_digest'")
        .fetch_one(pool)
        .await?
        .try_get(0)?;
    let digest: [u8; 32] = digest
        .try_into()
        .map_err(|_| StoreError::Invalid("stored catalog digest is not 32 bytes"))?;
    let operations = sqlx::query("SELECT service,operation FROM catalog_operations")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| Ok((row.try_get("service")?, row.try_get("operation")?)))
        .collect::<Result<std::collections::BTreeSet<(String, String)>, StoreError>>()?;
    super::relation_permissions::load_registry(pool, digest, load_schemas(pool).await?, operations)
        .await
}

async fn load_schemas(pool: &SqlitePool) -> Result<Vec<Arc<KindSchema>>, StoreError> {
    let rows = sqlx::query("SELECT kind,table_name,definition FROM kind_schemas ORDER BY kind")
        .fetch_all(pool)
        .await?;
    rows.into_iter()
        .map(|row| {
            let kind: String = row.try_get("kind")?;
            let table: String = row.try_get("table_name")?;
            let definition: Vec<u8> = row.try_get("definition")?;
            decode_schema(&kind, &table, &definition).map(Arc::new)
        })
        .collect()
}

fn encode_schema(schema: &KindSchema) -> Result<Vec<u8>, StoreError> {
    let count = u16::try_from(schema.columns().len())
        .map_err(|_| StoreError::Invalid("too many schema columns"))?;
    let mut bytes = count.to_be_bytes().to_vec();
    for column in schema.columns() {
        bytes.push(
            u8::try_from(column.path().len())
                .map_err(|_| StoreError::Invalid("schema path too long"))?,
        );
        for field in column.path() {
            bytes.extend_from_slice(&field.value().to_be_bytes());
        }
        bytes.push(scalar_code(column.scalar()));
    }
    Ok(bytes)
}

fn decode_schema(kind: &str, table: &str, bytes: &[u8]) -> Result<KindSchema, StoreError> {
    let mut cursor = SchemaCursor { bytes, offset: 0 };
    let count = usize::from(cursor.u16()?);
    let columns = (0..count)
        .map(|_| decode_column(&mut cursor))
        .collect::<Result<Vec<_>, _>>()?;
    if !cursor.finished() {
        return Err(StoreError::Invalid("trailing schema bytes"));
    }
    Ok(KindSchema::new(
        inventory_core::inventory::Kind::parse(kind)?,
        inventory_core::inventory::SqlName::parse(table)?,
        columns,
    )?)
}

fn decode_column(
    cursor: &mut SchemaCursor<'_>,
) -> Result<inventory_core::inventory::Column, StoreError> {
    let len = usize::from(cursor.byte()?);
    if len > 8 {
        return Err(StoreError::Invalid("schema path exceeds bounds"));
    }
    let path = (0..len)
        .map(|_| inventory_core::record::FieldId::new(cursor.u32()?).map_err(StoreError::from))
        .collect::<Result<Vec<_>, _>>()?;
    let scalar = decode_scalar(cursor.byte()?)?;
    if path.is_empty() {
        return Ok(inventory_core::inventory::Column::selected_scalar(scalar));
    }
    inventory_core::inventory::Column::new(path, scalar).map_err(StoreError::from)
}

fn scalar_code(scalar: ScalarType) -> u8 {
    match scalar {
        ScalarType::Bool => 0,
        ScalarType::Integer => 1,
        ScalarType::Real => 2,
        ScalarType::Text => 3,
        ScalarType::Blob => 4,
        ScalarType::Timestamp => 5,
    }
}

fn decode_scalar(code: u8) -> Result<ScalarType, StoreError> {
    match code {
        0 => Ok(ScalarType::Bool),
        1 => Ok(ScalarType::Integer),
        2 => Ok(ScalarType::Real),
        3 => Ok(ScalarType::Text),
        4 => Ok(ScalarType::Blob),
        5 => Ok(ScalarType::Timestamp),
        _ => Err(StoreError::Invalid("unknown schema scalar")),
    }
}

struct SchemaCursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl SchemaCursor<'_> {
    fn byte(&mut self) -> Result<u8, StoreError> {
        let value = self
            .bytes
            .get(self.offset)
            .copied()
            .ok_or(StoreError::Invalid("truncated schema definition"))?;
        self.offset = self
            .offset
            .checked_add(1)
            .ok_or(StoreError::Invalid("schema offset overflow"))?;
        Ok(value)
    }
    fn u16(&mut self) -> Result<u16, StoreError> {
        Ok(u16::from_be_bytes([self.byte()?, self.byte()?]))
    }
    fn u32(&mut self) -> Result<u32, StoreError> {
        Ok(u32::from_be_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }
    fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
