use std::collections::HashSet;

use crate::HarnessError;
use crate::provenance::file_sha256;

const MAX_ADDITIONS: usize = 512;
const SOURCE_PREFIX: &str = "crates/aws/src/";

fn hex64(value: &str) -> bool {
    if value.len() != 64 {
        return false;
    }
    value.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn source_path(path: &str) -> bool {
    if !path.starts_with(SOURCE_PREFIX) {
        return false;
    }
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return false;
        }
    }
    true
}

fn addition_sites(row: &serde_json::Value, ident: &str) -> Result<Vec<serde_json::Value>, HarnessError> {
    let values = row
        .get("source_calls")
        .and_then(serde_json::Value::as_array);
    let values = match values {
        Some(v) if !v.is_empty() => v,
        _ => return Err(HarnessError::invalid("invalid reviewed additions document")),
    };

    let mut seen = HashSet::new();
    for site in values {
        let path = site
            .get("path")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| HarnessError::invalid("invalid reviewed additions document"))?;
        let function = site
            .get("function")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| HarnessError::invalid("invalid reviewed additions document"))?;
        if !source_path(path) || function.is_empty() {
            return Err(HarnessError::invalid("invalid reviewed additions document"));
        }

        for name in ["builder_line", "operation_line", "send_line"] {
            let line = site.get(name).and_then(serde_json::Value::as_i64);
            match line {
                Some(l) if l < 1 => {
                    return Err(HarnessError::invalid(format!(
                        "invalid reviewed callsite lines for {}",
                        ident
                    )));
                }
                None => {
                    return Err(HarnessError::invalid(format!(
                        "invalid reviewed callsite lines for {}",
                        ident
                    )));
                }
                _ => {}
            }
        }

        let key = (path, function);
        if !seen.insert(key) {
            return Err(HarnessError::invalid("invalid reviewed additions document"));
        }
    }

    Ok(values.clone())
}

fn addition_sources(
    root: &std::path::Path,
    row: &serde_json::Value,
    sites: &[serde_json::Value],
) -> Result<(), HarnessError> {
    let hashes = row
        .get("source_sha256")
        .and_then(serde_json::Value::as_object);
    let hashes = match hashes {
        Some(h) => h,
        None => return Err(HarnessError::invalid("invalid reviewed additions document")),
    };

    let paths: HashSet<&str> = sites
        .iter()
        .filter_map(|s| s.get("path").and_then(serde_json::Value::as_str))
        .collect();

    if hashes.keys().len() != paths.len() {
        return Err(HarnessError::invalid("invalid reviewed additions document"));
    }

    for (path, digest) in hashes {
        let digest_str = match digest {
            serde_json::Value::String(s) => s.as_str(),
            _ => {
                return Err(HarnessError::invalid(
                    "invalid reviewed additions document",
                ));
            }
        };
        if !source_path(path) || !hex64(digest_str) {
            return Err(HarnessError::invalid("invalid reviewed additions document"));
        }
        let current = file_sha256(&root.join(path))
            .map_err(|e| HarnessError::invalid(format!("failed to hash {}: {}", path, e)))?;
        if current != *digest {
            return Err(HarnessError::invalid(format!(
                "reviewed addition source changed since review: {}",
                path
            )));
        }
    }

    Ok(())
}

pub fn load_additions(
    root: &std::path::Path,
    matrix: &serde_json::Value,
    matrix_sha: &str,
) -> Result<Option<serde_json::Value>, HarnessError> {
    let path = root.join("catalog/aws-operation-additions.json");
    if !path.is_file() {
        return Ok(None);
    }

    let raw = std::fs::read(&path)
        .map_err(|e| HarnessError::invalid(format!("failed to read additions: {}", e)))?;
    let document: serde_json::Value = serde_json::from_slice(&raw)
        .map_err(|e| HarnessError::invalid(format!("invalid additions JSON: {}", e)))?;

    let schema = document
        .get("schema_version")
        .and_then(serde_json::Value::as_i64);
    if schema != Some(1) {
        return Err(HarnessError::invalid("invalid reviewed additions document"));
    }

    let binds = document
        .get("binds")
        .and_then(serde_json::Value::as_object);
    let binds = match binds {
        Some(b) => b,
        None => return Err(HarnessError::invalid("invalid reviewed additions document")),
    };

    let historical = binds
        .get("historical_matrix")
        .and_then(serde_json::Value::as_object);
    let historical = match historical {
        Some(h) => h,
        None => return Err(HarnessError::invalid("invalid reviewed additions document")),
    };

    let bound_sha = historical
        .get("sha256")
        .and_then(serde_json::Value::as_str);
    if bound_sha != Some(matrix_sha) {
        return Err(HarnessError::invalid(
            "reviewed additions bind a different historical operation matrix",
        ));
    }

    Ok(Some(document))
}

pub fn addition_expected(additions: Option<&serde_json::Value>) -> Vec<serde_json::Value> {
    match additions {
        Some(doc) => {
            let rows = doc
                .get("additions")
                .and_then(serde_json::Value::as_array);
            let rows = match rows {
                Some(r) => r,
                None => return Vec::new(),
            };

            let mut expected = Vec::new();
            for row in rows {
                let id = row
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let sites = row
                    .get("source_calls")
                    .and_then(serde_json::Value::as_array);
                if let Some(sites) = sites {
                    for site in sites {
                        let mut enriched = site.clone();
                        enriched["id"] = serde_json::Value::String(id.to_string());
                        enriched["origin"] =
                            serde_json::Value::String("reviewed_addition".to_string());
                        expected.push(enriched);
                    }
                }
            }
            expected
        }
        None => Vec::new(),
    }
}