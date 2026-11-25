use super::*;

pub(super) struct Reachability {
    direct: BTreeMap<usize, Vec<model::Declaration>>,
    resources: BTreeMap<usize, BTreeSet<String>>,
}

impl Reachability {
    pub fn new(
        service: Service<'_>,
        resources: &resources::Resources<'_, '_>,
    ) -> Result<Self, RuntimeCatalogCompileError> {
        let mut parents = BTreeMap::<usize, Vec<usize>>::new();
        let mut direct = BTreeMap::new();
        let mut reachable = BTreeMap::<_, BTreeSet<_>>::new();
        for (index, shape) in service.shapes()?.enumerate() {
            let shape = shape?;
            let declarations = model::direct(shape, resources)?;
            if !declarations.is_empty() {
                reachable.insert(
                    index,
                    declarations
                        .iter()
                        .map(|row| row.resource.clone())
                        .collect(),
                );
                direct.insert(index, declarations);
            }
            for member in shape.members()? {
                if let Target::Shape(child) = member?.target {
                    parents.entry(child).or_default().push(index);
                }
            }
            for target in [shape.element()?, shape.map_value()?].into_iter().flatten() {
                if let Target::Shape(child) = target {
                    parents.entry(child).or_default().push(index);
                }
            }
        }
        let mut next = reachable.keys().copied().collect::<Vec<_>>();
        while let Some(index) = next.pop() {
            let resources = reachable
                .get(&index)
                .cloned()
                .ok_or(RuntimeCatalogCompileError::Capability)?;
            for parent in parents.get(&index).into_iter().flatten() {
                let entry = reachable.entry(*parent).or_default();
                let before = entry.len();
                for resource in &resources {
                    if !entry.contains(resource) {
                        entry.insert(resource.clone());
                    }
                }
                if entry.len() != before {
                    next.push(*parent);
                }
            }
        }
        Ok(Self {
            direct,
            resources: reachable,
        })
    }

    pub fn contains(&self, target: Target) -> bool {
        match target {
            Target::Shape(index) => self.resources.contains_key(&index),
            _ => false,
        }
    }

    pub fn direct(&self, target: Target) -> &[model::Declaration] {
        match target {
            Target::Shape(index) => self.direct.get(&index).map_or(&[], Vec::as_slice),
            _ => &[],
        }
    }

    pub fn resources(&self, target: Target) -> impl Iterator<Item = &str> {
        let resources = match target {
            Target::Shape(index) => self.resources.get(&index),
            _ => None,
        };
        resources.into_iter().flatten().map(String::as_str)
    }
}
