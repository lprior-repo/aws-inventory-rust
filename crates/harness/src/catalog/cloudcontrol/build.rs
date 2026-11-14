use std::collections::BTreeSet;

use crate::HarnessError;
use crate::catalog::archive::Archive;
use crate::catalog::json::Json;
use crate::catalog::rules::{GLOBAL_SENSITIVE, SELECTED};

use super::descriptor::descriptor;
use super::{EXPECTED_TYPES, archive_name};

const SOURCE_KIND: &str = "official_aws_cloudformation_registry_schema_archive";
const SOURCE_URL: &str =
    "https://schema.cloudformation.us-east-1.amazonaws.com/CloudformationSchema.zip";
const RETRIEVED_AT: &str = "2026-11-12T15:31:46.572550+00:00";
const SELECTION: &str = "397 reviewed regional types across architectural service families; read and list handlers; empty list model accepted; documented parent-model exceptions excluded";
const LIST_MODEL_URL: &str =
    "https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-operations-list.html";
const SUPPORT_URL: &str =
    "https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/resource-types.html";
const SCHEMA_URL: &str =
    "https://docs.aws.amazon.com/cloudformation-cli/latest/userguide/resource-type-schema.html";

type Built = (Json, Vec<(String, usize)>);

type TypeRows = (Vec<Json>, Vec<(String, usize)>);

pub fn build_resources(
    archive: &Archive,
    additions: &[(String, String)],
) -> Result<Built, HarnessError> {
    let global = global_sensitive();
    let added_names = added_names(additions);
    let groups = groups(additions);
    let (types, counts) = build_types(&groups, archive, &added_names, &global)?;
    require_reviewed_types(&types)?;
    Ok((
        Json::object(vec![
            ("schema_version", Json::count(1)),
            ("evidence", evidence(archive, &global)),
            ("resource_types", Json::array(types)),
        ]),
        counts,
    ))
}

fn global_sensitive() -> BTreeSet<String> {
    GLOBAL_SENSITIVE
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

fn added_names(additions: &[(String, String)]) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (service, resources) in additions {
        for resource in resources.split_whitespace() {
            names.insert(format!("AWS::{service}::{resource}"));
        }
    }
    names
}

fn groups(additions: &[(String, String)]) -> Vec<(&str, &str)> {
    SELECTED
        .iter()
        .copied()
        .chain(
            additions
                .iter()
                .map(|(service, names)| (service.as_str(), names.as_str())),
        )
        .collect()
}

fn build_types(
    groups: &[(&str, &str)],
    archive: &Archive,
    added_names: &BTreeSet<String>,
    global: &BTreeSet<String>,
) -> Result<TypeRows, HarnessError> {
    let mut types = Vec::new();
    let mut counts: Vec<(String, usize)> = Vec::new();
    for &(service, resources) in groups {
        for resource in resources.split_whitespace() {
            let name = format!("AWS::{service}::{resource}");
            types.push(type_row(
                archive,
                &name,
                service,
                resource,
                added_names,
                global,
            )?);
            tally(&mut counts, service);
        }
    }
    Ok((types, counts))
}

fn type_row(
    archive: &Archive,
    name: &str,
    service: &str,
    resource: &str,
    added_names: &BTreeSet<String>,
    global: &BTreeSet<String>,
) -> Result<Json, HarnessError> {
    let entry = archive
        .get(name)
        .ok_or_else(|| HarnessError::invalid(format!("schema archive lacks {name}")))?;
    let is_new = added_names.contains(name);
    descriptor(
        service,
        resource,
        name,
        entry,
        &entry.schema,
        is_new,
        global,
    )
}

fn tally(counts: &mut Vec<(String, usize)>, service: &str) {
    match counts.iter_mut().find(|(key, _)| key == service) {
        Some((_, count)) => *count = count.saturating_add(1),
        None => counts.push((service.to_string(), 1)),
    }
}

fn require_reviewed_types(types: &[Json]) -> Result<(), HarnessError> {
    let unique: BTreeSet<&str> = types
        .iter()
        .filter_map(|row| row.get("type_name").and_then(Json::as_str))
        .collect();
    if types.len() != EXPECTED_TYPES || unique.len() != EXPECTED_TYPES {
        return Err(HarnessError::invalid(format!(
            "expected exactly {EXPECTED_TYPES} reviewed unique types, got {}",
            types.len()
        )));
    }
    Ok(())
}

fn evidence(archive: &Archive, global: &BTreeSet<String>) -> Json {
    let mut entries = evidence_head();
    entries.push(("archive_schema_count", Json::count(archive.all_count)));
    entries.push((
        "archive_read_and_list_handler_count",
        Json::count(archive.eligible_count),
    ));
    entries.extend(evidence_tail());
    entries.push(("live_validated", Json::Bool(false)));
    entries.push(("limitations", limitations()));
    entries.push(("global_sensitive_property_names", sensitive_names(global)));
    Json::object(entries)
}

fn evidence_head() -> Vec<(&'static str, Json)> {
    vec![
        ("source_kind", Json::string(SOURCE_KIND)),
        ("url", Json::string(SOURCE_URL)),
        (
            "archive_file",
            Json::string(format!("upstream/{}", archive_name())),
        ),
        (
            "archive_sha256",
            Json::string(crate::catalog::ARCHIVE_SHA256),
        ),
        ("region", Json::string("us-east-1")),
        ("retrieved_at", Json::string(RETRIEVED_AT)),
    ]
}

fn evidence_tail() -> Vec<(&'static str, Json)> {
    vec![
        ("selection", Json::string(SELECTION)),
        ("list_model_exceptions_url", Json::string(LIST_MODEL_URL)),
        ("support_semantics_url", Json::string(SUPPORT_URL)),
        ("schema_semantics_url", Json::string(SCHEMA_URL)),
    ]
}

fn limitations() -> Json {
    Json::array(vec![
        Json::string(
            "Pinned schema capabilities are not proof that every selected region, account, permission set or provider version will succeed.",
        ),
        Json::string(
            "No claimed intersection with the supported-resources HTML table: direct acquisition returned a SiteUnavailable page and was discarded.",
        ),
        Json::string(
            "Regional scope is a reviewed catalog decision; global IAM, CloudFront, Route53 and S3 Bucket enumeration are deliberately excluded.",
        ),
        Json::string(
            "Readable provider properties can contain sensitive values; schema write-only plus explicit descriptor and common-key redaction applies before storage and edges.",
        ),
    ])
}

fn sensitive_names(global: &BTreeSet<String>) -> Json {
    Json::array(
        global
            .iter()
            .map(|name| Json::string(name.as_str()))
            .collect(),
    )
}
