use crate::response::{DecodeError, ResponseContext, defaults, headers, xml::http};
use inventory_core::{
    catalog::{Member, Shape},
    record::{KeyRef, Record, RecordBuilder, ValueRef},
};

pub(in crate::response::xml) fn raw_payload(
    context: ResponseContext<'_>,
    output: Shape<'_>,
    payload: Member<'_>,
) -> Result<Record, DecodeError> {
    if context.body.len() > context.limits.body_bytes() {
        return Err(DecodeError::Limit("XML body"));
    }
    let mut builder = RecordBuilder::new(
        context.limits.body_bytes(),
        context.limits.nodes(),
        context.limits.depth(),
    )?;
    builder.begin_map(None)?;
    headers::decode(context, output, &mut builder)?;
    add_payload(&mut builder, payload, context)?;
    builder.end()?;
    builder.finish().map_err(Into::into)
}

fn add_payload(
    builder: &mut RecordBuilder,
    payload: Member<'_>,
    context: ResponseContext<'_>,
) -> Result<(), DecodeError> {
    if context.limits.excluded_members.contains(payload.id) {
        return Ok(());
    }
    if context.body.is_empty() {
        return defaults::apply_member(builder, payload, context.service, context.limits);
    }
    let (kind, _) = http::scalar_target(context, payload.target)?;
    let key = Some(KeyRef::Field(payload.id));
    match kind {
        "blob" => builder.scalar(key, ValueRef::Bytes(context.body))?,
        "string" | "enum" => {
            let text = std::str::from_utf8(context.body)
                .map_err(|_| DecodeError::Invalid("invalid XML payload text"))?;
            builder.scalar(key, ValueRef::Text(text))?;
        }
        _ => return Err(DecodeError::Invalid("raw XML payload target")),
    }
    Ok(())
}
