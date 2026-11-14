use super::{
    DecodeError, Document, HttpBinding, Member, Protocol, ResponseContext, Shape, Target,
    element_name, shape,
};
use inventory_core::catalog::XmlNamespace;

pub(super) fn content(
    document: &Document<'_>,
    context: ResponseContext<'_>,
    output: Shape<'_>,
) -> Result<usize, DecodeError> {
    if !document.has_root {
        return Ok(document.root);
    }
    match context.protocol {
        Protocol::AwsQuery | Protocol::Ec2Query => query(document, context),
        Protocol::RestXml => rest(document, context, output),
        _ => Err(DecodeError::Unsupported("non-XML protocol")),
    }
}

fn query(document: &Document<'_>, context: ResponseContext<'_>) -> Result<usize, DecodeError> {
    document.validate_namespace(document.root, context.service.wire()?.xml_namespace()?)?;
    let operation = context.operation.name()?;
    if element_name(document, document.root)?.strip_suffix("Response") != Some(operation) {
        return Err(DecodeError::Invalid("query response root"));
    }
    if context.protocol == Protocol::Ec2Query {
        return Ok(document.root);
    }
    let cursor = document
        .root
        .checked_add(1)
        .ok_or(DecodeError::Invalid("XML element"))?;
    let child = document
        .next_child(document.root, cursor)
        .ok_or(DecodeError::Invalid("query result wrapper"))?;
    if element_name(document, child)?.strip_suffix("Result") != Some(operation) {
        return Err(DecodeError::Invalid("query result wrapper"));
    }
    Ok(child)
}

fn rest(
    document: &Document<'_>,
    context: ResponseContext<'_>,
    output: Shape<'_>,
) -> Result<usize, DecodeError> {
    let payload = payload(output)?;
    let expected = expected_name(context, output, payload)?;
    document.validate_namespace(document.root, namespace(context, output, payload)?)?;
    if element_name(document, document.root)? != expected {
        return Err(DecodeError::Invalid("REST XML root"));
    }
    Ok(document.root)
}

fn payload(output: Shape<'_>) -> Result<Option<Member<'_>>, DecodeError> {
    for item in output.members()? {
        let member = item?;
        if member.wire()?.binding()? == HttpBinding::Payload {
            return Ok(Some(member));
        }
    }
    Ok(None)
}

fn expected_name<'a>(
    context: ResponseContext<'a>,
    output: Shape<'a>,
    payload: Option<Member<'a>>,
) -> Result<&'a str, DecodeError> {
    match payload {
        Some(member) => match member.wire()?.xml_name()? {
            Some(name) => Ok(name),
            None => shape_name(context, member.target),
        },
        None => Ok(output
            .wire()?
            .xml_name()?
            .map_or(output.name()?, |name| name)),
    }
}

fn namespace<'a>(
    context: ResponseContext<'a>,
    output: Shape<'a>,
    payload: Option<Member<'a>>,
) -> Result<Option<XmlNamespace<'a>>, DecodeError> {
    let namespace = match payload {
        Some(member) => member
            .wire()?
            .xml_namespace()?
            .or(target_namespace(context, member.target)?),
        None => output.wire()?.xml_namespace()?,
    };
    Ok(namespace.or(context.service.wire()?.xml_namespace()?))
}

fn shape_name<'a>(context: ResponseContext<'a>, target: Target) -> Result<&'a str, DecodeError> {
    let shape = shape(context, target)?;
    Ok(shape.wire()?.xml_name()?.map_or(shape.name()?, |name| name))
}

fn target_namespace<'a>(
    context: ResponseContext<'a>,
    target: Target,
) -> Result<Option<XmlNamespace<'a>>, DecodeError> {
    match target {
        Target::Shape(_) => Ok(shape(context, target)?.wire()?.xml_namespace()?),
        Target::Prelude(_) => Ok(None),
        _ => Err(DecodeError::Unsupported("XML namespace target")),
    }
}
