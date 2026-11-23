use super::{Action, finish};
use crate::inventory_seed::{SeedError, model::members};
use inventory_core::{
    catalog::{Service, Shape, Target},
    record::{Key, Value},
};

type Task<'a> = Action<(Target, &'a Value)>;

pub(in crate::inventory_seed) fn resolve(
    service: Service<'_>,
    target: Target,
    value: &Value,
) -> Result<Value, SeedError> {
    super::validate(value)?;
    resolve_value(service, target, value)
}

pub(in crate::inventory_seed) fn resolve_template(
    service: Service<'_>,
    target: Target,
    value: &Value,
) -> Result<Value, SeedError> {
    super::validate_template(value)?;
    resolve_value(service, target, value)
}

fn resolve_value(service: Service<'_>, target: Target, value: &Value) -> Result<Value, SeedError> {
    let mut tasks = vec![Action::Visit((target, value), 0)];
    let mut values = Vec::new();
    for _ in 0..514 {
        match tasks.pop() {
            Some(Action::Visit((target, value), depth)) => {
                visit(service, target, value, (&mut tasks, &mut values, depth))?
            }
            Some(Action::Finish(keys, length)) => finish(keys, length, &mut values)?,
            None => return complete_modeled(values),
        }
    }
    Err(SeedError::Invalid("seed literal exceeds its node limit"))
}

fn complete_modeled(mut values: Vec<Value>) -> Result<Value, SeedError> {
    if values.len() != 1 {
        return Err(SeedError::Invalid("modeled seed literal is incomplete"));
    }
    values
        .pop()
        .ok_or(SeedError::Invalid("modeled seed literal is absent"))
}

fn shape(service: Service<'_>, target: Target) -> Result<Shape<'_>, SeedError> {
    let Target::Shape(index) = target else {
        return Err(SeedError::Invalid(
            "seed container does not target a modeled shape",
        ));
    };
    service
        .shape(index)?
        .ok_or(SeedError::Invalid("seed container shape is missing"))
}

fn visit<'a>(
    service: Service<'_>,
    target: Target,
    value: &'a Value,
    state: (&mut Vec<Task<'a>>, &mut Vec<Value>, usize),
) -> Result<(), SeedError> {
    let (tasks, values, depth) = state;
    let next = depth
        .checked_add(1)
        .ok_or(SeedError::Invalid("seed literal depth overflow"))?;
    match value {
        Value::List(items) => {
            let element = members::element(service, target)?;
            tasks.push(Action::Finish(None, items.len()));
            tasks.extend(items.iter().rev().map(|item| Action::Visit((element, item), next)));
        }
        Value::Map(items) => map(service, target, items, (tasks, next))?,
        value => values.push(value.clone()),
    }
    Ok(())
}

fn map<'a>(
    service: Service<'_>,
    target: Target,
    items: &'a std::collections::BTreeMap<Key, Value>,
    (tasks, depth): (&mut Vec<Task<'a>>, usize),
) -> Result<(), SeedError> {
    let fields = items
        .iter()
        .map(|(key, value)| {
            let (key, target) = map_field(service, target, key)?;
            Ok((key, target, value))
        })
        .collect::<Result<Vec<_>, SeedError>>()?;
    tasks.push(Action::Finish(
        Some(fields.iter().map(|(key, _, _)| key.clone()).collect()),
        fields.len(),
    ));
    tasks.extend(
        fields
            .into_iter()
            .rev()
            .map(|(_, target, value)| Action::Visit((target, value), depth)),
    );
    Ok(())
}

fn map_field(
    service: Service<'_>,
    target: Target,
    key: &Key,
) -> Result<(Key, Target), SeedError> {
    let Key::Name(name) = key else {
        return Err(SeedError::Invalid("seed literal requires named map keys"));
    };
    if members::is_document(service, target)? {
        return Ok((key.clone(), target));
    }
    let shape = shape(service, target)?;
    match shape.kind()? {
        "structure" | "union" => {
            let member = members::member(service, target, name)?;
            Ok((Key::Field(member.id), member.target))
        }
        "map" => Ok((key.clone(), shape.map_value()?.ok_or(SeedError::Invalid(
            "seed map value shape is missing",
        ))?)),
        _ => Err(SeedError::Invalid("seed map targets a non-map shape")),
    }
}
