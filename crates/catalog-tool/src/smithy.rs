mod compile;
mod fields;
mod model;
mod registry;
mod report;
mod stats;
mod totals;

pub(crate) use crate::report::write_stdout;
pub use compile::{CompileError, build_catalog, compile};
pub use fields::fields;
pub use registry::{Error as RegistryError, Registry};
pub(crate) use stats::READ_PREFIXES;

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

const REPORT_PATH: &str = "catalog/aws-models.report";

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    ModelSource(#[from] crate::model_source::Error),
    #[error("invalid Smithy model {file}: {message}")]
    InvalidModel { file: PathBuf, message: String },

    #[error("{file}: {source}")]
    Io {
        file: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("no service shape in {0}")]
    NoService(PathBuf),

    #[error("several service shapes in {0}")]
    SeveralServices(PathBuf),

    #[error("{file}: {source}")]
    InvalidInput {
        file: PathBuf,
        #[source]
        source: crate::model_source::InputError,
    },

    #[error(transparent)]
    InputResolution(#[from] crate::model_source::InputError),

    #[error(transparent)]
    Compile(#[from] compile::CompileError),

    #[error(transparent)]
    Report(#[from] crate::report::ReportError),

    #[error("report formatting failed: {0}")]
    Formatting(#[from] std::fmt::Error),

    #[error(transparent)]
    Registry(#[from] RegistryError),

    #[error(
        "{new_names} member names are not in the registry, first: {first}; run `catalog-tool models fields --write`"
    )]
    RegistryStale {
        path: PathBuf,
        new_names: usize,
        first: String,
    },

    #[error("registry changed during the run; rerun")]
    RegistryChanged { path: PathBuf },

    #[error("{path}: another run is writing or a previous run crashed; remove the temporary file")]
    WriteInProgress { path: PathBuf },

    #[error("{path}: registry exceeds {limit} bytes")]
    RegistryTooLarge { path: PathBuf, limit: u64 },

    #[error("{original}; failed to remove temporary {path}: {cleanup}")]
    PublicationCleanup {
        #[source]
        original: Box<Error>,
        path: PathBuf,
        cleanup: io::Error,
    },
}

pub fn run(root: &Path, source: &Path, mode: crate::ReportMode) -> Result<(), Error> {
    let text = build_report(source)?;
    crate::report::publish(&root.join(REPORT_PATH), &text, mode)?;
    Ok(())
}
pub fn build_report(source: &Path) -> Result<String, Error> {
    let files = crate::model_source::discover_models(&source.join("models"))?;
    if files.is_empty() {
        return Err(crate::model_source::Error::NoModels.into());
    }
    let totals = files.iter().try_fold(
        totals::Totals::default(),
        |mut totals, file| -> Result<_, Error> {
            totals.add_model(read_model_file(file)?, file)?;
            Ok(totals)
        },
    )?;
    Ok(report::render_report(&totals)?)
}

fn io_error(file: &Path, source: io::Error) -> Error {
    Error::Io {
        file: file.to_path_buf(),
        source,
    }
}

fn read_model_file(file: &Path) -> Result<model::ModelFile, Error> {
    let model = crate::model_source::read_model(file)?;
    stats::validate_inputs(&model, file)?;
    let model = model::mixins::expand(model).map_err(|message| Error::InvalidModel {
        file: file.to_path_buf(),
        message,
    })?;
    Ok(model)
}

#[cfg(test)]
mod bounds_tests;
#[cfg(test)]
mod tests;
