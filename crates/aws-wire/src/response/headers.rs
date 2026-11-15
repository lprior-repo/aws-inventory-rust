use super::{DecodeError, ResponseContext, defaults};
use inventory_core::{
    catalog::{HttpBinding, Member, Shape, Target},
    record::{KeyRef, RecordBuilder, ValueRef},
};
mod items;
mod values;

pub(super) fn decode(
    context: ResponseContext<'_>,
    shape: Shape<'_>,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    for member in shape.members()? {
        decode_member(context, member?, builder)?;
    }
    Ok(())
}

pub(super) fn decode_member(
    context: ResponseContext<'_>,
    member: Member<'_>,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    let wire = member.wire()?;
    match wire.binding()? {
        HttpBinding::Header(name) => single_header(context, name, member, builder),
        HttpBinding::PrefixHeaders(prefix) => prefix_headers(context, prefix, member, builder),
        HttpBinding::ResponseCode => {
            builder.scalar(
                Some(KeyRef::Field(member.id)),
                ValueRef::Int(i64::from(context.status.as_u16())),
            )?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn prefix_headers(
    context: ResponseContext<'_>,
    prefix: &str,
    member: Member<'_>,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    let model = prefix_model(context, member)?;
    builder.begin_map(Some(KeyRef::Field(member.id)))?;
    for name in context.headers.keys() {
        let Some(suffix) = suffix(name.as_str(), prefix) else {
            continue;
        };
        values::write(
            context,
            name.as_str(),
            Some(KeyRef::Name(suffix)),
            &model,
            builder,
        )?;
    }
    builder.end()?;
    Ok(())
}

fn map_shape(context: ResponseContext<'_>, target: Target) -> Result<Shape<'_>, DecodeError> {
    let Target::Shape(index) = target else {
        return Err(DecodeError::Invalid("prefix headers must target a map"));
    };
    let shape = context
        .service
        .shape(index)?
        .ok_or(DecodeError::Invalid("missing prefix header map"))?;
    if shape.kind()? != "map" {
        return Err(DecodeError::Invalid("prefix headers must target a map"));
    }
    Ok(shape)
}

fn suffix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
    name.get(..prefix.len())?
        .eq_ignore_ascii_case(prefix)
        .then_some(())?;
    name.get(prefix.len()..)
}

fn single_header(
    context: ResponseContext<'_>,
    name: &str,
    member: Member<'_>,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    if !context.headers.contains_key(name) {
        return defaults::apply_member(builder, member, context.service, context.limits);
    }
    let model = values::model(
        context.service,
        member.target,
        member.wire()?.timestamp_format()?,
    )?;
    values::write(
        context,
        name,
        Some(KeyRef::Field(member.id)),
        &model,
        builder,
    )
}

fn prefix_model<'a>(
    context: ResponseContext<'a>,
    member: Member<'_>,
) -> Result<values::Model<'a>, DecodeError> {
    let map = map_shape(context, member.target)?;
    let target = map
        .map_value()?
        .ok_or(DecodeError::Invalid("missing prefix header value"))?;
    values::model(
        context.service,
        target,
        map.map_value_wire()?.timestamp_format()?,
    )
}
