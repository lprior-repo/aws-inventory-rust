use super::{CallablePlan, PlanPurpose, RuntimeCatalogError};
use crate::record::{FieldId, MapRef, ValueRef};

const MAX_SKIPS: usize = 64;

pub(super) fn read(fields: MapRef<'_>) -> Result<Box<[Box<str>]>, RuntimeCatalogError> {
    let Some(value) = fields.get_field(FieldId::new(26)?)? else {
        return Ok(Box::new([]));
    };
    let list = super::list_value(value)?;
    let count = list.count()?;
    if count > MAX_SKIPS {
        return Err(RuntimeCatalogError::Bounds);
    }
    let mut names = Vec::<Box<str>>::new();
    names
        .try_reserve_exact(count)
        .map_err(|_| RuntimeCatalogError::Bounds)?;
    list.iter()?.try_for_each(|entry| {
        let name = name(entry?)?;
        if names.iter().any(|prior| prior.as_ref() == name) {
            return Err(RuntimeCatalogError::Capability);
        }
        names.push(name.into());
        Ok(())
    })?;
    Ok(names.into_boxed_slice())
}

fn name(value: ValueRef<'_>) -> Result<&str, RuntimeCatalogError> {
    let ValueRef::Text(value) = value else {
        return Err(RuntimeCatalogError::Capability);
    };
    if value.is_empty()
        || value.len() > 256
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(value)
}

pub(super) fn validate(plans: &[CallablePlan<'_>]) -> Result<(), RuntimeCatalogError> {
    let mut parents = plans
        .iter()
        .filter(|plan| !plan.skips().is_empty())
        .peekable();
    if parents.peek().is_none() {
        return Ok(());
    }

    let mut indexed = Vec::new();
    indexed
        .try_reserve_exact(plans.len())
        .map_err(|_| RuntimeCatalogError::Bounds)?;
    indexed.extend(plans);
    indexed.sort_unstable_by_key(|&plan| plan_key(plan));

    parents.try_for_each(|parent| {
        if !matches!(parent.purpose(), PlanPurpose::Enumeration) {
            return Err(RuntimeCatalogError::Capability);
        }
        parent
            .skips()
            .iter()
            .try_for_each(|child| validate_child(&indexed, parent, child))
    })
}

fn plan_key<'p>(plan: &'p CallablePlan<'_>) -> (&'p str, &'p str) {
    (plan.service_name(), plan.operation_name())
}

fn validate_child(
    indexed: &[&CallablePlan<'_>],
    parent: &CallablePlan<'_>,
    child_name: &str,
) -> Result<(), RuntimeCatalogError> {
    let key = (parent.service_name(), child_name);
    if key == plan_key(parent) {
        return Err(RuntimeCatalogError::Capability);
    }
    let start = indexed.partition_point(|plan| plan_key(plan) < key);
    let mut matches = indexed
        .get(start..)
        .ok_or(RuntimeCatalogError::Capability)?
        .iter()
        .take_while(|plan| plan_key(plan) == key);
    let child = matches.next().ok_or(RuntimeCatalogError::Capability)?;
    if matches.next().is_some() || !bound_child(parent, child) {
        return Err(RuntimeCatalogError::Capability);
    }
    Ok(())
}

fn bound_child(parent: &CallablePlan<'_>, child: &CallablePlan<'_>) -> bool {
    child
        .branches()
        .iter()
        .map(Some)
        .chain(std::iter::once(None).filter(|_| child.branches().is_empty()))
        .all(|scope| child.has_exclusive_skip_bindings(parent, scope))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{Key, Record, Value};

    #[test]
    fn catalogs_without_skip_field_default_to_no_skips() -> Result<(), Box<dyn std::error::Error>> {
        let record = Record::encode(&Value::Map(Default::default()))?;
        assert!(read(record.root())?.is_empty());
        let emit_record = Record::encode(&Value::Map(
            [(Key::Field(FieldId::new(25)?), Value::Int(1))].into(),
        ))?;
        assert!(read(emit_record.root())?.is_empty());
        Ok(())
    }

    #[test]
    fn decodes_operation_names() -> Result<(), Box<dyn std::error::Error>> {
        let record = Record::encode(&Value::Map(
            [(
                Key::Field(FieldId::new(26)?),
                Value::List(vec![Value::Text("DescribeThing".to_owned())]),
            )]
            .into(),
        ))?;
        assert_eq!(
            read(record.root())?.as_ref(),
            &[Box::<str>::from("DescribeThing")]
        );
        Ok(())
    }

    #[test]
    fn given_malformed_names_in_field_26_when_decoded_then_catalog_is_refused()
    -> Result<(), Box<dyn std::error::Error>> {
        for name in [
            "",
            "Describe-Thing",
            "Describe.Thing",
            "Describe Thing",
            "Décrire",
        ] {
            let record = Record::encode(&Value::Map(
                [(
                    Key::Field(FieldId::new(26)?),
                    Value::List(vec![Value::Text(name.to_owned())]),
                )]
                .into(),
            ))?;
            assert!(
                matches!(read(record.root()), Err(RuntimeCatalogError::Capability)),
                "{name:?}"
            );
        }
        Ok(())
    }
}
