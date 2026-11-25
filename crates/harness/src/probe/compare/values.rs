use super::super::ProbeError;
use inventory_core::{
    catalog::Service,
    record::{FieldId, Key, Record, Value},
};

const MAX_NODES: usize = 1_048_576;
type Task<'a> = (Option<&'a Value>, Option<&'a Value>, String);

pub(super) fn difference(
    service: Service<'_>,
    list: &Record,
    describe: &Record,
) -> Result<Option<String>, ProbeError> {
    let list = inventory_core::record::ValueRef::Map(list.root()).to_value()?;
    let describe = inventory_core::record::ValueRef::Map(describe.root()).to_value()?;
    compare(
        &|id| Ok(service.field_name(id)?.to_owned()),
        &list,
        &describe,
    )
}

fn compare(
    field_name: &impl Fn(FieldId) -> Result<String, ProbeError>,
    list: &Value,
    describe: &Value,
) -> Result<Option<String>, ProbeError> {
    let mut tasks = vec![(Some(list), Some(describe), "$".to_owned())];
    (0..MAX_NODES)
        .find_map(|_| {
            let Some((list, describe, path)) = tasks.pop() else {
                return Some(Ok(None));
            };
            match visit(field_name, list, describe, &path, &mut tasks) {
                Ok(true) => Some(Ok(Some(path))),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            }
        })
        .ok_or(ProbeError::Invalid("comparison node bound exceeded"))?
}

fn visit<'a>(
    field_name: &impl Fn(FieldId) -> Result<String, ProbeError>,
    list: Option<&'a Value>,
    describe: Option<&'a Value>,
    path: &str,
    tasks: &mut Vec<Task<'a>>,
) -> Result<bool, ProbeError> {
    match (list, describe) {
        (Some(Value::Map(list)), Some(Value::Map(describe))) => {
            for (key, value) in list
                .iter()
                .rev()
                .filter(|(key, _)| !describe.contains_key(*key))
            {
                let name = match key {
                    Key::Field(id) => field_name(*id)?,
                    Key::Name(name) => name.clone(),
                };
                tasks.try_reserve(1)?;
                tasks.push((Some(value), None, format!("{path}.{name}")));
            }
            tasks.try_reserve(describe.len())?;
            describe.iter().rev().try_for_each(|(key, value)| {
                let name = match key {
                    Key::Field(id) => field_name(*id)?,
                    Key::Name(name) => name.clone(),
                };
                tasks.push((list.get(key), Some(value), format!("{path}.{name}")));
                Ok::<_, ProbeError>(())
            })?;
            Ok(false)
        }
        (Some(Value::List(list)), Some(Value::List(describe))) if list.len() == describe.len() => {
            tasks.try_reserve(describe.len())?;
            tasks.extend(
                describe.iter().enumerate().rev().map(|(index, value)| {
                    (list.get(index), Some(value), format!("{path}[{index}]"))
                }),
            );
            Ok(false)
        }
        (Some(list), Some(describe)) => Ok(list != describe),
        (None, None) => Ok(false),
        _ => Ok(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inventory_core::record::{FieldId, Float, Key};

    fn record(
        entries: impl IntoIterator<Item = (u32, Value)>,
    ) -> Result<Record, Box<dyn std::error::Error>> {
        let fields = entries
            .into_iter()
            .map(|(id, value)| Ok((Key::Field(FieldId::new(id)?), value)))
            .collect::<Result<_, inventory_core::record::RecordError>>()?;
        Ok(Record::encode(&Value::Map(fields))?)
    }

    #[test]
    fn raw_null_and_filled_values_are_not_equal() -> Result<(), Box<dyn std::error::Error>> {
        let list = record([(1, Value::Null)])?;
        let describe = record([(1, Value::Text("filled".to_owned()))])?;
        let list = inventory_core::record::ValueRef::Map(list.root()).to_value()?;
        let describe = inventory_core::record::ValueRef::Map(describe.root()).to_value()?;
        assert_eq!(
            compare(&|_| Ok("Member".into()), &list, &describe)?,
            Some("$.Member".into())
        );
        Ok(())
    }

    #[test]
    fn extra_describe_member_is_missing_not_equal() -> Result<(), Box<dyn std::error::Error>> {
        let list = Value::Map(Default::default());
        let describe = Value::Map([(Key::Name("Extra".into()), Value::Null)].into());
        assert_eq!(
            compare(&|_| Ok("Member".into()), &list, &describe)?,
            Some("$.Extra".into())
        );
        Ok(())
    }

    #[test]
    fn equal_nested_values_have_no_difference() -> Result<(), Box<dyn std::error::Error>> {
        let item =
            Value::Map([(Key::Name("Nested".into()), Value::List(vec![Value::Int(7)]))].into());
        assert_eq!(compare(&|_| Ok("Member".into()), &item, &item)?, None);
        Ok(())
    }

    #[test]
    fn absent_null_and_populated_members_are_pairwise_distinct()
    -> Result<(), Box<dyn std::error::Error>> {
        let states = [
            Value::Map(Default::default()),
            Value::Map([(Key::Name("Member".into()), Value::Null)].into()),
            Value::Map([(Key::Name("Member".into()), Value::Text("filled".into()))].into()),
        ];
        for (left_index, left) in states.iter().enumerate() {
            for (right_index, right) in states.iter().enumerate() {
                let expected = (left_index != right_index).then(|| "$.Member".to_owned());
                assert_eq!(compare(&|_| Ok("Member".into()), left, right)?, expected);
            }
        }
        Ok(())
    }

    #[test]
    fn given_unequal_list_lengths_when_compared_then_the_list_member_differs()
    -> Result<(), Box<dyn std::error::Error>> {
        let short = Value::Map([(Key::Name("Values".into()), Value::List(vec![]))].into());
        let long =
            Value::Map([(Key::Name("Values".into()), Value::List(vec![Value::Int(7)]))].into());
        assert_eq!(
            compare(&|_| Ok("unused".into()), &short, &long)?,
            Some("$.Values".into())
        );
        assert_eq!(
            compare(&|_| Ok("unused".into()), &long, &short)?,
            Some("$.Values".into())
        );
        Ok(())
    }

    #[test]
    fn given_integer_and_equal_valued_float_when_compared_then_raw_types_prevent_skip()
    -> Result<(), Box<dyn std::error::Error>> {
        let integer = Value::Map([(Key::Name("Count".into()), Value::Int(1))].into());
        let float = Value::Map([(Key::Name("Count".into()), Value::Float(Float::new(1.0)))].into());
        assert_eq!(
            compare(&|_| Ok("unused".into()), &integer, &float)?,
            Some("$.Count".into())
        );
        assert_eq!(
            compare(&|_| Ok("unused".into()), &float, &integer)?,
            Some("$.Count".into())
        );
        Ok(())
    }
}
