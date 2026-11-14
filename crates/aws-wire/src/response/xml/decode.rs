use super::{
    DecodeError, ResponseContext,
    index::{Document, Token},
    model::{self, Model},
};
use crate::request::Protocol;
use inventory_core::{
    catalog::{HttpBinding, Member, Shape, Target},
    record::{Record, RecordBuilder},
};

mod root;

pub(super) fn decode(context: ResponseContext<'_>) -> Result<Record, DecodeError> {
    validate_context(context)?;
    let target = context.operation.output()?;
    if target == Target::Prelude("unit") {
        return decode_unit(context);
    }
    let output = shape(context, target)?;
    if let Some(payload) = raw_payload(context, output)? {
        return model::raw_payload(context, output, payload);
    }
    let document = Document::parse(context)?;
    let content = root::content(&document, context, output)?;
    let mut builder = builder(context)?;
    Model::new(&document, context, &mut builder).run(content, target)?;
    builder.finish().map_err(Into::into)
}

fn validate_context(context: ResponseContext<'_>) -> Result<(), DecodeError> {
    if !matches!(
        context.protocol,
        Protocol::AwsQuery | Protocol::Ec2Query | Protocol::RestXml
    ) {
        return Err(DecodeError::Unsupported("non-XML protocol"));
    }
    if !context.status.is_success() {
        return Err(DecodeError::Invalid("XML response status"));
    }
    Ok(())
}

fn raw_payload<'a>(
    context: ResponseContext<'a>,
    output: Shape<'a>,
) -> Result<Option<Member<'a>>, DecodeError> {
    if context.protocol != Protocol::RestXml {
        return Ok(None);
    }
    for item in output.members()? {
        let member = item?;
        if member.wire()?.binding()? == HttpBinding::Payload
            && matches!(
                super::http::scalar_target(context, member.target)?.0,
                "blob" | "string" | "enum"
            )
        {
            return Ok(Some(member));
        }
    }
    Ok(None)
}

fn builder(context: ResponseContext<'_>) -> Result<RecordBuilder, DecodeError> {
    RecordBuilder::new(
        context.limits.body_bytes(),
        context.limits.nodes(),
        context.limits.depth(),
    )
    .map_err(Into::into)
}

fn decode_unit(context: ResponseContext<'_>) -> Result<Record, DecodeError> {
    Document::parse(context)?;
    let mut builder = builder(context)?;
    builder.begin_map(None)?;
    builder.end()?;
    builder.finish().map_err(Into::into)
}

fn shape<'a>(context: ResponseContext<'a>, target: Target) -> Result<Shape<'a>, DecodeError> {
    match target {
        Target::Shape(index) => context
            .service
            .shape(index)?
            .ok_or(DecodeError::Invalid("missing XML output shape")),
        Target::Prelude(_) => Err(DecodeError::Unsupported("prelude XML output")),
        _ => Err(DecodeError::Unsupported("XML output target")),
    }
}

pub(super) fn element_name<'a>(
    document: &'a Document<'_>,
    index: usize,
) -> Result<&'a str, DecodeError> {
    match document.tokens.get(index) {
        Some(Token::Start(start) | Token::Empty(start)) => Ok(start.local_name().into_inner()),
        _ => Err(DecodeError::Invalid("XML element")),
    }
}
