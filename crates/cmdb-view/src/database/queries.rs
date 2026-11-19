use super::*;

pub(super) async fn read_detail(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    table: &SqlName,
    account: &str,
    region: &str,
    id: &str,
) -> Result<sqlx::sqlite::SqliteRow, ViewError> {
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT record,content_hash FROM \"");
    query
        .push(table.as_str())
        .push("\" WHERE account=")
        .push_bind(account)
        .push(" AND region=")
        .push_bind(region)
        .push(" AND id=")
        .push_bind(id);
    query
        .build()
        .fetch_one(&mut **tx)
        .await
        .map_err(ViewError::Sql)
}

/// Returns only the byte length of the persisted record payload, avoiding
/// loading the full payload for table-page projection.
pub(super) async fn record_byte_length(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    table: &SqlName,
    account: &str,
    region: &str,
    id: &str,
) -> Result<usize, ViewError> {
    let mut query = sqlx::QueryBuilder::<sqlx::Sqlite>::new("SELECT length(record) FROM \"");
    query
        .push(table.as_str())
        .push("\" WHERE account=")
        .push_bind(account)
        .push(" AND region=")
        .push_bind(region)
        .push(" AND id=")
        .push_bind(id);
    let result: i64 = query
        .build()
        .fetch_one(&mut **tx)
        .await
        .map_err(ViewError::Sql)?
        .try_get::<i64, _>(0)?;
    usize::try_from(result).map_err(|_| ViewError::StoredRow)
}

pub(super) async fn scalar_count(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    sql: &'static str,
) -> Result<u64, ViewError> {
    let value = sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(&mut **tx)
        .await?;
    u64::try_from(value).map_err(|_| ViewError::StoredRow)
}

pub(super) async fn count_filtered(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    filter: &ResourceFilter,
) -> Result<u64, ViewError> {
    let value = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM resources WHERE (? IS NULL OR account=?) AND (? IS NULL OR region=?) AND (? IS NULL OR kind=?) AND (? IS NULL OR instr(lower(coalesce(name,'') || ' ' || id || ' ' || kind),lower(?))>0) AND (? IS NULL OR EXISTS(SELECT 1 FROM tags t WHERE t.account=resources.account AND t.region=resources.region AND t.kind=resources.kind AND t.id=resources.id AND t.tag_key=? AND (? IS NULL OR t.tag_value=?)))")
        .bind(filter.account.as_deref()).bind(filter.account.as_deref())
        .bind(filter.region.as_deref()).bind(filter.region.as_deref())
        .bind(filter.kind.as_deref()).bind(filter.kind.as_deref())
        .bind(filter.text.as_deref()).bind(filter.text.as_deref())
        .bind(filter.tag.as_ref().map(|tag| tag.key.as_str()))
        .bind(filter.tag.as_ref().map(|tag| tag.key.as_str()))
        .bind(filter.tag.as_ref().and_then(|tag| tag.value.as_deref()))
        .bind(filter.tag.as_ref().and_then(|tag| tag.value.as_deref()))
        .fetch_one(&mut **tx).await?;
    u64::try_from(value).map_err(|_| ViewError::StoredRow)
}

pub(super) async fn fetch_page(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    filter: &ResourceFilter,
    request: &PageRequest,
) -> Result<Vec<sqlx::sqlite::SqliteRow>, ViewError> {
    let limit = i64::try_from(request.limit.checked_add(1).ok_or(ViewError::PageLimit)?)
        .map_err(|_| ViewError::PageLimit)?;
    let cursor = request.after.as_ref();
    let query = sqlx::query(
        "SELECT account,region,kind,id,name,content_hash FROM resources WHERE (? IS NULL OR account=?) AND (? IS NULL OR region=?) AND (? IS NULL OR kind=?) AND (? IS NULL OR instr(lower(coalesce(name,'') || ' ' || id || ' ' || kind),lower(?))>0) AND (? IS NULL OR EXISTS(SELECT 1 FROM tags t WHERE t.account=resources.account AND t.region=resources.region AND t.kind=resources.kind AND t.id=resources.id AND t.tag_key=? AND (? IS NULL OR t.tag_value=?))) AND (? IS NULL OR (account,region,kind,id)>(?,?,?,?)) ORDER BY account,region,kind,id LIMIT ?",
    );
    let rows = bind_filter(query, filter)
        .bind(cursor.map(|_| 1_i64))
        .bind(cursor.map(|v| v.account.as_str()))
        .bind(cursor.map(|v| v.region.as_str()))
        .bind(cursor.map(|v| v.kind.as_str()))
        .bind(cursor.map(|v| v.id.as_str()))
        .bind(limit)
        .fetch_all(&mut **tx)
        .await?;
    Ok(rows)
}

type SqliteQuery<'q> = sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments>;

fn bind_filter<'q>(query: SqliteQuery<'q>, filter: &'q ResourceFilter) -> SqliteQuery<'q> {
    query
        .bind(filter.account.as_deref())
        .bind(filter.account.as_deref())
        .bind(filter.region.as_deref())
        .bind(filter.region.as_deref())
        .bind(filter.kind.as_deref())
        .bind(filter.kind.as_deref())
        .bind(filter.text.as_deref())
        .bind(filter.text.as_deref())
        .bind(filter.tag.as_ref().map(|tag| tag.key.as_str()))
        .bind(filter.tag.as_ref().map(|tag| tag.key.as_str()))
        .bind(filter.tag.as_ref().and_then(|tag| tag.value.as_deref()))
        .bind(filter.tag.as_ref().and_then(|tag| tag.value.as_deref()))
}

fn bind_node<'q>(query: SqliteQuery<'q>, node: &'q GraphNode) -> SqliteQuery<'q> {
    query
        .bind(&node.account)
        .bind(&node.region)
        .bind(&node.kind)
        .bind(&node.id)
}

pub(super) async fn count_node_edges(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node: &GraphNode,
) -> Result<u64, ViewError> {
    let count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM edges WHERE (from_account=? AND from_region=? AND from_kind=? AND from_id=?) OR (to_account=? AND to_region=? AND to_kind=? AND to_id=?)")
        .bind(&node.account).bind(&node.region).bind(&node.kind).bind(&node.id)
        .bind(&node.account).bind(&node.region).bind(&node.kind).bind(&node.id)
        .fetch_one(&mut **tx).await?;
    u64::try_from(count).map_err(|_| ViewError::StoredRow)
}

pub(super) async fn fetch_node_edges(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node: &GraphNode,
    request: &EdgePageRequest,
) -> Result<Vec<sqlx::sqlite::SqliteRow>, ViewError> {
    let limit = i64::try_from(request.limit.checked_add(1).ok_or(ViewError::PageLimit)?)
        .map_err(|_| ViewError::PageLimit)?;
    let cursor = request.after.as_ref().map(|value| &value.edge);
    let query = sqlx::query(
        "SELECT from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source FROM edges WHERE ((from_account=? AND from_region=? AND from_kind=? AND from_id=?) OR (to_account=? AND to_region=? AND to_kind=? AND to_id=?)) AND (? IS NULL OR (from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source)>(?,?,?,?,?,?,?,?,?,?)) ORDER BY from_account,from_region,from_kind,from_id,to_account,to_region,to_kind,to_id,relation,source LIMIT ?",
    );
    let query = bind_edge_cursor(bind_node(bind_node(query, node), node), cursor);
    Ok(query.bind(limit).fetch_all(&mut **tx).await?)
}

fn bind_edge_cursor<'q>(query: SqliteQuery<'q>, cursor: Option<&'q GraphEdge>) -> SqliteQuery<'q> {
    query
        .bind(cursor.map(|_| 1_i64))
        .bind(cursor.map(|v| v.from.account.as_str()))
        .bind(cursor.map(|v| v.from.region.as_str()))
        .bind(cursor.map(|v| v.from.kind.as_str()))
        .bind(cursor.map(|v| v.from.id.as_str()))
        .bind(cursor.map(|v| v.to.account.as_str()))
        .bind(cursor.map(|v| v.to.region.as_str()))
        .bind(cursor.map(|v| v.to.kind.as_str()))
        .bind(cursor.map(|v| v.to.id.as_str()))
        .bind(cursor.map(|v| v.relation.as_str()))
        .bind(cursor.map(|v| v.provenance.as_str()))
}

pub(super) fn decode_edge(row: sqlx::sqlite::SqliteRow) -> Result<GraphEdge, ViewError> {
    Ok(GraphEdge {
        from: GraphNode {
            account: row.try_get("from_account")?,
            region: row.try_get("from_region")?,
            kind: row.try_get("from_kind")?,
            id: row.try_get("from_id")?,
        },
        to: GraphNode {
            account: row.try_get("to_account")?,
            region: row.try_get("to_region")?,
            kind: row.try_get("to_kind")?,
            id: row.try_get("to_id")?,
        },
        relation: row.try_get("relation")?,
        provenance: row.try_get("source")?,
    })
}

pub(super) fn edge_cursor(edge: &GraphEdge) -> EdgeCursor {
    EdgeCursor { edge: edge.clone() }
}

pub(super) fn cursor_of(resource: &StoredResource) -> PageCursor {
    PageCursor {
        account: resource.key.account.as_str().to_owned(),
        region: resource.key.region.as_str().to_owned(),
        kind: resource.key.kind.as_str().to_owned(),
        id: resource.key.id.as_str().to_owned(),
    }
}
