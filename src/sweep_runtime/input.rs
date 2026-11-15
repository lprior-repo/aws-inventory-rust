use inventory_core::record::{FieldId, Key, Record, Value};

use super::SweepError;

const MAX_PATH_FIELDS: usize = 8;

pub(super) struct InputField {
    pub path: Box<[FieldId]>,
    pub value: Value,
}

pub(super) fn requires_parent(plan: &inventory_core::catalog::runtime::CallablePlan<'_>) -> bool {
    use inventory_core::catalog::runtime::BindingSource;
    plan.bindings().iter().any(|binding| {
        matches!(
            binding.source,
            BindingSource::ParentIdentity { .. } | BindingSource::Operation { .. }
        )
    })
}

pub(super) fn root_bindings(
    plan: &inventory_core::catalog::runtime::CallablePlan<'_>,
    account: Option<&inventory_core::inventory::AccountId>,
    region: Option<&inventory_core::inventory::Region>,
) -> Result<Record, SweepError> {
    let mut fields = Vec::new();
    fields.try_reserve(plan.bindings().len())?;
    for binding in plan.bindings() {
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
    use inventory_core::catalog::runtime::{BindingSource, BindingValue};
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
        BindingSource::Fixed(value) => Ok(Some(match value {
            BindingValue::String(value) => Value::Text(value.clone()),
            BindingValue::Integer(value) => Value::Int(*value),
            BindingValue::Boolean(value) => Value::Bool(*value),
            BindingValue::Strings(values) => {
                Value::List(values.iter().cloned().map(Value::Text).collect())
            }
        })),
        BindingSource::ParentIdentity { .. } | BindingSource::Operation { .. } => Ok(None),
    }
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
    if target
        .insert(Key::Field(leaf), field.value.clone())
        .is_some()
    {
        return Err(SweepError::Operation("duplicate input binding path"));
    }
    Ok(())
}
