use super::DecodeError;
use inventory_core::{
    catalog::{HttpBinding, Member, Service, Target},
    record::{FieldId, Float, KeyRef, RecordBuilder, RecordError, Time, ValueRef},
};

pub(super) fn apply(
    builder: &mut RecordBuilder,
    target: Option<Target>,
    seen: &[FieldId],
    service: Service<'_>,
    limits: super::DecodeLimits,
) -> Result<(), DecodeError> {
    let Some(Target::Shape(index)) = target else {
        return Ok(());
    };
    let Some(shape) = service.shape(index)? else {
        return Ok(());
    };
    if shape.kind()? != "structure" {
        return Ok(());
    }
    for entry in shape.members()? {
        let member = entry?;
        if !seen.contains(&member.id) && is_document(member.wire()?.binding()?) {
            apply_member(builder, member, service, limits)?;
        }
    }
    Ok(())
}

pub(super) fn is_document(binding: HttpBinding<'_>) -> bool {
    matches!(
        binding,
        HttpBinding::Document
            | HttpBinding::Label
            | HttpBinding::Query(_)
            | HttpBinding::QueryParams
    )
}

pub(super) fn apply_member(
    builder: &mut RecordBuilder,
    member: Member<'_>,
    service: Service<'_>,
    limits: super::DecodeLimits,
) -> Result<(), DecodeError> {
    if limits.preserve_fields {
        return Ok(());
    }
    let wire = member.wire()?;
    if wire.client_optional()? || wire.streaming()? || streaming(member.target, service)? {
        return Ok(());
    }
    if let Some(value) = wire.default_value()?
        && !matches!(value, ValueRef::Null)
    {
        emit(builder, Some(KeyRef::Field(member.id)), value)?;
        return Ok(());
    }
    if member.required {
        correct(builder, member, service)?;
    }
    Ok(())
}

fn streaming(target: Target, service: Service<'_>) -> Result<bool, DecodeError> {
    if let Target::Shape(index) = target {
        return service
            .shape(index)?
            .ok_or(DecodeError::Invalid("missing member target"))?
            .wire()?
            .streaming()
            .map_err(DecodeError::from);
    }
    Ok(false)
}

fn correct(
    builder: &mut RecordBuilder,
    member: Member<'_>,
    service: Service<'_>,
) -> Result<(), DecodeError> {
    let kind = target_kind(member.target, service)?;
    let key = Some(KeyRef::Field(member.id));
    if kind == "list" {
        builder.begin_list(key)?;
        builder.end()?;
    } else if kind == "map" {
        builder.begin_map(key)?;
        builder.end()?;
    } else {
        builder.scalar(key, zero(kind)?)?;
    }
    Ok(())
}

fn target_kind<'a>(target: Target, service: Service<'a>) -> Result<&'a str, DecodeError> {
    match target {
        Target::Prelude(kind) => Ok(kind),
        Target::Shape(index) => service
            .shape(index)?
            .ok_or(DecodeError::Invalid("missing required member target"))?
            .kind()
            .map_err(DecodeError::from),
        _ => Err(DecodeError::Unsupported("required member target")),
    }
}

fn zero(kind: &str) -> Result<ValueRef<'static>, DecodeError> {
    Ok(match kind {
        "string" | "enum" => ValueRef::Text(""),
        "blob" => ValueRef::Bytes(&[]),
        "boolean" => ValueRef::Bool(false),
        "byte" | "short" | "integer" | "long" | "bigInteger" | "intEnum" => ValueRef::Int(0),
        "float" | "double" | "bigDecimal" => ValueRef::Float(Float::new(0.0)),
        "timestamp" => ValueRef::Time(Time::new(0, 0)?),
        _ => return Err(DecodeError::Invalid("uncorrectable required member")),
    })
}

enum Task<'a, L, M>
where
    L: Iterator<Item = Result<ValueRef<'a>, RecordError>>,
    M: Iterator<Item = Result<(KeyRef<'a>, ValueRef<'a>), RecordError>>,
{
    Emit(Option<KeyRef<'a>>, ValueRef<'a>),
    List(L),
    Map(M),
}

fn emit(
    builder: &mut RecordBuilder,
    key: Option<KeyRef<'_>>,
    value: ValueRef<'_>,
) -> Result<(), DecodeError> {
    let mut parents = Vec::new();
    let mut next = Some(Task::Emit(key, value));
    while let Some(task) = next.take().or_else(|| parents.pop()) {
        match task {
            Task::Emit(key, ValueRef::List(list)) => {
                builder.begin_list(key)?;
                next = Some(Task::List(list.iter()?));
            }
            Task::Emit(key, ValueRef::Map(map)) => {
                builder.begin_map(key)?;
                next = Some(Task::Map(map.iter()?));
            }
            Task::Emit(key, scalar) => builder.scalar(key, scalar)?,
            task => next = advance(builder, &mut parents, task)?,
        }
    }
    Ok(())
}

fn advance<'a, L, M>(
    builder: &mut RecordBuilder,
    parents: &mut Vec<Task<'a, L, M>>,
    mut task: Task<'a, L, M>,
) -> Result<Option<Task<'a, L, M>>, DecodeError>
where
    L: Iterator<Item = Result<ValueRef<'a>, RecordError>>,
    M: Iterator<Item = Result<(KeyRef<'a>, ValueRef<'a>), RecordError>>,
{
    let Some((key, value)) = next_child(&mut task)? else {
        builder.end()?;
        return Ok(None);
    };
    parents.try_reserve(1)?;
    parents.push(task);
    Ok(Some(Task::Emit(key, value)))
}

fn next_child<'a, L, M>(
    task: &mut Task<'a, L, M>,
) -> Result<Option<(Option<KeyRef<'a>>, ValueRef<'a>)>, DecodeError>
where
    L: Iterator<Item = Result<ValueRef<'a>, RecordError>>,
    M: Iterator<Item = Result<(KeyRef<'a>, ValueRef<'a>), RecordError>>,
{
    match task {
        Task::List(items) => Ok(items.next().transpose()?.map(|value| (None, value))),
        Task::Map(items) => Ok(items
            .next()
            .transpose()?
            .map(|(key, value)| (Some(key), value))),
        Task::Emit(_, _) => Err(DecodeError::Invalid("default traversal state")),
    }
}
