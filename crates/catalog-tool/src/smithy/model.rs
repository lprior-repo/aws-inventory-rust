//! Only the Smithy fields needed by the statistics reader.

use std::collections::BTreeMap;

use serde::{Deserialize, de::IgnoredAny};

#[derive(Debug, Deserialize)]
pub(super) struct ModelFile {
    pub shapes: BTreeMap<String, Shape>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Shape {
    #[serde(default)]
    pub traits: BTreeMap<String, IgnoredAny>,
    #[serde(flatten)]
    pub kind: ShapeKind,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(super) enum ShapeKind {
    Service,
    Operation {
        input: Option<Target>,
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
    #[serde(default)]
    pub traits: BTreeMap<String, IgnoredAny>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Target {
    pub target: String,
}
