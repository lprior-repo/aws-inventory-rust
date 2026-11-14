mod orchestration;
mod report;
mod service;
mod validation;
use crate::{
    classes::{self, Class, DenyList},
    plans,
    smithy::{self, ReportMode},
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

const REPORT_PATH: &str = "catalog/aws-overlay.report";
const MAX_SERVICES: usize = 4096;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CheckError {
    #[error(transparent)]
    Overlay(#[from] super::OverlayError),
    #[error(transparent)]
    Models(#[from] smithy::Error),
    #[error(transparent)]
    Decode(#[from] serde_json::Error),
    #[error(transparent)]
    Plans(#[from] plans::Error),
    #[error(transparent)]
    Classes(#[from] classes::Error),
    #[error(transparent)]
    DenyList(#[from] classes::ParseError),
    #[error("{service}: no model directory")]
    MissingService { service: String },
    #[error("{service}/{operation} {key}={value}: {rule}")]
    Invalid {
        service: String,
        operation: String,
        key: String,
        value: String,
        rule: &'static str,
    },
    #[error("{service}: incomplete review: {operations}")]
    IncompleteReview { service: String, operations: String },
    #[error("overlay validation failed: {}", report::format_violations(.0))]
    Violations(Vec<Violation>),
    #[error("{file}: committed overlay report differs")]
    Drift { file: PathBuf },
    #[error("{file}: {source}")]
    Io {
        file: PathBuf,
        source: std::io::Error,
    },
    #[error("report formatting failed: {0}")]
    Formatting(#[from] std::fmt::Error),
    #[error("overlay candidate count overflowed")]
    CountOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ViolationRule {
    UnknownService,
    UnknownOperation,
    InvalidPath,
    InvalidValue,
    OverlayPromotesDenied,
    IncompletePlan,
    IncompleteReview,
    BindingSourceNotCallable,
    BindingCycle,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    service: String,
    operation: String,
    key: String,
    value: String,
    rule: ViolationRule,
}

impl Violation {
    pub fn service(&self) -> &str {
        &self.service
    }
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn rule(&self) -> ViolationRule {
        self.rule
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableOperation {
    service: String,
    operation: String,
    class: EffectiveClass,
    plan: super::OperationOverlay,
}

impl CallableOperation {
    pub fn service(&self) -> &str {
        &self.service
    }
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub fn class(&self) -> EffectiveClass {
        self.class
    }
    pub fn plan(&self) -> &super::OperationOverlay {
        &self.plan
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveClass {
    Enumerate,
    Expand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewedService {
    service: String,
    callable: Vec<CallableOperation>,
}

impl ReviewedService {
    pub fn service(&self) -> &str {
        &self.service
    }
    pub fn callable(&self) -> &[CallableOperation] {
        &self.callable
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedOverlays {
    services: Vec<ReviewedService>,
    report: String,
}

impl CheckedOverlays {
    pub fn services(&self) -> &[ReviewedService] {
        &self.services
    }
    pub fn report(&self) -> &str {
        &self.report
    }
}

pub fn check(root: &Path, source: &Path) -> Result<CheckedOverlays, CheckError> {
    orchestration::check(root, source)
}

fn classifier(
    value: &Value,
    service: &str,
    deny: &DenyList<'_>,
) -> Result<BTreeMap<String, Class>, CheckError> {
    let model = classes::model::Model::deserialize(value)?;
    model.validate(Path::new(service))?;
    model
        .operations(service)
        .map(|facts| {
            let facts = facts?;
            Ok((facts.name.to_owned(), classes::classify(&facts, deny)))
        })
        .collect()
}

pub fn build_report(root: &Path, source: &Path) -> Result<String, CheckError> {
    Ok(check(root, source)?.report)
}

pub fn run(root: &Path, source: &Path, mode: ReportMode) -> Result<(), CheckError> {
    let report = build_report(root, source)?;
    if mode == ReportMode::Check {
        let path = root.join(REPORT_PATH);
        let current = fs::read_to_string(&path).map_err(|source| CheckError::Io {
            file: path.clone(),
            source,
        })?;
        if current != report {
            return Err(CheckError::Drift { file: path });
        }
    }
    classes::report::publish(root, REPORT_PATH, &report, mode)?;
    Ok(())
}
