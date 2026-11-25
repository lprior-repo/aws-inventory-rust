use super::{Error, LegacyCatalog, LegacyType, Rule, model, pointer};
use crate::kind_map::{KindMap, KindTarget};
use crate::overlay::Scope;
use pointer::{ResolvedPointer, Traversal};
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    fmt::{self, Write as _},
};

pub(super) struct Output {
    pub rows: String,
    pub report: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Row {
    kind: String,
    path: String,
    target: String,
    target_path: String,
    label: String,
    scope: &'static str,
    traversal: Traversal,
}

enum Reason {
    Source,
    Pointer(pointer::Error),
    Target,
    TargetPath,
    UnknownLegacy,
    AmbiguousLegacy,
    AmbiguousSource,
    AmbiguousTarget,
    NoEnumeratedKind,
    AmbiguousIdentifier,
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source => f.write_str("type not callable"),
            Self::Pointer(error) => write!(f, "{error}"),
            Self::Target => f.write_str("target kind not callable"),
            Self::TargetPath => f.write_str("target ARN or identity member unavailable"),
            Self::UnknownLegacy => f.write_str("unresolved legacy target name"),
            Self::AmbiguousLegacy => f.write_str("ambiguous legacy target name"),
            Self::AmbiguousSource => f.write_str("ambiguous source endpoints"),
            Self::AmbiguousTarget => f.write_str("ambiguous target endpoints"),
            Self::NoEnumeratedKind => f.write_str("no enumerated kind has this identifier"),
            Self::AmbiguousIdentifier => {
                f.write_str("identifier matches multiple enumerated kinds")
            }
        }
    }
}

impl Reason {
    fn failure_key(&self) -> (u8, &str) {
        match self {
            Self::Pointer(pointer::Error::Segment(segment)) => (0, segment),
            Self::Pointer(pointer::Error::Invalid) => (1, ""),
            Self::Pointer(pointer::Error::Terminal) => (2, ""),
            _ => (3, ""),
        }
    }
}

#[derive(Default)]
struct TypeOutput<'a> {
    rows: BTreeMap<Row, &'a Rule>,
    live: usize,
    held: usize,
    unmapped: usize,
    details: Vec<String>,
    held_rules: Vec<String>,
    reasons: BTreeMap<String, usize>,
}

pub(super) fn build(
    catalogs: &[LegacyCatalog],
    kinds: &KindMap,
    endpoints: &model::Endpoints<'_, '_>,
) -> Result<Output, Error> {
    let types = catalogs
        .iter()
        .flat_map(|catalog| &catalog.resource_types)
        .collect::<Vec<_>>();
    let total = types
        .iter()
        .try_fold(0usize, |total, ty| {
            total.checked_add(ty.relationships.len())
        })
        .ok_or(Error::Invalid("rule count overflow"))?;
    if total > super::MAX_RULES || types.len() > super::MAX_RULES {
        return Err(Error::Invalid("more than 10000 rules or types"));
    }
    let targets = target_types(&types);
    let mut output = TypeOutput::default();
    let mut reports = BTreeMap::new();
    types.iter().try_for_each(|ty| {
        append_type(ty, kinds, endpoints, &targets, &mut output, &mut reports)
    })?;
    output.held_rules.sort();
    let reasons = output
        .reasons
        .iter()
        .map(|(reason, count)| format!("unmapped_reason\t{reason}\t{count}\n"))
        .collect::<String>();
    let report = format!(
        "relations mapped_live={} mapped_held={} unmapped={} total={total}\n{}mapped, held: path crosses a list (needs E2.2) count={}\n{}{}",
        output.live,
        output.held,
        output.unmapped,
        reports.into_values().collect::<String>(),
        output.held,
        reasons,
        output.held_rules.concat()
    );
    Ok(Output {
        rows: render_rows(&output.rows)?,
        report,
    })
}

fn append_type<'a>(
    ty: &'a LegacyType,
    kinds: &KindMap,
    endpoints: &model::Endpoints<'_, '_>,
    targets: &BTreeMap<&str, BTreeSet<&str>>,
    output: &mut TypeOutput<'a>,
    reports: &mut BTreeMap<&'a str, String>,
) -> Result<(), Error> {
    let mapped = remap_type(ty, kinds, endpoints, targets)?;
    let report = format!(
        "type\t{}\tmapped_live={}\tmapped_held={}\tunmapped={}\n{}",
        ty.type_name,
        mapped.live,
        mapped.held,
        mapped.unmapped,
        mapped.details.concat()
    );
    if reports.insert(&ty.type_name, report).is_some() {
        return Err(Error::Invalid("duplicate CloudFormation type"));
    }
    output.live = output.live.saturating_add(mapped.live);
    output.held = output.held.saturating_add(mapped.held);
    output.unmapped = output.unmapped.saturating_add(mapped.unmapped);
    mapped.reasons.into_iter().for_each(|(reason, count)| {
        let entry = output.reasons.entry(reason).or_default();
        *entry = entry.saturating_add(count);
    });
    output.rows.extend(mapped.rows);
    output.held_rules.extend(mapped.held_rules);
    Ok(())
}

fn render_rows(rows: &BTreeMap<Row, &Rule>) -> Result<String, Error> {
    rows.keys()
        .try_fold(String::new(), |mut text, row| {
            let tag = match row.traversal {
                Traversal::Scalar => "relation",
                Traversal::List => "relation-held",
            };
            writeln!(
                text,
                "{tag}\t{}\t{}\t{}\t{}\t{}\t{}",
                row.kind, row.path, row.target, row.target_path, row.label, row.scope
            )?;
            Ok::<_, std::fmt::Error>(text)
        })
        .map_err(|_| Error::Invalid("row formatting failed"))
}

fn remap_type<'a>(
    ty: &'a LegacyType,
    kinds: &KindMap,
    endpoints: &model::Endpoints<'_, '_>,
    targets: &BTreeMap<&str, BTreeSet<&str>>,
) -> Result<TypeOutput<'a>, Error> {
    let mut output = TypeOutput::default();
    let mut seen = BTreeSet::new();
    ty.relationships.iter().try_for_each(|rule| {
        validate_rule(rule)?;
        if !seen.insert((
            &rule.pointer,
            &rule.relation,
            &rule.target_kind,
            &rule.target_format,
        )) {
            return Err(Error::Invalid("duplicate source rule"));
        }
        match map_rule(ty, rule, kinds, endpoints, targets)? {
            Ok(row) => match output.rows.entry(row) {
                Entry::Vacant(entry) => {
                    match entry.key().traversal {
                        Traversal::Scalar => output.live = output.live.saturating_add(1),
                        Traversal::List => output.held = output.held.saturating_add(1),
                    }
                    entry.insert(rule);
                }
                Entry::Occupied(mut entry) => {
                    if rule.pointer < entry.get().pointer {
                        entry.insert(rule);
                    }
                }
            },
            Err(reason) => {
                output.unmapped = output.unmapped.saturating_add(1);
                let count = output.reasons.entry(reason.to_string()).or_default();
                *count = count.saturating_add(1);
                output.details.push(format!(
                    "unmapped\t{}\t{}\t{}\t{}\t{}\t{reason}\n",
                    ty.type_name, rule.pointer, rule.relation, rule.target_kind, rule.target_format
                ));
            }
        }
        Ok::<_, Error>(())
    })?;
    output.details.sort();
    output.held_rules = output
        .rows
        .iter()
        .filter(|(row, _)| row.traversal == Traversal::List)
        .map(|(row, rule)| {
            format!(
                "held\t{}\t{}\t{}\t{}\t{}\n",
                ty.type_name, rule.pointer, row.path, row.target, row.target_path
            )
        })
        .collect();
    Ok(output)
}

fn target_types<'a>(types: &[&'a LegacyType]) -> BTreeMap<&'a str, BTreeSet<&'a str>> {
    let mut targets = BTreeMap::<_, BTreeSet<_>>::new();
    for ty in types {
        for name in [&ty.view_name, &ty.kind]
            .into_iter()
            .filter(|name| !name.is_empty())
        {
            targets
                .entry(name.as_str())
                .or_default()
                .insert(ty.type_name.as_str());
        }
    }
    targets
}

fn legacy_target<'a>(
    targets: &BTreeMap<&str, BTreeSet<&'a str>>,
    name: &str,
) -> Result<&'a str, Reason> {
    let candidates = targets.get(name).ok_or(Reason::UnknownLegacy)?;
    if candidates.len() != 1 {
        return Err(Reason::AmbiguousLegacy);
    }
    candidates.first().copied().ok_or(Reason::UnknownLegacy)
}

fn canonical<'a>(kinds: &'a KindMap, cfn: &str) -> Option<&'a str> {
    match &kinds.lookup(cfn)?.target {
        KindTarget::Mapped { kind, .. } => Some(kind),
        KindTarget::Unmapped { .. } => None,
    }
}

fn map_rule(
    ty: &LegacyType,
    rule: &Rule,
    kinds: &KindMap,
    endpoints: &model::Endpoints<'_, '_>,
    targets: &BTreeMap<&str, BTreeSet<&str>>,
) -> Result<Result<Row, Reason>, Error> {
    let source = canonical(kinds, &ty.type_name)
        .and_then(|kind| endpoints.get(kind).map(|items| (kind, items)));
    let Some((kind, sources)) = source else {
        return Ok(Err(Reason::Source));
    };
    let path = source_path(sources, &rule.pointer)?;
    let path = match path {
        Ok(path) => path,
        Err(reason) => return Ok(Err(reason)),
    };
    let target_cfn = if rule.target_kind == "aws_reference" {
        let Some(member) = path.path.rsplit('.').next() else {
            return Ok(Err(Reason::NoEnumeratedKind));
        };
        match inferred_target(member, rule, kind, kinds, endpoints) {
            Ok(target) => target,
            Err(reason) => return Ok(Err(reason)),
        }
    } else {
        match legacy_target(targets, &rule.target_kind) {
            Ok(target) => target,
            Err(reason) => return Ok(Err(reason)),
        }
    };
    let target = canonical(kinds, target_cfn);
    let Some((target, items)) =
        target.and_then(|kind| endpoints.get(kind).map(|items| (kind, items)))
    else {
        return Ok(Err(Reason::Target));
    };
    let (target_path, scope) = match target_path(items, &rule.target_format)? {
        Ok(path) => path,
        Err(reason) => return Ok(Err(reason)),
    };
    Ok(Ok(Row {
        kind: kind.to_owned(),
        path: path.path,
        target: target.to_owned(),
        target_path: target_path.path,
        label: rule.relation.clone(),
        scope: match scope {
            Some(Scope::Account) => "account",
            Some(Scope::HomeRegion) => "home_region",
            _ => "account_region",
        },
        traversal: path.traversal.merge(target_path.traversal),
    }))
}

fn inferred_target<'b>(
    member: &str,
    rule: &Rule,
    source_kind: &str,
    kinds: &KindMap,
    endpoints: &model::Endpoints<'_, 'b>,
) -> Result<&'b str, Reason> {
    let candidates = endpoints
        .iter()
        .filter(|(kind, _)| **kind != source_kind)
        .filter(|(kind, items)| {
            canonical(kinds, kind).is_some()
                && items.iter().any(|endpoint| {
                    model::identity_path(endpoint.identity, &rule.target_format)
                        .and_then(|path| path.rsplit('.').next().map(str::to_owned))
                        .is_some_and(|identifier| identifier == member)
                })
        })
        .map(|(kind, _)| *kind)
        .collect::<BTreeSet<_>>();
    match candidates.len() {
        0 => Err(Reason::NoEnumeratedKind),
        1 => candidates.first().copied().ok_or(Reason::NoEnumeratedKind),
        _ => Err(Reason::AmbiguousIdentifier),
    }
}

#[derive(Default)]
struct Candidates {
    resolved: Option<(ResolvedPointer, Option<Scope>)>,
    failure: Option<Reason>,
    ambiguous: bool,
}

impl Candidates {
    fn record(&mut self, result: Result<ResolvedPointer, Reason>, scope: Option<Scope>) {
        match result {
            Ok(path) => {
                let candidate = (path, scope);
                if let Some(previous) = &self.resolved {
                    self.ambiguous |= *previous != candidate;
                } else {
                    self.resolved = Some(candidate);
                }
            }
            Err(reason) => {
                if self
                    .failure
                    .as_ref()
                    .is_none_or(|previous| reason.failure_key() < previous.failure_key())
                {
                    self.failure = Some(reason);
                }
            }
        }
    }

    fn finish(
        self,
        missing: Reason,
        ambiguous: Reason,
    ) -> Result<(ResolvedPointer, Option<Scope>), Reason> {
        if self.ambiguous {
            return Err(ambiguous);
        }
        self.resolved.ok_or_else(|| self.failure.unwrap_or(missing))
    }
}

fn target_path(
    items: &[model::Endpoint<'_, '_>],
    format: &str,
) -> Result<Result<(ResolvedPointer, Option<Scope>), Reason>, Error> {
    let mut candidates = Candidates::default();
    for endpoint in items {
        if let Some(path) = model::identity_path(endpoint.identity, format) {
            let pointer = format!("/{}", path.replace('.', "/"));
            if let Ok(path) = checked_pointer(endpoint, &pointer)? {
                candidates.record(Ok(path), endpoint.scope);
            }
        }
    }
    Ok(candidates.finish(Reason::TargetPath, Reason::AmbiguousTarget))
}

fn source_path(
    items: &[model::Endpoint<'_, '_>],
    pointer: &str,
) -> Result<Result<ResolvedPointer, Reason>, Error> {
    let mut candidates = Candidates::default();
    for endpoint in items {
        candidates.record(checked_pointer(endpoint, pointer)?, endpoint.scope);
    }
    Ok(candidates
        .finish(Reason::Source, Reason::AmbiguousSource)
        .map(|(path, _)| path))
}

fn checked_pointer(
    endpoint: &model::Endpoint<'_, '_>,
    pointer: &str,
) -> Result<Result<ResolvedPointer, Reason>, Error> {
    match pointer::resolve(endpoint.service, endpoint.item, pointer) {
        Ok(path) => Ok(Ok(path)),
        Err(pointer::Error::Catalog(error)) => Err(error.into()),
        Err(error) => Ok(Err(Reason::Pointer(error))),
    }
}

fn validate_rule(rule: &Rule) -> Result<(), Error> {
    let valid = [
        &rule.pointer,
        &rule.relation,
        &rule.target_kind,
        &rule.target_format,
    ]
    .into_iter()
    .all(|value| !value.is_empty() && value.len() <= 1024 && !value.chars().any(char::is_control));
    if valid {
        Ok(())
    } else {
        Err(Error::Invalid(
            "empty, oversized or control-containing rule field",
        ))
    }
}
