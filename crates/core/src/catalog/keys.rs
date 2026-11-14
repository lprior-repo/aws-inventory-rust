//! Catalog-only keys; member identities use `record::FieldId`, never these numbers.
use crate::record::{FieldId, RecordError};

/// Version-one catalog map keys (a namespace separate from member IDs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CatalogKey {
    /// Root format version.
    Format,
    /// Pinned upstream Git commit.
    ModelsCommit,
    /// Service records, each stored as bytes.
    Services,
    /// Service directory name, or unqualified shape/operation name.
    Name,
    /// Protocol trait IDs.
    Protocols,
    /// Operation maps.
    Operations,
    /// Shape maps in full-ID byte order.
    Shapes,
    /// Resource names.
    Resources,
    /// Shape kind code.
    Kind,
    /// Structure/union member triples: ID, target, required.
    Members,
    /// List element target.
    Element,
    /// Map key target.
    MapKey,
    /// Map value target.
    MapValue,
    /// ARN template, or null.
    Arn,
    /// Operation flags: readonly=1, paginated=2, List/Describe=4, input-free=8, read-named=16.
    Flags,
    /// Operation input target.
    Input,
    /// Operation output target.
    Output,
    /// Full Smithy service shape ID (protocol targets and error namespace).
    ServiceShape,
}

impl CatalogKey {
    /// Convert a catalog key to the record writer's sole field identity.
    /// # Errors
    /// Only fails if the fixed nonzero vocabulary is changed incorrectly.
    pub fn field(self) -> Result<FieldId, RecordError> {
        FieldId::new(self.number())
    }

    fn number(self) -> u32 {
        match self {
            Self::Format => 1,
            Self::ModelsCommit => 2,
            Self::Services => 3,
            Self::Name => 4,
            Self::Protocols => 5,
            Self::Operations => 6,
            Self::Shapes => 7,
            Self::Resources => 8,
            Self::Kind => 9,
            Self::Members => 10,
            Self::Element => 11,
            Self::MapKey => 12,
            Self::MapValue => 13,
            Self::Arn => 14,
            Self::Flags => 15,
            Self::Input => 16,
            Self::Output => 17,
            Self::ServiceShape => 18,
        }
    }
}

/// Shape kind codes. Position is the wire code, shared by compiler and reader.
/// Scalars occupy codes 0..=13; containers and Smithy metadata follow them.
pub const SHAPE_KINDS: [&str; 23] = [
    "blob",
    "boolean",
    "string",
    "byte",
    "short",
    "integer",
    "long",
    "float",
    "double",
    "bigInteger",
    "bigDecimal",
    "timestamp",
    "document",
    "unit",
    "structure",
    "union",
    "list",
    "map",
    "enum",
    "intEnum",
    "service",
    "operation",
    "resource",
];

/// Resolve a Smithy kind to its stable wire code.
pub fn kind_code(name: &str) -> Option<usize> {
    SHAPE_KINDS.iter().position(|kind| *kind == name)
}
