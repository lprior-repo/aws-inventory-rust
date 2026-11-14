mod parser;
mod scalar;

use super::{DecodeError, ResponseContext};
use crate::request::Protocol;
use inventory_core::{
    catalog::Target,
    record::{Record, RecordBuilder},
};

pub fn decode(context: ResponseContext<'_>) -> Result<Record, DecodeError> {
    validate_context(context)?;
    let target = context.operation.output()?;
    let mut builder = RecordBuilder::new(
        context.limits.body_bytes(),
        context.limits.nodes(),
        context.limits.depth(),
    )?;
    builder.begin_map(None)?;
    decode_output(context, target, &mut builder)?;
    builder.end()?;
    Ok(builder.finish()?)
}

fn validate_context(context: ResponseContext<'_>) -> Result<(), DecodeError> {
    if !context.status.is_success() {
        return Err(DecodeError::Invalid(
            "non-success response belongs to error decoder",
        ));
    }
    if !matches!(
        context.protocol,
        Protocol::AwsJson10 | Protocol::AwsJson11 | Protocol::RestJson1
    ) {
        return Err(DecodeError::Unsupported("JSON response protocol"));
    }
    if context.body.len() > context.limits.body_bytes() {
        return Err(DecodeError::Limit("response body"));
    }
    Ok(())
}

fn decode_output(
    context: ResponseContext<'_>,
    target: Target,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    match target {
        Target::Shape(index) => {
            let shape = context
                .service
                .shape(index)?
                .ok_or(DecodeError::Invalid("missing output shape"))?;
            if let Some(member) = parser::payload_member(shape)? {
                parser::decode_payload(context, member, builder)?;
            } else {
                parser::decode_object(context, target, builder)?;
            }
            crate::response::headers::decode(context, shape, builder)
        }
        Target::Prelude("unit") => parser::decode_object(context, target, builder),
        Target::Prelude(_) => Err(DecodeError::Unsupported("non-structure output")),
        _ => Err(DecodeError::Unsupported("non-exhaustive output target")),
    }
}
