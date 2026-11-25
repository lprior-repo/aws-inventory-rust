use super::*;

type Query<'q> = sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>;

const FILTER: &str = "($1::text IS NULL OR r.account=$1) AND ($2::text IS NULL OR r.region=$2) AND ($3::text IS NULL OR r.kind=$3) AND ($4::text IS NULL OR (coalesce(r.name,'') || ' ' || r.id || ' ' || r.kind) COLLATE \"C\" ILIKE $4 ESCAPE '!') AND ($5::text IS NULL OR EXISTS(SELECT 1 FROM inventory.tags t WHERE (t.account,t.region,t.kind,t.id)=(r.account,r.region,r.kind,r.id) AND t.tag_key=$5 AND ($6::text IS NULL OR t.tag_value=$6)))";
const METADATA: &str = "SELECT r.account,r.region,r.kind,r.id,r.name,r.content_hash,tags.tag_pairs,o.coverage,o.observed_at,o.last_complete_at FROM inventory.resources r LEFT JOIN LATERAL (SELECT coalesce(array_agg((t.tag_key,t.tag_value) ORDER BY t.tag_key COLLATE \"C\"),ARRAY[]::record[]) AS tag_pairs FROM (SELECT tag_key,tag_value FROM inventory.tags t WHERE (t.account,t.region,t.kind,t.id)=(r.account,r.region,r.kind,r.id) ORDER BY tag_key COLLATE \"C\" LIMIT 1001) t) tags ON true LEFT JOIN inventory.tag_observations o USING(account,region,kind,id)";
const RESOURCE_CURSOR: &str = " AND ($7::text IS NULL OR (r.account COLLATE \"C\",r.region COLLATE \"C\",r.kind COLLATE \"C\",r.id COLLATE \"C\")>($7,$8,$9,$10)) ORDER BY r.account COLLATE \"C\",r.region COLLATE \"C\",r.kind COLLATE \"C\",r.id COLLATE \"C\" LIMIT $11";
const NODE_EDGES: &str = "((from_account,from_region,from_kind,from_id)=($1,$2,$3,$4) OR (to_account,to_region,to_kind,to_id)=($1,$2,$3,$4))";
const EDGE_PAGE: &str = " AND ($5::text IS NULL OR (from_account COLLATE \"C\",from_region COLLATE \"C\",from_kind COLLATE \"C\",from_id COLLATE \"C\",to_account COLLATE \"C\",to_region COLLATE \"C\",to_kind COLLATE \"C\",to_id COLLATE \"C\",relation COLLATE \"C\",source COLLATE \"C\")>($5,$6,$7,$8,$9,$10,$11,$12,$13,$14)) ORDER BY from_account COLLATE \"C\",from_region COLLATE \"C\",from_kind COLLATE \"C\",from_id COLLATE \"C\",to_account COLLATE \"C\",to_region COLLATE \"C\",to_kind COLLATE \"C\",to_id COLLATE \"C\",relation COLLATE \"C\",source COLLATE \"C\" LIMIT $15";

fn pattern(text: &str) -> String {
    let mut pattern = String::with_capacity(text.len().saturating_mul(2).saturating_add(2));
    pattern.push('%');
    for character in text.chars() {
        if matches!(character, '!' | '%' | '_') {
            pattern.push('!');
        }
        pattern.push(character);
    }
    pattern.push('%');
    pattern
}

fn bind_filter<'q>(query: Query<'q>, filter: &'q ResourceFilter) -> Query<'q> {
    query
        .bind(filter.account.as_deref())
        .bind(filter.region.as_deref())
        .bind(filter.kind.as_deref())
        .bind(filter.text.as_deref().map(pattern))
        .bind(filter.tag.as_ref().map(|tag| tag.key.as_str()))
        .bind(filter.tag.as_ref().and_then(|tag| tag.value.as_deref()))
}

pub(super) async fn count_filtered(
    tx: &mut Transaction<'_, Postgres>,
    filter: &ResourceFilter,
) -> Result<u64, ViewError> {
    let mut builder = sqlx::QueryBuilder::<Postgres>::new(
        "SELECT count(*) AS count FROM inventory.resources r WHERE ",
    );
    builder.push(FILTER);
    #[cfg(test)]
    probes::query();
    let row = bind_filter(builder.build(), filter)
        .fetch_one(&mut **tx)
        .await?;
    unsigned(&row, "count")
}

pub(super) async fn fetch_page(
    tx: &mut Transaction<'_, Postgres>,
    filter: &ResourceFilter,
    request: &PageRequest,
) -> Result<Vec<PgRow>, ViewError> {
    let mut builder = sqlx::QueryBuilder::<Postgres>::new(METADATA);
    builder.push(" WHERE ").push(FILTER).push(RESOURCE_CURSOR);
    let cursor = request.after.as_ref();
    #[cfg(test)]
    probes::query();
    Ok(bind_filter(builder.build(), filter)
        .bind(cursor.map(|v| v.account.as_str()))
        .bind(cursor.map(|v| v.region.as_str()))
        .bind(cursor.map(|v| v.kind.as_str()))
        .bind(cursor.map(|v| v.id.as_str()))
        .bind(page_limit(request.limit)?)
        .fetch_all(&mut **tx)
        .await?)
}

pub(super) async fn detail(
    tx: &mut Transaction<'_, Postgres>,
    node: &GraphNode,
) -> Result<Option<PgRow>, ViewError> {
    let mut builder = sqlx::QueryBuilder::<Postgres>::new(METADATA);
    builder.push(" WHERE (r.account,r.region,r.kind,r.id)=($1,$2,$3,$4)");
    #[cfg(test)]
    probes::query();
    Ok(bind_node(builder.build(), node)
        .fetch_optional(&mut **tx)
        .await?)
}

pub(super) async fn table_for<'a>(
    snapshot: &'a mut Snapshot,
    kind: &str,
) -> Result<&'a SqlName, ViewError> {
    if !snapshot.tables.contains_key(kind) {
        #[cfg(test)]
        probes::query();
        let table: String =
            sqlx::query_scalar("SELECT table_name FROM inventory.kinds WHERE kind=$1")
                .bind(kind)
                .fetch_optional(&mut *snapshot.tx)
                .await?
                .ok_or(ViewError::StoredRow)?;
        if table.len() > 63 {
            return Err(ViewError::SqlName);
        }
        let table = SqlName::parse(&table).map_err(|_| ViewError::SqlName)?;
        snapshot.tables.insert(kind.to_owned(), table);
    }
    snapshot.tables.get(kind).ok_or(ViewError::StoredRow)
}

pub(super) async fn read_detail(
    snapshot: &mut Snapshot,
    key: &ResourceKey,
) -> Result<PgRow, ViewError> {
    let table = table_for(snapshot, key.kind.as_str()).await?;
    let sql = format!(
        "SELECT record,content_hash FROM inventory.\"{}\" WHERE account=$1 AND region=$2 AND id=$3",
        table.as_str()
    );
    #[cfg(test)]
    probes::query();
    sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
        .bind(key.account.as_str())
        .bind(key.region.as_str())
        .bind(key.id.as_str())
        .fetch_optional(&mut *snapshot.tx)
        .await?
        .ok_or(ViewError::StoredRow)
}

pub(super) async fn record_lengths(
    snapshot: &mut Snapshot,
    rows: &[PgRow],
) -> Result<BTreeMap<ResourceKey, usize>, ViewError> {
    let groups = rows.iter().try_fold(
        BTreeMap::<String, Vec<ResourceKey>>::new(),
        |mut groups, row| {
            let key = row_key(row)?;
            groups
                .entry(key.kind.as_str().to_owned())
                .or_default()
                .push(key);
            Ok::<_, ViewError>(groups)
        },
    )?;
    stream::iter(groups.into_iter().map(Ok))
        .try_fold((snapshot, BTreeMap::new()), |(snapshot, mut lengths), (kind, keys)| async move {
            let table = table_for(snapshot, &kind).await?;
            let sql = format!("SELECT k.account,k.region,k.id,octet_length(k.record)::bigint AS bytes FROM inventory.\"{}\" k JOIN unnest($1::text[],$2::text[],$3::text[]) AS p(account,region,id) USING(account,region,id)", table.as_str());
            let accounts: Vec<&str> = keys.iter().map(|key| key.account.as_str()).collect();
            let regions: Vec<&str> = keys.iter().map(|key| key.region.as_str()).collect();
            let ids: Vec<&str> = keys.iter().map(|key| key.id.as_str()).collect();
            #[cfg(test)]
            probes::query();
            let rows = sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).bind(accounts).bind(regions).bind(ids).fetch_all(&mut *snapshot.tx).await?;
            rows.into_iter().try_for_each(|row| {
                let key = resource_key([row.try_get("account")?, row.try_get("region")?, &kind, row.try_get("id")?])?;
                let bytes = usize::try_from(row.try_get::<i64, _>("bytes")?).map_err(|_| ViewError::StoredRow)?;
                lengths.insert(key, bytes);
                Ok::<_, ViewError>(())
            })?;
            Ok::<_, ViewError>((snapshot, lengths))
        }).await.map(|(_, lengths)| lengths)
}

fn bind_node<'q>(query: Query<'q>, node: &'q GraphNode) -> Query<'q> {
    query
        .bind(&node.account)
        .bind(&node.region)
        .bind(&node.kind)
        .bind(&node.id)
}

pub(super) async fn count_node_edges(
    tx: &mut Transaction<'_, Postgres>,
    node: &GraphNode,
) -> Result<u64, ViewError> {
    let mut builder =
        sqlx::QueryBuilder::<Postgres>::new("SELECT count(*) AS count FROM inventory.edges WHERE ");
    builder.push(NODE_EDGES);
    let row = bind_node(builder.build(), node)
        .fetch_one(&mut **tx)
        .await?;
    unsigned(&row, "count")
}

pub(super) async fn fetch_node_edges(
    tx: &mut Transaction<'_, Postgres>,
    node: &GraphNode,
    request: &EdgePageRequest,
) -> Result<Vec<GraphEdge>, ViewError> {
    let mut builder = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM inventory.edges WHERE ");
    builder.push(NODE_EDGES).push(EDGE_PAGE);
    let cursor = request.after.as_ref().map(|cursor| &cursor.edge);
    let rows = bind_node(builder.build(), node)
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
        .bind(page_limit(request.limit)?)
        .fetch_all(&mut **tx)
        .await?;
    rows.into_iter().map(decode_edge).collect()
}

fn decode_edge(row: PgRow) -> Result<GraphEdge, ViewError> {
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

pub(super) fn unsigned(row: &PgRow, column: &str) -> Result<u64, ViewError> {
    u64::try_from(row.try_get::<i64, _>(column)?).map_err(|_| ViewError::StoredRow)
}

fn page_limit(limit: usize) -> Result<i64, ViewError> {
    i64::try_from(limit.checked_add(1).ok_or(ViewError::PageLimit)?)
        .map_err(|_| ViewError::PageLimit)
}
