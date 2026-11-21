mod defaults;
mod encode;
mod input;
mod metadata;
mod operation;
mod provenance;
mod registry_wire;
mod runtime_inputs;
mod service;
mod shapes;
#[cfg(test)]
mod tests;
use super::{Error, Registry, fields, write_stdout};
use crate::ReportMode;
use inventory_core::{
    catalog::{CATALOG_VERSION, Catalog, CatalogKey, MAX_CATALOG_BYTES},
    record::Value,
};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CompileError {
    #[error(transparent)]
    Record(#[from] inventory_core::record::RecordError),

    #[error(transparent)]
    Catalog(#[from] inventory_core::catalog::CatalogError),

    #[error(transparent)]
    RuntimeInputs(Box<runtime_inputs::RuntimeInputsError>),

    #[error("member {0:?} is missing; run `catalog-tool models fields --write`")]
    MissingMember(String),

    #[error("unknown or missing shape target: {0}")]
    Target(String),

    #[error("unsupported Smithy shape kind")]
    Kind,
    #[error("invalid ARN template on {0}")]
    Arn(String),
    #[error("invalid known trait payload: {0}")]
    Trait(String),
    #[error("cannot read model Git commit: {0}")]
    Commit(String),

    #[error("catalog differs; run `catalog-tool models compile --write`")]
    Stale,

    #[error("model must be inside a named service directory: {0}")]
    ModelPath(std::path::PathBuf),

    #[error("duplicate service directory: {0}")]
    DuplicateService(String),

    #[error("duplicate service shape ID: {0}")]
    DuplicateServiceShape(String),

    #[error("model source {path} is not its repository root {repository}")]
    SourceNotRepositoryRoot {
        path: std::path::PathBuf,

        repository: std::path::PathBuf,
    },

    #[error("model source has uncommitted changes: {0}")]
    SourceModified(std::path::PathBuf),

    #[error("service {service}: {source}")]
    Service {
        service: String,

        source: Box<CompileError>,
    },

    #[error("shape {shape}: {source}")]
    Shape {
        shape: String,

        source: Box<CompileError>,
    },
}

impl From<runtime_inputs::RuntimeInputsError> for CompileError {
    fn from(source: runtime_inputs::RuntimeInputsError) -> Self {
        Self::RuntimeInputs(Box::new(source))
    }
}

impl CompileError {
    fn in_service(self, service: &str) -> Self {
        Self::Service {
            service: service.to_owned(),
            source: Box::new(self),
        }
    }
    fn in_shape(self, shape: &str) -> Self {
        Self::Shape {
            shape: shape.to_owned(),
            source: Box::new(self),
        }
    }
}

pub fn compile(
    root: &Path,
    source: &Path,
    mode: ReportMode,
    pending_report: Option<String>,
) -> Result<(), Error> {
    let registry = fields::load_registry(&root.join("catalog/field-ids.txt"))?;
    let path = root.join("catalog/aws-api.catalog");
    let limit = u64::try_from(MAX_CATALOG_BYTES).map_err(|_| CompileError::Kind)?;
    let original = fields::publish::read_bytes(&path, limit)?;
    let source_pin = provenance::capture(source)?;
    let bytes = build_catalog(source, &registry, &source_pin.commit)?;
    let (bytes, relations) =
        runtime_inputs::augment(root, source, &bytes, &source_pin).map_err(CompileError::from)?;
    if let Some(report_path) = &pending_report {
        write_pending_report(report_path, &relations.inactive)?;
    }
    provenance::verify(source, &source_pin)?;
    match mode {
        ReportMode::Write => {
            fields::publish::publish_bytes(&path, original.as_deref(), &bytes, limit)?
        }
        ReportMode::Check if original.as_deref() != Some(bytes.as_slice()) => {
            return Err(CompileError::Stale.into());
        }
        _ => {}
    }
    if mode == ReportMode::Print {
        write_stdout(&format!("catalog {} bytes\n", bytes.len()))?;
    }
    Ok(())
}

fn write_pending_report(
    path: &str,
    inactive: &std::collections::BTreeMap<String, String>,
) -> Result<(), Error> {
    let mut lines = Vec::new();
    for (id, reason) in inactive {
        lines.push(format!("{id}\t{reason}"));
    }
    let content = lines.join("\n");
    let mut file = std::fs::File::create(path).map_err(|_| CompileError::Kind)?;
    std::io::Write::write_all(&mut file, content.as_bytes()).map_err(|_| CompileError::Kind)?;
    Ok(())
}

pub fn build_catalog(
    source: &Path,
    registry: &Registry,
    models_commit: &str,
) -> Result<Vec<u8>, Error> {
    let models = source.join("models");
    let files = crate::model_source::discover_models(&models)?;
    if files.is_empty() {
        return Err(crate::model_source::Error::NoModels.into());
    }
    let state = files
        .iter()
        .try_fold(input::Compilation::default(), |state, file| {
            state.add(file, registry, &models)
        })?;
    encode_root(models_commit, registry, state.services)
}

fn encode_root(
    models_commit: &str,
    registry: &Registry,
    services: Vec<Value>,
) -> Result<Vec<u8>, Error> {
    let (names, offsets) = registry_wire::encode(registry)?;
    let root = encode::map([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (
            CatalogKey::ModelsCommit,
            Value::Text(models_commit.to_owned()),
        ),
        (CatalogKey::FieldNames, names),
        (CatalogKey::FieldNameOffsets, offsets),
        (CatalogKey::Services, Value::List(services)),
    ])?;
    let bytes = inventory_core::record::encode_container(&root, MAX_CATALOG_BYTES)
        .map_err(CompileError::from)?;
    Catalog::borrow(&bytes).map_err(CompileError::from)?;
    Ok(bytes)
}
