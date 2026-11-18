use super::{Entries, Items, Key, Model, Task, Value, schema};
use crate::response::{
    DecodeError, defaults, headers,
    xml::{http, index::Token, scalar},
};
use inventory_core::{
    catalog::{HttpBinding, Member, Shape},
    record::KeyRef,
};

#[derive(Clone, Copy)]
enum Layout {
    Wrapped,
    Unwrapped,
}

impl<'a, 'd> Model<'a, 'd, '_> {
    pub(super) fn structure(
        &mut self,
        node: usize,
        shape: Shape<'a>,
        key: Key<'d>,
    ) -> Result<(), DecodeError> {
        let layout = self.layout(&key)?;
        self.builder.begin_map(key.reference())?;
        self.push(Task::End)?;
        let count = shape.members()?.try_fold(0_usize, |count, item| {
            let present = self.member(node, item?, layout)?;
            count
                .checked_add(usize::from(present))
                .ok_or(DecodeError::Limit("XML members"))
        })?;
        if shape.kind()? == "union" && count != 1 {
            return Err(DecodeError::Invalid("XML union variant count"));
        }
        Ok(())
    }

    fn layout(&self, key: &Key<'_>) -> Result<Layout, DecodeError> {
        if matches!(key, Key::Root) && self.context.operation.s3_unwrapped_xml_output()? {
            Ok(Layout::Unwrapped)
        } else {
            Ok(Layout::Wrapped)
        }
    }

    fn member(
        &mut self,
        node: usize,
        member: Member<'a>,
        layout: Layout,
    ) -> Result<bool, DecodeError> {
        match member.wire()?.binding()? {
            HttpBinding::Header(_) | HttpBinding::PrefixHeaders(_) | HttpBinding::ResponseCode => {
                headers::decode_member(self.context, member, self.builder)?;
                Ok(false)
            }
            HttpBinding::Payload => self.payload(node, member),
            HttpBinding::Document
            | HttpBinding::Label
            | HttpBinding::Query(_)
            | HttpBinding::QueryParams => self.body_member(node, member, layout),
        }
    }

    fn payload(&mut self, node: usize, member: Member<'a>) -> Result<bool, DecodeError> {
        if !self.document.has_root {
            return self.absent(member);
        }
        self.push(Task::Value(Value {
            node,
            target: member.target,
            key: Key::Field(member.id),
            timestamp: member.wire()?.timestamp_format()?,
        }))?;
        Ok(true)
    }

    fn absent(&mut self, member: Member<'a>) -> Result<bool, DecodeError> {
        defaults::apply_member(
            self.builder,
            member,
            self.context.service,
            self.context.limits,
        )?;
        Ok(false)
    }

    fn body_member(
        &mut self,
        node: usize,
        member: Member<'a>,
        layout: Layout,
    ) -> Result<bool, DecodeError> {
        if member.wire()?.xml_attribute()? {
            return self.attribute(node, member);
        }
        if matches!(layout, Layout::Unwrapped) {
            return self.payload(node, member);
        }
        let nodes = schema::matching_children(self.document, node, schema::member_name(member)?)?;
        if nodes.is_empty() {
            return self.absent(member);
        }
        let task = self.bound_member(member, nodes)?;
        self.push(task)?;
        Ok(true)
    }

    fn bound_member(
        &self,
        member: Member<'a>,
        nodes: Vec<usize>,
    ) -> Result<Task<'a, 'd>, DecodeError> {
        let shape = schema::target_shape(self.context, member.target)?;
        if member.wire()?.xml_flattened()? || schema::flattened_collection(shape)? {
            let shape = shape.ok_or(DecodeError::Invalid("XML collection"))?;
            return self.flattened_member(member, shape, nodes);
        }
        schema::validate_nodes(self.document, &nodes, member.wire()?.xml_namespace()?)?;
        if nodes.len() != 1 {
            return Err(DecodeError::Invalid("duplicate XML member"));
        }
        Ok(Task::Value(Value {
            node: schema::only_node(&nodes)?,
            target: member.target,
            key: Key::Field(member.id),
            timestamp: member.wire()?.timestamp_format()?,
        }))
    }

    fn flattened_member(
        &self,
        member: Member<'a>,
        shape: Shape<'a>,
        nodes: Vec<usize>,
    ) -> Result<Task<'a, 'd>, DecodeError> {
        match shape.kind()? {
            "list" => self.flattened_list(member, shape, nodes),
            "map" => {
                schema::validate_nodes(self.document, &nodes, member.wire()?.xml_namespace()?)?;
                Ok(Task::MapEntries(Entries {
                    nodes,
                    shape,
                    key: Key::Field(member.id),
                    timestamp: member.wire()?.timestamp_format()?,
                }))
            }
            _ => Err(DecodeError::Invalid(
                "flattened XML member is not a collection",
            )),
        }
    }

    fn flattened_list(
        &self,
        member: Member<'a>,
        shape: Shape<'a>,
        nodes: Vec<usize>,
    ) -> Result<Task<'a, 'd>, DecodeError> {
        let wire = shape.element_wire()?;
        let namespace = wire.xml_namespace()?.or(member.wire()?.xml_namespace()?);
        schema::validate_nodes(self.document, &nodes, namespace)?;
        Ok(Task::ListItems(Items {
            nodes,
            target: shape
                .element()?
                .ok_or(DecodeError::Invalid("XML list element"))?,
            key: Key::Field(member.id),
            timestamp: member
                .wire()?
                .timestamp_format()?
                .or(wire.timestamp_format()?),
        }))
    }

    fn attribute(&mut self, node: usize, member: Member<'a>) -> Result<bool, DecodeError> {
        if !self.document.has_root {
            return self.absent(member);
        }
        let start = match self.document.tokens.get(node) {
            Some(Token::Start(start) | Token::Empty(start)) => start,
            _ => return Err(DecodeError::Invalid("XML structure")),
        };
        for attribute in start.attributes().with_checks(true) {
            let attribute = attribute.map_err(|_| DecodeError::Invalid("XML attribute"))?;
            if attribute.key.local_name().as_ref() == schema::member_name(member)? {
                self.attribute_value(node, member, attribute)?;
                return Ok(true);
            }
        }
        self.absent(member)
    }

    fn attribute_value(
        &mut self,
        node: usize,
        member: Member<'a>,
        attribute: quick_xml::events::attributes::Attribute<'_>,
    ) -> Result<(), DecodeError> {
        let namespace = member.wire()?.xml_namespace()?.map(|value| value.uri);
        self.document.validate_attribute_namespace(
            node,
            attribute.key.as_ref().as_bytes(),
            namespace,
        )?;
        let value = attribute
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .map_err(|_| DecodeError::Invalid("XML attribute value"))?;
        let (kind, shape_format) = http::scalar_target(self.context, member.target)?;
        let format = member.wire()?.timestamp_format()?.or(shape_format);
        scalar::parse(kind, value, format)?.write(self.builder, Some(KeyRef::Field(member.id)))
    }
}
