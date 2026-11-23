use std::collections::{BTreeMap, btree_map::Entry};
use std::fmt::Write;

use super::{Exceptions, MAX_SQL_NAME_BYTES, NameError, SqlName, derive_full_name};

pub fn render_names<'a>(
    type_names: &[&'a str],
    exceptions: &Exceptions,
) -> Result<Vec<(&'a str, SqlName)>, NameError> {
    render(type_names, exceptions, None)
}

pub(super) fn render_report(
    type_names: &[&str],
    exceptions: &Exceptions,
) -> Result<String, NameError> {
    let mut report = String::new();
    let rows = render(type_names, exceptions, Some(&mut report))?;
    let long = report.lines().count();
    writeln!(report, "names total={} long={long} distinct={}", rows.len(), rows.len())?;
    Ok(report)
}

fn render<'a>(
    type_names: &[&'a str],
    exceptions: &Exceptions,
    mut report: Option<&mut String>,
) -> Result<Vec<(&'a str, SqlName)>, NameError> {
    let mut used = if report.is_none() { vec![false; exceptions.entries.len()] } else { Vec::new() };
    let names = type_names
        .iter()
        .try_fold(BTreeMap::new(), |mut names, &type_name| {
            let name = derive_row(type_name, exceptions, &mut used, report.as_deref_mut())?;
            insert_name(&mut names, name, type_name)?;
            Ok::<_, NameError>(names)
        })?;
    if report.is_none() {
        check_used(exceptions, &used)?;
    }
    let mut rows: Vec<_> = names.into_iter().map(|(name, kind)| (kind, name)).collect();
    rows.sort_unstable_by_key(|(kind, _)| *kind);
    Ok(rows)
}

fn derive_row(
    type_name: &str,
    exceptions: &Exceptions,
    used: &mut [bool],
    report: Option<&mut String>,
) -> Result<SqlName, NameError> {
    let full = derive_full_name(type_name, exceptions, used)?;
    if full.len() > MAX_SQL_NAME_BYTES && let Some(report) = report {
        write!(report, "long {full} ")?;
        let name = SqlName::new(full, type_name)?;
        writeln!(report, "{name}")?;
        return Ok(name);
    }
    SqlName::new(full, type_name)
}

fn insert_name<'a>(
    names: &mut BTreeMap<SqlName, &'a str>,
    name: SqlName,
    type_name: &'a str,
) -> Result<(), NameError> {
    match names.entry(name) {
        Entry::Occupied(entry) => Err(NameError::DuplicateSqlName {
            name: entry.key().clone(),
            first_type: (*entry.get()).to_owned(),
            second_type: type_name.to_owned(),
        }),
        Entry::Vacant(entry) => {
            entry.insert(type_name);
            Ok(())
        }
    }
}

fn check_used(exceptions: &Exceptions, used: &[bool]) -> Result<(), NameError> {
    let unused = exceptions
        .entries
        .iter()
        .zip(used)
        .zip(1..)
        .find(|((_, used), _)| !**used);
    if let Some((((spelling, _), _), line)) = unused {
        return Err(NameError::UnusedException {
            line,
            spelling: spelling.clone(),
        });
    }
    Ok(())
}
