use std::path::Path;

use super::{
    Error,
    model::{ModelFile, Shape, ShapeKind},
};

pub(crate) const READ_PREFIXES: [&str; 9] = [
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

pub(super) fn validate_inputs(model: &ModelFile, file: &Path) -> Result<(), Error> {
    for (id, shape) in &model.shapes {
        let ShapeKind::Operation { input, .. } = &shape.kind else {
            continue;
        };
        let (service, operation) = id.split_once('#').unwrap_or(("", id.as_str()));
        crate::model_source::resolve_input(
            model,
            crate::model_source::InputSpec {
                service,
                operation,
                target: input.as_ref().map(|input| input.target.as_str()),
            },
        )
        .map_err(|source| Error::InvalidInput {
            file: file.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

pub(super) fn compute_stats(model: &ModelFile) -> Result<ModelStats, Error> {
    let mut stats = ModelStats::default();
    for (id, shape) in &model.shapes {
        stats.arn_templates = stats
            .arn_templates
            .saturating_add(usize::from(shape.traits.contains_key("aws.api#arn")));
        match &shape.kind {
            ShapeKind::Operation { .. } => count_operation(&mut stats, id, shape, model)?,
            ShapeKind::Resource { .. } => stats.resources = stats.resources.saturating_add(1),
            ShapeKind::Structure { members } | ShapeKind::Union { members } => {
                stats.members = stats.members.saturating_add(members.len());
            }
            _ => {}
        }
    }
    Ok(stats)
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

fn count_operation(
    stats: &mut ModelStats,
    id: &str,
    shape: &Shape,
    model: &ModelFile,
) -> Result<(), Error> {
    let (service, name) = id.split_once('#').unwrap_or(("", id));
    let ShapeKind::Operation { input, .. } = &shape.kind else {
        return Ok(());
    };
    let has_required = crate::model_source::resolve_input(
        model,
        crate::model_source::InputSpec {
            service,
            operation: name,
            target: input.as_ref().map(|input| input.target.as_str()),
        },
    )
    .map_err(Error::InputResolution)?;
    count_metrics(stats, name, shape, has_required);
    Ok(())
}

fn count_metrics(stats: &mut ModelStats, name: &str, shape: &Shape, has_required: bool) {
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
        list_describe && !has_required,
    );
}

fn increment(counter: &mut usize, condition: bool) {
    *counter = counter.saturating_add(usize::from(condition));
}
