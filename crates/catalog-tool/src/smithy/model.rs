pub(super) mod mixins;
mod traits;

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct ModelFile {
    #[serde(deserialize_with = "crate::model_source::unique_map")]
    pub shapes: BTreeMap<String, Shape>,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct Shape {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "traits::read")]
    pub traits: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub errors: Vec<Target>,
    #[serde(default)]
    pub mixins: Vec<Target>,
    #[serde(flatten)]
    pub kind: ShapeKind,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(super) enum ShapeKind {
    Service,
    Operation {
        input: Option<Target>,
        output: Option<Target>,
    },
    Structure {
        #[serde(default, deserialize_with = "crate::model_source::unique_map")]
        members: BTreeMap<String, Member>,
    },
    Union {
        #[serde(default, deserialize_with = "crate::model_source::unique_map")]
        members: BTreeMap<String, Member>,
    },
    Resource {
        list: Option<Target>,
        read: Option<Target>,
        #[serde(default)]
        identifiers: BTreeMap<String, Target>,
    },
    List {
        member: Target,
    },
    Map {
        key: Target,
        value: Target,
    },
    Blob,
    Boolean,
    String,
    Byte,
    Short,
    Integer,
    Long,
    Float,
    Double,
    #[serde(rename = "bigInteger")]
    BigInteger,
    #[serde(rename = "bigDecimal")]
    BigDecimal,
    Timestamp,
    Document,
    Enum {
        #[serde(default, deserialize_with = "crate::model_source::unique_map")]
        members: BTreeMap<String, Member>,
    },
    #[serde(rename = "intEnum")]
    IntEnum {
        #[serde(default, deserialize_with = "crate::model_source::unique_map")]
        members: BTreeMap<String, Member>,
    },
    #[serde(other)]
    Other,
}

impl Shape {
    pub fn into_members(self) -> BTreeMap<String, Member> {
        match self.kind {
            ShapeKind::Structure { members } | ShapeKind::Union { members } => members,
            _ => BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct Member {
    pub target: Option<String>,
    #[serde(default, deserialize_with = "traits::read")]
    pub traits: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct Target {
    pub target: String,
    #[serde(default, deserialize_with = "traits::read")]
    pub traits: BTreeMap<String, serde_json::Value>,
}
pub(super) fn validate_shape_traits(shape: &Shape) -> Result<(), String> {
    traits::validate_shape(shape)
}

impl crate::model_source::InputModel for ModelFile {
    fn structure(&self, target: &str) -> Option<bool> {
        self.shapes
            .get(target)
            .map(|shape| matches!(&shape.kind, ShapeKind::Structure { .. }))
    }

    fn mixins<'a>(&'a self, target: &str, visit: &mut dyn FnMut(&'a str)) {
        if let Some(shape) = self.shapes.get(target) {
            shape.mixins.iter().for_each(|mixin| visit(&mixin.target));
        }
    }

    fn members<'a>(&'a self, target: &str, visit: &mut dyn FnMut(&'a str, bool)) {
        if let Some(Shape {
            kind: ShapeKind::Structure { members },
            ..
        }) = self.shapes.get(target)
        {
            members.iter().for_each(|(name, member)| {
                visit(name, member.traits.contains_key("smithy.api#required"));
            });
        }
    }
}
