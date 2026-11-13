//! Parent-workflow candidate derivation from the pinned CloudFormation schemas.

mod inference;
mod model;
mod schema;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use zip::ZipArchive;

use inference::{infer, names_by_suffix, reviewed_parents};
use model::{
    Candidate, Report, ReviewedCatalog, Signal, Source, Summary, UniversalCatalog, UniversalRow,
};
use schema::{Archive, schema_filename, schema_required};

const UNIVERSAL: &str = "catalog/cloudcontrol-universal.json";
const REVIEWED: &str = "catalog/cloudcontrol-parent-resources.json";
const ARCHIVE: &str = "catalog/upstream/cloudformation-us-east-1-2026-11-12.zip";
const OUTPUT: &str = "catalog/cloudcontrol-parent-candidates.json";

pub fn run(root: &Path, write: bool, check: bool) -> Result<(), Box<dyn std::error::Error>> {
    if write && check {
        return Err("--write and --check are mutually exclusive".into());
    }
    let report = build(root)?;
    let rendered = format!("{}\n", serde_json::to_string_pretty(&report)?);
    let output = root.join(OUTPUT);
    if check {
        let existing = fs::read_to_string(&output)?;
        if existing != rendered {
            return Err(format!("{} is stale; regenerate with --write", output.display()).into());
        }
        println!("{}", serde_json::to_string(&report.summary)?);
        return Ok(());
    }
    if write {
        fs::write(&output, rendered)?;
        println!("{}", serde_json::to_string(&report.summary)?);
        return Ok(());
    }
    print!("{rendered}");
    Ok(())
}

fn build(root: &Path) -> Result<Report, Box<dyn std::error::Error>> {
    let universal: UniversalCatalog = load(root, UNIVERSAL)?;
    let reviewed: ReviewedCatalog = load(root, REVIEWED)?;
    let type_names = type_names(&universal);
    let implemented = reviewed_parents(&reviewed, &type_names);
    let global = names_by_suffix(&type_names);
    let (archive_sha256, mut archive) = open_archive(root)?;
    let candidates = candidates(
        &universal.resource_types,
        &type_names,
        &global,
        &implemented,
        &mut archive,
    )?;
    let summary = summarize(&candidates, reviewed.resource_types.len());
    Ok(Report {
        schema_version: 1,
        source: source(archive_sha256),
        summary,
        candidates,
    })
}

fn load<T: serde::de::DeserializeOwned>(
    root: &Path,
    relative: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(
        root.join(relative),
    )?)?)
}

fn type_names(universal: &UniversalCatalog) -> Vec<&str> {
    universal
        .resource_types
        .iter()
        .map(|row| row.type_name.as_str())
        .collect()
}

fn open_archive(root: &Path) -> Result<(String, Archive), Box<dyn std::error::Error>> {
    let bytes = fs::read(root.join(ARCHIVE))?;
    let digest = hex(&Sha256::digest(&bytes));
    let archive = ZipArchive::new(std::io::Cursor::new(bytes))?;
    Ok((digest, archive))
}

fn source(archive_sha256: String) -> Source {
    Source {
        universal_catalog: UNIVERSAL,
        parent_catalog: REVIEWED,
        schema_archive: ARCHIVE,
        archive_sha256,
    }
}

fn candidates(
    rows: &[UniversalRow],
    type_names: &[&str],
    global: &BTreeMap<String, Vec<String>>,
    implemented: &BTreeMap<String, Option<String>>,
    archive: &mut Archive,
) -> Result<Vec<Candidate>, Box<dyn std::error::Error>> {
    let mut candidates = Vec::new();
    for row in rows.iter().filter(|row| row.capability == "parent") {
        candidates.push(candidate(row, type_names, global, implemented, archive)?);
    }
    Ok(candidates)
}

fn candidate(
    row: &UniversalRow,
    type_names: &[&str],
    global: &BTreeMap<String, Vec<String>>,
    implemented: &BTreeMap<String, Option<String>>,
    archive: &mut Archive,
) -> Result<Candidate, Box<dyn std::error::Error>> {
    let filename = schema_filename(&row.type_name)?;
    let required = schema_required(archive, &filename)?;
    let signals = infer(&row.type_name, &required, type_names, global);
    let chosen = proposed(&signals);
    Ok(Candidate {
        type_name: row.type_name.clone(),
        exclusion: row.exclusion.clone(),
        reviewed_parent: implemented.get(&row.type_name).cloned().flatten(),
        confidence: confidence(&signals, chosen.as_deref()),
        signals,
        proposed_parent: chosen,
    })
}

fn proposed(signals: &[Signal]) -> Option<String> {
    signals
        .iter()
        .find(|signal| matches!(signal.confidence, "high" | "medium"))
        .and_then(|signal| signal.candidate_parent.clone())
}

fn confidence(signals: &[Signal], chosen: Option<&str>) -> &'static str {
    if chosen.is_some() {
        return signals
            .iter()
            .find(|signal| signal.candidate_parent.as_deref() == chosen)
            .map_or("unmapped", |signal| signal.confidence);
    }
    if signals.iter().any(|signal| signal.confidence == "low") {
        return "low";
    }
    "unmapped"
}

fn summarize(candidates: &[Candidate], workflows: usize) -> Summary {
    let reviewed = candidates
        .iter()
        .filter(|row| row.reviewed_parent.is_some())
        .count();
    let remaining: Vec<&Candidate> = candidates
        .iter()
        .filter(|row| row.reviewed_parent.is_none())
        .collect();
    Summary {
        total_parent_types: candidates.len(),
        reviewed_children: reviewed,
        implemented_workflows_total: workflows,
        remaining_children: candidates.len().saturating_sub(reviewed),
        inferred_high: confidence_count(&remaining, "high"),
        inferred_medium: confidence_count(&remaining, "medium"),
        low_or_unmapped: remaining
            .iter()
            .filter(|row| !matches!(row.confidence, "high" | "medium"))
            .count(),
        reviewed_inference_mismatches: mismatch_count(candidates),
    }
}

/// Count review candidates at an exact confidence level.
fn confidence_count(remaining: &[&Candidate], confidence: &str) -> usize {
    remaining
        .iter()
        .filter(|row| row.confidence == confidence)
        .count()
}

/// Count reviewed rows whose inferred parent disagrees with the review.
fn mismatch_count(candidates: &[Candidate]) -> usize {
    candidates
        .iter()
        .filter(|row| row.reviewed_parent.is_some())
        .filter(|row| row.proposed_parent.is_some() && row.proposed_parent != row.reviewed_parent)
        .count()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_shape_is_stable() {
        let candidate = Candidate {
            type_name: "AWS::Test::Child".to_owned(),
            exclusion: None,
            reviewed_parent: None,
            signals: vec![],
            proposed_parent: None,
            confidence: "unmapped",
        };
        let summary = summarize(&[candidate], 21);
        assert_eq!(summary.total_parent_types, 1);
        assert_eq!(summary.implemented_workflows_total, 21);
        assert_eq!(summary.remaining_children, 1);
    }
}
