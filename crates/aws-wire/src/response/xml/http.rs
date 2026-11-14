use super::{DecodeError, ResponseContext};
use inventory_core::catalog::{Target, TimestampFormat};

pub(super) fn scalar_target<'a>(
    context: ResponseContext<'a>,
    target: Target,
) -> Result<(&'a str, Option<TimestampFormat>), DecodeError> {
    match target {
        Target::Shape(index) => {
            let shape = context
                .service
                .shape(index)?
                .ok_or(DecodeError::Invalid("XML scalar shape"))?;
            Ok((shape.kind()?, shape.wire()?.timestamp_format()?))
        }
        Target::Prelude(kind) => Ok((kind, None)),
        _ => Err(DecodeError::Unsupported("XML scalar target")),
    }
}
