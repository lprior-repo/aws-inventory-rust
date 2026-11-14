//! Pure per-model counters; member names stay in the model until accumulation.

use std::path::Path;

use super::{
    Error,
    model::{ModelFile, Shape, ShapeKind},
};

const READ_PREFIXES: [&str; 9] = [
    "List", "Describe", "Get", "BatchGet", "Search", "Scan", "Query", "Lookup", "Select",
];

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct ModelStats {
    pub operations: usize,
    pub read_named: usize,
    pub readonly: usize,
    pub list_describe: usize,
    pub list_describe_root: usize,
    pub paginated: usize,
    pub resources: usize,
    pub arn_templates: usize,
    pub members: usize,
}

pub(super) fn compute_stats(model: &ModelFile) -> ModelStats {
    model
        .shapes
        .iter()
        .fold(ModelStats::default(), |mut stats, (id, shape)| {
            stats.arn_templates = stats
                .arn_templates
                .saturating_add(usize::from(shape.traits.contains_key("aws.api#arn")));
            match &shape.kind {
                ShapeKind::Operation { .. } => count_operation(&mut stats, id, shape, model),
                ShapeKind::Resource => stats.resources = stats.resources.saturating_add(1),
                ShapeKind::Structure { members } | ShapeKind::Union { members } => {
                    stats.members = stats.members.saturating_add(members.len());
                }
                _ => {}
            }
            stats
        })
}

pub(super) fn service<'a>(model: &'a ModelFile, file: &Path) -> Result<&'a Shape, Error> {
    let mut services = model
        .shapes
        .values()
        .filter(|shape| matches!(&shape.kind, ShapeKind::Service));
    let first = services
        .next()
        .ok_or_else(|| Error::NoService(file.to_path_buf()))?;
    if services.next().is_some() {
        return Err(Error::SeveralServices(file.to_path_buf()));
    }
    Ok(first)
}

fn count_operation(stats: &mut ModelStats, id: &str, shape: &Shape, model: &ModelFile) {
    let name = id.rsplit_once('#').map_or(id, |(_, name)| name);
    let list_describe = name.starts_with("List") || name.starts_with("Describe");
    stats.operations = stats.operations.saturating_add(1);
    increment(
        &mut stats.read_named,
        READ_PREFIXES.iter().any(|prefix| name.starts_with(prefix)),
    );
    increment(
        &mut stats.readonly,
        shape.traits.contains_key("smithy.api#readonly"),
    );
    increment(
        &mut stats.paginated,
        shape.traits.contains_key("smithy.api#paginated"),
    );
    increment(&mut stats.list_describe, list_describe);
    increment(
        &mut stats.list_describe_root,
        list_describe && input_has_no_required(model, shape),
    );
}

fn increment(counter: &mut usize, condition: bool) {
    *counter = counter.saturating_add(usize::from(condition));
}

fn input_has_no_required(model: &ModelFile, operation: &Shape) -> bool {
    let ShapeKind::Operation { input } = &operation.kind else {
        return true;
    };
    let target = input
        .as_ref()
        .filter(|input| input.target != "smithy.api#Unit");
    match target
        .and_then(|input| model.shapes.get(&input.target))
        .map(|shape| &shape.kind)
    {
        Some(ShapeKind::Structure { members }) => members
            .values()
            .all(|member| !member.traits.contains_key("smithy.api#required")),
        _ => true,
    }
}
