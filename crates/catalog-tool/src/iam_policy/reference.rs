use super::*;
use serde::Deserialize;
use std::process::Command;

const SOURCE: &str = "https://servicereference.us-east-1.amazonaws.com/";

#[derive(Deserialize)]
struct Entry {
    service: String,
    url: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Service {
    name: String,
    actions: Vec<Action>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Action {
    name: String,
    annotations: Annotations,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Annotations {
    properties: Properties,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Properties {
    is_write: bool,
}

pub(super) fn parse(text: &str) -> Result<BTreeMap<String, &str>, Error> {
    text.lines()
        .filter(|line| !line.starts_with('#'))
        .try_fold(BTreeMap::new(), |mut map, line| {
            let mut fields = line.split('\t');
            let (Some(prefix), Some(name), Some(access), None) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                return Err(Error::BadReferenceRow {
                    row: line.to_owned(),
                });
            };
            if !matches!(access, "read" | "write")
                || prefix.is_empty()
                || name.is_empty()
                || !prefix
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            {
                return Err(Error::BadReferenceRow {
                    row: line.to_owned(),
                });
            }
            if map.insert(format!("{prefix}:{name}"), access).is_some() {
                return Err(Error::DuplicateReferenceRow {
                    row: line.to_owned(),
                });
            }
            Ok(map)
        })
}

/// Refresh the tracked reference, fetching only services used by callable plans/mappings.
/// Date is explicit so invocation and fixtures never depend on a wall clock.
pub fn refresh(root: &Path, date: &str) -> Result<(), Error> {
    validate_date(date)?;
    let bytes = std::fs::read(root.join("catalog/aws-api.catalog"))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let mappings: Mappings = toml::from_str(&crate::report::read_text(&root.join(MAPPINGS_PATH))?)?;
    let mut prefixes: BTreeSet<String> = operations(&catalog)?
        .iter()
        .map(|op| op.prefix.to_owned())
        .collect();
    catalog
        .plans()
        .iter()
        .try_for_each(|plan| -> Result<(), Error> {
            if let Some(prefix) = plan.service().arn_namespace()? {
                prefixes.insert(prefix.to_owned());
            }
            Ok(())
        })?;
    mappings
        .operation
        .values()
        .filter_map(|action| action.split_once(':'))
        .for_each(|(prefix, _)| {
            prefixes.insert(prefix.to_owned());
        });
    let rows = fetch_rows(&prefixes)?;
    let text = format!(
        "# Source: {SOURCE}\n# Retrieved: {date}\n# service\taction\taccess\n{}",
        rows.into_iter().collect::<String>()
    );
    parse(&text)?;
    Ok(crate::report::publish(
        &root.join(REFERENCE_PATH),
        &text,
        ReportMode::Write,
    )?)
}

fn fetch_rows(prefixes: &BTreeSet<String>) -> Result<BTreeSet<String>, Error> {
    let entries: Vec<Entry> = serde_json::from_slice(&fetch(SOURCE)?)?;
    let rows = entries
        .into_iter()
        .filter(|entry| prefixes.contains(&entry.service))
        .try_fold(BTreeSet::new(), |mut rows, entry| -> Result<_, Error> {
            let url = entry_url(&entry)?;
            let service = service_document(&entry, &fetch(url)?)?;
            rows.extend(service.actions.into_iter().map(|action| {
                format!(
                    "{}\t{}\t{}\n",
                    service.name,
                    action.name,
                    if action.annotations.properties.is_write {
                        "write"
                    } else {
                        "read"
                    }
                )
            }));
            Ok(rows)
        })?;
    Ok(rows)
}

fn fetch(url: &str) -> Result<Vec<u8>, Error> {
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--max-time",
            "60",
            "--max-filesize",
            "8388608",
            url,
        ])
        .output()?;
    if !output.status.success() {
        return Err(Error::ReferenceFetch {
            url: url.to_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(output.stdout)
}
pub(super) fn validate_date(date: &str) -> Result<(), Error> {
    let invalid = || Error::InvalidReferenceDate {
        date: date.to_owned(),
    };
    let mut fields = date.split('-');
    let (Some(year), Some(month), Some(day), None) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(invalid());
    };
    if year.len() != 4
        || month.len() != 2
        || day.len() != 2
        || ![year, month, day]
            .iter()
            .all(|field| field.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(invalid());
    }
    let year: u16 = year.parse().map_err(|_| invalid())?;
    let month: u16 = month.parse().map_err(|_| invalid())?;
    let day: u16 = day.parse().map_err(|_| invalid())?;
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28_u16.saturating_add(u16::from(leap)),
        _ => return Err(invalid()),
    };
    if year == 0 || day == 0 || day > days {
        return Err(invalid());
    }
    Ok(())
}

fn entry_url(entry: &Entry) -> Result<&str, Error> {
    if !entry.url.starts_with(SOURCE) {
        return Err(Error::ForeignReferenceUrl {
            url: entry.url.clone(),
        });
    }
    Ok(&entry.url)
}

fn service_document(entry: &Entry, bytes: &[u8]) -> Result<Service, Error> {
    let service: Service = serde_json::from_slice(bytes)?;
    if service.name != entry.service {
        return Err(Error::ReferenceServiceMismatch {
            expected: entry.service.clone(),
            actual: service.name,
        });
    }
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_calendar_dates_when_validated_then_invalid_dates_fail() {
        ["2026-11-24", "2024-02-29", "2000-02-29"]
            .iter()
            .for_each(|date| assert!(validate_date(date).is_ok()));
        [
            "9999-99-99",
            "2026-02-29",
            "1900-02-29",
            "2026-04-31",
            "2026-00-01",
            "2026-01-00",
            "0000-01-01",
            "2026-1-01",
            "----------",
            "2026a11a24",
        ]
        .iter()
        .for_each(|date| assert!(validate_date(date).is_err()));
    }

    #[test]
    fn given_reference_entries_when_validated_then_foreign_urls_and_service_mismatches_fail()
    -> Result<(), Error> {
        let entry = Entry {
            service: "s3".into(),
            url: format!("{SOURCE}v1/s3/s3.json"),
        };
        assert_eq!(entry_url(&entry)?, entry.url);
        let bytes = br#"{"Name":"s3","Actions":[{"Name":"ListBuckets","Annotations":{"Properties":{"IsWrite":false}}}]}"#;
        assert_eq!(service_document(&entry, bytes)?.actions.len(), 1);
        let foreign = Entry {
            service: "s3".into(),
            url: "https://example.test/s3.json".into(),
        };
        assert!(entry_url(&foreign).is_err());
        let wrong = Entry {
            service: "ec2".into(),
            url: entry.url,
        };
        assert!(service_document(&wrong, bytes).is_err());
        Ok(())
    }
}
