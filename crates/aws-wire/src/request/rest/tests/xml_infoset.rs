use quick_xml::{events::Event, name::{QName, ResolveResult}, reader::NsReader};

#[derive(Debug, Eq, PartialEq)]
enum Item { Start(String, Vec<(String, String)>), End(String), Text(String) }

pub(super) fn equal(left: &[u8], right: &[u8]) -> bool {
    let (Ok(left), Ok(right)) = (std::str::from_utf8(left), std::str::from_utf8(right)) else { return false };
    match (parse(left), parse(right)) { (Ok(left), Ok(right)) => left == right, _ => false }
}

fn parse(source: &str) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let mut reader = NsReader::from_str(source);
    let mut result = Vec::new();
    let mut text = String::new();
    loop {
        let (namespace, event) = reader.read_resolved_event()?;
        match event {
            Event::Start(element) => {
                flush_text(&mut text, &mut result);
                result.push(Item::Start(expanded(namespace, element.name().as_ref()), attributes(&reader, &element)?));
            }
            Event::Empty(element) => {
                flush_text(&mut text, &mut result);
                let name = expanded(namespace, element.name().as_ref());
                result.push(Item::Start(name.clone(), attributes(&reader, &element)?));
                result.push(Item::End(name));
            }
            Event::End(element) => {
                flush_text(&mut text, &mut result);
                result.push(Item::End(expanded(namespace, element.name().as_ref())));
            }
            Event::Text(value) => text.push_str(&quick_xml::escape::unescape(value.as_ref())?),
            Event::CData(value) => text.push_str(std::str::from_utf8(value.as_ref())?),
            Event::GeneralRef(reference) => {
                if let Some(character) = reference.resolve_char_ref()? { text.push(character); }
                else { text.push_str(named_entity(reference.as_ref())?); }
            }
            Event::Eof => break,
            Event::Decl(_) | Event::PI(_) | Event::Comment(_) | Event::DocType(_) => {}
        }
    }
    flush_text(&mut text, &mut result);
    Ok(result)
}

fn attributes(reader: &NsReader<&[u8]>, element: &quick_xml::events::BytesStart<'_>) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut attributes = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute?;
        if attribute.key.as_ref() == b"xmlns" || attribute.key.as_ref().starts_with(b"xmlns:") { continue; }
        let (namespace, local) = reader.resolver().resolve_attribute(QName(attribute.key.as_ref()));
        attributes.push((expanded(namespace, local.as_ref()), attribute.unescape_value()?.into_owned()));
    }
    attributes.sort();
    Ok(attributes)
}

fn expanded(namespace: ResolveResult<'_>, local: &[u8]) -> String {
    let local = String::from_utf8_lossy(local);
    match namespace {
        ResolveResult::Bound(uri) => format!("{{{}}}{local}", uri.as_ref()),
        ResolveResult::Unbound => local.into_owned(),
        _ => panic!("unresolved XML namespace"),
    }
}

fn flush_text(text: &mut String, result: &mut Vec<Item>) {
    if !text.trim().is_empty() { result.push(Item::Text(std::mem::take(text))); }
    else { text.clear(); }
}
