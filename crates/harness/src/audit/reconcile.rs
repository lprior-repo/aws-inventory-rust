//! Build the source-boundary reconciliation report without writing evidence.

mod callsites;
mod inputs;
mod report;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::{Value, json};

use crate::HarnessError;
use crate::provenance::{sha256_hex, source_fingerprint};

use super::calls::CallSite;
use super::files::{array_field, text_field};
use super::inventory::source_inventory;
use super::mask::{location, mask};
use super::matrix::{catalogs, descriptor_names, runtime_digest, source_rows};
use super::operations::inventory::load as load_inventory;
use super::pins::{addition_evidence, dependency_evidence};
use super::resolve::{function_at, provider_builder_offsets};

use report::{Report, report};

pub(super) use inputs::Inputs;

use inputs::inputs;

/// Historical and reviewed dependency evidence collected together.
type DependencyFacts = (Vec<Value>, Vec<Value>, Value, CatalogFacts, Vec<Value>);

pub(super) struct CatalogFacts {
    pub(super) catalog_sha256: String,
    pub(super) parent_sha256: String,
    pub(super) runtime_sha: String,
    pub(super) descriptor_count: usize,
    pub(super) root_types: usize,
    pub(super) parent_types: usize,
    pub(super) parent_plans: usize,
    pub(super) relationship_rules: usize,
}

pub(super) struct Facts {
    pub(super) callsites: Vec<Value>,
    pub(super) position_changes: usize,
    pub(super) provider_sites: Vec<Value>,
    pub(super) provider_dependency: Value,
    pub(super) modules: Vec<Value>,
    pub(super) rows: Vec<Value>,
    pub(super) addition_modules: Vec<Value>,
    pub(super) services: usize,
    pub(super) nonempty: usize,
}

/// Build the reconciliation report without writing any evidence.
pub fn reconcile(root: &Path) -> Result<Value, HarnessError> {
    let before = source_fingerprint(root)?;
    let inputs = inputs(root)?;
    let (sources, calls, nonempty) = source_inventory(root)?;
    check_closure(&before, &sources)?;
    inputs.additions.bind(
        &load_inventory(root)?,
        &matrix_ids(&inputs.matrix)?,
        &sources,
    )?;
    let (facts, catalogs) = computed(root, &inputs, &sources, &calls, nonempty)?;
    report(
        root,
        Report {
            before: &before,
            inputs: &inputs,
            catalogs: &catalogs,
            facts: &facts,
        },
    )
}

fn computed(
    root: &Path,
    inputs: &Inputs,
    sources: &BTreeMap<String, String>,
    calls: &[CallSite],
    nonempty: usize,
) -> Result<(Facts, CatalogFacts), HarnessError> {
    let (callsites, position_changes) =
        callsites::rows(&inputs.expected, calls, inputs.additions.is_present())?;
    let provider_sites = provider_boundary(&inputs.matrix, sources)?;
    let (modules, addition_modules, provider_dependency, catalogs, rows) =
        dependencies(root, inputs, sources)?;
    let facts = gather(Gathered {
        callsites,
        position_changes,
        provider_sites,
        provider_dependency,
        modules,
        rows,
        addition_modules,
        calls,
        nonempty,
    });
    Ok((facts, catalogs))
}

struct Gathered<'a> {
    callsites: Vec<Value>,
    position_changes: usize,
    provider_sites: Vec<Value>,
    provider_dependency: Value,
    modules: Vec<Value>,
    rows: Vec<Value>,
    addition_modules: Vec<Value>,
    calls: &'a [CallSite],
    nonempty: usize,
}

fn gather(data: Gathered<'_>) -> Facts {
    Facts {
        callsites: sorted_callsites(data.callsites),
        position_changes: data.position_changes,
        provider_sites: data.provider_sites,
        provider_dependency: data.provider_dependency,
        modules: data.modules,
        rows: data.rows,
        addition_modules: data.addition_modules,
        services: service_count(data.calls),
        nonempty: data.nonempty,
    }
}

fn dependencies(
    root: &Path,
    inputs: &Inputs,
    sources: &BTreeMap<String, String>,
) -> Result<DependencyFacts, HarnessError> {
    let (modules, provider_dependency) = dependency_evidence(root, &inputs.matrix)?;
    let addition_modules = addition_evidence(root, &inputs.additions)?;
    let catalogs = catalog_facts(root)?;
    let rows = source_rows(&inputs.matrix, &inputs.previous, sources)?;
    Ok((
        modules,
        addition_modules,
        provider_dependency,
        catalogs,
        rows,
    ))
}

fn matrix_ids(matrix: &Value) -> Result<BTreeSet<String>, HarnessError> {
    let mut ids = BTreeSet::new();
    for operation in array_field(matrix, "operations")? {
        ids.insert(text_field(operation, "id")?.to_owned());
    }
    Ok(ids)
}

fn service_count(calls: &[CallSite]) -> usize {
    calls
        .iter()
        .map(|call| call.sdk_service.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

fn check_closure(before: &Value, sources: &BTreeMap<String, String>) -> Result<(), HarnessError> {
    let mut closure_files: BTreeMap<&str, &str> = BTreeMap::new();
    for row in array_field(before, "files")? {
        closure_files.insert(text_field(row, "file")?, text_field(row, "sha256")?);
    }
    for (path, source) in sources {
        let digest = sha256_hex(source.as_bytes());
        if closure_files.get(path.as_str()) != Some(&digest.as_str()) {
            return Err(HarnessError::invalid(
                "AWS source changed while the audit snapshot was being read",
            ));
        }
    }
    Ok(())
}

fn provider_boundary(
    matrix: &Value,
    sources: &BTreeMap<String, String>,
) -> Result<Vec<Value>, HarnessError> {
    let provider_sites = provider_sites(sources)?;
    let providers = array_field(matrix, "credential_provider_boundaries")?;
    let provider = providers.first().ok_or_else(|| {
        HarnessError::invalid("unexpected historical credential-provider boundaries")
    })?;
    validate_provider(providers, provider)?;
    let historical_provider = provider_counts(array_field(provider, "source_calls")?)?;
    if historical_provider != provider_counts(&provider_sites)? {
        return Err(HarnessError::invalid(
            "credential-provider boundary changed",
        ));
    }
    Ok(provider_sites)
}

fn validate_provider(providers: &[Value], provider: &Value) -> Result<(), HarnessError> {
    if providers.len() != 1
        || provider.get("id").and_then(Value::as_str) != Some("sts.assume_role")
        || provider.get("explicit_sdk_send") != Some(&Value::Bool(false))
    {
        return Err(HarnessError::invalid(
            "unexpected historical credential-provider boundaries",
        ));
    }
    Ok(())
}

fn catalog_facts(root: &Path) -> Result<CatalogFacts, HarnessError> {
    let frozen = catalogs(root)?;
    let (descriptors, _names) = descriptor_names(&frozen.merged)?;
    let relationship_rules: usize = descriptors
        .iter()
        .map(|row| {
            row.get("relationships")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0)
        })
        .sum();
    Ok(CatalogFacts {
        descriptor_count: descriptors.len(),
        runtime_sha: runtime_digest(&frozen.merged),
        root_types: array_field(&frozen.catalog, "resource_types")?.len(),
        parent_types: array_field(&frozen.parents, "resource_types")?.len(),
        parent_plans: array_field(&frozen.parents, "parent_plans")?.len(),
        catalog_sha256: frozen.catalog_sha256,
        parent_sha256: frozen.parent_sha256,
        relationship_rules,
    })
}

fn sorted_callsites(mut callsites: Vec<Value>) -> Vec<Value> {
    callsites.sort_by(|left, right| sort_key(left).cmp(&sort_key(right)));
    callsites
}

fn sort_key(row: &Value) -> (&str, u64) {
    (
        row.get("path").and_then(Value::as_str).unwrap_or_default(),
        row.get("send_line").and_then(Value::as_u64).unwrap_or(0),
    )
}

fn provider_sites(sources: &BTreeMap<String, String>) -> Result<Vec<Value>, HarnessError> {
    let mut sites = Vec::new();
    for (path, source) in sources {
        let characters: Vec<char> = source.chars().collect();
        let masked = mask(&characters).map_err(|error| HarnessError::invalid(error.message))?;
        for start in provider_builder_offsets(&masked) {
            let (function, _) = function_at(&masked, start)?;
            sites.push(json!({
                "path": path,
                "function": function,
                "provider_builder_line": location(&characters, start).0,
            }));
        }
    }
    Ok(sites)
}

fn provider_counts(sites: &[Value]) -> Result<BTreeMap<(String, String), usize>, HarnessError> {
    let mut counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for site in sites {
        let key = (
            text_field(site, "path")?.to_owned(),
            text_field(site, "function")?.to_owned(),
        );
        let count = counts.entry(key).or_insert(0);
        *count = count.saturating_add(1);
    }
    Ok(counts)
}
