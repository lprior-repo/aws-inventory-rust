//! Exact IAM grants for the compiled catalog's callable plans.
mod documents;
mod gaps;
mod reference;
pub use reference::refresh;

use crate::{ReportError, ReportMode};
use inventory_core::catalog::runtime::RuntimeCatalog;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::Path,
};
use thiserror::Error;

pub const REFERENCE_PATH: &str = "catalog/iam-actions.tsv";
pub const MAPPINGS_PATH: &str = "catalog/overlay/iam-actions.toml";
pub const EXCEPTIONS_PATH: &str = "catalog/overlay/iam-reference-exceptions.tsv";
const GAPS_PATH: &str = "deploy/read-only-policy-gaps.txt";
const POLICY_PATH: &str = "catalog/iam-read-only-policy.json";
const ROLE_PATH: &str = "deploy/read-only-role.yaml";

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Report(#[from] ReportError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Catalog(#[from] inventory_core::catalog::CatalogError),
    #[error(transparent)]
    Runtime(#[from] inventory_core::catalog::runtime::RuntimeCatalogError),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Formatting(#[from] std::fmt::Error),
    #[error(
        "{operation}: {action} is absent from {REFERENCE_PATH}; add a verified overlay mapping or pinned unconfirmed exception"
    )]
    Unmapped { operation: String, action: String },
    #[error("{operation}: {action} is not read-only")]
    NotReadOnly { operation: String, action: String },
    #[error("{operation}: empty IAM mapping")]
    EmptyMapping { operation: String },
    #[error("invalid reference exception row {row:?}: {reason}")]
    BadException { row: String, reason: ExceptionFault },
    #[error("{path}: output is not a file", path = path.display())]
    OutputNotFile { path: std::path::PathBuf },
    #[error("{operation}: missing signing name")]
    MissingSigningName { operation: String },
    #[error("no callable actions")]
    NoCallableActions,
    #[error("{path}: first difference at byte {byte}", path = path.display())]
    ArtifactDrift {
        path: std::path::PathBuf,
        byte: usize,
    },
    #[error("{path}: output parent missing", path = path.display())]
    OutputParent { path: std::path::PathBuf },
    #[error("{path}: publish failed: {source}; rollback failures: {rollback:?}", path = path.display())]
    Publication {
        path: std::path::PathBuf,
        source: std::io::Error,
        rollback: Vec<RollbackFailure>,
    },
    #[error("reference pin date missing")]
    MissingReferenceDate,
    #[error("invalid IAM reference row: {row}")]
    BadReferenceRow { row: String },
    #[error("duplicate IAM reference row: {row}")]
    DuplicateReferenceRow { row: String },
    #[error("invalid date {date}; expected calendar YYYY-MM-DD")]
    InvalidReferenceDate { date: String },
    #[error("reference fetch {url}: {stderr}")]
    ReferenceFetch { url: String, stderr: String },
    #[error("foreign reference URL: {url}")]
    ForeignReferenceUrl { url: String },
    #[error("reference service mismatch: expected {expected}, received {actual}")]
    ReferenceServiceMismatch { expected: String, actual: String },
    #[error("action exceeds managed-policy size {limit}: {action}")]
    ActionTooLarge { action: String, limit: usize },
    #[error("{count} managed policies exceed role maximum {limit}")]
    TooManyPolicies { count: usize, limit: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ExceptionFault {
    #[error("expected service, operation, signing name and pinned reason")]
    Malformed,
    #[error("operation is no longer callable")]
    NoLongerCallable,
    #[error("signing name or reference pin date differs")]
    PinMismatch,
    #[error("stale exception; action is published or mapped")]
    Stale,
    #[error("duplicate exception")]
    Duplicate,
}

#[derive(Debug, Error)]
#[error("{path}: {source}", path = path.display())]
pub struct RollbackFailure {
    pub path: std::path::PathBuf,
    pub source: std::io::Error,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Operation<'a> {
    pub service: &'a str,
    pub name: &'a str,
    pub prefix: &'a str,
}

impl<'a> Operation<'a> {
    pub fn new(service: &'a str, name: &'a str, prefix: &'a str) -> Self {
        Self {
            service,
            name,
            prefix,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Rendered {
    /// Bundle of individually attachable managed policy documents.
    pub text: String,
    pub template: String,
    pub actions: BTreeSet<String>,
    pub policies: Vec<String>,
    /// Default-name grants absent from the pinned reference, not confirmed by AWS.
    pub unconfirmed: BTreeSet<String>,
    pub gap_report: String,
    pub operation_actions: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mappings {
    #[serde(default)]
    operation: BTreeMap<String, String>,
}

pub fn operations<'a>(catalog: &'a RuntimeCatalog<'_>) -> Result<Vec<Operation<'a>>, Error> {
    catalog
        .plans()
        .iter()
        .map(|plan| {
            Ok(Operation {
                service: plan.service_name(),
                name: plan.operation_name(),
                prefix: plan.service().signing_name()?.ok_or_else(|| {
                    Error::MissingSigningName {
                        operation: format!("{}/{}", plan.service_name(), plan.operation_name()),
                    }
                })?,
            })
        })
        .collect()
}

/// Only callable plans enter this boundary; deny/defer decisions never grant actions.
pub fn render(
    operations: &[Operation<'_>],
    mappings: &str,
    reference: &str,
    exceptions: &str,
) -> Result<Rendered, Error> {
    let mappings: Mappings = toml::from_str(mappings)?;
    let known = reference::parse(reference)?;
    let unconfirmed = gaps::parse(exceptions, reference, &known, operations, &mappings)?;
    let operation_actions: BTreeMap<_, _> = operations
        .iter()
        .map(|op| resolve_action(op, &mappings, &known, &unconfirmed))
        .collect::<Result<_, _>>()?;
    let actions: BTreeSet<_> = operation_actions.values().cloned().collect();
    if actions.is_empty() {
        return Err(Error::NoCallableActions);
    }
    let policies = documents::split(&actions)?;
    Ok(Rendered {
        text: format!("[\n{}\n]\n", policies.join(",\n")),
        template: documents::template(&policies),
        actions,
        policies,
        gap_report: gaps::report(&unconfirmed),
        unconfirmed,
        operation_actions,
    })
}

fn resolve_action(
    op: &Operation<'_>,
    mappings: &Mappings,
    known: &BTreeMap<String, &str>,
    unconfirmed: &BTreeSet<String>,
) -> Result<(String, String), Error> {
    let id = format!("{}/{}", op.service, op.name);
    let action = match mappings.operation.get(&id) {
        Some(action) if action.is_empty() => {
            return Err(Error::EmptyMapping { operation: id });
        }
        Some(action) => action.clone(),
        None => format!("{}:{}", op.prefix, op.name),
    };
    match known.get(&action) {
        Some(&"read") => {}
        Some(_) => {
            return Err(Error::NotReadOnly {
                operation: id,
                action,
            });
        }
        None if unconfirmed.contains(&id) => {
            if !crate::smithy::READ_PREFIXES
                .iter()
                .any(|verb| op.name.starts_with(verb))
            {
                return Err(Error::NotReadOnly {
                    operation: id,
                    action,
                });
            }
        }
        None => {
            return Err(Error::Unmapped {
                operation: id,
                action,
            });
        }
    }
    Ok((id, action))
}

pub fn run(root: &Path, mode: ReportMode) -> Result<(), Error> {
    let bytes = std::fs::read(root.join("catalog/aws-api.catalog"))?;
    let catalog = RuntimeCatalog::from_bytes(&bytes)?;
    let mappings = crate::report::read_text(&root.join(MAPPINGS_PATH))?;
    let mut output = render(
        &operations(&catalog)?,
        &mappings,
        &crate::report::read_text(&root.join(REFERENCE_PATH))?,
        &crate::report::read_text(&root.join(EXCEPTIONS_PATH))?,
    )?;
    if mode == ReportMode::Print {
        return Ok(crate::report::write_stdout(&output.text)?);
    }
    output.gap_report.push_str(&gaps::shared(
        &catalog,
        &mappings,
        &output.operation_actions,
    )?);
    let artifacts = [
        (POLICY_PATH, output.text),
        (ROLE_PATH, output.template),
        (GAPS_PATH, output.gap_report),
    ];
    if mode == ReportMode::Write {
        let staged = artifacts
            .iter()
            .map(|(path, text)| stage(&root.join(path), text))
            .collect::<Result<Vec<_>, _>>()?;
        return install(&staged);
    }
    artifacts
        .iter()
        .try_for_each(|(path, text)| check_artifact(&root.join(path), text))
}

fn check_artifact(path: &Path, expected: &str) -> Result<(), Error> {
    let actual = crate::report::read_text(path)?;
    if actual == expected {
        return Ok(());
    }
    let byte = actual
        .bytes()
        .zip(expected.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    Err(Error::ArtifactDrift {
        path: path.to_owned(),
        byte,
    })
}

struct Staged {
    path: std::path::PathBuf,
    fresh: tempfile::NamedTempFile,
    original: Option<tempfile::NamedTempFile>,
}

fn stage(path: &Path, text: &str) -> Result<Staged, Error> {
    let parent = path.parent().ok_or_else(|| Error::OutputParent {
        path: path.to_owned(),
    })?;
    std::fs::create_dir_all(parent)?;
    let fresh = tempfile::NamedTempFile::new_in(parent)?;
    crate::package::atomic_write(fresh.path(), |file| -> Result<(), Error> {
        file.write_all(text.as_bytes())?;
        Ok(file.sync_all()?)
    })?;
    let original = match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => {
            let backup = tempfile::NamedTempFile::new_in(parent)?;
            std::fs::copy(path, backup.path())?;
            backup.as_file().sync_all()?;
            Some(backup)
        }
        Ok(_) => {
            return Err(Error::OutputNotFile {
                path: path.to_owned(),
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    Ok(Staged {
        path: path.to_owned(),
        fresh,
        original,
    })
}

fn install(staged: &[Staged]) -> Result<(), Error> {
    staged.iter().enumerate().try_for_each(|(index, file)| {
        if let Err(error) = std::fs::rename(file.fresh.path(), &file.path) {
            let rollback = staged
                .iter()
                .take(index)
                .rev()
                .filter_map(|old| {
                    let result = match &old.original {
                        Some(backup) => std::fs::rename(backup.path(), &old.path),
                        None => std::fs::remove_file(&old.path),
                    };
                    result.err().map(|source| RollbackFailure {
                        path: old.path.clone(),
                        source,
                    })
                })
                .collect();
            return Err(Error::Publication {
                path: file.path.clone(),
                source: error,
                rollback,
            });
        }
        Ok(())
    })
}

#[cfg(test)]
mod output_tests {
    use super::*;

    #[test]
    fn given_later_install_failure_when_publishing_then_previous_files_are_restored()
    -> Result<(), Error> {
        let directory = tempfile::tempdir()?;
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        std::fs::write(&first, "old first")?;
        std::fs::write(&second, "old second")?;
        let staged = [stage(&first, "new first")?, stage(&second, "new second")?];
        std::fs::remove_file(&second)?;
        std::fs::create_dir(&second)?;
        let result = install(&staged);
        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(first)?, "old first");
        assert!(second.is_dir());
        Ok(())
    }

    #[test]
    fn given_new_output_when_later_publication_fails_then_rollback_removes_it() -> Result<(), Error>
    {
        let directory = tempfile::tempdir()?;
        let first = directory.path().join("new");
        let second = directory.path().join("blocked");
        let staged = [stage(&first, "new")?, stage(&second, "later")?];
        std::fs::create_dir(&second)?;
        assert!(install(&staged).is_err());
        assert!(!first.try_exists()?);
        assert!(second.is_dir());
        Ok(())
    }
}

#[cfg(test)]
mod refusal_tests {
    use super::*;

    #[test]
    fn given_generator_refusals_when_returned_then_errors_are_not_bare_strings()
    -> Result<(), Box<dyn std::error::Error>> {
        let operations = [Operation::new("s3", "ListBuckets", "s3")];
        let reference = "# Retrieved: 2026-11-24\nec2\tDeleteVpc\twrite\n";
        let cases = [
            ("", ""),
            ("[operation]\n\"s3/ListBuckets\" = \"ec2:DeleteVpc\"", ""),
            ("[operation]\n\"s3/ListBuckets\" = \"\"", ""),
            ("", "bad row"),
        ];
        for (index, (mapping, exceptions)) in cases.into_iter().enumerate() {
            let error = render(&operations, mapping, reference, exceptions)
                .err()
                .ok_or("invalid input was accepted")?;
            let kind = match error {
                Error::Unmapped { operation, action }
                    if operation == "s3/ListBuckets" && action == "s3:ListBuckets" =>
                {
                    0
                }
                Error::NotReadOnly { operation, action }
                    if operation == "s3/ListBuckets" && action == "ec2:DeleteVpc" =>
                {
                    1
                }
                Error::EmptyMapping { operation } if operation == "s3/ListBuckets" => 2,
                Error::BadException {
                    row,
                    reason: ExceptionFault::Malformed,
                } if row == "bad row" => 3,
                other => return Err(format!("unexpected refusal: {other}").into()),
            };
            assert_eq!(kind, index);
        }
        let directory = tempfile::tempdir()?;
        let error = stage(directory.path(), "new")
            .err()
            .ok_or("directory accepted")?;
        assert!(matches!(error, Error::OutputNotFile { path } if path == directory.path()));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn given_staged_output_when_installed_then_mode_matches_atomic_write() -> Result<(), Error> {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir()?;
        let reference = directory.path().join("reference");
        crate::package::atomic_write(&reference, |file| -> std::io::Result<()> {
            file.write_all(b"reference")
        })?;
        let path = directory.path().join("published");
        install(&[stage(&path, "published")?])?;
        let expected = std::fs::metadata(reference)?.permissions().mode() & 0o777;
        assert_eq!(
            std::fs::metadata(path)?.permissions().mode() & 0o777,
            expected
        );
        Ok(())
    }

    #[test]
    fn given_two_rollback_failures_when_publication_fails_then_both_paths_are_reported()
    -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let paths = ["first", "second", "third"].map(|name| directory.path().join(name));
        let staged = paths
            .iter()
            .map(|path| -> Result<_, Error> {
                std::fs::write(path, "old")?;
                stage(path, "new")
            })
            .collect::<Result<Vec<_>, _>>()?;
        for file in staged.iter().take(2) {
            std::fs::remove_file(file.original.as_ref().ok_or("backup missing")?.path())?;
        }
        let third = paths.last().ok_or("third path missing")?;
        std::fs::remove_file(third)?;
        std::fs::create_dir(third)?;
        let error = install(&staged)
            .err()
            .ok_or("publish unexpectedly succeeded")?;
        let message = error.to_string();
        for path in paths.iter().take(2) {
            assert!(message.contains(&path.display().to_string()), "{message}");
        }
        let Error::Publication { rollback, .. } = error else {
            return Err("publication error lost its typed rollback failures".into());
        };
        assert_eq!(rollback.len(), 2);
        Ok(())
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::*;
    use inventory_core::catalog::runtime::ReviewDisposition;
    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    fn root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn compile_real_catalog() -> TestResult<Vec<u8>> {
        let root = root();
        let source = root.join(".local/aws-models");
        let fields = crate::smithy::Registry::parse(&std::fs::read_to_string(
            root.join("catalog/field-ids.txt"),
        )?)?;
        let base = crate::smithy::build_catalog(&source, &fields, "iam-policy-pinned-models")?;
        // Only tracked TOML is input: no generated CFN JSON, kinds TSV or binary catalog.
        let cfn: toml::Value = toml::from_str(&std::fs::read_to_string(
            root.join("catalog/cloudcontrol-capabilities.toml"),
        )?)?;
        let cfn = serde_json::to_vec(&cfn)?;
        let checked = crate::overlay::check(&root, &source)?;
        let hand = crate::kind_map::embedded(&cfn)?;
        let values = crate::overlay::read_model_values(&source)?;
        let generated = crate::kind_map::generate::generate(&values, &checked, &hand, &cfn)?;
        let kinds = crate::kind_map::load_with_cfn(&cfn, generated.text.as_bytes())?;
        let endpoints = crate::endpoints::verified_endpoints(&source)?;
        Ok(crate::runtime_catalog::augment(
            &base, &checked, &kinds, &endpoints,
        )?)
    }

    #[test]
    fn given_real_catalog_when_generated_then_policy_is_exactly_callable_action_union() -> TestResult
    {
        let bytes = compile_real_catalog()?;
        let catalog = RuntimeCatalog::from_bytes(&bytes)?;
        let mappings = std::fs::read_to_string(root().join(MAPPINGS_PATH))?;
        let output = render(
            &operations(&catalog)?,
            &mappings,
            &std::fs::read_to_string(root().join(REFERENCE_PATH))?,
            &std::fs::read_to_string(root().join(EXCEPTIONS_PATH))?,
        )?;
        assert_review_decisions(&catalog, &output, &toml::from_str::<Mappings>(&mappings)?)?;
        println!(
            "{}",
            gaps::shared(&catalog, &mappings, &output.operation_actions)?
        );
        assert!(
            output.policies.iter().all(|policy| policy
                .chars()
                .filter(|c| !c.is_whitespace())
                .count()
                <= 6144)
        );
        assert!(output.policies.len() <= 20);
        assert!(output.template.len() <= 1_000_000);
        for (operation, denied) in [("DescribeClusters", true), ("DescribeLoggingStatus", false)] {
            let decision = catalog
                .decisions()
                .iter()
                .find(|decision| decision.service == "redshift" && decision.operation == operation)
                .ok_or("read-named redshift decision missing")?;
            assert!(if denied {
                matches!(decision.disposition, ReviewDisposition::Denied { .. })
            } else {
                matches!(decision.disposition, ReviewDisposition::Deferred { .. })
            });
        }
        Ok(())
    }

    fn assert_review_decisions(
        catalog: &RuntimeCatalog<'_>,
        output: &Rendered,
        mappings: &Mappings,
    ) -> TestResult {
        let policy = parsed_actions(&output.text)?;
        let (mut required, mut excluded) = (BTreeSet::new(), BTreeSet::new());
        let mut prefixes = BTreeMap::new();
        for service in catalog.catalog().services()? {
            let service = service?;
            prefixes.insert(service.name()?, service.signing_name()?);
        }
        let mut callable = 0_usize;
        for decision in catalog.decisions() {
            let id = format!("{}/{}", decision.service, decision.operation);
            let plan = catalog.lookup(&decision.service, &decision.operation);
            let action = match mappings.operation.get(&id) {
                Some(action) => Some(action.clone()),
                None => prefixes
                    .get(decision.service.as_str())
                    .copied()
                    .flatten()
                    .map(|prefix| format!("{prefix}:{}", decision.operation)),
            };
            match &decision.disposition {
                ReviewDisposition::Callable => {
                    let action = action.ok_or("callable signing name missing")?;
                    assert!(plan.is_some(), "{id}: callable decision has no plan");
                    assert_eq!(output.operation_actions.get(&id), Some(&action), "{id}");
                    required.insert(action);
                    callable = callable.saturating_add(1);
                }
                _ => {
                    assert!(plan.is_none(), "{id}: excluded decision has a plan");
                    assert!(
                        !output.operation_actions.contains_key(&id),
                        "{id}: excluded decision grants"
                    );
                    if let Some(action) = action {
                        excluded.insert(action);
                    }
                }
            }
        }
        assert_eq!(policy, required);
        assert_eq!(policy, output.actions);
        for action in excluded.difference(&required) {
            assert!(
                !policy.contains(action),
                "non-callable-only action {action}"
            );
        }
        assert_eq!(callable, catalog.plans().len());
        assert_eq!(callable, output.operation_actions.len());
        Ok(())
    }

    fn parsed_actions(text: &str) -> TestResult<BTreeSet<String>> {
        let documents: Vec<serde_json::Value> = serde_json::from_str(text)?;
        documents
            .iter()
            .try_fold(BTreeSet::new(), |mut actions, document| {
                assert_eq!(
                    document
                        .get("Statement")
                        .and_then(serde_json::Value::as_array)
                        .map(Vec::len),
                    Some(1)
                );
                assert_eq!(
                    document
                        .pointer("/Statement/0/Effect")
                        .and_then(serde_json::Value::as_str),
                    Some("Allow")
                );
                let names = document
                    .pointer("/Statement/0/Action")
                    .and_then(serde_json::Value::as_array)
                    .ok_or("policy actions absent")?;
                for name in names {
                    actions.insert(name.as_str().ok_or("non-string policy action")?.to_owned());
                }
                Ok(actions)
            })
    }
}
