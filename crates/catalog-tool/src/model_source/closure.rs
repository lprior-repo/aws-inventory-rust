use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
struct View {
    #[serde(default)]
    shapes: BTreeMap<String, Shape>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Shape {
    #[serde(default, rename = "type")]
    role: Role,
    #[serde(default)]
    operations: Vec<Target>,
    #[serde(default)]
    collection_operations: Vec<Target>,
    #[serde(default)]
    resources: Vec<Target>,
    #[serde(default)]
    create: Option<Target>,
    #[serde(default)]
    put: Option<Target>,
    #[serde(default)]
    read: Option<Target>,
    #[serde(default)]
    update: Option<Target>,
    #[serde(default)]
    delete: Option<Target>,
    #[serde(default)]
    list: Option<Target>,
}

#[derive(Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Role {
    Service,
    Operation,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct Target {
    target: String,
}

impl Shape {
    fn bound(&self) -> impl Iterator<Item = &str> {
        let lifecycle = [
            &self.create,
            &self.put,
            &self.read,
            &self.update,
            &self.delete,
            &self.list,
        ];
        self.operations
            .iter()
            .chain(&self.collection_operations)
            .chain(lifecycle.into_iter().flatten())
            .map(|target| target.target.as_str())
    }
}

pub(super) fn unbound_operations(bytes: &[u8]) -> Result<Vec<String>, serde_json::Error> {
    let view: View = serde_json::from_slice(bytes)?;
    let mut pending = view
        .shapes
        .iter()
        .filter(|(_, shape)| shape.role == Role::Service)
        .map(|(id, _)| id.as_str())
        .collect::<Vec<_>>();
    if pending.is_empty() {
        return Ok(Vec::new());
    }
    let namespaces = pending
        .iter()
        .map(|id| namespace(id))
        .collect::<BTreeSet<_>>();
    let mut visited = BTreeSet::new();
    let mut bound = BTreeSet::new();
    while let Some(id) = pending.pop() {
        let Some(shape) = view.shapes.get(id).filter(|_| visited.insert(id)) else {
            continue;
        };
        bound.extend(shape.bound());
        pending.extend(shape.resources.iter().map(|target| target.target.as_str()));
    }
    Ok(view
        .shapes
        .iter()
        .filter(|(id, shape)| {
            shape.role == Role::Operation
                && !bound.contains(id.as_str())
                && !namespaces.contains(namespace(id))
        })
        .map(|(id, _)| id.clone())
        .collect())
}

fn namespace(id: &str) -> &str {
    id.split_once('#').map_or(id, |(namespace, _)| namespace)
}

#[cfg(test)]
mod tests {
    use super::unbound_operations;

    #[test]
    fn foreign_operations_outside_the_service_resource_tree_are_unbound()
    -> Result<(), serde_json::Error> {
        let model = br#"{"shapes":{
            "n#Service":{"type":"service","operations":[{"target":"n#ListTags"}],"resources":[{"target":"n#Outer"}]},
            "n#Outer":{"type":"resource","list":{"target":"n#ListOuter"},"resources":[{"target":"n#Inner"}]},
            "n#Inner":{"type":"resource","read":{"target":"n#GetInner"},"collectionOperations":[{"target":"n#Sweep"}],"resources":[{"target":"n#Outer"}]},
            "n#Loose":{"type":"resource","operations":[{"target":"other#Transform"}]},
            "n#ListTags":{"type":"operation"},
            "n#ListOuter":{"type":"operation"},
            "n#GetInner":{"type":"operation"},
            "n#Sweep":{"type":"operation"},
            "n#Unlisted":{"type":"operation"},
            "other#ListTags":{"type":"operation"},
            "other#Transform":{"type":"operation"},
            "n#List":{"type":"list","member":{"target":"smithy.api#String"}}
        }}"#;

        assert_eq!(
            unbound_operations(model)?,
            ["other#ListTags", "other#Transform"]
        );
        Ok(())
    }

    #[test]
    fn a_model_without_a_service_keeps_every_operation() -> Result<(), serde_json::Error> {
        let model = br#"{"shapes":{"n#Call":{"type":"operation"}}}"#;

        assert_eq!(unbound_operations(model)?, Vec::<String>::new());
        Ok(())
    }
}
