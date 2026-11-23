use super::*;

pub(super) fn strings(
    service: Service<'_>,
    target: Target,
    values: &[String],
    depth: usize,
    state: &mut State<'_, '_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    collection(values.len())?;
    let child = string_element(service, target, depth, values.is_empty())?;
    charge_nodes(state.budget, values.len())?;
    Ok(Value::List(
        values
            .iter()
            .map(|text| {
                text_bound(text.len(), &mut state.budget.bytes)?;
                enum_value(service, child, ValueRef::Text(text))?;
                Ok(Value::Text(text.clone()))
            })
            .collect::<Result<_, RuntimeCatalogCompileError>>()?,
    ))
}

fn string_element(
    service: Service<'_>,
    target: Target,
    depth: usize,
    empty: bool,
) -> Result<Target, RuntimeCatalogCompileError> {
    let child = shape(service, target)?.element()?.ok_or_else(unsupported)?;
    if !matches!(
        super::super::super::model_path::kind(service, child)?,
        "string" | "enum"
    ) || (depth >= BindingValue::MAX_DEPTH && !empty)
    {
        return Err(unsupported());
    }
    Ok(child)
}

fn charge_nodes(budget: &mut Budget, count: usize) -> Result<(), RuntimeCatalogCompileError> {
    budget.nodes = budget.nodes.checked_add(count).ok_or_else(unsupported)?;
    if budget.nodes > BindingValue::MAX_NODES {
        return Err(unsupported());
    }
    Ok(())
}

pub(super) fn list<'a>(
    service: Service<'_>,
    target: Target,
    values: &'a [FixedValue],
    depth: usize,
    state: &mut State<'a, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    collection(values.len())?;
    let child = shape(service, target)?.element()?.ok_or_else(unsupported)?;
    state.work.push(Work::List(values.len()));
    state.work.extend(
        values
            .iter()
            .rev()
            .map(|value| Work::Read(value, child, depth.saturating_add(1))),
    );
    Ok(())
}

pub(super) fn structure<'a>(
    service: Service<'_>,
    target: Target,
    values: &'a std::collections::BTreeMap<String, FixedValue>,
    depth: usize,
    state: &mut State<'a, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    collection(values.len())?;
    let members = shape(service, target)?
        .members()?
        .collect::<Result<Vec<_>, _>>()?;
    required_members(&members, values)?;
    let children = structure_children(&members, values, &mut state.budget.bytes)?;
    state.work.push(Work::Object(
        children.iter().map(|(key, _, _)| key.clone()).collect(),
    ));
    state.work.extend(
        children
            .into_iter()
            .rev()
            .map(|(_, value, target)| Work::Read(value, target, depth.saturating_add(1))),
    );
    Ok(())
}

fn required_members(
    members: &[inventory_core::catalog::Member<'_>],
    values: &std::collections::BTreeMap<String, FixedValue>,
) -> Result<(), RuntimeCatalogCompileError> {
    if members.iter().any(|member| {
        member.required
            && member
                .name()
                .map_or(true, |name| !values.contains_key(name))
    }) {
        return Err(unsupported());
    }
    Ok(())
}

fn structure_children<'a>(
    members: &[inventory_core::catalog::Member<'_>],
    values: &'a std::collections::BTreeMap<String, FixedValue>,
    bytes: &mut usize,
) -> Result<Vec<(Key, &'a FixedValue, Target)>, RuntimeCatalogCompileError> {
    values
        .iter()
        .map(|(name, value)| {
            text_bound(name.len(), bytes)?;
            let member = members
                .iter()
                .find(|member| member.name().is_ok_and(|field| field == name))
                .ok_or_else(unsupported)?;
            Ok((Key::Field(member.id), value, member.target))
        })
        .collect()
}

pub(super) fn map<'a>(
    service: Service<'_>,
    target: Target,
    values: &'a std::collections::BTreeMap<String, FixedValue>,
    depth: usize,
    state: &mut State<'a, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    collection(values.len())?;
    let (key, child) = map_targets(service, target)?;
    values.keys().try_for_each(|name| {
        text_bound(name.len(), &mut state.budget.bytes)?;
        enum_value(service, key, ValueRef::Text(name))
    })?;
    state.work.push(Work::Object(
        values.keys().cloned().map(Key::Name).collect(),
    ));
    state.work.extend(
        values
            .values()
            .rev()
            .map(|value| Work::Read(value, child, depth.saturating_add(1))),
    );
    Ok(())
}

fn map_targets(
    service: Service<'_>,
    target: Target,
) -> Result<(Target, Target), RuntimeCatalogCompileError> {
    let shape = shape(service, target)?;
    let key = shape.map_key()?.ok_or_else(unsupported)?;
    super::super::super::model_path::string_target(service, key)?;
    let child = shape.map_value()?.ok_or_else(unsupported)?;
    Ok((key, child))
}

fn collection(length: usize) -> Result<(), RuntimeCatalogCompileError> {
    if length > BindingValue::MAX_COLLECTION {
        Err(unsupported())
    } else {
        Ok(())
    }
}
