mod required;
use super::Error;
use crate::classes::{
    OperationFacts,
    model::{Model as ClassModel, Shape},
};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{collections::BTreeSet, path::Path, sync::LazyLock};
static PRELUDE_SHAPES: LazyLock<Map<String, Value>> = LazyLock::new(|| {
    [
        ("smithy.api#Blob", "blob"),
        ("smithy.api#Boolean", "boolean"),
        ("smithy.api#Byte", "byte"),
        ("smithy.api#Short", "short"),
        ("smithy.api#Integer", "integer"),
        ("smithy.api#Long", "long"),
        ("smithy.api#Float", "float"),
        ("smithy.api#Double", "double"),
        ("smithy.api#PrimitiveBoolean", "boolean"),
        ("smithy.api#PrimitiveByte", "byte"),
        ("smithy.api#PrimitiveShort", "short"),
        ("smithy.api#PrimitiveInteger", "integer"),
        ("smithy.api#PrimitiveLong", "long"),
        ("smithy.api#PrimitiveFloat", "float"),
        ("smithy.api#PrimitiveDouble", "double"),
        ("smithy.api#BigInteger", "bigInteger"),
        ("smithy.api#BigDecimal", "bigDecimal"),
        ("smithy.api#String", "string"),
        ("smithy.api#Timestamp", "timestamp"),
        ("smithy.api#Document", "document"),
        ("smithy.api#Unit", "unit"),
    ]
    .into_iter()
    .map(|(id, kind)| (id.to_owned(), serde_json::json!({"type": kind})))
    .collect()
});

#[derive(Debug)]
#[non_exhaustive]
pub struct Model<'a> {
    pub(super) shapes: &'a Map<String, Value>,
    pub(super) service: &'a str,
    classifier: ClassModel,
    pub(super) paginator: Option<&'a Value>,
}

impl<'a> Model<'a> {
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

    pub(crate) fn of_type<'b>(
        &'b self,
        kind: &'b str,
    ) -> impl Iterator<Item = (&'a str, &'a Value)> + 'b {
        let shapes: &'a Map<String, Value> = self.shapes;
        shapes
            .iter()
            .filter(move |(_, shape)| shape_type(shape) == Some(kind))
            .map(|(id, shape)| (id.as_str(), shape))
    }

    pub(crate) fn operation_names(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.of_type("operation").map(|(id, _)| name(id))
    }

    pub(crate) fn operation_id(&self, operation: &str) -> Option<&'a str> {
        self.of_type("operation")
            .map(|(id, _)| id)
            .find(|id| name(id) == operation)
    }

    pub(crate) fn side_root(&self, operation: &str, side: &str) -> Option<ShapeRef<'a>> {
        let id = self.operation_id(operation)?;
        let target = self.operation(id).ok()?.get(side).and_then(target)?;
        self.shapes.get(target).map(|shape| ShapeRef {
            id: target,
            value: shape,
        })
    }

    pub(crate) fn resolve_path(
        &self,
        operation: &str,
        side: &str,
        path: &str,
    ) -> Result<Option<ShapeRef<'a>>, &'static str> {
        let Some(root) = self.side_root(operation, side) else {
            return Ok(None);
        };
        self.resolve_path_from(root, path).map(Some)
    }

    pub(crate) fn resolve_path_from(
        &self,
        mut current: ShapeRef<'a>,
        path: &str,
    ) -> Result<ShapeRef<'a>, &'static str> {
        for segment in path.split('.') {
            current = self.unwrap_aggregates(current)?;
            if !matches!(shape_type(current.value), Some("structure" | "union")) {
                return Err("path traverses neither a structure nor a union");
            }
            let member = current
                .value
                .get("members")
                .and_then(Value::as_object)
                .and_then(|members| members.get(segment))
                .ok_or("member does not exist")?;
            let target = target(member).ok_or("member target is absent")?;
            current = self.shape(target).ok_or("member target does not exist")?;
        }
        Ok(current)
    }
    pub(crate) fn resolve_member(
        &self,
        operation: &str,
        side: &str,
        path: &str,
    ) -> Result<Option<(&'a Value, ShapeRef<'a>)>, &'static str> {
        self.side_root(operation, side)
            .map(|root| resolve_member_path(self, root, path))
            .transpose()
    }

    pub(crate) fn unwrap_aggregates(
        &self,
        mut shape: ShapeRef<'a>,
    ) -> Result<ShapeRef<'a>, &'static str> {
        let mut visited = BTreeSet::new();
        for _ in 0..32 {
            if !visited.insert(shape.id) {
                return Err("aggregate cycle without path progress");
            }
            let Some(target) = aggregate_target(shape.value)? else {
                return Ok(shape);
            };
            shape = self
                .shape(target)
                .ok_or("aggregate target does not exist")?;
        }
        Err("aggregate traversal limit exceeded")
    }

    pub(crate) fn shape(&self, id: &str) -> Option<ShapeRef<'a>> {
        self.shapes
            .get_key_value(id)
            .map(|(id, value)| ShapeRef { id, value })
            .or_else(|| {
                PRELUDE_SHAPES
                    .get_key_value(id)
                    .map(|(id, value)| ShapeRef { id, value })
            })
    }

    pub(super) fn operation(&self, id: &str) -> Result<&'a Value, Error> {
        self.shapes
            .get(id)
            .filter(|s| s.get("type").and_then(Value::as_str) == Some("operation"))
            .ok_or_else(|| self.invalid(id, "operation"))
    }

    pub(crate) fn members(
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

    pub(crate) fn facts(&self, id: &str) -> Result<OperationFacts<'_>, Error> {
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
fn resolve_member_path<'a>(
    model: &Model<'a>,
    mut current: ShapeRef<'a>,
    path: &str,
) -> Result<(&'a Value, ShapeRef<'a>), &'static str> {
    let mut segments = path.split('.').peekable();
    while let Some(segment) = segments.next() {
        current = model.unwrap_aggregates(current)?;
        let (member, shape) = resolve_member_segment(model, current, segment)?;
        if segments.peek().is_none() {
            return Ok((member, shape));
        }
        current = shape;
    }
    Err("empty path")
}

fn resolve_member_segment<'a>(
    model: &Model<'a>,
    current: ShapeRef<'a>,
    segment: &str,
) -> Result<(&'a Value, ShapeRef<'a>), &'static str> {
    if !matches!(shape_type(current.value), Some("structure" | "union")) {
        return Err("path traverses neither a structure nor a union");
    }
    let member = current
        .value
        .get("members")
        .and_then(Value::as_object)
        .and_then(|members| members.get(segment))
        .ok_or("member does not exist")?;
    let target = target(member).ok_or("member target is absent")?;
    let shape = model.shape(target).ok_or("member target does not exist")?;
    Ok((member, shape))
}

fn aggregate_target(value: &Value) -> Result<Option<&str>, &'static str> {
    match shape_type(value) {
        Some("list") => value
            .get("member")
            .and_then(target)
            .map(Some)
            .ok_or("aggregate target is absent"),
        Some("map") => value
            .get("value")
            .and_then(target)
            .map(Some)
            .ok_or("aggregate target is absent"),
        _ => Ok(None),
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ShapeRef<'a> {
    pub(crate) id: &'a str,
    pub(crate) value: &'a Value,
}

pub(crate) fn shape_type(value: &Value) -> Option<&str> {
    value.get("type").and_then(Value::as_str)
}

pub(crate) fn name(id: &str) -> &str {
    id.rsplit_once('#').map_or(id, |(_, name)| name)
}
pub(crate) fn target(value: &Value) -> Option<&str> {
    value.get("target").and_then(Value::as_str)
}
pub(super) fn trait_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.get("traits").and_then(|t| t.get(key))
}
pub(super) fn required(value: &Value) -> bool {
    trait_value(value, "smithy.api#required").is_some()
}
