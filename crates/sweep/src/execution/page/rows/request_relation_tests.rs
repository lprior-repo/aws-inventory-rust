use super::*;
#[path = "request_relation_tests/accelerator.rs"]
mod accelerator;
#[path = "request_relation_tests/declared.rs"]
mod declared;
#[path = "request_relation_tests/typed.rs"]
mod typed;
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
        edge_source: inventory_core::inventory::EdgeSource::ReviewedRule,
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
