use super::*;

pub(super) fn parse_meta(
    cells: &[&str],
    line: usize,
    state: &mut ParseState,
) -> Result<(), KindMapError> {
    let [_, key, value] = cells else {
        return Err(row_error(line, "metadata requires key and value"));
    };
    let valid = match *key {
        "version" if state.version.is_none() => set_version(value, state),
        "reviewer" if state.reviewer.is_none() && valid_reviewer(value) => {
            state.reviewer = Some((*value).to_owned());
            true
        }
        "reviewed" if state.reviewed.is_none() && valid_date(value) => {
            state.reviewed = Some((*value).to_owned());
            true
        }
        _ => false,
    };
    valid
        .then_some(())
        .ok_or_else(|| row_error(line, "duplicate or invalid metadata"))
}

fn set_version(value: &str, state: &mut ParseState) -> bool {
    let valid = value.parse::<u16>().is_ok_and(|parsed| parsed == 1);
    if valid {
        state.version = Some(1);
    }
    valid
}

fn valid_reviewer(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
}

pub(super) fn parse_cfn(
    cells: &[&str],
    line: usize,
    exceptions: &Exceptions,
    state: &mut ParseState,
) -> Result<(), KindMapError> {
    let [_, cfn, action, value, display_path] = cells else {
        return Err(row_error(line, "invalid or duplicate CFN type"));
    };
    if !valid_cfn_type(cfn) || !state.keys.insert((*cfn).to_owned()) {
        return Err(row_error(line, "invalid or duplicate CFN type"));
    }
    let display_name_path = display_path_value(display_path, line)?;
    let cells = (*cfn, *action, *value);
    let target = cfn_target(cells, display_name_path.is_none(), exceptions, state, line)?;
    let cfn_type = (*cfn).to_owned();
    state.entries.push(KindEntry {
        cfn_type,
        target,
        display_name_path,
    });
    Ok(())
}

fn cfn_target(
    (cfn, action, value): (&str, &str, &str),
    no_display: bool,
    exceptions: &Exceptions,
    state: &mut ParseState,
    line: usize,
) -> Result<KindTarget, KindMapError> {
    match action {
        "map" => mapped(cfn, value, exceptions, &mut state.used, line),
        "unmapped" if valid_reason(value) && no_display => Ok(KindTarget::Unmapped {
            reason: value.to_owned(),
        }),
        _ => Err(row_error(
            line,
            "mapping must be explicit map or unmapped with reason",
        )),
    }
}

fn display_path_value(display_path: &str, line: usize) -> Result<Option<String>, KindMapError> {
    match display_path {
        "-" => Ok(None),
        path if valid_path(path) => Ok(Some(path.to_owned())),
        _ => Err(row_error(line, "invalid explicit display name path")),
    }
}

pub(super) fn parse_api(
    cells: &[&str],
    line: usize,
    exceptions: &Exceptions,
    state: &mut ParseState,
) -> Result<(), KindMapError> {
    let [_, kind, marker, reason] = cells else {
        return Err(row_error(line, "API-only row requires kind and a reason"));
    };
    let fresh = valid_cfn_type(kind) && state.keys.insert((*kind).to_owned());
    if !fresh || !valid_reason(reason) || *marker != "kind" {
        return Err(row_error(line, "API-only row requires kind and a reason"));
    }
    let table_name = sql_name_used(kind, exceptions, &mut state.used).map_err(|source| {
        let kind = (*kind).to_owned();
        KindMapError::Name { kind, source }
    })?;
    let (kind, reason) = ((*kind).to_owned(), (*reason).to_owned());
    state.api_only.push(ApiOnlyKind {
        kind,
        table_name,
        reason,
    });
    Ok(())
}

pub(super) fn mapped(
    cfn: &str,
    kind: &str,
    exceptions: &Exceptions,
    used: &mut Vec<bool>,
    line: usize,
) -> Result<KindTarget, KindMapError> {
    if !valid_cfn_type(kind) {
        return Err(row_error(line, "invalid canonical kind"));
    }
    let name = sql_name_used(cfn, exceptions, used).map_err(|source| KindMapError::Name {
        kind: cfn.to_owned(),
        source,
    })?;
    Ok(KindTarget::Mapped {
        kind: kind.to_owned(),
        table_name: name,
    })
}
pub(super) fn parse_relation(
    cells: &[&str],
    line: usize,
    relations: &mut Vec<ReviewedRelation>,
) -> Result<(), KindMapError> {
    let fields: [&str; 6] = cells
        .get(1..)
        .and_then(|rest| rest.try_into().ok())
        .ok_or_else(|| row_error(line, "invalid reviewed relation"))?;
    let relation =
        reviewed_relation(fields).ok_or_else(|| row_error(line, "invalid reviewed relation"))?;
    relations.push(relation);
    Ok(())
}

fn reviewed_relation(fields: [&str; 6]) -> Option<ReviewedRelation> {
    let [
        source_kind,
        source_path,
        target_kind,
        target_path,
        relation,
        scope,
    ] = fields;
    let valid = valid_cfn_type(source_kind)
        && valid_path(source_path)
        && valid_cfn_type(target_kind)
        && valid_path(target_path)
        && valid_label(relation)
        && valid_label(scope);
    valid.then(|| ReviewedRelation {
        source_kind: source_kind.to_owned(),
        source_path: source_path.to_owned(),
        target_kind: target_kind.to_owned(),
        target_path: target_path.to_owned(),
        relation: relation.to_owned(),
        scope: scope.to_owned(),
    })
}
pub(super) fn validate_canonical_kinds(
    entries: &[KindEntry],
    api_only: &[ApiOnlyKind],
) -> Result<(), KindMapError> {
    let mapped = entries.iter().filter_map(|entry| match &entry.target {
        KindTarget::Mapped { kind, .. } => Some(kind.as_str()),
        KindTarget::Unmapped { .. } => None,
    });
    let kinds = mapped.chain(api_only.iter().map(|entry| entry.kind.as_str()));
    let mut seen = BTreeSet::new();
    kinds.into_iter().try_for_each(|kind| {
        if seen.insert(kind.to_owned()) {
            return Ok(());
        }
        Err(KindMapError::DuplicateCanonicalKind(kind.to_owned()))
    })
}
pub(super) fn validate_collisions(
    entries: &[KindEntry],
    api_only: &[ApiOnlyKind],
) -> Result<(), KindMapError> {
    validate_canonical_kinds(entries, api_only)?;
    let mapped = entries.iter().filter_map(|entry| match &entry.target {
        KindTarget::Mapped { table_name, .. } => Some(table_name.as_str()),
        KindTarget::Unmapped { .. } => None,
    });
    let names = mapped.chain(api_only.iter().map(|entry| entry.table_name.as_str()));
    let mut seen = BTreeSet::new();
    names.into_iter().try_for_each(|name| {
        if seen.insert(name.to_owned()) {
            Ok(())
        } else {
            Err(KindMapError::Duplicate(name.to_owned()))
        }
    })
}
