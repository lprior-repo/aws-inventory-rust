//! One input model per service directory; identity sets are bounded by discovered files.
use super::{CompileError, Error, Registry, encode};
use inventory_core::record::{MAX_RECORD_BYTES, RecordError, Value};
use std::{
    collections::BTreeSet,
    path::{Component, Path},
};

#[derive(Default)]
pub(super) struct Compilation {
    pub services: Vec<Value>,
    size: usize,
    names: BTreeSet<String>,
    identities: BTreeSet<String>,
}
impl Compilation {
    pub(super) fn add(
        mut self,
        file: &Path,
        registry: &Registry,
        models: &Path,
    ) -> Result<Self, Error> {
        let model = crate::smithy::read_model_file(file)?;
        crate::smithy::stats::service(&model, file)?;
        let name = service_name(file, models)?;
        let bytes = self
            .service_record(&model, registry, name)
            .map_err(|error| error.in_service(name))?;
        self.size = self
            .size
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_RECORD_BYTES)
            .ok_or(RecordError::SizeLimit)
            .map_err(|error| CompileError::from(error).in_service(name))?;
        self.services.push(Value::Bytes(bytes));
        Ok(self)
    }
    fn service_record(
        &mut self,
        model: &crate::smithy::model::ModelFile,
        registry: &Registry,
        name: &str,
    ) -> Result<Vec<u8>, CompileError> {
        let (identity, _) = encode::service_shape(model)?;
        self.register(name, identity)?;
        Ok(encode::service(model, registry, name)?.into_bytes())
    }
    fn register(&mut self, name: &str, identity: &str) -> Result<(), CompileError> {
        if !self.identities.insert(identity.to_owned()) {
            return Err(CompileError::DuplicateServiceShape(identity.to_owned()));
        }
        if !self.names.insert(name.to_owned()) {
            return Err(CompileError::DuplicateService(name.to_owned()));
        }
        Ok(())
    }
}
fn service_name<'a>(file: &'a Path, models: &Path) -> Result<&'a str, CompileError> {
    let relative = file
        .strip_prefix(models)
        .map_err(|_| CompileError::ModelPath(file.into()))?;
    let mut components = relative.components();
    let Some(Component::Normal(name)) = components.next() else {
        return Err(CompileError::ModelPath(file.into()));
    };
    if components.next().is_none() {
        return Err(CompileError::ModelPath(file.into()));
    }
    name.to_str()
        .ok_or_else(|| CompileError::ModelPath(file.into()))
}
