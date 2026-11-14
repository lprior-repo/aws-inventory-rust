use super::{DecodeError, ResponseContext, items};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use inventory_core::{
    catalog::{Service, Shape, Target, TimestampFormat},
    record::{Float, KeyRef, RecordBuilder, ValueRef},
};

struct Spec<'a> {
    kind: &'a str,
    format: Option<TimestampFormat>,
    base64: bool,
}
pub(super) struct Model<'a> {
    scalar: Spec<'a>,
    list: bool,
}

pub(super) fn model<'a>(
    service: Service<'a>,
    target: Target,
    format: Option<TimestampFormat>,
) -> Result<Model<'a>, DecodeError> {
    let Target::Shape(index) = target else {
        return Ok(Model {
            scalar: scalar_model(service, target, format)?,
            list: false,
        });
    };
    let shape = service
        .shape(index)?
        .ok_or(DecodeError::Invalid("missing header shape"))?;
    shape_model(service, shape, format)
}

fn scalar_model<'a>(
    service: Service<'a>,
    target: Target,
    format: Option<TimestampFormat>,
) -> Result<Spec<'a>, DecodeError> {
    match target {
        Target::Prelude(kind) => Ok(Spec {
            kind,
            format,
            base64: false,
        }),
        Target::Shape(index) => {
            let shape = service
                .shape(index)?
                .ok_or(DecodeError::Invalid("missing header scalar"))?;
            shape_spec(shape, shape.kind()?, format)
        }
        _ => Err(DecodeError::Unsupported("header scalar target")),
    }
}

fn shape_spec<'a>(
    shape: Shape<'a>,
    kind: &'a str,
    format: Option<TimestampFormat>,
) -> Result<Spec<'a>, DecodeError> {
    let wire = shape.wire()?;
    Ok(Spec {
        kind,
        format: format.or(wire.timestamp_format()?),
        base64: wire.media_type()?.is_some(),
    })
}

pub(super) fn write(
    context: ResponseContext<'_>,
    name: &str,
    key: Option<KeyRef<'_>>,
    model: &Model<'_>,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    if model.list {
        builder.begin_list(key)?;
        let date = model.scalar.http_date();
        for value in context.headers.get_all(name) {
            let text = bounded_text(value, context)?;
            items::visit(text, date, |item| {
                scalar(builder, None, item, &model.scalar)
            })?;
        }
        builder.end()?;
    } else if let Some(value) = context.headers.get(name) {
        scalar(builder, key, bounded_text(value, context)?, &model.scalar)?;
    }
    Ok(())
}

fn bounded_text<'a>(
    value: &'a http::HeaderValue,
    context: ResponseContext<'_>,
) -> Result<&'a str, DecodeError> {
    if value.as_bytes().len() > context.limits.body_bytes() {
        return Err(DecodeError::Limit("response header bytes"));
    }
    std::str::from_utf8(value.as_bytes()).map_err(|_| DecodeError::Invalid("HTTP header UTF-8"))
}

fn scalar(
    builder: &mut RecordBuilder,
    key: Option<KeyRef<'_>>,
    text: &str,
    model: &Spec<'_>,
) -> Result<(), DecodeError> {
    let value = match model.kind {
        "string" | "enum" if !model.base64 => ValueRef::Text(text),
        "string" | "enum" | "blob" => return decoded(builder, key, text, model.kind),
        "timestamp" => ValueRef::Time(crate::response::timestamp::parse(
            text.trim(),
            model.format.or(Some(TimestampFormat::HttpDate)),
        )?),
        "boolean" => ValueRef::Bool(boolean(text.trim())?),
        "byte" | "short" | "integer" | "long" | "bigInteger" | "intEnum" => {
            ValueRef::Int(integer(text)?)
        }
        "float" | "double" => ValueRef::Float(Float::new(floating(text)?)),
        _ => return Err(DecodeError::Unsupported("HTTP header scalar kind")),
    };
    builder.scalar(key, value)?;
    Ok(())
}

fn decoded(
    builder: &mut RecordBuilder,
    key: Option<KeyRef<'_>>,
    text: &str,
    kind: &str,
) -> Result<(), DecodeError> {
    let bytes = STANDARD
        .decode(text)
        .map_err(|_| DecodeError::Invalid("HTTP header base64"))?;
    let value = if kind == "blob" {
        ValueRef::Bytes(&bytes)
    } else {
        ValueRef::Text(
            std::str::from_utf8(&bytes).map_err(|_| DecodeError::Invalid("HTTP header UTF-8"))?,
        )
    };
    builder.scalar(key, value)?;
    Ok(())
}

fn shape_model<'a>(
    service: Service<'a>,
    shape: Shape<'a>,
    format: Option<TimestampFormat>,
) -> Result<Model<'a>, DecodeError> {
    let kind = shape.kind()?;
    if kind != "list" {
        return Ok(Model {
            scalar: shape_spec(shape, kind, format)?,
            list: false,
        });
    }
    let target = shape
        .element()?
        .ok_or(DecodeError::Invalid("missing header list element"))?;
    let format = format.or(shape.element_wire()?.timestamp_format()?);
    Ok(Model {
        scalar: scalar_model(service, target, format)?,
        list: true,
    })
}

fn boolean(text: &str) -> Result<bool, DecodeError> {
    match text {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(DecodeError::Invalid("HTTP header boolean")),
    }
}

impl Spec<'_> {
    fn http_date(&self) -> bool {
        self.kind == "timestamp" && matches!(self.format, None | Some(TimestampFormat::HttpDate))
    }
}

fn integer(text: &str) -> Result<i64, DecodeError> {
    text.trim()
        .parse()
        .map_err(|_| DecodeError::Invalid("HTTP header integer"))
}

fn floating(text: &str) -> Result<f64, DecodeError> {
    text.trim()
        .parse()
        .map_err(|_| DecodeError::Invalid("HTTP header float"))
}
