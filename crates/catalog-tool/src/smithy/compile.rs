//! Deterministic model compiler; only one parsed JSON model survives at a time.
mod encode;
mod metadata;
mod operation;
mod service;
mod input;
mod provenance;
mod registry_wire;
mod shapes;
#[cfg(test)]
mod tests;
use super::{Error, Registry, ReportMode, fields, find_model_files, write_stdout};
use inventory_core::{
    catalog::{CATALOG_VERSION, Catalog, CatalogKey},
    record::{MAX_RECORD_BYTES, Record, Value},
};
use std::path::Path;

/// Catalog compilation failures, retaining missing member/target identities.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CompileError {
    /// Underlying record writer rejected the size, depth or content.
    #[error(transparent)]
    Record(#[from] inventory_core::record::RecordError),
    /// Generated catalog failed its independent reader.
    #[error(transparent)]
    Catalog(#[from] inventory_core::catalog::CatalogError),
    /// Registry input is stale; compilation never changes it.
    #[error("member {0:?} is missing; run `catalog-tool models fields --write`")]
    MissingMember(String),
    /// A model target is absent or unknown.
    #[error("unknown or missing shape target: {0}")]
    Target(String),
    /// A future unsupported Smithy shape kind was encountered.
    #[error("unsupported Smithy shape kind")]
    Kind,
    #[error("invalid ARN template on {0}")]
    Arn(String),
    #[error("invalid known trait payload: {0}")]
    Trait(String),
    #[error("cannot read model Git commit: {0}")]
    Commit(String),
    /// Committed bytes differ from a fresh compile.
    #[error("catalog differs; run `catalog-tool models compile --write`")]
    Stale,
    /// Service identity must come from a directory below `models`.
    #[error("model must be inside a named service directory: {0}")]
    ModelPath(std::path::PathBuf),
    /// One directory cannot contain multiple service models.
    #[error("duplicate service directory: {0}")]
    DuplicateService(String),
    /// Two service directories cannot share a Smithy service identity.
    #[error("duplicate service shape ID: {0}")]
    DuplicateServiceShape(String),
    /// The source must itself be a Git repository root, not a directory in another checkout.
    #[error("model source {path} is not its repository root {repository}")]
    SourceNotRepositoryRoot {
        /// Canonical requested source directory.
        path: std::path::PathBuf,
        /// Canonical root reported by Git.
        repository: std::path::PathBuf,
    },
    /// A tracked or untracked model differs from the source commit.
    #[error("model source has uncommitted changes: {0}")]
    SourceModified(std::path::PathBuf),
    /// Compilation failed inside this model-directory service.
    #[error("service {service}: {source}")]
    Service {
        /// Stable model-directory service identity.
        service: String,
        /// Original typed compilation failure.
        source: Box<CompileError>,
    },
    /// Encoding failed inside this full Smithy shape identity.
    #[error("shape {shape}: {source}")]
    Shape {
        /// Full owning shape ID.
        shape: String,
        /// Original typed encoding failure.
        source: Box<CompileError>,
    },
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

/// Compile, print byte size, and optionally atomically publish or compare the catalog.
/// Registry reads are bounded by 16 MiB; model files by the shared 64 MiB reader.
/// # Errors
/// Returns input, registry, provenance, compilation, publication or drift errors.
pub fn compile(root: &Path, source: &Path, mode: ReportMode) -> Result<(), Error> {
    let registry = fields::load_registry(&root.join("catalog/field-ids.txt"))?;
    let path = root.join("catalog/aws-api.catalog");
    let limit = u64::try_from(MAX_RECORD_BYTES).map_err(|_| CompileError::Kind)?;
    let original = fields::publish::read_bytes(&path, limit)?;
    let bytes = build_catalog(source, &registry, &provenance::commit(source)?)?;
    match mode {
        ReportMode::Write => {
            fields::publish::publish_bytes(&path, original.as_deref(), &bytes, limit)?
        }
        ReportMode::Check if original.as_deref() != Some(bytes.as_slice()) => {
            return Err(CompileError::Stale.into());
        }
        _ => {}
    }
    write_stdout(&format!("catalog {} bytes\n", bytes.len()))
}

/// Compile sorted models and a read-only member registry into deterministic record bytes.
/// `models_commit` is the caller's upstream Git provenance, not a timestamp or path.
/// The complete root and each service must fit `MAX_RECORD_BYTES`; bounds are never raised.
/// # Errors
/// Returns named model errors, missing IDs/targets, unknown kinds, or record limits.
pub fn build_catalog(
    source: &Path,
    registry: &Registry,
    models_commit: &str,
) -> Result<Vec<u8>, Error> {
    let models = source.join("models");
    let files = find_model_files(&models)?;
    if files.is_empty() {
        return Err(Error::NoModels);
    }
    let state = files
        .iter()
        .try_fold(input::Compilation::default(), |state, file| {
            state.add(file, registry, &models)
        })?;
    encode_root(models_commit, registry, state.services)
}

/// Encode the root record and prove the independent reader accepts it.
fn encode_root(models_commit: &str, registry: &Registry, services: Vec<Value>) -> Result<Vec<u8>, Error> {
    let (names, offsets) = registry_wire::encode(registry)?;
    let root = encode::map([
        (CatalogKey::Format, Value::Int(CATALOG_VERSION)),
        (CatalogKey::ModelsCommit, Value::Text(models_commit.to_owned())),
        (CatalogKey::FieldNames, names),
        (CatalogKey::FieldNameOffsets, offsets),
        (CatalogKey::Services, Value::List(services)),
    ])?;
    let bytes = Record::encode(&root)
        .map_err(CompileError::from)?
        .into_bytes();
    Catalog::borrow(&bytes).map_err(CompileError::from)?;
    Ok(bytes)
}
