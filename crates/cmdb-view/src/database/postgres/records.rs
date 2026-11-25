use super::*;

pub(super) async fn read_details(
    snapshot: &mut Snapshot,
    rows: &[PgRow],
) -> Result<Vec<Option<PgRow>>, ViewError> {
    let groups = rows.iter().enumerate().try_fold(
        BTreeMap::<&str, Vec<(usize, &PgRow)>>::new(),
        |mut groups, (index, row)| {
            groups
                .entry(row.try_get("kind")?)
                .or_default()
                .push((index, row));
            Ok::<_, ViewError>(groups)
        },
    )?;
    let details: Vec<Option<PgRow>> = std::iter::repeat_with(|| None).take(rows.len()).collect();
    stream::iter(groups.into_iter().map(Ok))
        .try_fold(
            (snapshot, details),
            |(snapshot, mut details), (kind, rows)| async move {
                let payloads = fetch_details(snapshot, kind, &rows).await?;
                payloads.into_iter().try_for_each(|row| {
                    let index = usize::try_from(row.try_get::<i64, _>("row_index")?)
                        .map_err(|_| ViewError::StoredRow)?;
                    let slot = details.get_mut(index).ok_or(ViewError::StoredRow)?;
                    if slot.replace(row).is_some() {
                        return Err(ViewError::StoredRow);
                    }
                    Ok(())
                })?;
                Ok::<_, ViewError>((snapshot, details))
            },
        )
        .await
        .map(|(_, details)| details)
}

async fn fetch_details(
    snapshot: &mut Snapshot,
    kind: &str,
    rows: &[(usize, &PgRow)],
) -> Result<Vec<PgRow>, ViewError> {
    let table = queries::table_for(snapshot, kind).await?;
    let statement = format!(
        "SELECT p.row_index,k.record,k.content_hash FROM inventory.\"{}\" k JOIN unnest($1::text[],$2::text[],$3::text[],$4::bigint[]) AS p(account,region,id,row_index) USING(account,region,id)",
        table.as_str()
    );
    let accounts = rows
        .iter()
        .map(|(_, row)| row.try_get::<&str, _>("account"))
        .collect::<Result<Vec<_>, _>>()?;
    let regions = rows
        .iter()
        .map(|(_, row)| row.try_get::<&str, _>("region"))
        .collect::<Result<Vec<_>, _>>()?;
    let ids = rows
        .iter()
        .map(|(_, row)| row.try_get::<&str, _>("id"))
        .collect::<Result<Vec<_>, _>>()?;
    let indices = rows
        .iter()
        .map(|(index, _)| i64::try_from(*index).map_err(|_| ViewError::StoredRow))
        .collect::<Result<Vec<_>, _>>()?;
    #[cfg(test)]
    probes::query();
    Ok(sqlx::query(sqlx::AssertSqlSafe(statement.as_str()))
        .bind(accounts)
        .bind(regions)
        .bind(ids)
        .bind(indices)
        .fetch_all(&mut *snapshot.tx)
        .await?)
}
