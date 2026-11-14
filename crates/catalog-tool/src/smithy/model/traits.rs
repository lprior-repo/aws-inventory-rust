//! Retain only ARN payloads; protocol/classification traits need presence, not JSON trees.
use serde::{
    Deserializer,
    de::{IgnoredAny, MapAccess, Visitor},
};
use serde_json::Value;
use std::{collections::BTreeMap, fmt};

pub(super) fn read<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Value>, D::Error> {
    deserializer.deserialize_map(Traits)
}
struct Traits;
impl<'de> Visitor<'de> for Traits {
    type Value = BTreeMap<String, Value>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a Smithy trait map")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        // Each next_entry consumes a key/value pair from the bounded model input.
        std::iter::from_fn(|| next_entry(&mut access).transpose()).collect()
    }
}
fn next_entry<'de, A: MapAccess<'de>>(access: &mut A) -> Result<Option<(String, Value)>, A::Error> {
    let Some(name) = access.next_key::<String>()? else {
        return Ok(None);
    };
    let value = if name == "aws.api#arn" {
        access.next_value::<Value>()?
    } else {
        access.next_value::<IgnoredAny>()?;
        Value::Null
    };
    Ok(Some((name, value)))
}
