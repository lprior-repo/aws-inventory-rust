use crate::{
    error::{Error, Result, syntax},
    scalar::Kind,
    sink::Sink,
};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
pub fn parse(input: &[u8], sink: &mut Sink) -> Result<()> {
    let mut parser = serde_json::Deserializer::from_slice(input);
    let mut failure = None;
    let result = Seed {
        sink,
        depth: 0,
        failure: &mut failure,
    }
    .deserialize(&mut parser);
    if let Some(error) = failure {
        return Err(error);
    }
    result.map_err(syntax)?;
    parser.end().map_err(syntax)
}
struct Seed<'a> {
    sink: &'a mut Sink,
    depth: usize,
    failure: &'a mut Option<Error>,
}
impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        if self.depth >= 96 {
            return Err(de::Error::custom("depth limit"));
        }
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Seed<'_> {
    type Value = ();
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JSON response")
    }
    fn visit_map<A: MapAccess<'de>>(mut self, mut map: A) -> std::result::Result<(), A::Error> {
        self.sink
            .begin(true)
            .map_err(|e| capture(self.failure, e))?;
        std::iter::from_fn(|| match map.next_key::<String>() {
            Ok(Some(key)) => Some(self.member(&mut map, &key)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        })
        .try_for_each(|r| r)?;
        self.sink.end(true).map_err(|e| capture(self.failure, e))
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> std::result::Result<(), A::Error> {
        self.sink
            .begin(false)
            .map_err(|e| capture(self.failure, e))?;
        std::iter::from_fn(|| {
            match seq.next_element_seed(Seed {
                sink: self.sink,
                depth: self.depth.saturating_add(1),
                failure: self.failure,
            }) {
                Ok(Some(())) => Some(Ok(())),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        })
        .try_for_each(|r| r)?;
        self.sink.end(false).map_err(|e| capture(self.failure, e))
    }
    fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<(), E> {
        self.sink
            .scalar(v, Kind::Text)
            .map_err(|e| capture(self.failure, e))
    }
    fn visit_string<E: de::Error>(self, v: String) -> std::result::Result<(), E> {
        self.visit_str(&v)
    }
    fn visit_bool<E: de::Error>(self, v: bool) -> std::result::Result<(), E> {
        self.sink
            .scalar(if v { "true" } else { "false" }, Kind::Bool)
            .map_err(|e| capture(self.failure, e))
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<(), E> {
        self.sink
            .scalar("", Kind::Null)
            .map_err(|e| capture(self.failure, e))
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<(), E> {
        self.numeric(v)
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<(), E> {
        self.numeric(v)
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<(), E> {
        self.numeric(v)
    }
}
impl Seed<'_> {
    fn member<'de, A: MapAccess<'de>>(
        &mut self,
        map: &mut A,
        key: &str,
    ) -> std::result::Result<(), A::Error> {
        self.sink.key(key).map_err(|e| capture(self.failure, e))?;
        map.next_value_seed(Seed {
            sink: self.sink,
            depth: self.depth.saturating_add(1),
            failure: self.failure,
        })
    }
    fn numeric<E: de::Error>(self, value: impl std::fmt::Display) -> std::result::Result<(), E> {
        self.sink
            .scalar(&value.to_string(), Kind::Number)
            .map_err(|e| capture(self.failure, e))
    }
}
fn capture<E: de::Error>(failure: &mut Option<Error>, error: Error) -> E {
    let translated = E::custom(&error);
    *failure = Some(error);
    translated
}
