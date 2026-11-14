use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(super) struct UniversalCatalog {
    pub resource_types: Vec<UniversalRow>,
}

#[derive(Deserialize)]
pub(super) struct UniversalRow {
    pub type_name: String,
    pub capability: String,
    pub exclusion: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct ReviewedCatalog {
    pub resource_types: Vec<ReviewedRow>,
}

#[derive(Deserialize)]
pub(super) struct ReviewedRow {
    pub type_name: String,
    #[serde(default)]
    pub list_model_validation: Option<String>,
    #[serde(default)]
    pub primary_identifier: Vec<String>,
}

#[derive(Serialize)]
pub(super) struct Report {
    pub schema_version: u32,
    pub source: Source,
    pub summary: Summary,
    pub candidates: Vec<Candidate>,
}

#[derive(Serialize)]
pub(super) struct Source {
    pub universal_catalog: &'static str,
    pub parent_catalog: &'static str,
    pub schema_archive: &'static str,
    pub archive_sha256: String,
}

#[derive(Serialize)]
pub(super) struct Summary {
    pub total_parent_types: usize,
    pub reviewed_children: usize,
    pub implemented_workflows_total: usize,
    pub remaining_children: usize,
    pub inferred_high: usize,
    pub inferred_medium: usize,
    pub low_or_unmapped: usize,
    pub reviewed_inference_mismatches: usize,
}

#[derive(Serialize)]
pub(super) struct Candidate {
    pub type_name: String,
    pub exclusion: Option<String>,
    pub reviewed_parent: Option<String>,
    pub signals: Vec<Signal>,
    pub proposed_parent: Option<String>,
    pub confidence: &'static str,
}

#[derive(Serialize)]
pub(super) struct Signal {
    pub property: String,
    pub candidate_parent: Option<String>,
    pub confidence: &'static str,
}
