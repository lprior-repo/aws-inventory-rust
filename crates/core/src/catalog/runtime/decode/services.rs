use super::RuntimeCatalogError;
use crate::catalog::{Catalog, CatalogError, Operation, Service};

struct Entry<'a> {
    name: &'a str,
    service: Service<'a>,
    operations: Vec<(&'a str, Operation<'a>)>,
}

pub(super) struct Services<'a>(Vec<Entry<'a>>);

impl<'a> Services<'a> {
    pub(super) fn open(catalog: Catalog<'a>) -> Result<Self, RuntimeCatalogError> {
        let mut entries = catalog
            .services()?
            .map(|service| entry(service?))
            .collect::<Result<Vec<_>, CatalogError>>()?;
        entries.sort_unstable_by_key(|entry| entry.name);
        Ok(Self(entries))
    }

    pub(super) fn operation(
        &self,
        service_name: &str,
        operation_name: &str,
    ) -> Result<(Service<'a>, Operation<'a>), RuntimeCatalogError> {
        let entry = match equal_run(&self.0, |entry| entry.name, service_name) {
            [entry] => entry,
            [] => return Err(RuntimeCatalogError::Capability),
            _ => return Err(CatalogError::Reference.into()),
        };
        match equal_run(&entry.operations, |(name, _)| *name, operation_name) {
            [(_, operation)] => Ok((entry.service, *operation)),
            _ => Err(RuntimeCatalogError::Capability),
        }
    }
}

fn entry(service: Service<'_>) -> Result<Entry<'_>, CatalogError> {
    let mut operations = service
        .operations()?
        .map(|operation| {
            let operation = operation?;
            Ok((operation.name()?, operation))
        })
        .collect::<Result<Vec<_>, CatalogError>>()?;
    operations.sort_unstable_by_key(|(name, _)| *name);
    Ok(Entry {
        name: service.name()?,
        service,
        operations,
    })
}

fn equal_run<'s, T>(sorted: &'s [T], key_of: impl Fn(&T) -> &str, key: &str) -> &'s [T] {
    let start = sorted.partition_point(|item| key_of(item) < key);
    let rest = sorted.get(start..).unwrap_or_default();
    rest.get(..rest.partition_point(|item| key_of(item) == key))
        .unwrap_or_default()
}
