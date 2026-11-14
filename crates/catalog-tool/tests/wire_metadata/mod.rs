mod compression;
mod corruption;
mod errors_registry;
mod member_shapes;
mod service_operations;
mod trait_validation;

use catalog_tool::smithy::{Registry, build_catalog};
use inventory_core::catalog::Catalog;
use std::{error::Error, fs};
use tempfile::TempDir;

type TestResult = Result<(), Box<dyn Error>>;

pub fn compile_model(model: &str, fields: &str) -> Result<(TempDir, Vec<u8>), Box<dyn Error>> {
    let source = tempfile::tempdir()?;
    let model_path = source.path().join("models/example/nested/model.json");
    fs::create_dir_all(model_path.parent().ok_or("fixture path has no parent")?)?;
    fs::write(model_path, model)?;
    let registry = Registry::parse(fields)?;
    let bytes = build_catalog(source.path(), &registry, "fixture-commit")?;
    Ok((source, bytes))
}

pub fn catalog(bytes: &[u8]) -> Result<Catalog<'_>, Box<dyn Error>> {
    Ok(Catalog::borrow(bytes)?)
}

pub fn fixture_service(
    bytes: &[u8],
) -> Result<inventory_core::catalog::Service<'_>, Box<dyn Error>> {
    catalog(bytes)?
        .service("example")?
        .ok_or_else(|| "example service missing".into())
}
