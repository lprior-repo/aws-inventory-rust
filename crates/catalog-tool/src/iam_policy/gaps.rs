use super::*;
use inventory_core::catalog::runtime::ReviewDisposition;
use std::fmt::Write as _;

pub(super) fn parse(
    text: &str,
    reference: &str,
    known: &BTreeMap<String, &str>,
    operations: &[Operation<'_>],
    mappings: &Mappings,
) -> Result<BTreeSet<String>, Error> {
    let date = reference
        .lines()
        .find_map(|line| line.strip_prefix("# Retrieved: "))
        .ok_or(Error::MissingReferenceDate)?;
    reference::validate_date(date)?;
    let reason = format!("unconfirmed: absent from the service authorization reference at {date}");
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .try_fold(BTreeSet::new(), |mut ids, line| {
            let invalid = |reason| Error::BadException {
                row: line.to_owned(),
                reason,
            };
            let mut fields = line.split('\t');
            let row = (
                fields.next(),
                fields.next(),
                fields.next(),
                fields.next(),
                fields.next(),
            );
            let (Some(service), Some(operation), Some(signing), Some(explanation), None) = row
            else {
                return Err(invalid(ExceptionFault::Malformed));
            };
            if [service, operation, signing].iter().any(|field| {
                field.is_empty()
                    || !field
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            }) {
                return Err(invalid(ExceptionFault::Malformed));
            }
            let id = format!("{service}/{operation}");
            let op = operations
                .iter()
                .find(|op| op.service == service && op.name == operation)
                .ok_or_else(|| invalid(ExceptionFault::NoLongerCallable))?;
            if op.prefix != signing || explanation != reason {
                return Err(invalid(ExceptionFault::PinMismatch));
            }
            if mappings.operation.contains_key(&id)
                || known.contains_key(&format!("{signing}:{operation}"))
            {
                return Err(invalid(ExceptionFault::Stale));
            }
            if !ids.insert(id) {
                return Err(invalid(ExceptionFault::Duplicate));
            }
            Ok(ids)
        })
}

pub(super) fn report(ids: &BTreeSet<String>) -> String {
    format!(
        "The policy grants default-name actions for these {} operations as unconfirmed; the pinned reference does not publish them.\n{}",
        ids.len(),
        ids.iter().map(|id| format!("{id}\n")).collect::<String>()
    )
}

type SharedRows<'a> = BTreeMap<String, (String, &'a Vec<&'a str>)>;

pub(super) fn shared(
    catalog: &RuntimeCatalog<'_>,
    mappings: &str,
    operation_actions: &BTreeMap<String, String>,
) -> Result<String, Error> {
    let mappings: Mappings = toml::from_str(mappings)?;
    let mut owners: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (operation, action) in operation_actions {
        owners.entry(action).or_default().push(operation);
    }
    let mut prefixes = BTreeMap::new();
    for service in catalog.catalog().services()? {
        let service = service?;
        prefixes.insert(service.name()?, service.signing_name()?);
    }
    let (mut denied, mut deferred, mut unclassified) =
        (BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
    for decision in catalog.decisions() {
        let rows = match decision.disposition {
            ReviewDisposition::Callable => continue,
            ReviewDisposition::Denied { .. } => &mut denied,
            ReviewDisposition::Deferred { .. } => &mut deferred,
            ReviewDisposition::Unclassified => &mut unclassified,
        };
        let id = format!("{}/{}", decision.service, decision.operation);
        let action = if let Some(action) = mappings.operation.get(&id) {
            action.clone()
        } else {
            let Some(prefix) = prefixes.get(decision.service.as_str()).copied().flatten() else {
                continue;
            };
            format!("{prefix}:{}", decision.operation)
        };
        if let Some(callable) = owners.get(action.as_str()) {
            rows.insert(id, (action, callable));
        }
    }
    let count = denied
        .len()
        .saturating_add(deferred.len())
        .saturating_add(unclassified.len());
    let mut text = format!(
        "\nshared\nActions use explicit operation overrides or model signing-name:operation; \
         non-callable operations whose IAM action is not derivable by that lookup are not listed.\n\
         count: {count} non-callable operations\n"
    );
    for (label, rows) in [
        ("DENIED (must-deny)", denied),
        ("deferred", deferred),
        ("unclassified", unclassified),
    ] {
        shared_group(&mut text, label, &rows)?;
    }
    broad(&mut text, &mappings, &owners)?;
    Ok(text)
}

fn shared_group(text: &mut String, label: &str, rows: &SharedRows<'_>) -> std::fmt::Result {
    writeln!(text, "{label}: {}", rows.len())?;
    for (operation, (action, owners)) in rows {
        for owner in *owners {
            writeln!(text, "{operation} {action} callable-by {owner}")?;
        }
    }
    Ok(())
}

fn broad(
    text: &mut String,
    mappings: &Mappings,
    owners: &BTreeMap<&str, Vec<&str>>,
) -> std::fmt::Result {
    let mut actions: BTreeSet<&str> = owners
        .iter()
        .filter_map(|(action, operations)| (operations.len() > 1).then_some(*action))
        .collect();
    for (operation, action) in &mappings.operation {
        if owners
            .get(action.as_str())
            .is_some_and(|operations| operations.iter().any(|owner| *owner != operation))
        {
            actions.insert(action);
        }
    }
    writeln!(
        text,
        "\nbroad\nExisting aliases or callable mappings associate these granted actions with \
         multiple operation IDs, so they can permit other requests.\ncount: {} actions",
        actions.len()
    )?;
    for action in actions {
        if let Some(operations) = owners.get(action) {
            for operation in operations {
                writeln!(text, "{action} callable-by {operation}")?;
            }
        }
    }
    Ok(())
}
