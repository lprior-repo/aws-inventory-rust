//! Atomic page writes and batched observation insertion.
use super::*;

pub(super) async fn persist_page(
    mut tx: SqlTransaction,
    scan_id: &str,
    page: PreparedPage,
) -> Result<(), StoreError> {
    if page_already_committed(&mut tx, scan_id, &page).await? {
        tx.commit().await?;
        return Ok(());
    }
    initialize_page(&mut tx, scan_id, &page).await?;
    let tx = insert_resources(tx, scan_id, &page).await?;
    let mut tx = insert_edges(tx, scan_id, &page).await?;
    sqlx::query("UPDATE scope_checkpoints SET last_page=?3,next_token=?4,complete=?5 WHERE scan_id=?1 AND scope_key=?2")
        .bind(scan_id).bind(&page.scope_key).bind(i64::from(page.page_number))
        .bind(&page.next_token).bind(page.next_token.is_none()).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn initialize_page(
    tx: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<(), StoreError> {
    require_running(tx, scan_id).await?;
    insert_scope(tx, scan_id, &page.scope_key, &page.scope).await?;
    validate_page_sequence(tx, scan_id, page).await?;
    let resources =
        i64::try_from(page.resources.len()).map_err(|_| StoreError::Limit("page resources"))?;
    let edges = i64::try_from(page.edges.len()).map_err(|_| StoreError::Limit("page edges"))?;
    sqlx::query("INSERT INTO collection_pages(scan_id,scope_key,page_number,fingerprint,next_token,resource_count,edge_count)
                 VALUES (?1,?2,?3,?4,?5,?6,?7)")
        .bind(scan_id).bind(&page.scope_key).bind(i64::from(page.page_number)).bind(&page.fingerprint)
        .bind(&page.next_token).bind(resources).bind(edges).execute(tx).await?;
    Ok(())
}

pub(super) async fn insert_resources(
    tx: SqlTransaction,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<SqlTransaction, StoreError> {
    stream::iter(page.resources.chunks(INSERT_BATCH_ROWS))
        .map(Ok)
        .try_fold(tx, |mut tx, rows| async move {
            if let [resource] = rows {
                insert_one_resource(&mut tx, scan_id, page, resource).await?;
            } else {
                let mut query = resource_batch(scan_id, page, rows);
                query.build().execute(&mut *tx).await?;
            }
            Ok(tx)
        })
        .await
}

pub(super) async fn insert_one_resource(
    connection: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
    resource: &PreparedResource,
) -> Result<(), StoreError> {
    sqlx::query("INSERT INTO resource_observations(scan_id,scope_key,page_number,resource_id,account_id,
        region,kind,name,source,source_priority,observed_at,resource_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)")
        .bind(scan_id).bind(&page.scope_key).bind(i64::from(page.page_number)).bind(&resource.id)
        .bind(&resource.account_id).bind(&resource.region).bind(&resource.kind).bind(&resource.name)
        .bind(resource.source.as_str()).bind(i64::from(resource.source.priority()))
        .bind(resource.observed_at).bind(&resource.json).execute(connection).await?;
    Ok(())
}

pub(super) fn resource_batch(
    scan_id: &str,
    page: &PreparedPage,
    rows: &[PreparedResource],
) -> sqlx::QueryBuilder<Sqlite> {
    let mut query = sqlx::QueryBuilder::new(
        "INSERT INTO resource_observations(scan_id,scope_key,page_number,resource_id,account_id,
         region,kind,name,source,source_priority,observed_at,resource_json) ",
    );
    query.push_values(rows, |mut values, resource| {
        values
            .push_bind(scan_id)
            .push_bind(&page.scope_key)
            .push_bind(i64::from(page.page_number))
            .push_bind(&resource.id)
            .push_bind(&resource.account_id)
            .push_bind(&resource.region)
            .push_bind(&resource.kind)
            .push_bind(&resource.name)
            .push_bind(resource.source.as_str())
            .push_bind(i64::from(resource.source.priority()))
            .push_bind(resource.observed_at)
            .push_bind(&resource.json);
    });
    query
}

pub(super) async fn insert_edges(
    tx: SqlTransaction,
    scan_id: &str,
    page: &PreparedPage,
) -> Result<SqlTransaction, StoreError> {
    stream::iter(page.edges.chunks(INSERT_BATCH_ROWS))
        .map(Ok)
        .try_fold(tx, |mut tx, rows| async move {
            if let [edge] = rows {
                insert_one_edge(&mut tx, scan_id, page, edge).await?;
            } else {
                let mut query = edge_batch(scan_id, page, rows);
                query.build().execute(&mut *tx).await?;
            }
            Ok(tx)
        })
        .await
}

pub(super) async fn insert_one_edge(
    connection: &mut SqliteConnection,
    scan_id: &str,
    page: &PreparedPage,
    edge: &PreparedEdge,
) -> Result<(), StoreError> {
    sqlx::query(
        "INSERT INTO edge_observations(scan_id,scope_key,page_number,from_id,to_id,relation,
        source,source_priority,observed_at,edge_json) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
    )
    .bind(scan_id)
    .bind(&page.scope_key)
    .bind(i64::from(page.page_number))
    .bind(&edge.from_id)
    .bind(&edge.to_id)
    .bind(&edge.relation)
    .bind(edge.source.as_str())
    .bind(i64::from(edge.source.priority()))
    .bind(edge.observed_at)
    .bind(&edge.json)
    .execute(connection)
    .await?;
    Ok(())
}

pub(super) fn edge_batch(
    scan_id: &str,
    page: &PreparedPage,
    rows: &[PreparedEdge],
) -> sqlx::QueryBuilder<Sqlite> {
    let mut query = sqlx::QueryBuilder::new(
        "INSERT INTO edge_observations(scan_id,scope_key,page_number,from_id,to_id,relation,
         source,source_priority,observed_at,edge_json) ",
    );
    query.push_values(rows, |mut values, edge| {
        values
            .push_bind(scan_id)
            .push_bind(&page.scope_key)
            .push_bind(i64::from(page.page_number))
            .push_bind(&edge.from_id)
            .push_bind(&edge.to_id)
            .push_bind(&edge.relation)
            .push_bind(edge.source.as_str())
            .push_bind(i64::from(edge.source.priority()))
            .push_bind(edge.observed_at)
            .push_bind(&edge.json);
    });
    query
}
