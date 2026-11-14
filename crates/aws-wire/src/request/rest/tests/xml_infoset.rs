use inventory_core::catalog::{Operation, Service};
use quick_xml::{XmlVersion, events::Event, name::ResolveResult, reader::NsReader};
#[path = "../../../../tests/xml_infoset/compare.rs"]
mod compare;
const MAX_XML_BYTES: usize = 16 * 1024 * 1024;
const MAX_XML_EVENTS: usize = 65_536;
const MAX_XML_DEPTH: usize = 256;

#[derive(Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct Node {
    name: String,
    attributes: Vec<(String, String)>,
    text: String,
    children: Vec<Node>,
}

pub(super) fn equal(
    left: &[u8],
    right: &[u8],
    service: Service<'_>,
    operation: Operation<'_>,
) -> bool {
    let (Ok(left), Ok(right)) = (std::str::from_utf8(left), std::str::from_utf8(right)) else {
        return false;
    };
    let (Ok(left), Ok(right)) = (parse(left), parse(right)) else {
        return false;
    };
    let Some((target, name)) = compare::root_target(service, operation) else {
        return false;
    };
    compare::equal_roots(&left, &right, service, target, name.as_deref())
}

fn parse(source: &str) -> Result<Node, Box<dyn std::error::Error>> {
    if source.len() > MAX_XML_BYTES {
        return Err("XML byte limit exceeded".into());
    }
    let mut event_count = 0usize;
    let mut reader = NsReader::from_str(source);
    let mut stack = vec![Node::default()];
    loop {
        event_count = event_count
            .checked_add(1)
            .ok_or("XML event limit exceeded")?;
        if event_count > MAX_XML_EVENTS {
            return Err("XML event limit exceeded".into());
        }
        let (namespace, event) = reader.read_resolved_event()?;
        match event {
            Event::Start(element) => {
                if stack.len() >= MAX_XML_DEPTH {
                    return Err("XML depth limit exceeded".into());
                }
                let name = expanded(namespace, element.name().as_ref())?;
                stack.push(Node {
                    name,
                    attributes: attributes(&reader, &element)?,
                    ..Node::default()
                });
            }
            Event::Empty(element) => {
                let name = expanded(namespace, element.name().as_ref())?;
                let child = Node {
                    name,
                    attributes: attributes(&reader, &element)?,
                    ..Node::default()
                };
                attach(&mut stack, child)?;
            }
            Event::End(_) => {
                let child = stack.pop().ok_or("unbalanced XML")?;
                attach(&mut stack, child)?;
            }
            Event::Text(value) => {
                append(&mut stack, &quick_xml::escape::unescape(value.as_ref())?)?;
            }
            Event::CData(value) => append(&mut stack, value.as_ref())?,
            Event::GeneralRef(reference) => {
                if let Some(character) = reference.resolve_char_ref()? {
                    append(&mut stack, character.encode_utf8(&mut [0; 4]))?;
                } else {
                    append(
                        &mut stack,
                        quick_xml::escape::resolve_xml_entity(reference.as_ref())
                            .ok_or("unknown XML entity")?,
                    )?;
                }
            }
            Event::Eof => break,
            Event::Decl(_) | Event::PI(_) | Event::Comment(_) | Event::DocType(_) => {}
        }
    }
    let root = stack.pop().ok_or("unbalanced XML")?;
    if stack.is_empty() {
        Ok(root)
    } else {
        Err("unclosed XML element".into())
    }
}

fn attach(stack: &mut [Node], child: Node) -> Result<(), Box<dyn std::error::Error>> {
    stack
        .last_mut()
        .ok_or("unbalanced XML")?
        .children
        .push(child);
    Ok(())
}

fn append(stack: &mut [Node], text: &str) -> Result<(), Box<dyn std::error::Error>> {
    stack
        .last_mut()
        .ok_or("unbalanced XML")?
        .text
        .push_str(text);
    Ok(())
}

fn attributes(
    reader: &NsReader<&[u8]>,
    element: &quick_xml::events::BytesStart<'_>,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut attributes = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        if key == "xmlns" || key.starts_with("xmlns:") {
            continue;
        }
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        attributes.push((
            expanded(namespace, local.as_ref())?,
            attribute
                .normalized_value(XmlVersion::Implicit1_0)?
                .into_owned(),
        ));
    }
    attributes.sort();
    Ok(attributes)
}

fn expanded(namespace: ResolveResult<'_>, local: &str) -> Result<String, &'static str> {
    match namespace {
        ResolveResult::Bound(uri) => Ok(format!("{{{}}}{local}", uri.as_ref())),
        ResolveResult::Unbound => Ok(local.to_owned()),
        _ => Err("unresolved XML namespace"),
    }
}
#[cfg(test)]
#[path = "../../../../tests/xml_infoset/tests.rs"]
mod behavior_tests;
