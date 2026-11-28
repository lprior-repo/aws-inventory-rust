use super::*;
pub(super) struct Summary {
    pub(super) code: i32,
    pub(super) writes: u64,
    pub(super) rows: u64,
    pub(super) complete: u64,
    pub(super) partial: u64,
    pub(super) failed: u64,
    pub(super) pending: u64,
    pub(super) requests: u64,
    pub(super) pages: u64,
    pub(super) claims: u64,
}

pub(super) fn assert_summary(output: &Output, expected: Summary) -> Result<()> {
    assert_eq!(
        output.status.code(),
        Some(expected.code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    super::stdout::assert(std::str::from_utf8(&output.stdout)?, &expected)?;
    Ok(())
}

pub(super) async fn assert_resources(path: &Path, ids: &[&str]) -> Result<()> {
    let target = path.to_str().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "non-UTF-8 database path")
    })?;
    let database = Database::open(target).await?;
    let page = database
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 100,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.total_matching, u64::try_from(ids.len())?);
    assert_eq!(page.next, None);
    let actual = page
        .resources
        .iter()
        .map(|resource| {
            let scalar = match scalar_item(resource.record.root())? {
                Some(ValueRef::Text(value)) => value,
                _ => anyhow::bail!("persisted scalar item was not a text value"),
            };
            Ok((
                resource.key.account.as_str(),
                resource.key.region.as_str(),
                resource.key.kind.as_str(),
                resource.key.id.as_str(),
                scalar,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let expected = ids
        .iter()
        .map(|id| (ACCOUNT, REGION, KIND, *id, *id))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    Ok(())
}
pub(super) async fn assert_nested_resources(path: &Path, owner: FieldId) -> Result<()> {
    let target = path.to_str().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "non-UTF-8 database path")
    })?;
    let database = Database::open(target).await?;
    let page = database
        .page(
            &ResourceFilter::default(),
            PageRequest {
                limit: 100,
                after: None,
            },
        )
        .await?;
    assert_eq!(page.total_matching, 3);
    let actual = page
        .resources
        .iter()
        .map(|resource| {
            let owner = match resource.record.root().get_field(owner)? {
                Some(ValueRef::Text(owner)) => owner,
                _ => anyhow::bail!("nested owner was not retained on the instance"),
            };
            Ok((
                resource.key.account.as_str(),
                resource.key.region.as_str(),
                resource.key.kind.as_str(),
                resource.key.id.as_str(),
                owner,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(
        actual,
        vec![
            (ACCOUNT, REGION, "AWS::ApiOnly::NestedEc2Instance", "i-owned-one", ACCOUNT),
            (ACCOUNT, REGION, "AWS::ApiOnly::NestedEc2Instance", "i-owned-three", ACCOUNT),
            (ACCOUNT, REGION, "AWS::ApiOnly::NestedEc2Instance", "i-owned-two", ACCOUNT),
        ]
    );
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .pragma("query_only", "ON");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    let tags = sqlx::query_as::<_, (String, String, String)>(
        "SELECT id,tag_key,tag_value FROM tags ORDER BY id,tag_key",
    )
    .fetch_all(&pool)
    .await?;
    assert_eq!(
        tags,
        vec![
            ("i-owned-one".to_owned(), "Name".to_owned(), "owned-one".to_owned()),
            ("i-owned-three".to_owned(), "Name".to_owned(), "owned-three".to_owned()),
            ("i-owned-two".to_owned(), "Name".to_owned(), "owned-two".to_owned()),
        ]
    );
    Ok(())
}

pub(super) async fn expire_leases(path: &Path) -> Result<()> {
    let options = SqliteConnectOptions::new().filename(path);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    sqlx::query("UPDATE work_queue SET lease_until=0 WHERE state=1")
        .execute(&pool)
        .await?;
    Ok(())
}

pub(super) async fn assert_queue(
    path: &Path,
    active: i64,
    outcomes: &[(&str, i64)],
    digests: i64,
) -> Result<()> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .pragma("query_only", "ON");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;
    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_queue WHERE state IN(0,1)")
        .fetch_one(&pool)
        .await?;
    let cursor_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM work_queue WHERE state IN(0,1) AND token IS NOT NULL",
    )
    .fetch_one(&pool)
    .await?;
    let stored = sqlx::query_as::<_, (String, i64)>(
        "SELECT outcome,COUNT(*) FROM scope_outcomes GROUP BY outcome ORDER BY outcome",
    )
    .fetch_all(&pool)
    .await?;
    let token_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM work_tokens")
        .fetch_one(&pool)
        .await?;
    pool.close().await;
    assert_eq!(
        (pending, cursor_count, token_count),
        (active, active, digests)
    );
    assert_eq!(
        stored,
        outcomes
            .iter()
            .map(|(outcome, count)| ((*outcome).to_owned(), *count))
            .collect::<Vec<_>>()
    );
    Ok(())
}
