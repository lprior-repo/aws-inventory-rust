//! Bounded schema traversal used by relationship compilation.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::HarnessError;
use crate::catalog::schema::{escape, target};

const MAX_SCHEMA_NODES: usize = 40_000;
const MAX_SCHEMA_DEPTH: usize = 32;

/// Resolve the declared properties and arrays of a retained provider schema.
///
/// Opaque maps are left opaque and a recursive definition is visited once per
/// branch, so a cycle cannot justify unbounded generated paths.
pub fn schema_nodes(schema: &Value) -> Result<BTreeMap<String, Value>, HarnessError> {
    let mut traversal = Traversal {
        root: schema,
        visited: 0,
        output: BTreeMap::new(),
    };
    traversal.walk(schema, "", &[], 0)?;
    Ok(traversal.output)
}

struct Traversal<'a> {
    root: &'a Value,
    visited: usize,
    output: BTreeMap<String, Value>,
}

impl Traversal<'_> {
    fn walk(
        &mut self,
        node: &Value,
        path: &str,
        refs: &[String],
        depth: usize,
    ) -> Result<(), HarnessError> {
        self.bump(depth)?;
        let Some((current, branch_refs)) = self.current(node, refs)? else {
            return Ok(());
        };
        if !path.is_empty() {
            self.output
                .entry(path.to_string())
                .or_insert_with(|| current.clone());
        }
        self.walk_alternatives(&current, path, &branch_refs, depth)?;
        self.walk_properties(&current, path, &branch_refs, depth)?;
        self.walk_items(&current, path, &branch_refs, depth)
    }

    fn bump(&mut self, depth: usize) -> Result<(), HarnessError> {
        self.visited = self.visited.saturating_add(1);
        if self.visited > MAX_SCHEMA_NODES || depth > MAX_SCHEMA_DEPTH {
            let name = self
                .root
                .get("typeName")
                .and_then(Value::as_str)
                .unwrap_or("None");
            return Err(HarnessError::invalid(format!(
                "schema traversal budget exceeded: {name}"
            )));
        }
        Ok(())
    }

    fn current(
        &self,
        node: &Value,
        refs: &[String],
    ) -> Result<Option<(Value, Vec<String>)>, HarnessError> {
        if node.as_object().is_none() {
            return Ok(None);
        }
        let Some(reference) = node.get("$ref").and_then(Value::as_str) else {
            return Ok(Some((node.clone(), refs.to_vec())));
        };
        if refs.iter().any(|known| known == reference) {
            return Ok(None);
        }
        if !reference.starts_with("#/") {
            return Err(HarnessError::invalid(
                "external schema reference cannot establish a local mapping",
            ));
        }
        let mut branch_refs = refs.to_vec();
        branch_refs.push(reference.to_string());
        let merged = self.merged_target(node, reference)?;
        Ok(Some((merged, branch_refs)))
    }

    fn merged_target(&self, node: &Value, reference: &str) -> Result<Value, HarnessError> {
        let mut merged = target(self.root, reference)?.clone();
        if let (Some(merged_map), Some(original)) = (merged.as_object_mut(), node.as_object()) {
            for (key, value) in original {
                if key != "$ref" {
                    merged_map.insert(key.clone(), value.clone());
                }
            }
        }
        Ok(merged)
    }

    fn walk_alternatives(
        &mut self,
        node: &Value,
        path: &str,
        refs: &[String],
        depth: usize,
    ) -> Result<(), HarnessError> {
        for alternative in ["allOf", "anyOf", "oneOf"] {
            for branch in branches(node, alternative) {
                self.walk(branch, path, refs, depth.saturating_add(1))?;
            }
        }
        Ok(())
    }

    fn walk_properties(
        &mut self,
        node: &Value,
        path: &str,
        refs: &[String],
        depth: usize,
    ) -> Result<(), HarnessError> {
        let Some(Value::Object(properties)) = node.get("properties") else {
            return Ok(());
        };
        for (name, child) in properties {
            let child_path = format!("{path}/{}", escape(name));
            self.walk(child, &child_path, refs, depth.saturating_add(1))?;
        }
        Ok(())
    }

    fn walk_items(
        &mut self,
        node: &Value,
        path: &str,
        refs: &[String],
        depth: usize,
    ) -> Result<(), HarnessError> {
        let Some(items @ Value::Object(_)) = node.get("items") else {
            return Ok(());
        };
        self.walk(items, &format!("{path}/*"), refs, depth.saturating_add(1))
    }
}

fn branches<'a>(node: &'a Value, alternative: &str) -> Vec<&'a Value> {
    match node.get(alternative) {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(value @ Value::Object(_)) => vec![value],
        _ => Vec::new(),
    }
}
