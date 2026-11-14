use crate::request::{common::BoundedBody, EncodeError};
use inventory_core::catalog::XmlNamespace;

pub(super) fn element_namespace(body: &mut BoundedBody, name: &str, value: &str, namespace: Option<XmlNamespace<'_>>) -> Result<(), EncodeError> {
    open_namespace(body, name, namespace)?;
    body.write(b">")?;
    escape_text(body, value)?;
    close(body, name)
}

pub(super) fn empty_namespace(body: &mut BoundedBody, name: &str, namespace: Option<XmlNamespace<'_>>) -> Result<(), EncodeError> {
    open_namespace(body, name, namespace)?;
    body.write(b"/>")
}

pub(super) fn binary_namespace(body: &mut BoundedBody, name: &str, value: &[u8], namespace: Option<XmlNamespace<'_>>) -> Result<(), EncodeError> {
    open_namespace(body, name, namespace)?;
    body.write(b">")?;
    super::super::json::base64(body, value)?;
    close(body, name)
}

pub(super) fn open_namespace(body: &mut BoundedBody, name: &str, namespace: Option<XmlNamespace<'_>>) -> Result<(), EncodeError> {
    open(body, name)?;
    if let Some(namespace) = namespace {
        body.write(b" xmlns")?;
        if let Some(prefix) = namespace.prefix { body.write(b":")?; body.write(prefix.as_bytes())?; }
        body.write(b"=\"")?;
        escape_attr(body, namespace.uri)?;
        body.write(b"\"")?;
    }
    Ok(())
}

pub(super) fn open(body: &mut BoundedBody, name: &str) -> Result<(), EncodeError> {
    body.write(b"<")?;
    body.write(name.as_bytes())
}

pub(super) fn close(body: &mut BoundedBody, name: &str) -> Result<(), EncodeError> {
    body.write(b"</")?;
    body.write(name.as_bytes())?;
    body.write(b">")
}

fn escape_text(body: &mut BoundedBody, value: &str) -> Result<(), EncodeError> {
    value.chars().try_for_each(|character| escape(body, character))
}

pub(super) fn escape_attr(body: &mut BoundedBody, value: &str) -> Result<(), EncodeError> {
    value.chars().try_for_each(|character| escape(body, character))
}

fn escape(body: &mut BoundedBody, character: char) -> Result<(), EncodeError> {
    match character {
        '&' => body.write(b"&amp;"), '<' => body.write(b"&lt;"), '>' => body.write(b"&gt;"),
        '"' => body.write(b"&quot;"), '\'' => body.write(b"&apos;"),
        _ => { let mut bytes = [0_u8; 4]; body.write(character.encode_utf8(&mut bytes).as_bytes()) }
    }
}
