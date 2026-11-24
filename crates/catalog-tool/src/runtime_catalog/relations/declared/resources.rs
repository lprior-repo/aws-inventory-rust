use super::*;

pub(super) struct Resource<'s, 'p> {
    pub identifiers: BTreeSet<&'s str>,
    pub callable: Vec<usize>,
    pub pending: BTreeSet<&'p str>,
}

pub(super) struct Resources<'s, 'p> {
    namespace: &'s str,
    rows: BTreeMap<&'s str, Resource<'s, 'p>>,
}

impl<'s, 'p> Resources<'s, 'p> {
    pub fn new(
        service: Service<'s>,
        name: &str,
        context: &ValidationContext<'p, '_>,
        sources: &[ResourceSource<'p>],
    ) -> Result<Self, RuntimeCatalogCompileError> {
        let namespace = service
            .shape_id()?
            .split_once('#')
            .ok_or(RuntimeCatalogCompileError::Capability)?
            .0;
        let operations = sources
            .iter()
            .enumerate()
            .filter(|(_, source)| source.service == name)
            .fold(
                BTreeMap::<_, Vec<_>>::new(),
                |mut operations, (index, source)| {
                    operations.entry(source.operation).or_default().push(index);
                    operations
                },
            );
        let kinds = context
            .kinds
            .operations()
            .iter()
            .filter(|row| row.service == name)
            .fold(BTreeMap::<_, BTreeSet<_>>::new(), |mut kinds, row| {
                kinds
                    .entry(row.operation.as_str())
                    .or_default()
                    .insert(row.kind.as_str());
                kinds
            });
        let rows = service
            .shapes()?
            .try_fold(BTreeMap::new(), |mut rows, shape| {
                let shape = shape?;
                if shape.kind()? == "resource" {
                    let resource = resource(shape, &operations, &kinds, sources)?;
                    rows.insert(shape.name()?, resource);
                }
                Ok::<_, RuntimeCatalogCompileError>(rows)
            })?;
        Ok(Self { namespace, rows })
    }

    pub fn get(&self, id: &str) -> Option<&Resource<'s, 'p>> {
        let (namespace, name) = id.split_once('#')?;
        (namespace == self.namespace)
            .then(|| self.rows.get(name))
            .flatten()
    }
}

fn resource<'s, 'p>(
    shape: Shape<'s>,
    operations: &BTreeMap<&str, Vec<usize>>,
    kinds: &BTreeMap<&str, BTreeSet<&'p str>>,
    sources: &[ResourceSource<'p>],
) -> Result<Resource<'s, 'p>, RuntimeCatalogCompileError> {
    let names = shape
        .resource_operations()?
        .collect::<Result<BTreeSet<_>, _>>()?;
    let callable = names
        .iter()
        .flat_map(|name| operations.get(name).into_iter().flatten())
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let pending = names
        .iter()
        .flat_map(|name| kinds.get(name).into_iter().flatten())
        .copied()
        .filter(|kind| {
            !callable
                .iter()
                .filter_map(|index| sources.get(*index))
                .any(|source| source.emits_kind(kind))
        })
        .collect();
    let identifiers = shape.resource_identifiers()?.collect::<Result<_, _>>()?;
    Ok(Resource {
        identifiers,
        callable,
        pending,
    })
}
