use crate::{endpoints, kind_map, overlay, report, runtime_catalog};
use std::path::Path;
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeInputsError {
    #[error(transparent)]
    Read(#[from] report::ReportError),
    #[error(transparent)]
    Names(#[from] crate::names::NameError),
    #[error(transparent)]
    Kinds(#[from] kind_map::KindMapError),
    #[error(transparent)]
    Review(Box<overlay::CheckError>),
    #[error(transparent)]
    Endpoints(Box<endpoints::Error>),
    #[error(transparent)]
    Compile(#[from] runtime_catalog::RuntimeCatalogCompileError),
    #[error("compiled runtime catalog is rejected by the runtime reader: {0}")]
    Reader(#[from] inventory_core::catalog::runtime::RuntimeCatalogError),
    #[error("CloudFormation source changed during compilation")]
    SourceChanged,
}

impl From<overlay::CheckError> for RuntimeInputsError {
    fn from(error: overlay::CheckError) -> Self {
        Self::Review(Box::new(error))
    }
}

impl From<endpoints::Error> for RuntimeInputsError {
    fn from(error: endpoints::Error) -> Self {
        Self::Endpoints(Box::new(error))
    }
}

pub(super) fn augment(
    root: &Path,
    source: &Path,
    bytes: &[u8],
    source_pin: &super::provenance::SourcePin,
) -> Result<Vec<u8>, RuntimeInputsError> {
    let cfn = report::read_text(&root.join("catalog/cloudcontrol-universal.json"))?;
    let kinds = kind_map::embedded(cfn.as_bytes())?;
    let checked = overlay::check(root, source)?;
    let endpoints = endpoints::verified_endpoints(source)?;
    let sources = runtime_catalog::CatalogSources::repository(
        source_pin.commit.clone(), source_pin.sha256, Sha256::digest(cfn.as_bytes()).into(),
    );
    let augmented = runtime_catalog::augment_with_sources(bytes, &checked, &kinds, &endpoints, &sources)?;
    inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&augmented)?;
    if report::read_text(&root.join("catalog/cloudcontrol-universal.json"))? != cfn {
        return Err(RuntimeInputsError::SourceChanged);
    }
    Ok(augmented)
}
