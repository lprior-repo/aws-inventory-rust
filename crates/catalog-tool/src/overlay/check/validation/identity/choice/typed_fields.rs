use super::{Model, ShapeRef, shape_type, text};

pub(super) fn integer(shape: Option<ShapeRef<'_>>, minimum: i64, maximum: i64) -> bool {
    let Some(shape) = shape else {
        return false;
    };
    let (low, high) = match shape_type(shape.value) {
        Some("byte") => (i64::from(i8::MIN), i64::from(i8::MAX)),
        Some("short") => (i64::from(i16::MIN), i64::from(i16::MAX)),
        Some("integer") => (i64::from(i32::MIN), i64::from(i32::MAX)),
        Some("long") => (i64::MIN, i64::MAX),
        _ => return false,
    };
    minimum <= maximum && minimum >= low && maximum <= high
}

pub(super) fn string_list(
    model: &Model<'_>,
    shape: Option<ShapeRef<'_>>,
    max_items: u32,
    max_item_bytes: u32,
) -> bool {
    if !(1..=64).contains(&max_items) || !(1..=4096).contains(&max_item_bytes) {
        return false;
    }
    shape.is_some_and(|shape| {
        shape_type(shape.value) == Some("list")
            && text(
                shape
                    .value
                    .get("member")
                    .and_then(crate::plans::model::target)
                    .and_then(|id| model.shape(id)),
            )
    })
}

pub(super) fn digest(shape: Option<ShapeRef<'_>>) -> bool {
    shape.is_some_and(|shape| {
        matches!(
            shape_type(shape.value),
            Some(
                "string"
                    | "enum"
                    | "intEnum"
                    | "boolean"
                    | "byte"
                    | "short"
                    | "integer"
                    | "long"
                    | "float"
                    | "double"
                    | "bigInteger"
                    | "bigDecimal"
                    | "blob"
                    | "timestamp"
                    | "structure"
                    | "union"
                    | "list"
                    | "map"
            )
        )
    })
}
