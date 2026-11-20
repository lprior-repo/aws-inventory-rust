use super::{
    Observation, TagCoverage, TagMapping, TagObservation, TestResult, coverage, snapshot, state,
    support,
};

type RoleRow = (String, String, Option<String>, Option<String>);

async fn roles(pool: &sqlx::SqlitePool, id: &str) -> TestResult<Vec<RoleRow>> {
    Ok(
        sqlx::query_as(
            "SELECT id,role,tag_key,value FROM resource_roles WHERE id=?1 ORDER BY role",
        )
        .bind(id)
        .fetch_all(pool)
        .await?,
    )
}

fn role(id: &str, role: &str, key: Option<&str>, value: Option<&str>) -> RoleRow {
    (
        id.to_owned(),
        role.to_owned(),
        key.map(str::to_owned),
        value.map(str::to_owned),
    )
}

/// Observations and tag snapshots for three resources: all tags, a case-variant owner
/// only, and no tags. Plain data: nothing here reaches a service.
fn baseline() -> TestResult<(Vec<Observation>, Vec<TagObservation>)> {
    Ok((
        vec![
            support::observation("one", "a")?,
            support::observation("two", "b")?,
            support::observation("three", "c")?,
        ],
        vec![
            snapshot(
                "one",
                TagCoverage::Complete,
                &[
                    ("owner", "alice"),
                    ("Owner", "bob"),
                    ("team", ""),
                    ("app", "checkout"),
                ],
            )?,
            snapshot("two", TagCoverage::Complete, &[("Owner", "carol")])?,
            snapshot("three", TagCoverage::Complete, &[])?,
        ],
    ))
}

#[tokio::test]
async fn seeded_tags_derive_literal_roles_and_unmapped_roles_are_unassigned() -> TestResult {
    // Given three resources: all tags, a case-variant owner only, and no tags.
    let (directory, store, _) = support::fixture().await?;
    let (observations, tags) = baseline()?;
    state::commit_resources(&store, "baseline", observations, tags, 100).await?;
    let pool = state::pool(&directory).await?;

    // When the mapping lists owner keys in priority order and an empty team list.
    store
        .replace_tag_mapping(&TagMapping::parse(
            "owner = [\"owner\", \"Owner\"]\napplication = [\"app\"]\n",
        )?)
        .await?;

    // Then owner takes the first key, the empty-valued team tag is unassigned,
    // and the resource with no tags has every role unassigned.
    assert_eq!(
        roles(&pool, "one").await?,
        vec![
            role("one", "application", Some("app"), Some("checkout")),
            role("one", "owner", Some("owner"), Some("alice")),
            role("one", "team", None, None),
        ]
    );
    assert_eq!(
        roles(&pool, "two").await?,
        vec![
            role("two", "application", None, None),
            role("two", "owner", Some("Owner"), Some("carol")),
            role("two", "team", None, None),
        ]
    );
    assert_eq!(
        roles(&pool, "three").await?,
        vec![
            role("three", "application", None, None),
            role("three", "owner", None, None),
            role("three", "team", None, None),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn mapping_change_recomputes_roles_without_fetch_and_sweeps_keep_it() -> TestResult {
    // Given observed resources committed once. The fixture data is plain values:
    // no fetch seam is involved, and replace_tag_mapping takes only a store and a mapping.
    let (directory, store, _) = support::fixture().await?;
    let (observations, tags) = baseline()?;
    state::commit_resources(&store, "baseline", observations, tags, 100).await?;
    let pool = state::pool(&directory).await?;
    store
        .replace_tag_mapping(&TagMapping::parse("owner = [\"owner\"]\n")?)
        .await?;
    let image = state::resource_image(&pool).await?;
    let tags_before = state::read_tags(&store, &support::resource("one")?).await?;
    let coverage_before = coverage(&directory).await?;

    // When the mapping changes: owner is now found under team-style keys.
    store
        .replace_tag_mapping(&TagMapping::parse(
            "team = [\"owner\", \"Owner\"]\napplication = [\"app\"]\n",
        )?)
        .await?;

    // Then the derived roles follow the new mapping, with no fetch.
    // The mapping write is store-only: no fetch seam exists to call here.
    assert_eq!(
        roles(&pool, "one").await?,
        vec![
            role("one", "application", Some("app"), Some("checkout")),
            role("one", "owner", None, None),
            role("one", "team", Some("owner"), Some("alice")),
        ]
    );
    assert_eq!(
        roles(&pool, "two").await?,
        vec![
            role("two", "application", None, None),
            role("two", "owner", None, None),
            role("two", "team", Some("Owner"), Some("carol")),
        ]
    );
    // The observed record, tags and coverage are untouched by the mapping change.
    assert_eq!(state::resource_image(&pool).await?, image);
    assert_eq!(
        state::read_tags(&store, &support::resource("one")?).await?,
        tags_before
    );
    assert_eq!(coverage(&directory).await?, coverage_before);

    // When a later sweep changes a tag value, it reflects under the same mapping.
    state::commit_tags(
        &store,
        "tags-2",
        vec![snapshot(
            "one",
            TagCoverage::Complete,
            &[("owner", "alice"), ("app", "payments")],
        )?],
        200,
    )
    .await?;
    assert_eq!(
        roles(&pool, "one").await?,
        vec![
            role("one", "application", Some("app"), Some("payments")),
            role("one", "owner", None, None),
            role("one", "team", Some("owner"), Some("alice")),
        ]
    );

    // When the mapping is emptied, every role is unassigned again.
    store.replace_tag_mapping(&TagMapping::parse("")?).await?;
    assert_eq!(
        roles(&pool, "one").await?,
        vec![
            role("one", "application", None, None),
            role("one", "owner", None, None),
            role("one", "team", None, None),
        ]
    );
    Ok(())
}

#[tokio::test]
async fn whitespace_only_tag_value_is_blank_and_falls_through_to_next_key() -> TestResult {
    // Given one resource whose first owner key is whitespace-only.
    let (directory, store, _) = support::fixture().await?;
    let observations = vec![support::observation("one", "a")?];
    let tags = vec![snapshot(
        "one",
        TagCoverage::Complete,
        &[("owner", " "), ("Owner", "bob")],
    )?];
    state::commit_resources(&store, "baseline", observations, tags, 100).await?;
    let pool = state::pool(&directory).await?;

    // When owner lists the blank key first, then Owner.
    store
        .replace_tag_mapping(&TagMapping::parse("owner = [\"owner\", \"Owner\"]\n")?)
        .await?;

    // Then the blank value is not an owner and the next key supplies it.
    assert_eq!(
        roles(&pool, "one").await?,
        vec![
            role("one", "application", None, None),
            role("one", "owner", Some("Owner"), Some("bob")),
            role("one", "team", None, None),
        ]
    );
    Ok(())
}
