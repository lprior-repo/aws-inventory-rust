use super::{Entries, Items, Key, Model, Task, Value, schema};
use crate::response::{
    DecodeError,
    xml::{index::Document, scalar},
};
use inventory_core::catalog::{Shape, Target, TimestampFormat, WireTraits};
use std::borrow::Cow;

struct MapSpec<'a> {
    key: WireTraits<'a>,
    value: WireTraits<'a>,
    target: Target,
    timestamp: Option<TimestampFormat>,
}

impl<'a, 'd> Model<'a, 'd, '_> {
    pub(super) fn list(&mut self, shape: Shape<'a>, value: Value<'d>) -> Result<(), DecodeError> {
        let wire = shape.element_wire()?;
        let name = wire.xml_name()?.map_or("member", |name| name);
        let nodes = schema::matching_children(self.document, value.node, name)?;
        schema::validate_nodes(self.document, &nodes, wire.xml_namespace()?)?;
        self.list_nodes(Items {
            nodes,
            target: shape
                .element()?
                .ok_or(DecodeError::Invalid("XML list element"))?,
            key: value.key,
            timestamp: value.timestamp.or(wire.timestamp_format()?),
        })
    }

    pub(super) fn list_nodes(&mut self, items: Items<'d>) -> Result<(), DecodeError> {
        self.builder.begin_list(items.key.reference())?;
        self.push(Task::End)?;
        for node in items.nodes.into_iter().rev() {
            self.push(Task::Value(Value {
                node,
                target: items.target,
                key: Key::List,
                timestamp: items.timestamp,
            }))?;
        }
        Ok(())
    }

    pub(super) fn map(&mut self, shape: Shape<'a>, value: Value<'d>) -> Result<(), DecodeError> {
        let name = shape.wire()?.xml_name()?.map_or("entry", |name| name);
        let nodes = schema::matching_children(self.document, value.node, name)?;
        self.map_nodes(Entries {
            nodes,
            shape,
            key: value.key,
            timestamp: value.timestamp,
        })
    }

    pub(super) fn map_nodes(&mut self, entries: Entries<'a, 'd>) -> Result<(), DecodeError> {
        let key_target = entries
            .shape
            .map_key()?
            .ok_or(DecodeError::Invalid("XML map key"))?;
        self.validate_map_key(key_target)?;
        let spec = MapSpec::new(entries.shape, entries.timestamp)?;
        self.builder.begin_map(entries.key.reference())?;
        self.push(Task::End)?;
        for entry in entries.nodes {
            self.map_entry(entry, &spec)?;
        }
        Ok(())
    }

    fn map_entry(&mut self, entry: usize, spec: &MapSpec<'a>) -> Result<(), DecodeError> {
        let (name, node) = spec.entry(self.document, entry)?;
        self.push(Task::Value(Value {
            node,
            target: spec.target,
            key: Key::Name(name),
            timestamp: spec.timestamp,
        }))
    }

    fn validate_map_key(&self, target: Target) -> Result<(), DecodeError> {
        match target {
            Target::Prelude("string") => Ok(()),
            Target::Shape(index) => {
                let shape = self
                    .context
                    .service
                    .shape(index)?
                    .ok_or(DecodeError::Invalid("XML map key"))?;
                if shape.kind()? == "string" {
                    Ok(())
                } else {
                    Err(DecodeError::Invalid("XML map key must be a string"))
                }
            }
            _ => Err(DecodeError::Invalid("XML map key must be a string")),
        }
    }
}

impl<'a> MapSpec<'a> {
    fn new(shape: Shape<'a>, timestamp: Option<TimestampFormat>) -> Result<Self, DecodeError> {
        let value = shape.map_value_wire()?;
        Ok(Self {
            key: shape.map_key_wire()?,
            value,
            target: shape
                .map_value()?
                .ok_or(DecodeError::Invalid("XML map value"))?,
            timestamp: timestamp.or(value.timestamp_format()?),
        })
    }

    fn entry<'d>(
        &self,
        document: &'d Document<'_>,
        entry: usize,
    ) -> Result<(Cow<'d, str>, usize), DecodeError> {
        let key = self.child(document, entry, self.key, "key")?;
        let value = self.child(document, entry, self.value, "value")?;
        Ok((scalar::text(document, key)?, value))
    }

    fn child(
        &self,
        document: &Document<'_>,
        entry: usize,
        wire: WireTraits<'_>,
        fallback: &str,
    ) -> Result<usize, DecodeError> {
        let nodes = schema::matching_children(
            document,
            entry,
            wire.xml_name()?.map_or(fallback, |name| name),
        )?;
        if nodes.len() != 1 {
            return Err(DecodeError::Invalid("XML map entry"));
        }
        let node = schema::only_node(&nodes)?;
        document.validate_namespace(node, wire.xml_namespace()?)?;
        Ok(node)
    }
}
