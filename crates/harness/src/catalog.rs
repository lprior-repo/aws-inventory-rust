mod archive;
mod capabilities;
mod cloudcontrol;
mod json;
mod parent;
mod rules;
mod schema;
mod tables;
pub(crate) mod toml_doc;
pub mod universal;

use std::path::Path;

use crate::HarnessError;
use crate::provenance::{file_sha256, sha256_hex};
pub use json::Json;

pub const ARCHIVE: &str = "catalog/upstream/cloudformation-us-east-1-2026-11-12.zip";

pub const ARCHIVE_SHA256: &str = "8226b9cd82f432997fe3918b134ef155245636f35ac37f8006de5abe88dd54a8";

pub const ADDITIONS: &str = "catalog/cloudcontrol-regional-additions.toml";

pub const ROOT_CATALOG: &str = "catalog/cloudcontrol-resources.json";

pub const CAPABILITY_MATRIX: &str = "catalog/cloudcontrol-capabilities.json";

pub const PARENT_CATALOG: &str = "catalog/cloudcontrol-parent-resources.toml";

pub const UPSTREAM_MANIFEST: &str = "catalog/upstream/cloudcontrol-upstream.json";

#[derive(Debug, Clone)]
pub struct Report {
    pub file: String,

    pub contents: String,
}

pub fn reproduce(root: &Path) -> Result<Vec<Report>, HarnessError> {
    let archive = archive::load(root)?;
    let additions = archive::additions(root)?;
    let (resources, _counts) = cloudcontrol::build_resources(&archive, &additions)?;
    let resources_text = format!("{}\n", resources.pretty());
    let resources_sha256 = sha256_hex(resources_text.as_bytes());
    let matrix = capabilities::build(&archive, &resources, &resources_sha256)?;
    let manifest = upstream_manifest(root)?;
    let parents = parent::build(root, &archive)?;
    Ok(vec![
        Report {
            file: ROOT_CATALOG.to_string(),
            contents: resources_text,
        },
        document(CAPABILITY_MATRIX, &matrix),
        document(UPSTREAM_MANIFEST, &manifest),
        Report {
            file: PARENT_CATALOG.to_string(),
            contents: toml_doc::render(&parents.to_serde()?)?,
        },
    ])
}

fn document(file: &str, contents: &Json) -> Report {
    Report {
        file: file.to_string(),
        contents: format!("{}\n", contents.pretty()),
    }
}

pub fn check(root: &Path) -> Result<(), HarnessError> {
    let root_bytes = std::fs::read(root.join(ROOT_CATALOG))?;
    for report in reproduce(root)? {
        let committed = std::fs::read(root.join(&report.file))?;
        if committed == report.contents.as_bytes() {
            continue;
        }
        if report.file == CAPABILITY_MATRIX {
            verify_matrix(&report.contents, &committed)?;
            continue;
        }
        return Err(mismatch(&report.file, &report.contents, &committed));
    }
    verify_parent_binding(root, &root_bytes)
}

fn verify_parent_binding(root: &Path, root_bytes: &[u8]) -> Result<(), HarnessError> {
    let text = std::fs::read_to_string(root.join(PARENT_CATALOG))?;
    let parent = toml_doc::parse(&text)?;
    let recorded = parent
        .pointer("/evidence/root_catalog_sha256")
        .and_then(serde_json::Value::as_str);
    if recorded != Some(sha256_hex(root_bytes).as_str()) {
        return Err(HarnessError::invalid(
            "parent catalog does not bind the committed root catalog digest",
        ));
    }
    Ok(())
}

fn verify_matrix(generated: &str, committed: &[u8]) -> Result<(), HarnessError> {
    let text = std::str::from_utf8(committed)
        .map_err(|_| HarnessError::invalid("capability matrix is not UTF-8"))?;
    let mut projected = Json::parse(text)?;
    capabilities::without_executed_evidence(&mut projected);
    let expected = format!("{}\n", projected.pretty());
    if expected == generated {
        return Ok(());
    }
    Err(mismatch(CAPABILITY_MATRIX, generated, expected.as_bytes()))
}

fn mismatch(file: &str, expected: &str, actual: &[u8]) -> HarnessError {
    let line = first_difference_line(expected, actual);
    let action = if file == PARENT_CATALOG {
        "parent catalog differs; regenerate with `inventory-harness catalog universal`"
    } else if file == UPSTREAM_MANIFEST {
        "upstream manifest differs; regenerate with `inventory-harness catalog universal`"
    } else if file == CAPABILITY_MATRIX {
        "capability matrix differs; regenerate with `inventory-harness catalog universal`"
    } else {
        "catalog differs; regenerate with `inventory-harness catalog universal`"
    };
    HarnessError::invalid(format!("{action} (first difference at line {line})"))
}

fn first_difference_line(expected: &str, actual: &[u8]) -> usize {
    let expected_bytes = expected.as_bytes();
    let mut index = 0_usize;
    loop {
        match (expected_bytes.get(index), actual.get(index)) {
            (Some(left), Some(right)) if left == right => {
                index = index.saturating_add(1);
            }
            _ => break,
        }
    }
    let prefix = expected_bytes.get(..index).unwrap_or_default();
    1_usize.saturating_add(prefix.iter().filter(|byte| **byte == b'\n').count())
}

fn upstream_manifest(root: &Path) -> Result<Json, HarnessError> {
    let text = std::fs::read_to_string(root.join(UPSTREAM_MANIFEST))?;
    let mut manifest = Json::parse(&text)?;
    let name = ARCHIVE.rsplit('/').next().unwrap_or(ARCHIVE);
    manifest.set("sources", Json::array(matching_sources(&manifest, name)));
    manifest.set("discarded_acquisitions", discarded_acquisitions());
    Ok(manifest)
}

fn matching_sources(manifest: &Json, name: &str) -> Vec<Json> {
    manifest
        .get("sources")
        .and_then(Json::as_array)
        .unwrap_or_default()
        .iter()
        .filter(|entry| entry.get("file").and_then(Json::as_str) == Some(name))
        .cloned()
        .collect()
}

fn discarded_acquisitions() -> Json {
    Json::array(vec![Json::object(vec![
        (
            "url",
            Json::string(
                "https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/supported-resources.html",
            ),
        ),
        (
            "reason",
            Json::string(
                "HTTP 200 returned a 195-byte SiteUnavailable page; not retained or used as capability evidence",
            ),
        ),
    ])])
}

pub fn verify_archive(root: &Path) -> Result<(), HarnessError> {
    if file_sha256(&root.join(ARCHIVE))? != ARCHIVE_SHA256 {
        return Err(HarnessError::invalid("upstream archive checksum mismatch"));
    }
    Ok(())
}
