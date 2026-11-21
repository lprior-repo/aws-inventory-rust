#[allow(dead_code, reason = "shared via #[path]; each test target uses a subset of the helpers")]
mod support {
    include!("inventory_store/support.rs");

    pub(super) fn list_catalog(source: i64, mode: i64) -> TestResult<Vec<u8>> {
        let record = Record::from_bytes(catalog_bytes()?)?;
        let Value::Map(mut root) =
            inventory_core::record::ValueRef::Map(record.root()).to_value()?
        else {
            return Err("catalog root is not a map".into());
        };
        let Some(Value::List(services)) = root.get_mut(&Key::Field(CatalogKey::Services.field()?))
        else {
            return Err("catalog services missing".into());
        };
        let Some(Value::Bytes(bytes)) = services.first_mut() else {
            return Err("service missing".into());
        };
        let record = Record::from_bytes(bytes.clone())?;
        let Value::Map(mut service) =
            inventory_core::record::ValueRef::Map(record.root()).to_value()?
        else {
            return Err("service is not a map".into());
        };
        let Some(Value::List(shapes)) = service.get_mut(&Key::Field(CatalogKey::Shapes.field()?))
        else {
            return Err("service shapes missing".into());
        };
        let shape_index = if source == 0 { 1 } else { 0 };
        let Some(Value::Map(shape)) = shapes.get_mut(shape_index) else {
            return Err("source shape missing".into());
        };
        let Some(Value::List(members)) = shape.get_mut(&Key::Field(CatalogKey::Members.field()?))
        else {
            return Err("source members missing".into());
        };
        let member_id = Key::Field(CatalogKey::MemberId.field()?);
        let replacement = catalog_map([
            (CatalogKey::MemberId, Value::Int(2)),
            (CatalogKey::MemberTarget, Value::Int(2)),
            (CatalogKey::MemberRequired, Value::Bool(false)),
        ])?;
        if let Some(member) = members.iter_mut().find(|member|
            matches!(member, Value::Map(fields) if fields.get(&member_id) == Some(&Value::Int(2)))) {
            *member = replacement;
        } else {
            members.push(replacement);
        }
        shapes.push(catalog_map([
            (CatalogKey::Name, Value::Text("References".into())),
            (CatalogKey::Kind, Value::Int(16)),
            (CatalogKey::Element, Value::Int(-3)),
        ])?);
        let offsets = ListRef::from_bytes(&encode_list(shapes)?)?
            .encoded_item_offsets()?
            .into_iter()
            .flat_map(u32::to_be_bytes)
            .collect();
        service.insert(
            Key::Field(CatalogKey::ShapeOffsets.field()?),
            Value::Bytes(offsets),
        );
        *bytes = Record::encode(&Value::Map(service))?.into_bytes();
        let Some(Value::Map(extension)) = root.get_mut(&Key::Field(FieldId::new(60_000)?)) else {
            return Err("runtime extension missing".into());
        };
        let target_path = if mode == 4 {
            Vec::new()
        } else {
            vec![Value::Int(1)]
        };
        extension.insert(
            Key::Field(FieldId::new(4)?),
            Value::List(vec![wire_map([
                (1, Value::Text("AWS::Example::Thing".into())),
                (2, Value::List(vec![Value::Int(2)])),
                (3, Value::Text("AWS::Example::Thing".into())),
                (4, Value::List(target_path)),
                (5, Value::Text("references".into())),
                (6, Value::Int(0)),
                (10, Value::Int(source)),
                (11, Value::Int(0)),
                (12, Value::Int(mode)),
            ])?]),
        );
        let mut root = Value::Map(root);
        seal_catalog(&mut root)?;
        Ok(Record::encode(&root)?.into_bytes())
    }

    pub(super) fn scalar_term_catalog(component: bool) -> TestResult<Vec<u8>> {
        let mut fields = vec![
            (1, Value::Text("AWS::Example::Thing".into())),
            (2, Value::List(vec![Value::Int(2)])),
            (3, Value::Text("AWS::Example::Thing".into())),
            (4, Value::List(vec![])),
            (5, Value::Text("references".into())),
            (6, Value::Int(0)),
            (10, Value::Int(1)),
            (11, Value::Int(0)),
            (12, Value::Int(4)),
        ];
        if component {
            fields.push((
                14,
                Value::List(vec![wire_map([
                    (1, Value::Int(1)),
                    (2, Value::List(vec![Value::Int(3)])),
                    (3, Value::Int(0)),
                    (4, Value::List(vec![Value::Int(1)])),
                    (5, Value::Int(0)),
                ])?]),
            ));
        } else {
            fields.push((
                13,
                wire_map([
                    (1, Value::Int(1)),
                    (2, Value::Int(0)),
                    (3, Value::List(vec![Value::Int(3)])),
                    (4, Value::Text("target-name".into())),
                ])?,
            ));
        }
        typed_relationship_catalog(vec![wire_map(fields)?], false)
    }
}
#[path = "scalar_replay_contract/snapshot.rs"]
mod snapshot;

use inventory_core::{
    catalog::runtime::{RelationTargetMode, RuntimeCatalog},
    inventory::{
        Edge, EdgeSource, Observation, ReferenceTarget, Relation, ResourceId, UnresolvedReference,
    },
    record::{Exclusions, FieldId, Key, Record, Value},
};
use inventory_engine::inventory::{
    BinaryStore, ProjectionEdgeHeader, SchemaRegistry, StoreError, SweepId, WorkSpec,
};
use std::collections::BTreeMap;
use support::*;

fn record(fields: impl IntoIterator<Item = (u32, Value)>) -> TestResult<Record> {
    let fields = fields
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()?;
    Ok(Record::encode(&Value::Map(fields))?)
}

fn source_observation(value: Value) -> TestResult<Observation> {
    Ok(Observation::new(
        resource("source")?,
        record([(1, Value::Text("source-name".into())), (2, value)])?,
        &Exclusions::none(),
        None,
    )?)
}

fn reference(value: &str, mode: RelationTargetMode) -> TestResult<UnresolvedReference> {
    let key = resource("target")?;
    let path = if mode == RelationTargetMode::CanonicalId {
        vec![]
    } else {
        vec![FieldId::new(1)?]
    };
    Ok(UnresolvedReference {
        from: resource("source")?,
        relation: Relation::parse("references")?,
        source: EdgeSource::InferredExact,
        target: ReferenceTarget::typed(
            key.account,
            key.region,
            key.kind,
            path,
            ResourceId::parse(value)?,
            mode,
        )?,
    })
}

async fn fixture(source: i64, mode: i64) -> TestResult<(tempfile::TempDir, BinaryStore)> {
    let directory = tempfile::tempdir()?;
    let bytes = list_catalog(source, mode)?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let registry = SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?;
    let store = BinaryStore::create(&directory.path().join("inventory.sqlite"), registry).await?;
    Ok((directory, store))
}

async fn list_page(
    store: &BinaryStore,
    source: i64,
    values: Value,
) -> TestResult<inventory_engine::inventory::PageCommit> {
    let bindings = if source == 1 {
        record([(2, values.clone())])?
    } else {
        record([])?
    };
    store
        .seed_work(WorkSpec::new(
            SweepId::new("membership".into())?,
            scope("us-east-1")?,
            None,
            bindings.into_bytes(),
        )?)
        .await?;
    let lease = store
        .claim(&SweepId::new("membership".into())?, 100, 20)
        .await?
        .ok_or("lease missing")?;
    let observations = vec![
        source_observation(if source == 0 { values } else { Value::Null })?,
        observation("target", "target-name")?,
        observation("other", "other-name")?,
    ];
    Ok(page(lease, observations, true, None, 101))
}

async fn assert_edges(store: &BinaryStore, expected: &[(&str, EdgeSource)]) -> TestResult {
    let actual = store
        .projection_edges_for(&resource("source")?, None, 10)
        .await?;
    let edges = expected
        .iter()
        .map(|(id, source)| {
            Ok(ProjectionEdgeHeader {
                from: resource("source")?,
                to: resource(id)?,
                relation: Relation::parse("references")?,
                source: source.as_str().to_owned(),
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(actual.edges, edges);
    assert_eq!(actual.next, None);
    Ok(())
}

#[tokio::test]
async fn given_reviewed_text_list_when_exact_members_commit_then_both_edges_survive_reopen()
-> TestResult {
    for source in [0, 1] {
        for (mode, wire_mode, members) in [
            (
                RelationTargetMode::SelectedFields,
                0,
                ["target-name", "other-name"],
            ),
            (RelationTargetMode::CanonicalId, 4, ["target", "other"]),
        ] {
            // Given a checked selected-output or bound-request text-list rule and two real targets.
            let (directory, store) = fixture(source, wire_mode).await?;
            let values = Value::List(
                std::iter::once(Value::Null)
                    .chain(std::iter::once(Value::Text(String::new())))
                    .chain(members.iter().map(|member| Value::Text((*member).into())))
                    .collect(),
            );
            let mut page = list_page(&store, source, values).await?;
            page.unresolved = members
                .iter()
                .map(|value| reference(value, mode))
                .collect::<TestResult<Vec<_>>>()?;
            // When each exact list member is committed through the public persistence API.
            store.commit_page(page).await?;
            // Then persisted authority resolves both members, rather than stringifying or dropping the list.
            let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
            assert_edges(
                &reopened,
                &[
                    ("other", EdgeSource::InferredExact),
                    ("target", EdgeSource::InferredExact),
                ],
            )
            .await?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_reviewed_text_list_when_direct_edge_names_a_member_then_its_exact_pin_survives_reopen()
-> TestResult {
    for source in [0, 1] {
        for (wire_mode, value) in [(0, "target-name"), (4, "target")] {
            // Given a list whose second member, not its first or a stringified value, identifies the target.
            let (directory, store) = fixture(source, wire_mode).await?;
            let mut page = list_page(
                &store,
                source,
                Value::List(vec![
                    Value::Text("unrelated".into()),
                    Value::Text(value.into()),
                ]),
            )
            .await?;
            page.edges.push(Edge {
                from: resource("source")?,
                to: resource("target")?,
                relation: Relation::parse("references")?,
                source: EdgeSource::Model,
            });
            // When an existing supported same-page direct edge is persisted.
            store.commit_page(page).await?;
            // Then target-mode matching selects the member even when target Name differs from its canonical ID.
            let reopened = BinaryStore::open(&directory.path().join("inventory.sqlite")).await?;
            assert_edges(&reopened, &[("target", EdgeSource::Model)]).await?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_reviewed_list_when_member_is_absent_or_a_later_element_is_malformed_then_whole_page_is_unchanged()
-> TestResult {
    for source in [0, 1] {
        for direct in [false, true] {
            for (values, expected) in [
                (
                    vec![],
                    "relationship evidence is not permitted by the checked catalog",
                ),
                (
                    vec![Value::Text("target-name-suffix".into())],
                    "relationship evidence is not permitted by the checked catalog",
                ),
                (
                    vec![Value::Text("target-name".into()), Value::Bool(true)],
                    "relationship source list element is not modeled text",
                ),
                (
                    vec![Value::Text("target-name".into()), Value::Int(7)],
                    "relationship source list element is not modeled text",
                ),
            ] {
                // Given a real checked list source and a snapshot of durable state before the attempted page.
                let (directory, store) = fixture(source, 0).await?;
                let mut page = list_page(&store, source, Value::List(values)).await?;
                let pool = sqlx::sqlite::SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_with(
                        sqlx::sqlite::SqliteConnectOptions::new()
                            .filename(directory.path().join("inventory.sqlite")),
                    )
                    .await?;
                let before = snapshot::read_durable_state(&pool).await?;
                if direct {
                    page.edges.push(Edge {
                        from: resource("source")?,
                        to: resource("target")?,
                        relation: Relation::parse("references")?,
                        source: EdgeSource::Model,
                    });
                } else {
                    page.unresolved.push(reference(
                        "target-name",
                        RelationTargetMode::SelectedFields,
                    )?);
                }
                // When absent-member forgery or malformed trailing elements reach persistence.
                let result = store.commit_page(page).await;
                // Then full-list validation rejects the exact error and no resource, lease or edge mutates.
                assert!(matches!(result, Err(StoreError::Invalid(message)) if message == expected));
                assert_eq!(snapshot::read_durable_state(&pool).await?, before);
                pool.close().await;
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn given_scalar_guard_or_component_when_only_a_list_contains_its_value_then_no_authority_is_minted()
-> TestResult {
    for component in [false, true] {
        for direct in [false, true] {
            // Given an independently checked scalar guard or compound component, not a primary membership term.
            let directory = tempfile::tempdir()?;
            let bytes = scalar_term_catalog(component)?;
            let catalog = RuntimeCatalog::from_bytes(&bytes)?;
            let store = BinaryStore::create(
                &directory.path().join("inventory.sqlite"),
                SchemaRegistry::new(&bytes, catalog.schemas().to_vec())?,
            )
            .await?;
            let bindings = record([
                (2, Value::Text("target".into())),
                (3, Value::List(vec![Value::Text("target-name".into())])),
            ])?;
            store
                .seed_work(WorkSpec::new(
                    SweepId::new("scalar-only".into())?,
                    scope("us-east-1")?,
                    None,
                    bindings.into_bytes(),
                )?)
                .await?;
            let lease = store
                .claim(&SweepId::new("scalar-only".into())?, 100, 20)
                .await?
                .ok_or("lease missing")?;
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
            if direct {
                page.edges.push(Edge {
                    from: resource("source")?,
                    to: resource("target")?,
                    relation: Relation::parse("references")?,
                    source: EdgeSource::Model,
                });
            } else {
                let mut reference = reference("target", RelationTargetMode::CanonicalId)?;
                if component {
                    reference.target = reference.target.with_components(vec![
                        inventory_core::inventory::ReferenceComponent {
                            mode: RelationTargetMode::SelectedFields,
                            path: vec![FieldId::new(1)?].into_boxed_slice(),
                            value: ResourceId::parse("target-name")?,
                        },
                    ])?;
                }
                page.unresolved.push(reference);
            }
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(
                    sqlx::sqlite::SqliteConnectOptions::new()
                        .filename(directory.path().join("inventory.sqlite")),
                )
                .await?;
            let before = snapshot::read_durable_state(&pool).await?;
            // When matching text occurs only inside a list at a scalar-only selector.
            let result = store.commit_page(page).await;
            // Then primary membership support cannot authorize guards or compound terms and writes remain atomic.
            assert!(matches!(
                result,
                Err(StoreError::Invalid(
                    "relationship evidence is not permitted by the checked catalog"
                ))
            ));
            assert_eq!(snapshot::read_durable_state(&pool).await?, before);
            pool.close().await;
        }
    }
    Ok(())
}
