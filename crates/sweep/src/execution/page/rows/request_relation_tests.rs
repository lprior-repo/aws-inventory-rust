use super::*;
use crate::execution::page::consumer_index::ConsumerIndex;
use crate::{
    execution::{
        page::child::{Context, policy_dfgk_tests::project_children},
        routing,
    },
    input,
    record::policy_dfgk_tests::fixture,
};
use inventory_core::catalog::runtime::RelationshipPlan;
use inventory_core::{
    inventory::{AccountId, OperationId, ScopeKey, ServiceId},
    record::Value,
};
use inventory_engine::inventory::{
    BinaryStore, PageCommit, SchemaRegistry, SweepId, WorkLease, WorkSpec,
};
use serde_json::json;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn ga_request_parent_relations_reject_hostile_scope_and_type_without_selected_parent_fields()
-> TestResult {
    let catalog = fixture::catalog()?;
    let consumers = ConsumerIndex::build(&catalog)?;
    let account = AccountId::parse("123456789012")?;
    let home = RegionId::parse("eu-west-1")?;
    let sweep = SweepId::new("request-origin-hostile-scope".to_owned())?;
    for (root_operation, child_operation) in [
        ("ListAccelerators", "ListListeners"),
        (
            "ListCustomRoutingAccelerators",
            "ListCustomRoutingListeners",
        ),
    ] {
        // Given real compiled discovery, a durable root lease and actual bound request.
        let directory = tempfile::tempdir()?;
        let registry = SchemaRegistry::new(catalog.raw_bytes(), catalog.schemas().to_vec())?;
        let store = BinaryStore::create(&directory.path().join("inventory.db"), registry).await?;
        let parent = fixture::plan(&catalog, "global-accelerator", root_operation)?;
        let child = fixture::plan(&catalog, "global-accelerator", child_operation)?;
        let parent_arn = "arn:aws:globalaccelerator::123456789012:accelerator/parent";
        let scope = ScopeKey::new(
            account.clone(),
            Region::Global,
            ServiceId::parse(parent.service_name())?,
            OperationId::parse(root_operation)?,
            None,
        )?;
        let bindings = input::root_bindings(parent, Some(&account), Some(&Region::Global))?;
        store
            .seed_work(WorkSpec::new(
                sweep.clone(),
                scope,
                None,
                bindings.into_bytes(),
            )?)
            .await?;
        let root_lease = store
            .claim(&sweep, 1000, 100_000)
            .await?
            .ok_or("root lease absent")?;
        let root_request = actual_request(parent, &root_lease)?;
        let response = fixture::decode(
            parent,
            &serde_json::to_vec(&json!({"Accelerators":[{"AcceleratorArn":parent_arn}]}))?,
        )?;
        let context = Context::new(parent, &sweep, &account, &home, &consumers);
        let ancestry = crate::execution::page::preparation::ancestry(
            root_lease.context(),
            root_lease.scope().context(),
        )?;
        let (parent_observations, children) = project_children(
            &catalog,
            &context,
            root_lease.scope(),
            &root_request,
            &response,
            &ancestry,
        )?;
        assert!(
            children
                .iter()
                .any(|work| work.scope().operation().as_str() == child_operation),
            "real child discovery did not schedule listener enumeration"
        );
        store
            .commit_page(PageCommit {
                lease: root_lease,
                observations: parent_observations,
                edges: Vec::new(),
                unresolved: Vec::new(),
                tags: Vec::new(),
                children,
                next_token: None,
                continuation_digest: None,
                done: true,
                observed_at_ms: 1001,
            })
            .await?;
        // Claim generated work, rather than exposing private WorkSpec binding/context fields.
        let lease = loop {
            let lease = store
                .claim(&sweep, 1002, 100_000)
                .await?
                .ok_or("generated listener lease absent")?;
            if lease.scope().operation().as_str() == child_operation {
                break lease;
            }
        };
        let request = actual_request(child, &lease)?;
        let ancestry = crate::execution::page::preparation::ancestry(
            lease.context(),
            lease.scope().context(),
        )?;
        let response = fixture::decode(
            child,
            &serde_json::to_vec(&json!({"Listeners":[{
                "ListenerArn":"arn:aws:globalaccelerator::123456789012:accelerator/parent/listener/child"
            }]}))?,
        )?;
        let context = Context::new(child, &sweep, &account, &home, &consumers);
        let (observations, children) = project_children(
            &catalog,
            &context,
            lease.scope(),
            &request,
            &response,
            &ancestry,
        )?;
        let rows = record::item_records(&response, child.items())?;
        let rule = catalog
            .relationships()
            .iter()
            .find(|rule| {
                rule.source_kind == *child.kind() && rule.source == RelationSource::Request
            })
            .ok_or("reviewed request-origin listener relationship absent")?;
        assert!(child.supports_relationship(rule)?);
        let request_region =
            routing::request_region(child, lease.scope().region(), &home, lease.scope().branch())?;
        let valid = references(
            &catalog,
            child,
            &request_region,
            &rows,
            &observations,
            &request,
            lease.scope().parent(),
        )?;
        assert_eq!(valid.len(), 1);
        assert_eq!(
            valid
                .first()
                .ok_or("valid request parent reference absent")?
                .target
                .value()
                .as_str(),
            parent_arn
        );
        for observation in &observations {
            assert!(record::path_value(observation.record().root(), &rule.source_path)?.is_none());
            assert!(
                matches!(record::path_value(request.root(), &rule.source_path)?,
                Some(ValueRef::Text(value)) if value == parent_arn)
            );
        }
        // When only the actual bound parent request is hostile, no selected output may rescue it.
        for hostile in [
            Value::Text("arn:aws:globalaccelerator::999999999999:accelerator/foreign".to_owned()),
            Value::Text(
                "arn:aws-cn:globalaccelerator::123456789012:accelerator/foreign".to_owned(),
            ),
            Value::Text(
                "arn:aws:globalaccelerator:us-east-1:123456789012:accelerator/foreign".to_owned(),
            ),
            Value::Text("arn:broken".to_owned()),
            Value::Int(7),
        ] {
            let hostile_request =
                replace_bound_value(&request, &rule.source_path, hostile.clone())?;
            assert_eq!(
                record::path_value(hostile_request.root(), &rule.source_path)?
                    .map(|value| value.to_value())
                    .transpose()?,
                Some(hostile)
            );
            assert!(
                references(
                    &catalog,
                    child,
                    &request_region,
                    &rows,
                    &observations,
                    &hostile_request,
                    lease.scope().parent()
                )
                .is_err()
            );
        }
        // Then the same valid reference survives checked persistence and resolves to the discovered parent.
        let key = observations
            .first()
            .ok_or("listener observation absent")?
            .key()
            .clone();
        store
            .commit_page(PageCommit {
                lease,
                observations,
                unresolved: valid,
                children,
                edges: Vec::new(),
                tags: Vec::new(),
                next_token: None,
                continuation_digest: None,
                done: true,
                observed_at_ms: 1003,
            })
            .await?;
        let edges = store.projection_edges_for(&key, None, 10).await?;
        assert_eq!(edges.edges.len(), 1);
        let edge = edges
            .edges
            .first()
            .ok_or("persisted parent relation absent")?;
        assert_eq!(edge.from, key);
        assert_eq!(edge.relation, rule.relation);
        assert_eq!(edge.to.kind, rule.target_kind);
        assert_eq!(edge.to.id.as_str(), parent_arn);
        assert_eq!(edge.to.account, account);
        assert_eq!(edge.to.region, Region::Global);
    }
    Ok(())
}

fn actual_request(
    plan: &CallablePlan<'_>,
    lease: &WorkLease,
) -> Result<Record, Box<dyn std::error::Error>> {
    let bindings = Record::from_bytes(lease.bindings().to_vec())?;
    let cursor = crate::execution::page::cursor::Cursor::decode(lease.token())?;
    let mut fields = cursor.request_fields(plan.pagination())?;
    if let Some(size) = &plan.pagination().page_size {
        fields.push(input::InputField {
            path: size.path.clone(),
            value: size.value(),
        });
    }
    Ok(input::merge_input(&bindings, &fields)?)
}

fn replace_bound_value(
    request: &Record,
    path: &[inventory_core::record::FieldId],
    value: Value,
) -> Result<Record, Box<dyn std::error::Error>> {
    use inventory_core::record::Key;
    let Value::Map(mut root) = ValueRef::Map(request.root()).to_value()? else {
        return Err("bound request is not a map".into());
    };
    let (leaf, parents) = path.split_last().ok_or("bound request path is empty")?;
    let mut target = &mut root;
    for field in parents {
        let Some(Value::Map(children)) = target.get_mut(&Key::Field(*field)) else {
            return Err("bound request parent is absent or not a map".into());
        };
        target = children;
    }
    *target
        .get_mut(&Key::Field(*leaf))
        .ok_or("bound request leaf is absent")? = value;
    Ok(Record::encode(&Value::Map(root))?)
}

fn typed_rule()
-> Result<inventory_core::catalog::runtime::RelationshipPlan, Box<dyn std::error::Error>> {
    use inventory_core::{
        catalog::runtime::{
            ReferenceOwnerRule, RelationPathMode, RelationScope, RelationTargetMode,
        },
        inventory::{Kind, Relation},
        record::FieldId,
    };
    let kind = Kind::parse("AWS::ApiOnly::RelationFixture")?;
    Ok(RelationshipPlan {
        source_kind: kind.clone(),
        source: RelationSource::SelectedOutput,
        source_path: vec![FieldId::new(1)?].into_boxed_slice(),
        source_path_mode: RelationPathMode::Fields,
        target_kind: kind.clone(),
        target_kinds: vec![kind].into_boxed_slice(),
        target_path: vec![FieldId::new(1)?].into_boxed_slice(),
        relation: Relation::parse("LINKS_TO")?,
        target_mode: RelationTargetMode::SelectedFields,
        guard: None,
        components: Box::new([]),
        scope: RelationScope::AccountRegion,
        owner: ReferenceOwnerRule::ExecutionAccount,
    })
}

fn typed_observation(rule: &RelationshipPlan) -> Result<Observation, Box<dyn std::error::Error>> {
    use inventory_core::inventory::{ResourceId, ResourceKey};
    let record = Record::encode(&Value::Map(Default::default()))?;
    let key = ResourceKey {
        account: AccountId::parse("123456789012")?,
        region: Region::parse("us-east-1")?,
        kind: rule.source_kind.clone(),
        id: ResourceId::parse("child")?,
    };
    Ok(Observation::new(
        key,
        record,
        &inventory_core::record::Exclusions::none(),
        None,
    )?)
}

fn typed_record(
    fields: impl IntoIterator<Item = (u32, Value)>,
) -> Result<Record, Box<dyn std::error::Error>> {
    use inventory_core::record::{FieldId, Key};
    let fields = fields
        .into_iter()
        .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
        .collect::<Result<std::collections::BTreeMap<_, _>, inventory_core::record::RecordError>>(
        )?;
    Ok(Record::encode(&Value::Map(fields))?)
}

#[test]
fn given_guarded_membership_when_null_empty_or_hostile_types_arrive_then_only_exact_text_members_emit()
-> TestResult {
    use inventory_core::{
        catalog::runtime::{RelationGuard, RelationPathMode},
        record::FieldId,
    };
    // Given a reviewed list source and request guard, not a truthy/unconditional approximation.
    let mut rule = typed_rule()?;
    rule.guard = Some(RelationGuard {
        source: RelationSource::Request,
        path_mode: RelationPathMode::Fields,
        path: vec![FieldId::new(2)?].into_boxed_slice(),
        equals: "active".into(),
    });
    let observation = typed_observation(&rule)?;
    let row = typed_record([(
        1,
        Value::List(vec![
            Value::Null,
            Value::Text("".into()),
            Value::Text("one".into()),
            Value::Text("two".into()),
        ]),
    )])?;
    let home = RegionId::parse("us-east-1")?;
    // When the actual bound request guard matches.
    let request = typed_record([(2, Value::Text("active".into()))])?;
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 10)?;
    // Then every meaningful list member becomes an exact reference; no list is stringified.
    assert_eq!(
        output
            .iter()
            .map(|reference| reference.target.value().as_str())
            .collect::<Vec<_>>(),
        ["one", "two"]
    );
    for value in [Value::Null, Value::Text("inactive".into())] {
        let request = typed_record([(2, value)])?;
        let mut output = Vec::new();
        super::relations::Context {
            row: &row,
            request: &request,
            observation: &observation,
            home: &home,
            parent: None,
        }
        .append(&rule, &mut output, 10)?;
        assert_eq!(output.len(), 0);
    }
    let request = typed_record([(2, Value::Bool(true))])?;
    let error = super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut Vec::new(), 10);
    assert!(matches!(
        error,
        Err(SweepError::Operation(
            "relationship guard is not modeled text"
        ))
    ));
    Ok(())
}

#[test]
fn given_root_and_compound_selectors_when_extracted_then_namespaces_and_conjunction_survive()
-> TestResult {
    use inventory_core::{
        catalog::runtime::{RelationPathMode, RelationTargetMode, RelationshipComponentPlan},
        record::{FieldId, Key},
    };
    // Given a real scalar selected row and a separate actual request conjunct.
    let mut rule = typed_rule()?;
    rule.source_path = Box::new([]);
    rule.source_path_mode = RelationPathMode::RootText;
    rule.target_mode = RelationTargetMode::CanonicalId;
    rule.target_path = Box::new([]);
    rule.components = vec![RelationshipComponentPlan {
        source: RelationSource::Request,
        source_path: vec![FieldId::new(2)?].into_boxed_slice(),
        source_path_mode: RelationPathMode::Fields,
        target_mode: RelationTargetMode::BoundContextFields,
        target_path: vec![FieldId::new(3)?].into_boxed_slice(),
    }]
    .into_boxed_slice();
    let row = Record::encode(&Value::Map(std::collections::BTreeMap::from([(
        Key::Name("$value".into()),
        Value::Text("exact-id".into()),
    )])))?;
    let request = typed_record([(2, Value::Text("context-operand".into()))])?;
    let observation = typed_observation(&rule)?;
    let home = RegionId::parse("us-east-1")?;
    // When extracting through the production source projection.
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 1)?;
    // Then canonical ID and target context are distinct typed operands, never field-path sentinels.
    let reference = output.first().ok_or("root reference absent")?;
    assert_eq!(reference.target.value().as_str(), "exact-id");
    assert_eq!(reference.target.mode(), RelationTargetMode::CanonicalId);
    let term = reference
        .target
        .components()
        .first()
        .ok_or("compound term absent")?;
    assert_eq!(term.mode, RelationTargetMode::BoundContextFields);
    assert_eq!(term.path.as_ref(), &[FieldId::new(3)?]);
    assert_eq!(term.value.as_str(), "context-operand");
    let missing = typed_record([])?;
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &missing,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 1)?;
    assert_eq!(output.len(), 0);
    Ok(())
}

#[test]
fn given_parent_rule_when_selected_decoy_differs_then_only_authoritative_parent_scope_is_used()
-> TestResult {
    use inventory_core::{
        catalog::runtime::{RelationPathMode, RelationTargetMode},
        inventory::{Kind, ResourceId},
    };
    let mut rule = typed_rule()?;
    rule.source = RelationSource::Parent;
    rule.source_path = Box::new([]);
    rule.source_path_mode = RelationPathMode::RootText;
    rule.target_mode = RelationTargetMode::CanonicalId;
    rule.target_path = Box::new([]);
    rule.target_kind = Kind::parse("AWS::VpcLattice::Service")?;
    let network = Kind::parse("AWS::VpcLattice::ServiceNetwork")?;
    rule.target_kinds = vec![rule.target_kind.clone(), network.clone()].into_boxed_slice();
    let observation = typed_observation(&rule)?;
    let mut parent = observation.key().clone();
    parent.id = ResourceId::parse("authoritative-parent")?;
    parent.kind = network;
    let row = typed_record([(1, Value::Text("selected-decoy".into()))])?;
    let request = typed_record([])?;
    let home = RegionId::parse("us-east-1")?;
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: Some(&parent),
    }
    .append(&rule, &mut output, 1)?;
    assert_eq!(
        output
            .first()
            .ok_or("parent reference absent")?
            .target
            .value()
            .as_str(),
        "authoritative-parent"
    );
    let target = &output.first().ok_or("parent reference absent")?.target;
    assert_eq!(target.kind(), &parent.kind);
    assert_ne!(target.kind(), &rule.target_kind);
    assert_eq!(target.candidate_kinds(), rule.target_kinds.as_ref());
    assert!(matches!(
        super::relations::Context {
            row: &row,
            request: &request,
            observation: &observation,
            home: &home,
            parent: Some(&parent)
        }
        .append(&rule, &mut output, 1),
        Err(SweepError::Operation(
            "page exceeds the atomic reference bound"
        ))
    ));
    let mut output = Vec::new();
    super::relations::Context {
        row: &row,
        request: &request,
        observation: &observation,
        home: &home,
        parent: None,
    }
    .append(&rule, &mut output, 1)?;
    assert_eq!(output.len(), 0);
    Ok(())
}
