use crate::response::{
    DecodeError, ResponseContext,
    xml::{decode::element_name, index::Document},
};
use inventory_core::catalog::{Member, Shape, Target, XmlNamespace};

pub(super) fn target_shape<'a>(
    context: ResponseContext<'a>,
    target: Target,
) -> Result<Option<Shape<'a>>, DecodeError> {
    match target {
        Target::Shape(index) => context.service.shape(index).map_err(Into::into),
        Target::Prelude(_) => Ok(None),
        _ => Err(DecodeError::Unsupported("XML target")),
    }
}

pub(super) fn flattened_collection(shape: Option<Shape<'_>>) -> Result<bool, DecodeError> {
    let Some(shape) = shape else {
        return Ok(false);
    };
    match shape.kind()? {
        "list" => Ok(shape.wire()?.xml_flattened()? || shape.element_wire()?.xml_flattened()?),
        "map" => Ok(shape.wire()?.xml_flattened()?),
        _ => Ok(false),
    }
}

pub(super) fn member_name(member: Member<'_>) -> Result<&str, DecodeError> {
    let name = member
        .wire()?
        .xml_name()?
        .map_or(member.name()?, |name| name);
    Ok(local_name(name))
}

fn local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

pub(super) fn matching_children(
    document: &Document<'_>,
    parent: usize,
    expected: &str,
) -> Result<Vec<usize>, DecodeError> {
    let mut found = Vec::new();
    let mut cursor = parent
        .checked_add(1)
        .ok_or(DecodeError::Invalid("XML element"))?;
    while let Some(child) = document.next_child(parent, cursor) {
        if element_name(document, child)? == local_name(expected) {
            found.try_reserve(1)?;
            found.push(child);
        }
        cursor = document
            .mates
            .get(child)
            .copied()
            .ok_or(DecodeError::Invalid("XML element"))?
            .checked_add(1)
            .ok_or(DecodeError::Invalid("XML element"))?;
    }
    Ok(found)
}

pub(super) fn only_node(nodes: &[usize]) -> Result<usize, DecodeError> {
    nodes
        .first()
        .copied()
        .ok_or(DecodeError::Invalid("XML member"))
}

pub(super) fn validate_nodes(
    document: &Document<'_>,
    nodes: &[usize],
    namespace: Option<XmlNamespace<'_>>,
) -> Result<(), DecodeError> {
    nodes
        .iter()
        .try_for_each(|node| document.validate_namespace(*node, namespace))
}
