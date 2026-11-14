use super::super::{EncodeError, RequestContext};
use inventory_core::catalog::{HttpBinding, Shape};

pub(super) fn names<'a>(
    context: RequestContext<'a>,
    input: Shape<'a>,
) -> Result<Vec<&'a str>, EncodeError> {
    let mut names = Vec::new();
    for member in input.members()? {
        let member = member?;
        if let HttpBinding::Query(name) = member.wire()?.binding()? {
            insert(&mut names, name)?;
        }
    }
    let uri = context
        .operation
        .http()?
        .ok_or(EncodeError::Unsupported("operation without HTTP binding"))?
        .uri;
    add_literal_names(&mut names, uri)?;
    names.sort_unstable();
    names.dedup();
    Ok(names)
}

fn add_literal_names<'a>(names: &mut Vec<&'a str>, uri: &'a str) -> Result<(), EncodeError> {
    let Some((_, query)) = uri.split_once('?') else {
        return Ok(());
    };
    query
        .split('&')
        .try_for_each(|part| insert(names, part.split_once('=').map_or(part, |(name, _)| name)))
}

fn insert<'a>(names: &mut Vec<&'a str>, name: &'a str) -> Result<(), EncodeError> {
    names.try_reserve(1)?;
    names.push(name);
    Ok(())
}
