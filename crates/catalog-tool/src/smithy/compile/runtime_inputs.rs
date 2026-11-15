use crate::{endpoints, kind_map, names::Exceptions, overlay, report, runtime_catalog};
use std::path::Path;

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
) -> Result<Vec<u8>, RuntimeInputsError> {
    let names = report::read_text(&root.join("catalog/name-exceptions.tsv"))?;
    let exceptions = Exceptions::parse(&names)?;
    let kinds = report::read_text(&root.join("catalog/kinds.tsv"))?;
    let cfn = report::read_text(&root.join("catalog/cloudcontrol-universal.json"))?;
    let kinds = kind_map::load_with_cfn(kinds.as_bytes(), cfn.as_bytes(), &exceptions)?;
    let checked = overlay::check(root, source)?;
    let endpoints = endpoints::verified_endpoints(source)?;
    let augmented = runtime_catalog::augment(bytes, &checked, &kinds, &endpoints)?;
    inventory_core::catalog::runtime::RuntimeCatalog::from_bytes(&augmented)?;
    Ok(augmented)
}
