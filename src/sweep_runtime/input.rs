use inventory_core::record::{FieldId, Key, Record, Value};

use super::SweepError;

const MAX_PATH_FIELDS: usize = 8;

pub(super) struct InputField {
    pub path: Box<[FieldId]>,
    pub value: Value,
}

#[cfg(test)]
pub(super) fn requires_parent(plan: &inventory_core::catalog::runtime::CallablePlan<'_>) -> bool {
    plan.bindings().iter().any(|binding| dependent(&binding.source))
}

pub(in crate::sweep_runtime) fn requires_parent_branch(plan: &inventory_core::catalog::runtime::CallablePlan<'_>,
    branch: Option<&inventory_core::catalog::runtime::ExecutionBranch>) -> bool {
    plan.effective_bindings(branch).any(|binding| dependent(&binding.source))
}

fn dependent(source: &inventory_core::catalog::runtime::BindingSource) -> bool {
    use inventory_core::catalog::runtime::BindingSource;
    match source {
        BindingSource::ParentIdentity { .. } | BindingSource::Operation { .. }
        | BindingSource::OperationArn { .. } | BindingSource::OperationUrl { .. } | BindingSource::Context { .. }
        | BindingSource::Alternatives(_) | BindingSource::ContextFromBranch { .. } => true,
        BindingSource::Transformed { source,.. } => dependent(source),
        BindingSource::Account | BindingSource::Region | BindingSource::RegionList | BindingSource::Fixed(_) => false,
    }
}

pub(super) fn root_bindings(
    plan: &inventory_core::catalog::runtime::CallablePlan<'_>,
    account: Option<&inventory_core::inventory::AccountId>,
    region: Option<&inventory_core::inventory::Region>,
) -> Result<Record, SweepError> {
    root_bindings_branch(plan, None, account, region)
}

pub(in crate::sweep_runtime) fn root_bindings_branch(
    plan: &inventory_core::catalog::runtime::CallablePlan<'_>,
    branch: Option<&inventory_core::catalog::runtime::ExecutionBranch>,
    account: Option<&inventory_core::inventory::AccountId>,
    region: Option<&inventory_core::inventory::Region>,
) -> Result<Record, SweepError> {
    let mut fields = Vec::new();
    fields.try_reserve(plan.bindings().len().saturating_add(branch.map_or(0, |branch| branch.bindings.len())))?;
    for binding in plan.effective_bindings(branch) {
        let value = binding_value(&binding.source, account, region)?.ok_or(
            SweepError::Operation("dependent input binding requires a reviewed parent"),
        )?;
        fields.push(InputField {
            path: binding.path.clone(),
            value,
        });
    }
    merge_input(&empty_record()?, &fields)
}

fn empty_record() -> Result<Record, SweepError> {
    Record::encode(&Value::Map(std::collections::BTreeMap::new())).map_err(SweepError::from)
}

fn binding_value(
    source: &inventory_core::catalog::runtime::BindingSource,
    account: Option<&inventory_core::inventory::AccountId>,
    region: Option<&inventory_core::inventory::Region>,
) -> Result<Option<Value>, SweepError> {
    use inventory_core::catalog::runtime::BindingSource;
    match source {
        BindingSource::Account => account
            .map(|value| Value::Text(value.as_str().to_owned()))
            .map(Some)
            .ok_or(SweepError::Operation(
                "account binding unavailable during bootstrap",
            )),
        BindingSource::Region => region
            .map(|value| Value::Text(value.as_str().to_owned()))
            .map(Some)
            .ok_or(SweepError::Operation(
                "region binding unavailable during bootstrap",
            )),
        BindingSource::RegionList => match region {
            Some(inventory_core::inventory::Region::Regional(region)) =>
                Ok(Some(Value::List(vec![Value::Text(region.as_str().to_owned())]))),
            _ => Err(SweepError::Operation("region list binding requires the routed request region")),
        },
        BindingSource::Fixed(value) => super::request::fixed_input::value(value).map(Some),
        BindingSource::ParentIdentity { .. } | BindingSource::Operation { .. } | BindingSource::OperationArn { .. } | BindingSource::OperationUrl { .. } | BindingSource::Context { .. } | BindingSource::Alternatives(_) | BindingSource::ContextFromBranch { .. } => Ok(None),
        BindingSource::Transformed { source,transform } => binding_value(source,account,region)?
            .map(|value| transform_value(value,transform)).transpose(),
    }
}

pub(in crate::sweep_runtime) fn transform_value(
    value: Value, transform: &inventory_core::catalog::runtime::BindingTransform,
) -> Result<Value,SweepError> {
    use inventory_core::catalog::runtime::BindingTransform;
    let Value::Text(mut text) = value else { return Err(SweepError::Operation("reviewed binding transform requires text")); };
    match transform {
        BindingTransform::StripPrefix(prefix) => {
            if text.starts_with(prefix.as_ref()) { text.drain(..prefix.len()); }
        }
    }
    if text.is_empty() || text.contains('/') {
        return Err(SweepError::Operation("reviewed binding transform requires a nonempty bare identifier"));
    }
    Ok(Value::Text(text))
}
pub(super) fn merge_input(base: &Record, fields: &[InputField]) -> Result<Record, SweepError> {
    let mut root = std::collections::BTreeMap::new();
    for entry in base.root().iter()? {
        let (key, value) = entry?;
        root.insert(key.to_key(), value.to_value()?);
    }
    fields
        .iter()
        .try_for_each(|field| insert_path(&mut root, field))?;
    Ok(Record::encode(&Value::Map(root))?)
}

fn insert_path(
    root: &mut std::collections::BTreeMap<Key, Value>,
    field: &InputField,
) -> Result<(), SweepError> {
    if field.path.is_empty() || field.path.len() > MAX_PATH_FIELDS {
        return Err(SweepError::Operation(
            "input binding path exceeds compiled bounds",
        ));
    }
    let parents = field.path.len().saturating_sub(1);
    let mut target = root;
    for id in field.path.iter().take(parents) {
        let value = target
            .entry(Key::Field(*id))
            .or_insert_with(|| Value::Map(std::collections::BTreeMap::new()));
        target = match value {
            Value::Map(children) => children,
            _ => return Err(SweepError::Operation("input binding paths collide")),
        };
    }
    let leaf = field
        .path
        .last()
        .copied()
        .ok_or(SweepError::Operation("input binding path is empty"))?;
    // Idempotent per branch: later binding wins over base (branch overrides). Deduplicate same-path identical or conflicting by last-wins to avoid spurious duplicate error.
    let _ = target.insert(Key::Field(leaf), field.value.clone());
    Ok(())
}

#[cfg(test)]
mod transform_tests {
    use super::*;
    use inventory_core::catalog::runtime::BindingTransform;

    #[test]
    fn reviewed_prefix_removes_only_one_exact_type_prefix() -> Result<(),SweepError> {
        let transform = BindingTransform::StripPrefix("/hostedzone/".into());
        for text in ["Z123","/hostedzone/Z123"] {
            assert_eq!(transform_value(Value::Text(text.to_owned()),&transform)?,Value::Text("Z123".to_owned()));
        }
        for text in ["","/hostedzone/","/hostedzone//Z123","/delegationset/N123","Z/123"] {
            assert!(transform_value(Value::Text(text.to_owned()),&transform).is_err(),"{text}");
        }
        assert!(transform_value(Value::Int(1),&transform).is_err());
        Ok(())
    }
}
