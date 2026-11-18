#[path = "scalar_replay_contract/snapshot.rs"]
mod snapshot;
#[path = "inventory_store/support.rs"]
mod support;

use inventory_core::{
    catalog::runtime::{RelationTargetMode, RuntimeCatalog},
    inventory::{
        Edge, EdgeSource, Observation, ReferenceComponent, ReferenceTarget, Relation, ResourceId,
        ScopeOutcome, UnresolvedReference,
    },
    record::{FieldId, Key, Value},
};
use inventory_engine::inventory::{
    BinaryStore, ProjectionEdgeHeader, SchemaRegistry, StoreError, SweepId,
};
use std::collections::BTreeMap;
use support::*;

fn map(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Value> {
    Ok(Value::Map(
        fields
            .into_iter()
            .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
            .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?,
    ))
}

fn path(fields: &[u32]) -> Value {
    Value::List(
        fields
            .iter()
            .map(|field| Value::Int(i64::from(*field)))
            .collect(),
    )
}

fn rule(
    mode: i64,
    target_path: &[u32],
    components: Vec<Value>,
    guard: Option<Value>,
) -> TestResult<Value> {
    let mut fields = vec![
        (1, Value::Text("AWS::Example::Thing".into())),
        (2, path(&[2])),
        (3, Value::Text("AWS::Example::Thing".into())),
        (4, path(target_path)),
        (5, Value::Text("references".into())),
        (6, Value::Int(0)),
        (10, Value::Int(1)),
        (11, Value::Int(0)),
        (12, Value::Int(mode)),
        (14, Value::List(components)),
    ];
    if let Some(guard) = guard {
        fields.push((13, guard));
    }
    map(fields)
}

fn bound_component() -> TestResult<Value> {
    map([
        (1, Value::Int(1)),
        (2, path(&[3])),
        (3, Value::Int(0)),
        (4, path(&[3])),
        (5, Value::Int(2)),
    ])
}

async fn fixture_for(rule: Value, dual: bool) -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let bytes = typed_relationship_catalog(vec![rule], dual)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store))
}

fn reference(
    mode: RelationTargetMode,
    target_path: &[u32],
    value: &str,
    namespace: Option<&str>,
) -> TestResult<UnresolvedReference> {
    let key = resource("target")?;
    let target = ReferenceTarget::typed(
        key.account,
        key.region,
        key.kind,
        target_path
            .iter()
            .map(|field| FieldId::new(*field))
            .collect::<Result<Vec<_>, _>>()?,
        ResourceId::parse(value)?,
        mode,
    )?;
    let components = namespace
        .map(|namespace| {
            Ok(ReferenceComponent {
                mode: RelationTargetMode::BoundContextFields,
                path: vec![FieldId::new(3)?].into_boxed_slice(),
                value: ResourceId::parse(namespace)?,
            })
        })
        .into_iter()
        .collect::<TestResult<Vec<_>>>()?;
    Ok(UnresolvedReference {
        from: resource("source")?,
        relation: Relation::parse("references")?,
        source: EdgeSource::InferredExact,
        target: target.with_components(components)?,
    })
}

async fn commit_partial_observation(
    store: &BinaryStore,
    sweep: &str,
    at: i64,
    request: Option<&str>,
    namespace: Option<&str>,
    observations: Vec<Observation>,
    references: Vec<UnresolvedReference>,
    direct: bool,
) -> TestResult {
    store
        .seed_work(relationship_work(sweep, "us-east-1", request, namespace)?)
        .await?;
    let lease = store
        .claim(&SweepId::new(sweep.to_owned())?, at, 20)
        .await?
        .ok_or("lease missing")?;
    // These are deliberately per-resource observations, not exhaustive root listings.
    // Commit a real continuation page, then explicitly settle its scope as Partial.
    let mut page = page(lease, observations, false, Some(vec![1]), at + 1);
    page.unresolved = references;
    if direct {
        page.edges.push(Edge {
            from: resource("source")?,
            to: resource("target")?,
            relation: Relation::parse("references")?,
            source: EdgeSource::Model,
        });
    }
    store.commit_page(page).await?;
    let sweep_id = SweepId::new(sweep.to_owned())?;
    let continuation = store
        .claim(&sweep_id, at + 2, 20)
        .await?
        .ok_or("continuation lease missing")?;
    store
        .finish_scope(continuation, ScopeOutcome::Partial, at + 3)
        .await?;
    assert_eq!(store.sweep_summary(&sweep_id).await?.partial_scopes, 1);
    Ok(())
}

async fn assert_edges(store: &BinaryStore, ids: &[&str], source: EdgeSource) -> TestResult {
    let actual = store
        .projection_edges_for(&resource("source")?, None, 10)
        .await?;
    let expected = ids
        .iter()
        .map(|id| {
            Ok(ProjectionEdgeHeader {
                from: resource("source")?,
                to: resource(id)?,
                relation: Relation::parse("references")?,
                source: source.as_str().to_owned(),
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(actual.edges, expected);
    assert_eq!(actual.next, None);
    Ok(())
}

async fn assert_reopened_bound_reference(
    directory: &tempfile::TempDir,
    ids: &[&str],
    source: EdgeSource,
    status: &str,
) -> TestResult {
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_edges(&reopened, ids, source).await?;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    let statuses: Vec<String> = sqlx::query_scalar(
        "SELECT resolution FROM unresolved_refs WHERE from_id=?1 AND relation=?2 AND source=?3",
    )
    .bind("source")
    .bind("references")
    .bind(source.as_str())
    .fetch_all(&pool)
    .await?;
    assert_eq!(statuses, vec![status]);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_canonical_id_reference_when_target_arrives_in_either_order_then_names_do_not_affect_reopen_or_replay()
-> TestResult {
    for source_first in [true, false] {
        // Given a checked canonical-ID rule, not a selected-root or Name rule.
        let (directory, store) = fixture_for(rule(4, &[], vec![], None)?, false).await?;
        if !source_first {
            commit_partial_observation(
                &store,
                "target-first",
                10,
                None,
                None,
                vec![observation("target", "unrelated-name")?],
                vec![],
                false,
            )
            .await?;
        }
        // When the reference is committed before or after its real canonical target.
        commit_partial_observation(
            &store,
            "source",
            100,
            Some("target"),
            None,
            vec![observation("source", "source-name")?],
            vec![reference(
                RelationTargetMode::CanonicalId,
                &[],
                "target",
                None,
            )?],
            false,
        )
        .await?;
        assert_edges(
            &store,
            if source_first { &[] } else { &["target"] },
            EdgeSource::InferredExact,
        )
        .await?;
        let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        commit_partial_observation(
            &reopened,
            "target-late-or-renamed",
            200,
            None,
            None,
            vec![observation("target", "still-not-its-id")?],
            vec![],
            false,
        )
        .await?;
        // Then exact identity survives SQLite reopen and an identical source replay.
        assert_edges(&reopened, &["target"], EdgeSource::InferredExact).await?;
        commit_partial_observation(
            &reopened,
            "source-replay",
            300,
            Some("target"),
            None,
            vec![observation("source", "source-name")?],
            vec![reference(
                RelationTargetMode::CanonicalId,
                &[],
                "target",
                None,
            )?],
            false,
        )
        .await?;
        assert_edges(&reopened, &["target"], EdgeSource::InferredExact).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_bound_context_target_when_only_output_name_matches_then_only_the_persisted_checked_request_resolves()
-> TestResult {
    // Given output rows whose names cannot serve as validated request context.
    let (directory, store) = fixture_for(rule(2, &[3], vec![], None)?, false).await?;
    commit_partial_observation(
        &store,
        "source",
        100,
        Some("namespace-x"),
        Some("source-context"),
        vec![observation("source", "source-name")?],
        vec![reference(
            RelationTargetMode::BoundContextFields,
            &[3],
            "namespace-x",
            None,
        )?],
        false,
    )
    .await?;
    commit_partial_observation(
        &store,
        "wrong-context",
        200,
        None,
        Some("namespace-y"),
        vec![observation("decoy", "namespace-x")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&store, &[], EdgeSource::InferredExact).await?;
    // When the real target arrives under the matching checked request and SQLite is reopened.
    commit_partial_observation(
        &store,
        "right-context",
        300,
        None,
        Some("namespace-x"),
        vec![observation("target", "unrelated-name")?],
        vec![],
        false,
    )
    .await?;
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    // Then output data never substitutes for request provenance.
    assert_edges(&reopened, &["target"], EdgeSource::InferredExact).await?;
    Ok(())
}

#[tokio::test]
async fn given_bound_primary_reference_when_distinct_targets_match_then_ambiguity_retracts_until_one_witness_retires()
-> TestResult {
    // Given an unprojected request path and output names that differ from its value.
    let (directory, store) = fixture_for(rule(2, &[3], vec![], None)?, false).await?;
    commit_partial_observation(
        &store,
        "source",
        100,
        Some("namespace-x"),
        Some("source-context"),
        vec![observation("source", "source-name")?],
        vec![reference(
            RelationTargetMode::BoundContextFields,
            &[3],
            "namespace-x",
            None,
        )?],
        false,
    )
    .await?;
    commit_partial_observation(
        &store,
        "unique",
        200,
        None,
        Some("namespace-x"),
        vec![observation("target", "unrelated-name")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&store, &["target"], EdgeSource::InferredExact).await?;
    assert_reopened_bound_reference(
        &directory,
        &["target"],
        EdgeSource::InferredExact,
        "resolved",
    )
    .await?;
    // When another resource has the same checked context, the formerly unique edge disappears.
    commit_partial_observation(
        &store,
        "ambiguous",
        300,
        None,
        Some("namespace-x"),
        vec![observation("second", "another-name")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&store, &[], EdgeSource::InferredExact).await?;
    assert_reopened_bound_reference(&directory, &[], EdgeSource::InferredExact, "ambiguous")
        .await?;
    // Then one complete miss retains ambiguity; only the second retires its witness.
    for (sweep, source_sweep, source_at, at, second_survives) in [
        ("first-miss", "source-before-first-miss", 390, 400, true),
        ("second-miss", "source-before-second-miss", 490, 500, false),
    ] {
        let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        commit_partial_observation(
            &reopened,
            source_sweep,
            source_at,
            Some("namespace-x"),
            Some("source-context"),
            vec![observation("source", "source-name")?],
            vec![reference(
                RelationTargetMode::BoundContextFields,
                &[3],
                "namespace-x",
                None,
            )?],
            false,
        )
        .await?;
        reopened
            .seed_work(relationship_work(
                sweep,
                "us-east-1",
                None,
                Some("namespace-x"),
            )?)
            .await?;
        let sweep_id = SweepId::new(sweep.to_owned())?;
        let lease = reopened
            .claim(&sweep_id, at, 20)
            .await?
            .ok_or("complete listing lease missing")?;
        reopened
            .commit_page(page(
                lease,
                vec![observation("target", "unrelated-name")?],
                true,
                None,
                at + 1,
            ))
            .await?;
        assert_eq!(reopened.sweep_summary(&sweep_id).await?.complete_scopes, 1);
        assert!(
            reopened
                .projection_resource(&resource("source")?)
                .await?
                .is_some()
        );
        assert!(
            reopened
                .projection_resource(&resource("target")?)
                .await?
                .is_some()
        );
        assert_eq!(
            reopened
                .projection_resource(&resource("second")?)
                .await?
                .is_some(),
            second_survives
        );
        let ids: &[&str] = if second_survives { &[] } else { &["target"] };
        let status = if second_survives {
            "ambiguous"
        } else {
            "resolved"
        };
        assert_edges(&reopened, ids, EdgeSource::InferredExact).await?;
        assert_reopened_bound_reference(&directory, ids, EdgeSource::InferredExact, status).await?;
    }
    Ok(())
}

#[tokio::test]
async fn given_ambiguous_bound_primary_and_explicit_pin_when_candidates_refresh_then_only_the_pin_proves_its_edge()
-> TestResult {
    // Given the same bound tuple has both wildcard and exact-identity authority.
    let (directory, store) = fixture_for(rule(2, &[3], vec![], None)?, false).await?;
    let wildcard = || -> TestResult<_> {
        let mut reference = reference(
            RelationTargetMode::BoundContextFields,
            &[3],
            "namespace-x",
            None,
        )?;
        reference.source = EdgeSource::Model;
        Ok(reference)
    };
    commit_partial_observation(
        &store,
        "pin-and-wildcard",
        100,
        Some("namespace-x"),
        Some("namespace-x"),
        vec![
            observation("source", "source-name")?,
            observation("target", "unrelated-name")?,
        ],
        vec![wildcard()?],
        true,
    )
    .await?;
    assert_edges(&store, &["target"], EdgeSource::Model).await?;
    assert_reopened_bound_reference(&directory, &["target"], EdgeSource::Model, "resolved").await?;
    // When another distinct matching target arrives, ambiguity cannot erase or redirect the pin.
    commit_partial_observation(
        &store,
        "another-match",
        200,
        None,
        Some("namespace-x"),
        vec![observation("second", "another-name")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&store, &["target"], EdgeSource::Model).await?;
    assert_reopened_bound_reference(&directory, &["target"], EdgeSource::Model, "resolved").await?;
    // Then replacing the source facet without its direct proof leaves only ambiguity.
    commit_partial_observation(
        &store,
        "remove-pin",
        300,
        Some("namespace-x"),
        Some("namespace-x"),
        vec![observation("source", "source-name")?],
        vec![wildcard()?],
        false,
    )
    .await?;
    assert_edges(&store, &[], EdgeSource::Model).await?;
    assert_reopened_bound_reference(&directory, &[], EdgeSource::Model, "ambiguous").await?;
    Ok(())
}

#[tokio::test]
async fn given_compound_selected_and_bound_terms_when_targets_become_ambiguous_or_change_then_all_terms_control_the_edge()
-> TestResult {
    // Given one conjunct over selected Name and another over the same target's request.
    let (directory, store) =
        fixture_for(rule(0, &[1], vec![bound_component()?], None)?, false).await?;
    commit_partial_observation(
        &store,
        "source",
        100,
        Some("shared-name"),
        Some("namespace-x"),
        vec![observation("source", "source-name")?],
        vec![reference(
            RelationTargetMode::SelectedFields,
            &[1],
            "shared-name",
            Some("namespace-x"),
        )?],
        false,
    )
    .await?;
    commit_partial_observation(
        &store,
        "wrong-namespace",
        200,
        None,
        Some("namespace-y"),
        vec![observation("decoy", "shared-name")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&store, &[], EdgeSource::InferredExact).await?;
    commit_partial_observation(
        &store,
        "unique",
        300,
        None,
        Some("namespace-x"),
        vec![observation("target", "shared-name")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&store, &["target"], EdgeSource::InferredExact).await?;
    // When a second complete match appears, then ambiguity retracts rather than guesses.
    commit_partial_observation(
        &store,
        "ambiguous",
        400,
        None,
        Some("namespace-x"),
        vec![observation("second", "shared-name")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&store, &[], EdgeSource::InferredExact).await?;
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    commit_partial_observation(
        &reopened,
        "unique-again",
        500,
        None,
        Some("namespace-x"),
        vec![observation("second", "different")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&reopened, &["target"], EdgeSource::InferredExact).await?;
    commit_partial_observation(
        &reopened,
        "last-conjunct-lost",
        600,
        None,
        Some("namespace-x"),
        vec![observation("target", "different")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&reopened, &[], EdgeSource::InferredExact).await?;
    Ok(())
}

#[tokio::test]
async fn given_bound_components_from_different_target_facets_when_no_single_request_proves_both_then_no_cartesian_edge_exists()
-> TestResult {
    let left = map([
        (1, Value::Int(0)),
        (2, path(&[1])),
        (3, Value::Int(0)),
        (4, path(&[2])),
        (5, Value::Int(2)),
    ])?;
    // Given the target has two real facets: (left, wrong) and (wrong, right).
    let (directory, store) =
        fixture_for(rule(4, &[], vec![left, bound_component()?], None)?, false).await?;
    commit_partial_observation(
        &store,
        "facet-left",
        100,
        Some("left"),
        Some("wrong"),
        vec![observation("target", "target-name")?],
        vec![],
        false,
    )
    .await?;
    commit_partial_observation(
        &store,
        "facet-right",
        200,
        Some("wrong"),
        Some("right"),
        vec![observation("target", "target-name")?],
        vec![],
        false,
    )
    .await?;
    // When a canonical target requires both bound terms on one checked facet.
    let mut reference = reference(RelationTargetMode::CanonicalId, &[], "target", None)?;
    reference.target = reference.target.with_components(vec![
        ReferenceComponent {
            mode: RelationTargetMode::BoundContextFields,
            path: vec![FieldId::new(2)?].into_boxed_slice(),
            value: ResourceId::parse("left")?,
        },
        ReferenceComponent {
            mode: RelationTargetMode::BoundContextFields,
            path: vec![FieldId::new(3)?].into_boxed_slice(),
            value: ResourceId::parse("right")?,
        },
    ])?;
    commit_partial_observation(
        &store,
        "source",
        300,
        Some("target"),
        Some("right"),
        vec![observation("source", "left")?],
        vec![reference],
        false,
    )
    .await?;
    // Then neither target facet alone proves the conjunction, even after reopen.
    assert_edges(&store, &[], EdgeSource::InferredExact).await?;
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_edges(&reopened, &[], EdgeSource::InferredExact).await?;
    // A new request witness must refresh matches even when output bytes are unchanged.
    commit_partial_observation(
        &reopened,
        "complete-facet",
        400,
        Some("left"),
        Some("right"),
        vec![observation("target", "target-name")?],
        vec![],
        false,
    )
    .await?;
    assert_edges(&reopened, &["target"], EdgeSource::InferredExact).await?;
    Ok(())
}

#[tokio::test]
async fn given_checked_canonical_rule_when_a_selected_field_mode_is_forged_then_the_whole_page_is_unchanged()
-> TestResult {
    // Given a real checked catalog and a claimed work lease.
    let (directory, store) = fixture_for(rule(4, &[], vec![], None)?, false).await?;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    store
        .seed_work(relationship_work(
            "forged",
            "us-east-1",
            Some("target"),
            None,
        )?)
        .await?;
    let lease = store
        .claim(&SweepId::new("forged".to_owned())?, 100, 20)
        .await?
        .ok_or("lease missing")?;
    let before = snapshot::read_durable_state(&pool).await?;
    let mut page = page(
        lease,
        vec![
            observation("source", "source-name")?,
            observation("target", "target")?,
        ],
        true,
        None,
        101,
    );
    page.unresolved.push(reference(
        RelationTargetMode::SelectedFields,
        &[1],
        "target",
        None,
    )?);
    // When tuple values match but typed target authority does not.
    let result = store.commit_page(page).await;
    // Then neither resources, work completion, evidence nor projection may change.
    assert!(matches!(
        result,
        Err(StoreError::Invalid(
            "relationship evidence is not permitted by the checked catalog"
        ))
    ));
    assert_eq!(snapshot::read_durable_state(&pool).await?, before);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_canonical_rule_when_a_same_page_direct_edge_is_committed_then_its_pin_survives_reopen()
-> TestResult {
    // Given a direct model edge whose target Name is deliberately not its ID.
    let (directory, store) = fixture_for(rule(4, &[], vec![], None)?, false).await?;
    // When the exact canonical target and source are committed atomically.
    commit_partial_observation(
        &store,
        "direct",
        100,
        Some("target"),
        None,
        vec![
            observation("source", "source-name")?,
            observation("target", "different-name")?,
        ],
        vec![],
        true,
    )
    .await?;
    // Then the direct pin is encoded, decoded, and projected as model evidence.
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    assert_edges(&reopened, &["target"], EdgeSource::Model).await?;
    Ok(())
}

#[tokio::test]
async fn given_guarded_compound_rule_when_guard_or_component_is_forged_then_no_partial_page_is_committed()
-> TestResult {
    for (binding, component, accepted) in [
        ("enabled", "enabled", true),
        ("disabled", "disabled", false),
        ("enabled", "forged", false),
    ] {
        // Given a checked request guard and a second, independently typed target conjunct.
        let guard = map([
            (1, Value::Int(1)),
            (2, Value::Int(0)),
            (3, path(&[3])),
            (4, Value::Text("enabled".into())),
        ])?;
        let (directory, store) =
            fixture_for(rule(4, &[], vec![bound_component()?], Some(guard))?, false).await?;
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(directory.path().join("inventory.sqlite")),
            )
            .await?;
        store
            .seed_work(relationship_work(
                "guarded",
                "us-east-1",
                Some("target"),
                Some(binding),
            )?)
            .await?;
        let lease = store
            .claim(&SweepId::new("guarded".to_owned())?, 100, 20)
            .await?
            .ok_or("lease missing")?;
        let before = snapshot::read_durable_state(&pool).await?;
        let mut page = page(
            lease,
            vec![
                observation("source", "source-name")?,
                observation("target", "target-name")?,
            ],
            true,
            None,
            101,
        );
        page.unresolved.push(reference(
            RelationTargetMode::CanonicalId,
            &[],
            "target",
            Some(component),
        )?);
        // When a correct reference, a false guard, or a forged component reaches the public commit API.
        let result = store.commit_page(page).await;
        // Then only the complete, guarded conjunction can authorize durable evidence.
        if accepted {
            result?;
            let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
            assert_edges(&reopened, &["target"], EdgeSource::InferredExact).await?;
        } else {
            assert!(matches!(
                result,
                Err(StoreError::Invalid(
                    "relationship evidence is not permitted by the checked catalog"
                ))
            ));
            assert_eq!(snapshot::read_durable_state(&pool).await?, before);
        }
        pool.close().await;
    }
    Ok(())
}

#[tokio::test]
async fn given_v3_facets_without_request_column_when_opened_then_legacy_selected_evidence_is_preserved_without_inventing_context()
-> TestResult {
    // Given existing selected-field evidence using the unchanged V3 path/proof domain.
    let (directory, store) = fixture_for(rule(0, &[1], vec![], None)?, false).await?;
    commit_partial_observation(
        &store,
        "legacy-selected",
        100,
        Some("target-name"),
        None,
        vec![
            observation("source", "source-name")?,
            observation("target", "target-name")?,
        ],
        vec![reference(
            RelationTargetMode::SelectedFields,
            &[1],
            "target-name",
            None,
        )?],
        false,
    )
    .await?;
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(directory.path().join("inventory.sqlite")),
        )
        .await?;
    drop(store);
    sqlx::query("ALTER TABLE resource_facets DROP COLUMN bound_request")
        .execute(&pool)
        .await?;
    // When additive migration opens the older facet layout.
    let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
    // Then existing resources and selected references survive, but old requests remain unknown.
    assert_edges(&reopened, &["target"], EdgeSource::InferredExact).await?;
    let contexts: i64 =
        sqlx::query_scalar("SELECT count(*) FROM resource_facets WHERE bound_request IS NOT NULL")
            .fetch_one(&pool)
            .await?;
    assert_eq!(contexts, 0);
    let paths: Vec<Vec<u8>> = sqlx::query_scalar("SELECT target_path FROM unresolved_refs")
        .fetch_all(&pool)
        .await?;
    assert_eq!(paths, vec![1_u32.to_be_bytes().to_vec()]);
    pool.close().await;
    Ok(())
}

#[tokio::test]
async fn given_compound_reference_when_two_complete_sweeps_miss_only_target_then_source_survives_retirement_and_rearrival_restores_edge()
-> TestResult {
    // Given a selected-field AND checked-request relationship under a real root listing.
    let (directory, store) =
        fixture_for(rule(0, &[1], vec![bound_component()?], None)?, false).await?;
    for (sweep, at, includes_target, target_survives) in [
        ("full-listing", 100, true, true),
        ("first-complete-miss", 200, false, true),
        ("second-complete-miss", 300, false, false),
        ("target-reappears", 400, true, true),
    ] {
        store
            .seed_work(relationship_work(
                sweep,
                "us-east-1",
                Some("shared-name"),
                Some("namespace-x"),
            )?)
            .await?;
        let sweep_id = SweepId::new(sweep.to_owned())?;
        let lease = store
            .claim(&sweep_id, at, 20)
            .await?
            .ok_or("complete listing lease missing")?;
        // Every complete listing retains the source and its exact typed reference;
        // only the target's absence is certified, never the source's.
        let mut observations = vec![observation("source", "source-name")?];
        if includes_target {
            observations.push(observation("target", "shared-name")?);
        }
        let mut complete = page(lease, observations, true, None, at + 1);
        complete.unresolved.push(reference(
            RelationTargetMode::SelectedFields,
            &[1],
            "shared-name",
            Some("namespace-x"),
        )?);
        store.commit_page(complete).await?;
        assert_eq!(store.sweep_summary(&sweep_id).await?.complete_scopes, 1);
        // Then one miss keeps the target, two retire it, and re-arrival rematches
        // both typed conjuncts, including after reopening the persisted store.
        let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
        assert!(
            reopened
                .projection_resource(&resource("source")?)
                .await?
                .is_some(),
            "{sweep}: source must remain live"
        );
        assert_eq!(
            reopened
                .projection_resource(&resource("target")?)
                .await?
                .is_some(),
            target_survives,
            "{sweep}"
        );
        assert_edges(
            &reopened,
            if target_survives { &["target"] } else { &[] },
            EdgeSource::InferredExact,
        )
        .await?;
    }
    Ok(())
}
