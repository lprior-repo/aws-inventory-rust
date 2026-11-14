//! Shared tolerant Smithy inputs for reporting, registry extraction and compilation.
mod traits;

use std::collections::BTreeMap;

use serde::{Deserialize, de::IgnoredAny};

#[derive(Debug, Deserialize)]
pub(super) struct ModelFile {
    pub shapes: BTreeMap<String, Shape>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Shape {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "traits::read")]
    pub traits: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub errors: Vec<Target>,
    #[serde(flatten)]
    pub kind: ShapeKind,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(super) enum ShapeKind {
    Service,
    Operation {
        input: Option<Target>,
        output: Option<Target>,
    },
    Structure {
        #[serde(default)]
        members: BTreeMap<String, Member>,
    },
    Union {
        #[serde(default)]
        members: BTreeMap<String, Member>,
    },
    Resource,
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
    Enum,
    #[serde(rename = "intEnum")]
    IntEnum,
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

#[derive(Debug, Deserialize)]
pub(super) struct Member {
    pub target: Option<String>,
    #[serde(default, deserialize_with = "traits::read")]
    pub traits: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Target {
    pub target: String,
    #[serde(default, deserialize_with = "traits::read")]
    pub traits: BTreeMap<String, serde_json::Value>,
}
pub(super) fn validate_shape_traits(shape: &Shape) -> Result<(), String> {
    traits::validate_shape(shape)
}
