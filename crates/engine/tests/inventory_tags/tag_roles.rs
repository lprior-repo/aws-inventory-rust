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

/// Stands in for the AWS fetch. Every call is an API call, so the count proves
/// how many times a sweep reached the service.
#[derive(Default)]
struct CountingFake {
    calls: usize,
}

impl CountingFake {
    fn fetch(&mut self) -> TestResult<(Vec<Observation>, Vec<TagObservation>)> {
        self.calls = self.calls.saturating_add(1);
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
}

#[tokio::test]
async fn seeded_tags_derive_literal_roles_and_unmapped_roles_are_unassigned() -> TestResult {
    // Given three resources: all tags, a case-variant owner only, and no tags.
    let (directory, store, _) = support::fixture().await?;
    let mut fake = CountingFake::default();
    let (observations, tags) = fake.fetch()?;
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
async fn mapping_change_recomputes_roles_with_zero_api_calls_and_sweeps_keep_it() -> TestResult {
    // Given observed resources committed by one fetch.
    let (directory, store, _) = support::fixture().await?;
    let mut fake = CountingFake::default();
    let (observations, tags) = fake.fetch()?;
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
    assert_eq!(fake.calls, 1);
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
