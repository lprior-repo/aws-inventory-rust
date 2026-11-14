use super::super::DecodeError;
use inventory_core::catalog::{Service, Shape, Target};

pub(super) fn element_target(
    frame: Option<Target>,
    service: Service<'_>,
) -> Result<Option<Target>, DecodeError> {
    match frame {
        Some(Target::Prelude(kind)) => Ok(Some(Target::Prelude(kind))),
        Some(target) => Ok(shape(target, service)?.element()?),
        None => Ok(None),
    }
}

pub(super) fn target_kind(
    target: Target,
    service: Service<'_>,
) -> Result<&'static str, DecodeError> {
    match target {
        Target::Shape(_) => Ok(shape(target, service)?.kind()?),
        Target::Prelude(kind) => Ok(kind),
        _ => Err(DecodeError::Unsupported("catalog target kind")),
    }
}

pub(super) fn shape_option<'a>(
    target: Target,
    service: Service<'a>,
) -> Result<Option<Shape<'a>>, DecodeError> {
    match target {
        Target::Shape(index) => service.shape(index).map_err(DecodeError::Catalog),
        Target::Prelude(_) => Ok(None),
        _ => Err(DecodeError::Unsupported("catalog target")),
    }
}

pub(super) fn shape<'a>(target: Target, service: Service<'a>) -> Result<Shape<'a>, DecodeError> {
    shape_option(target, service)?.ok_or(DecodeError::Unsupported("prelude structure shape"))
}

pub(super) fn invalid_cbor() -> DecodeError {
    DecodeError::Invalid("malformed CBOR")
}
