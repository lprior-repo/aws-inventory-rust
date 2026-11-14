use super::Error;
use crate::classes::{
    OperationFacts,
    model::{Model as ClassModel, Shape},
};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::path::Path;

/// Bounded view of one model, with the existing classifier's validated representation.
#[derive(Debug)]
#[non_exhaustive]
pub struct Model<'a> {
    pub(super) shapes: &'a Map<String, Value>,
    pub(super) service: &'a str,
    classifier: ClassModel,
    pub(super) paginator: Option<&'a Value>,
}

impl<'a> Model<'a> {
    /// Borrow a model with at most 131,072 shapes and exactly one service.
    /// # Errors
    /// Rejects malformed classifier fields, shapes, service count or resource limits.
    pub fn parse(value: &'a Value, service: &'a str) -> Result<Self, Error> {
        let classifier = ClassModel::deserialize(value)?;
        classifier.validate(Path::new(service))?;
        let shapes = value.get("shapes").and_then(Value::as_object);
        let shapes = shapes.ok_or(Error::Limit)?;
        if shapes.len() > 131_072 {
            return Err(Error::Limit);
        }
        let paginator = shapes
            .values()
            .find(|s| s.get("type").and_then(Value::as_str) == Some("service"))
            .and_then(|s| trait_value(s, "smithy.api#paginated"));
        Ok(Self {
            shapes,
            service,
            classifier,
            paginator,
        })
    }

    pub(super) fn of_type(&self, kind: &'a str) -> impl Iterator<Item = (&'a str, &'a Value)> {
        self.shapes
            .iter()
            .filter(move |(_, s)| s.get("type").and_then(Value::as_str) == Some(kind))
            .map(|(id, shape)| (id.as_str(), shape))
    }

    pub(super) fn operation(&self, id: &str) -> Result<&'a Value, Error> {
        self.shapes
            .get(id)
            .filter(|s| s.get("type").and_then(Value::as_str) == Some("operation"))
            .ok_or_else(|| self.invalid(id, "operation"))
    }

    pub(super) fn members(
        &self,
        id: &str,
        side: &str,
    ) -> Result<Option<&'a Map<String, Value>>, Error> {
        let op = self.operation(id)?;
        let Some(target) = op.get(side).and_then(target) else {
            return Ok(None);
        };
        if target == "smithy.api#Unit" {
            return Ok(None);
        }
        let shape = self.structure(id, target)?;
        let members = shape.get("members").and_then(Value::as_object);
        if members.is_some_and(|m| m.len() > 4096) {
            return Err(Error::Limit);
        }
        Ok(members)
    }

    fn structure(&self, id: &str, target: &str) -> Result<&'a Value, Error> {
        self.shapes
            .get(target)
            .filter(|s| s.get("type").and_then(Value::as_str) == Some("structure"))
            .ok_or_else(|| self.invalid(id, "input/output structure"))
    }

    pub(super) fn facts(&self, id: &str) -> Result<OperationFacts<'_>, Error> {
        let (id, shape) = self
            .classifier
            .shapes
            .get_key_value(id)
            .ok_or_else(|| self.invalid(id, "operation"))?;
        let Shape::Operation { input, traits } = shape else {
            return Err(self.invalid(id, "operation"));
        };
        Ok(self.classifier.facts(self.service, id, input, traits)?)
    }

    pub(super) fn invalid(&self, id: &str, detail: &'static str) -> Error {
        Error::InvalidModel {
            service: self.service.to_owned(),
            operation: name(id).to_owned(),
            detail,
        }
    }
}

pub(super) fn name(id: &str) -> &str {
    id.rsplit_once('#').map_or(id, |(_, name)| name)
}
pub(super) fn target(value: &Value) -> Option<&str> {
    value.get("target").and_then(Value::as_str)
}
pub(super) fn trait_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.get("traits").and_then(|t| t.get(key))
}
pub(super) fn required(value: &Value) -> bool {
    trait_value(value, "smithy.api#required").is_some()
}
