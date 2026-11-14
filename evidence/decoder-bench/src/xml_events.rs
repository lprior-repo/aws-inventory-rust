use crate::{
    entities,
    error::{Error, Result, syntax},
    sink::Sink,
    xml_bridge::Bridge,
};
use quick_xml::{Reader, events::Event};
use xmlparser::{ElementEnd, Token, Tokenizer};
pub fn quick(input: &[u8], sink: &mut Sink) -> Result<()> {
    let mut reader = Reader::from_reader(input);
    let mut bridge = Bridge::new(sink);
    let mut done = false;
    std::iter::from_fn(|| {
        if done {
            return None;
        }
        Some(match reader.read_event().map_err(syntax) {
            Ok(Event::Eof) => {
                done = true;
                Ok(())
            }
            Ok(event) => quick_event(event, &mut bridge),
            Err(e) => Err(e),
        })
    })
    .take(input.len().saturating_add(1))
    .try_for_each(|r| r)?;
    if !done {
        return Err(Error::Bound);
    }
    bridge.finish()
}
fn quick_event(event: Event<'_>, b: &mut Bridge<'_>) -> Result<()> {
    match event {
        Event::Start(e) => {
            attributes(&e)?;
            b.start(e.name().as_ref())
        }
        Event::Empty(e) => {
            attributes(&e)?;
            b.start(e.name().as_ref())?;
            b.close()
        }
        Event::End(e) => b.end(e.name().as_ref()),
        Event::Text(e) => b.text(&e.xml_content(quick_xml::XmlVersion::Implicit1_0)),
        Event::CData(e) => b.text(&e.xml_content(quick_xml::XmlVersion::Implicit1_0)),
        Event::GeneralRef(e) => b.text(&entities::decode(&format!("&{};", e.as_ref()))?),
        Event::DocType(_) => Err(Error::Shape("DTD disabled")),
        _ => Ok(()),
    }
}
fn attributes(e: &quick_xml::events::BytesStart<'_>) -> Result<()> {
    e.attributes().try_for_each(|a| {
        let a = a.map_err(syntax)?;
        entities::decode(a.value.as_ref())?;
        Ok(())
    })
}
pub fn raw(input: &[u8], sink: &mut Sink) -> Result<()> {
    let text = std::str::from_utf8(input).map_err(syntax)?;
    let mut bridge = Bridge::new(sink);
    Tokenizer::from(text).try_for_each(|token| match token.map_err(syntax)? {
        Token::ElementStart { prefix, local, .. } => {
            if !prefix.as_str().is_empty() {
                return Err(Error::Shape("fixture namespace prefix"));
            }
            bridge.start(local.as_str())
        }
        Token::ElementEnd {
            end: ElementEnd::Close(_, local),
            ..
        } => bridge.end(local.as_str()),
        Token::ElementEnd {
            end: ElementEnd::Empty,
            ..
        } => bridge.close(),
        Token::Text { text } => bridge.text(&entities::decode(text.as_str())?),
        Token::Cdata { text, .. } => bridge.text(text.as_str()),
        Token::Attribute { value, .. } => {
            entities::decode(value.as_str())?;
            Ok(())
        }
        Token::DtdStart { .. } | Token::EmptyDtd { .. } => Err(Error::Shape("DTD disabled")),
        _ => Ok(()),
    })?;
    bridge.finish()
}
pub fn xml_rs(input: &[u8], sink: &mut Sink) -> Result<()> {
    let mut bridge = Bridge::new(sink);
    xml::reader::EventReader::new(input)
        .into_iter()
        .try_for_each(|event| {
            use xml::reader::XmlEvent;
            match event.map_err(syntax)? {
                XmlEvent::StartElement { name, .. } => bridge.start(&name.local_name),
                XmlEvent::EndElement { name } => bridge.end(&name.local_name),
                XmlEvent::Characters(text) | XmlEvent::CData(text) | XmlEvent::Whitespace(text) => {
                    bridge.text(&text)
                }
                _ => Ok(()),
            }
        })?;
    bridge.finish()
}
pub fn quick_reader(input: impl std::io::BufRead, sink: &mut Sink, bound: usize) -> Result<()> {
    let mut reader = Reader::from_reader(input);
    let mut bridge = Bridge::new(sink);
    let mut buffer = Vec::new();
    let mut done = false;
    std::iter::from_fn(|| {
        if done {
            return None;
        }
        buffer.clear();
        Some(match reader.read_event_into(&mut buffer).map_err(syntax) {
            Ok(Event::Eof) => {
                done = true;
                Ok(())
            }
            Ok(event) => quick_event(event, &mut bridge),
            Err(e) => Err(e),
        })
    })
    .take(bound.saturating_add(1))
    .try_for_each(|r| r)?;
    if !done {
        return Err(Error::Bound);
    }
    bridge.finish()
}
