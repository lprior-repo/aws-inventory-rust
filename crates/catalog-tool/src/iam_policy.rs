//! Read-only IAM policy generated from the pinned operation catalog.
//!
//! Actions come only from each callable operation's reviewed `iam_actions`
//! field. Nothing is derived from the service namespace or the operation name,
//! because IAM names do not follow SDK names (`s3.list_buckets` is
//! `s3:ListAllMyBuckets`; `cloudcontrol.*` is `cloudformation:*`).

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::report::{ReportError, ReportMode};

const OPERATIONS_PATH: &str = "catalog/aws-operations.json";
const POLICY_PATH: &str = "catalog/iam-read-only-policy.json";
const POLICY_VERSION: &str = "2012-10-17";
// Resource and condition data are absent from the catalog, so the policy
// grants read actions on every resource and states that limit explicitly.
const RESOURCE: &str = "*";
// IAM managed-policy limit, counted in non-whitespace characters.
const MAX_POLICY_NON_WHITESPACE: usize = 6144;
// A verb matches only when the name ends or the next character starts a new
// word, so `Getaway` and `Listen` are refused while `GetObject` is allowed.
const READ_VERBS: [&str; 6] = ["BatchGet", "Describe", "Get", "List", "Search", "Select"];

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Report(#[from] ReportError),

    #[error("{file}: {source}")]
    Io { file: PathBuf, source: io::Error },

    #[error("invalid operation catalog: {0}")]
    Json(#[from] serde_json::Error),

    #[error("no callable operations in the catalog")]
    Empty,

    #[error(
        "{} callable operation(s) have no iam_actions mapping; refusing to emit a partial policy: {}",
        .ids.len(),
        .ids.join(", ")
    )]
    Unmapped { ids: Vec<String> },

    #[error(
        "{id}: IAM action {action:?} is malformed; expected <service>:<ActionName> with no wildcard"
    )]
    Malformed { id: String, action: String },

    #[error("{id}: IAM action {action} is not on the read-only verb allowlist")]
    NotReadOnly { id: String, action: String },

    #[error("policy has {found} non-whitespace characters; the IAM limit is {limit}")]
    TooLarge { found: usize, limit: usize },

    #[error("unmapped operations are report-only and cannot be written or checked")]
    ReportOnlyPublication,
}

/// What to do with callable operations that have no `iam_actions` mapping.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnmappedPolicy {
    /// Fail closed: any unmapped operation aborts generation.
    #[default]
    Refuse,
    /// Print-only diagnostics: unmapped operations are listed and excluded.
    Report,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    pub text: String,
    /// Operations excluded from the policy; empty unless `UnmappedPolicy::Report`.
    pub unmapped: Vec<String>,
}

#[derive(Deserialize)]
struct Catalog {
    operations: Vec<Operation>,
}

#[derive(Deserialize)]
struct Operation {
    id: String,
    #[serde(default)]
    iam_actions: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Policy {
    version: &'static str,
    statement: Vec<Statement>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Statement {
    effect: &'static str,
    action: Vec<String>,
    resource: &'static str,
}

pub fn run(root: &Path, mode: ReportMode, unmapped: UnmappedPolicy) -> Result<Vec<String>, Error> {
    let operations_path = root.join(OPERATIONS_PATH);
    let source = fs::read_to_string(&operations_path).map_err(|source| Error::Io {
        file: operations_path.clone(),
        source,
    })?;
    let rendered = render(&source, unmapped)?;
    if mode != ReportMode::Print && unmapped == UnmappedPolicy::Report {
        return Err(Error::ReportOnlyPublication);
    }
    crate::report::publish(&root.join(POLICY_PATH), &rendered.text, mode)?;
    Ok(rendered.unmapped)
}

/// Renders the policy document for a catalog. Output is byte-stable: actions
/// are sorted and deduplicated, and the document ends with one newline.
pub fn render(catalog_json: &str, unmapped: UnmappedPolicy) -> Result<Rendered, Error> {
    let catalog: Catalog = serde_json::from_str(catalog_json)?;
    if catalog.operations.is_empty() {
        return Err(Error::Empty);
    }

    let mut actions = BTreeSet::new();
    let mut missing = Vec::new();
    for operation in &catalog.operations {
        if operation.iam_actions.is_empty() {
            missing.push(operation.id.clone());
            continue;
        }
        for action in &operation.iam_actions {
            validate_read_action(&operation.id, action)?;
            actions.insert(action.clone());
        }
    }

    if !missing.is_empty() && unmapped == UnmappedPolicy::Refuse {
        return Err(Error::Unmapped { ids: missing });
    }
    if actions.is_empty() {
        return Err(Error::Empty);
    }

    let policy = Policy {
        version: POLICY_VERSION,
        statement: vec![Statement {
            effect: "Allow",
            action: actions.into_iter().collect(),
            resource: RESOURCE,
        }],
    };
    let found = serde_json::to_string(&policy)?
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if found > MAX_POLICY_NON_WHITESPACE {
        return Err(Error::TooLarge {
            found,
            limit: MAX_POLICY_NON_WHITESPACE,
        });
    }

    let mut text = serde_json::to_string_pretty(&policy)?;
    text.push('\n');
    Ok(Rendered {
        text,
        unmapped: missing,
    })
}

fn validate_read_action(id: &str, action: &str) -> Result<(), Error> {
    let malformed = || Error::Malformed {
        id: id.to_owned(),
        action: action.to_owned(),
    };
    let (service, name) = action.split_once(':').ok_or_else(malformed)?;
    if !is_service_prefix(service) || name.is_empty() || !name.bytes().all(is_name_byte) {
        return Err(malformed());
    }
    if !is_read_only_name(name) {
        return Err(Error::NotReadOnly {
            id: id.to_owned(),
            action: action.to_owned(),
        });
    }
    Ok(())
}

fn is_service_prefix(service: &str) -> bool {
    !service.is_empty()
        && service
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

fn is_read_only_name(name: &str) -> bool {
    READ_VERBS.iter().any(|verb| {
        name.strip_prefix(verb).is_some_and(|rest| {
            rest.chars()
                .next()
                .is_none_or(|next| next.is_ascii_uppercase())
        })
    })
}
