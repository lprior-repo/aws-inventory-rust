use quick_xml::{Reader, events::Event};
use serde_json::Value;

pub(in crate::inventory_seed) fn message(body: &[u8]) -> String {
    if let Ok(value) = serde_json::from_slice::<Value>(body) {
        for pointer in ["/Message", "/message", "/Error/Message", "/error/message"] {
            if let Some(message) = value.pointer(pointer).and_then(Value::as_str) {
                return super::bounded_text(message);
            }
        }
    }
    let source = String::from_utf8_lossy(body);
    match xml(&source) {
        Some(text) => text,
        None => super::bounded_text(&source),
    }
}

fn xml(source: &str) -> Option<String> {
    let mut reader = Reader::from_str(source);
    for _ in 0..16_384 {
        match reader.read_event().ok()? {
            Event::Start(event) if event.local_name().as_ref().eq_ignore_ascii_case("message") => {
                let text = reader.read_text(event.name()).ok()?;
                let text = quick_xml::escape::unescape(&text).ok()?;
                return Some(super::bounded_text(&text));
            }
            Event::Eof => return None,
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn given_service_errors_when_reported_then_message_is_decoded_and_bounded() {
        assert_eq!(
            super::message(br#"{"Message":"not permitted"}"#),
            "not permitted"
        );
        assert_eq!(
            super::message(b"<Error><Message>not &amp; permitted</Message></Error>"),
            "not & permitted"
        );
        let message = super::message(format!("{{\"message\":\"{}\"}}", "x".repeat(201)).as_bytes());
        assert_eq!(message.len(), 200);
    }
}
