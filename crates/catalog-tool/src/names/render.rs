use std::collections::{BTreeMap, btree_map::Entry};

use super::{Exceptions, NameError, SqlName, derive_name};

pub fn render_names<'a>(
    type_names: &[&'a str],
    exceptions: &Exceptions,
) -> Result<Vec<(&'a str, SqlName)>, NameError> {
    let mut used = vec![false; exceptions.entries.len()];
    let names = type_names
        .iter()
        .try_fold(BTreeMap::new(), |mut names, &type_name| {
            let name = derive_name(type_name, exceptions, &mut used)?;
            insert_name(&mut names, name, type_name)?;
            Ok::<_, NameError>(names)
        })?;
    check_used(exceptions, &used)?;
    let mut rows: Vec<_> = names.into_iter().map(|(name, kind)| (kind, name)).collect();
    rows.sort_unstable_by_key(|(kind, _)| *kind);
    Ok(rows)
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
