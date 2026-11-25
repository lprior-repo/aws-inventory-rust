#[cfg(test)]
mod review_tests;
#[cfg(test)]
mod tests;
mod values;

use super::{SeedError, command::directory};
use inventory_core::{
    catalog::runtime::{
        IdentityChoicePlan, IdentityFieldPlan, IdentityLeafPlan, IdentityPart, IdentityPlan,
        ItemPlan, PlanPurpose, PlanScope, RuntimeCatalog,
    },
    catalog::{Operation, Service, Target},
    record::FieldId,
};
use std::{collections::BTreeSet, io::Write, path::Path};
use toml::Value;
type Table = toml::map::Map<String, Value>;

#[derive(Clone)]
struct Identifier {
    kind: String,
    name: String,
    scope: PlanScope,
    root_list: bool,
}

struct Produced {
    step: String,
    name: String,
    path: String,
}

#[derive(Clone)]
struct Leaf {
    name: String,
    path: String,
}

struct Candidate<'a> {
    operation: Operation<'a>,
    identifier: Identifier,
    leaf: Leaf,
    source: Source,
    ambiguous: bool,
}

enum DraftOperation<'a> {
    Ready(Candidate<'a>),
    Disabled {
        operation: Operation<'a>,
        kind: Option<String>,
        reason: &'static str,
    },
}

enum IdentityTask {
    Visit(Target, String, String),
    Exit(usize),
}

const IDENTITY_BUDGET_REASON: &str = "draft identity traversal exceeds node budget";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Response,
    Request,
}

pub fn generate(runtime: &RuntimeCatalog<'_>, service_name: &str) -> Result<String, SeedError> {
    let service = runtime
        .catalog()
        .service(service_name)?
        .ok_or(SeedError::Invalid("draft service is missing from catalog"))?;
    let identifiers = identifiers(runtime, service_name)?;
    let header = Table::from_iter([
        (String::from("format"), Value::Integer(1)),
        (String::from("service"), Value::String(service_name.into())),
    ]);
    let mut text = toml::to_string(&header).map_err(serialization)?;
    let mut earlier = Vec::new();
    let mut considered = false;
    for operation_result in service.operations()? {
        let operation = operation_result?;
        match candidate(service, operation, &identifiers)? {
            Some(DraftOperation::Ready(candidate)) => {
                considered = true;
                append(&mut text, &mut earlier, service, &candidate, &identifiers)?;
            }
            Some(DraftOperation::Disabled {
                operation,
                kind,
                reason,
            }) => {
                considered = true;
                write_step(
                    &mut text,
                    disabled_table(operation, kind.as_deref())?,
                    Some(reason),
                )?;
            }
            None => {}
        }
    }
    if !considered {
        return Err(SeedError::Invalid("draft has no enabled steps"));
    }
    if text.len() > super::MAX_VALUE_BYTES {
        return Err(SeedError::Invalid("draft exceeds seed file size limit"));
    }
    Ok(text)
}

pub fn run(service: &str, catalog: &Path, output: Option<&Path>) -> Result<(), SeedError> {
    if let Some(path) = output {
        validate_output(path)?;
    }
    let bytes = directory::read_bounded(catalog, 67_108_864)?;
    let runtime = RuntimeCatalog::from_bytes(&bytes)?;
    let text = generate(&runtime, service)?;
    match output {
        Some(path) => std::fs::write(path, text)?,
        None => std::io::stdout().lock().write_all(text.as_bytes())?,
    }
    Ok(())
}

fn validate_output(path: &Path) -> Result<(), SeedError> {
    let seeds = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("seeds")
        .canonicalize()?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or(Path::new("."), |p| p)
        .canonicalize()?;
    if parent.starts_with(&seeds) {
        return Err(SeedError::Invalid(
            "draft output must be outside crates/harness/seeds",
        ));
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    if metadata
        .as_ref()
        .is_some_and(|metadata| metadata.is_symlink())
        && !path.exists()
    {
        return Err(SeedError::Invalid(
            "draft output symlink target cannot be resolved",
        ));
    }
    let destination = if metadata.is_some() {
        path.canonicalize()?
    } else {
        parent.join(
            path.file_name()
                .ok_or(SeedError::Invalid("draft output has no file name"))?,
        )
    };
    if destination.starts_with(seeds) {
        return Err(SeedError::Invalid(
            "draft output must be outside crates/harness/seeds",
        ));
    }
    Ok(())
}

fn identifiers(
    runtime: &RuntimeCatalog<'_>,
    service_name: &str,
) -> Result<Vec<Identifier>, SeedError> {
    runtime
        .plans()
        .iter()
        .try_fold(Vec::new(), |mut identifiers, plan| {
            if plan.service().name()? != service_name
                || !matches!(plan.purpose(), PlanPurpose::Enumeration)
            {
                return Ok(identifiers);
            }
            plan.resource_emissions().iter().try_for_each(|emission| {
                let fallback = match emission.items() {
                    ItemPlan::Root => None,
                    ItemPlan::Path(path) | ItemPlan::Select { path, .. } => path.last().copied(),
                };
                let names = match emission.identity() {
                    IdentityPlan::MapArn { key, .. } => vec![(key.clone(), false)],
                    identity => identity_paths(identity)
                        .into_iter()
                        .filter_map(|path| {
                            path.last()
                                .copied()
                                .or(fallback)
                                .map(|id| (id, path.is_empty()))
                        })
                        .map(|(id, root)| {
                            let name = plan.service().field_name(id)?;
                            let name = if root {
                                root_name(
                                    plan.service(),
                                    plan.operation().output()?,
                                    emission.items(),
                                )?
                                .unwrap_or_else(|| name.to_owned())
                            } else {
                                name.to_owned()
                            };
                            Ok::<_, SeedError>((name, root))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                };
                identifiers.extend(names.into_iter().map(|(name, root_list)| Identifier {
                    kind: emission.kind().as_str().into(),
                    name,
                    scope: plan.scope(),
                    root_list,
                }));
                Ok::<_, SeedError>(())
            })?;
            Ok(identifiers)
        })
}

fn identity_paths(identity: &IdentityPlan) -> Vec<&[FieldId]> {
    match identity {
        IdentityPlan::Arn(path) => vec![path],
        IdentityPlan::Key(paths) => paths.iter().map(AsRef::as_ref).collect(),
        IdentityPlan::Template(parts) => template_paths(parts),
        IdentityPlan::CorrelatedKey(fields) => field_paths(fields),
        IdentityPlan::Choice(IdentityChoicePlan::PreferArn { path, fallback, .. }) => {
            std::iter::once(path.as_ref())
                .chain(leaf_paths(fallback))
                .collect()
        }
        IdentityPlan::Choice(IdentityChoicePlan::Discriminated(branches)) => branches
            .iter()
            .flat_map(|branch| leaf_paths(&branch.identity))
            .collect(),
        IdentityPlan::MapArn { path, .. } => vec![path],
    }
}

fn template_paths(parts: &[IdentityPart]) -> Vec<&[FieldId]> {
    parts
        .iter()
        .filter_map(|part| match part {
            IdentityPart::Field(path) => Some(path.as_ref()),
            _ => None,
        })
        .collect()
}

fn leaf_paths(identity: &IdentityLeafPlan) -> Vec<&[FieldId]> {
    match identity {
        IdentityLeafPlan::Arn(path) => vec![path],
        IdentityLeafPlan::Key(paths) => paths.iter().map(AsRef::as_ref).collect(),
        IdentityLeafPlan::Template(parts) => template_paths(parts),
        IdentityLeafPlan::Fields(fields) => field_paths(fields),
    }
}

fn field_paths(fields: &[IdentityFieldPlan]) -> Vec<&[FieldId]> {
    fields
        .iter()
        .map(|field| match field {
            IdentityFieldPlan::Selected(path)
            | IdentityFieldPlan::SelectedDigest(path)
            | IdentityFieldPlan::Request(path)
            | IdentityFieldPlan::SelectedArn(path)
            | IdentityFieldPlan::RequestArn(path)
            | IdentityFieldPlan::SelectedInteger { path, .. }
            | IdentityFieldPlan::RequestInteger { path, .. }
            | IdentityFieldPlan::SelectedStringList { path, .. }
            | IdentityFieldPlan::Context { path, .. }
            | IdentityFieldPlan::MapValue { path, .. } => path.as_ref(),
        })
        .collect()
}

fn candidate<'a>(
    service: Service<'a>,
    operation: Operation<'a>,
    identifiers: &[Identifier],
) -> Result<Option<DraftOperation<'a>>, SeedError> {
    let name = operation.name()?;
    if !["Create", "Put", "Register"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return Ok(None);
    }
    match select(service, operation, identifiers) {
        Ok(Some(candidate))
            if candidate.identifier.root_list
                && candidate.leaf.name != candidate.identifier.name
                && !singular_matches(&candidate.identifier.name, operation_noun(name)) =>
        {
            Ok(Some(DraftOperation::Disabled {
                operation,
                kind: Some(candidate.identifier.kind.clone()),
                reason: "root-list identifier singular form does not match a create input or output",
            }))
        }
        Ok(Some(candidate))
            if matches!(
                candidate.identifier.scope,
                PlanScope::Region | PlanScope::Account
            ) =>
        {
            Ok(Some(DraftOperation::Ready(candidate)))
        }
        Ok(Some(_)) => Ok(Some(DraftOperation::Disabled {
            operation,
            kind: None,
            reason: "home-region scope is not representable in seed files",
        })),
        Ok(None) => Ok(None),
        Err(SeedError::Invalid(IDENTITY_BUDGET_REASON)) => Ok(Some(DraftOperation::Disabled {
            operation,
            kind: None,
            reason: IDENTITY_BUDGET_REASON,
        })),
        Err(error) => Err(error),
    }
}

fn select<'a>(
    service: Service<'a>,
    operation: Operation<'a>,
    identifiers: &[Identifier],
) -> Result<Option<Candidate<'a>>, SeedError> {
    let outputs = leaves(service, operation.output()?, false)?;
    let inputs = leaves(service, operation.input()?, true)?;
    let mut matches = Vec::new();
    for (source, leaves) in [(Source::Response, &outputs), (Source::Request, &inputs)] {
        for leaf in leaves {
            let matching = identifiers
                .iter()
                .filter(|id| identifier_matches(id, &leaf.name))
                .cloned()
                .collect::<Vec<_>>();
            matches.extend(matching.into_iter().map(|id| (source, leaf.clone(), id)));
        }
    }

    let noun = operation_noun(operation.name()?);
    let preferred = matches
        .iter()
        .filter(|(_, _, id)| {
            id.kind
                .rsplit("::")
                .next()
                .is_some_and(|kind| kind.eq_ignore_ascii_case(noun))
        })
        .collect::<Vec<_>>();
    let choices = if preferred.is_empty() {
        matches.iter().collect::<Vec<_>>()
    } else {
        preferred
    };
    let Some((source, leaf, identifier)) = choices.first().copied() else {
        return Ok(None);
    };
    let kinds = choices
        .iter()
        .map(|(_, _, id)| &id.kind)
        .collect::<BTreeSet<_>>();
    Ok(Some(Candidate {
        operation,
        identifier: identifier.clone(),
        leaf: leaf.clone(),
        source: *source,
        ambiguous: kinds.len() != 1,
    }))
}

fn operation_noun(name: &str) -> &str {
    ["Create", "Put", "Register"]
        .into_iter()
        .find_map(|prefix| name.strip_prefix(prefix))
        .unwrap_or(name)
}

fn singular_matches(plural: &str, singular: &str) -> bool {
    if plural == singular {
        return true;
    }
    if !plural.ends_with('s') {
        return false;
    }
    if plural.ends_with("ss") || plural.ends_with("us") || plural.ends_with("Series") {
        return false;
    }
    if let Some(stem) = plural.strip_suffix("ies")
        && singular.strip_suffix('y') == Some(stem)
    {
        return true;
    }
    if let Some(stem) = plural.strip_suffix("es") {
        if stem.ends_with("ss") {
            return stem == singular;
        }
        if stem == singular
            && ["s", "x", "z", "sh", "ch", "o"]
                .iter()
                .any(|suffix| stem.ends_with(suffix))
        {
            return true;
        }
    }
    plural.strip_suffix('s') == Some(singular)
}

fn identifier_matches(identifier: &Identifier, name: &str) -> bool {
    identifier.name == name || (identifier.root_list && singular_matches(&identifier.name, name))
}
fn root_name(
    service: Service<'_>,
    mut target: Target,
    items: &ItemPlan,
) -> Result<Option<String>, SeedError> {
    let path = match items {
        ItemPlan::Path(path) | ItemPlan::Select { path, .. } => path,
        ItemPlan::Root => return Ok(None),
    };
    for id in path {
        let Some(mut shape) = values::shape(service, target)? else {
            return Ok(None);
        };
        if shape.kind()? == "list" {
            let Some(element) = shape.element()? else {
                return Ok(None);
            };
            let Some(element) = values::shape(service, element)? else {
                return Ok(None);
            };
            shape = element;
        }
        let mut found = None;
        for member in shape.members()? {
            let member = member?;
            if member.id == *id {
                found = Some(member.target);
                break;
            }
        }
        let Some(next) = found else { return Ok(None) };
        target = next;
    }
    let Some(list) = values::shape(service, target)? else {
        return Ok(None);
    };
    let Some(element) = list.element()? else {
        return Ok(None);
    };
    let Some(element) = values::shape(service, element)? else {
        return Ok(None);
    };
    Ok(element.name()?.rsplit('#').next().map(str::to_owned))
}

fn leaves(
    service: Service<'_>,
    target: Target,
    required_only: bool,
) -> Result<Vec<Leaf>, SeedError> {
    let mut pending = vec![IdentityTask::Visit(target, String::new(), String::new())];
    let mut active = BTreeSet::new();
    let mut leaves = Vec::new();
    let mut visited = 0usize;
    while let Some(task) = pending.pop() {
        let (target, name, path) = match task {
            IdentityTask::Visit(target, name, path) => (target, name, path),
            IdentityTask::Exit(index) => {
                active.remove(&index);
                continue;
            }
        };
        visited = visited.saturating_add(1);
        if visited > 4096 {
            return Err(SeedError::Invalid(IDENTITY_BUDGET_REASON));
        }
        let shape = values::shape(service, target)?;
        if let Target::Shape(index) = target {
            if !active.insert(index) {
                continue;
            }
            pending.push(IdentityTask::Exit(index));
        }
        match shape {
            Some(shape) if shape.kind()? == "structure" => {
                identity_members(shape, &path, required_only, &mut pending)?
            }
            Some(shape) if shape.kind()? == "list" => pending.push(IdentityTask::Visit(
                shape
                    .element()?
                    .ok_or(SeedError::Invalid("draft list element is missing"))?,
                name,
                format!("{path}.0"),
            )),
            Some(shape) if matches!(shape.kind()?, "string" | "enum") => {
                leaves.push(Leaf { name, path })
            }
            None if matches!(target, Target::Prelude("string" | "enum")) => {
                leaves.push(Leaf { name, path })
            }
            _ => {}
        }
        if pending.len().saturating_add(visited) > 8192 {
            return Err(SeedError::Invalid(IDENTITY_BUDGET_REASON));
        }
    }
    Ok(leaves)
}

fn identity_members(
    shape: inventory_core::catalog::Shape<'_>,
    path: &str,
    required_only: bool,
    pending: &mut Vec<IdentityTask>,
) -> Result<(), SeedError> {
    let start = pending.len();
    for member in shape.members()? {
        let member = member?;
        if required_only && !member.required {
            continue;
        }
        let name = member.name()?.to_owned();
        let path = if path.is_empty() {
            name.clone()
        } else {
            format!("{path}.{name}")
        };
        pending.push(IdentityTask::Visit(member.target, name, path));
        if pending.len() > 4096 {
            return Err(SeedError::Invalid(IDENTITY_BUDGET_REASON));
        }
    }
    pending
        .get_mut(start..)
        .ok_or(SeedError::Invalid(IDENTITY_BUDGET_REASON))?
        .reverse();
    Ok(())
}

fn disabled_table(operation: Operation<'_>, kind: Option<&str>) -> Result<Table, SeedError> {
    Ok(Table::from_iter([
        (
            String::from("id"),
            Value::String(operation.name()?.to_ascii_lowercase()),
        ),
        (
            String::from("operation"),
            Value::String(operation.name()?.into()),
        ),
        (
            String::from("kind"),
            Value::String(kind.unwrap_or_default().into()),
        ),
    ]))
}

fn append(
    text: &mut String,
    earlier: &mut Vec<Produced>,
    service: Service<'_>,
    candidate: &Candidate<'_>,
    identifiers: &[Identifier],
) -> Result<bool, SeedError> {
    let id = candidate.operation.name()?.to_ascii_lowercase();
    let own_input = match candidate.source {
        Source::Request => Some(candidate.leaf.path.as_str()),
        Source::Response => None,
    };
    let synthesis = values::Synthesis {
        service,
        step: &id,
        identifiers,
        earlier,
        own_input,
    };
    let input = if candidate.ambiguous {
        Err(SeedError::Invalid(
            "create identifier matches multiple enumeration kinds",
        ))
    } else {
        synthesis.input(candidate.operation.input()?)
    };
    let mut step = step_table(&id, candidate)?;
    let reason = match input {
        Ok(input) => {
            step.insert(String::from("input"), input);
            None
        }
        Err(SeedError::Invalid(reason)) => Some(String::from(reason)),
        Err(SeedError::Message(reason)) => Some(reason),
        Err(error) => return Err(error),
    };
    let enabled = reason.is_none();
    write_step(text, step, reason.as_deref())?;
    if enabled {
        earlier.extend(
            leaves(service, candidate.operation.output()?, false)?
                .into_iter()
                .map(|leaf| Produced {
                    step: id.clone(),
                    name: leaf.name,
                    path: leaf.path,
                }),
        );
    }
    if text.len() > super::MAX_VALUE_BYTES {
        return Err(SeedError::Invalid("draft exceeds seed file size limit"));
    }
    Ok(enabled)
}

fn write_step(text: &mut String, step: Table, reason: Option<&str>) -> Result<(), SeedError> {
    let document =
        Table::from_iter([(String::from("step"), Value::Array(vec![Value::Table(step)]))]);
    let encoded = toml::to_string(&document).map_err(serialization)?;
    let Some(reason) = reason else {
        text.push_str(&encoded);
        return Ok(());
    };
    text.push_str("# Disabled: ");
    text.push_str(reason);
    text.push('\n');
    for line in encoded.lines() {
        text.push('#');
        if !line.is_empty() {
            text.push(' ');
            text.push_str(line);
        }
        text.push('\n');
    }
    Ok(())
}

fn step_table(id: &str, candidate: &Candidate<'_>) -> Result<Table, SeedError> {
    let identity = match candidate.source {
        Source::Response => Value::String(candidate.leaf.path.clone()),
        Source::Request => Value::Table(Table::from_iter([(
            String::from("input"),
            Value::String(candidate.leaf.path.clone()),
        )])),
    };
    Ok(Table::from_iter([
        (String::from("id"), Value::String(id.into())),
        (
            String::from("operation"),
            Value::String(candidate.operation.name()?.into()),
        ),
        (
            String::from("kind"),
            Value::String(candidate.identifier.kind.clone()),
        ),
        (String::from("identity"), identity),
        (
            String::from("scope"),
            Value::String(
                match candidate.identifier.scope {
                    PlanScope::Account => "account",
                    PlanScope::Region => "region",
                    _ => {
                        return Err(SeedError::Invalid(
                            "home-region scope is not representable in seed files",
                        ));
                    }
                }
                .into(),
            ),
        ),
    ]))
}

fn serialization(error: toml::ser::Error) -> SeedError {
    SeedError::Message(format!("draft serialization failed: {error}"))
}
