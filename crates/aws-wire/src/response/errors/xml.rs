use super::super::{DecodeError, DecodeLimits};
use super::{Field, MAX_FIELD_BYTES};
use quick_xml::{
    Reader,
    events::{BytesRef, BytesStart, Event},
};

pub(super) fn scan(
    body: &[u8],
    limits: DecodeLimits,
    field: Field,
) -> Result<Option<String>, DecodeError> {
    if body.is_empty() {
        return Ok(None);
    }
    let source = std::str::from_utf8(body).map_err(syntax)?;
    let mut reader = Reader::from_str(source);
    let mut state = State::new(limits, field);
    loop {
        let event = reader.read_event().map_err(syntax)?;
        if matches!(event, Event::Eof) {
            return state.finish();
        }
        state.event(event)?;
    }
}

struct State {
    limits: DecodeLimits,
    depth: usize,
    nodes: usize,
    root_seen: bool,
    root_closed: bool,
    errors_wrapper: bool,
    error_depth: Option<usize>,
    field: Field,
    field_depth: Option<usize>,
    field_text: String,
    found: Option<String>,
}

impl State {
    fn new(limits: DecodeLimits, field: Field) -> Self {
        Self {
            limits,
            depth: 0,
            nodes: 0,
            root_seen: false,
            root_closed: false,
            errors_wrapper: false,
            error_depth: None,
            field,
            field_depth: None,
            field_text: String::new(),
            found: None,
        }
    }

    fn event(&mut self, event: Event<'_>) -> Result<(), DecodeError> {
        match event {
            Event::Start(element) => self.start(element, false),
            Event::Empty(element) => self.start(element, true),
            Event::Text(text) => self.text(text.as_ref(), false),
            Event::CData(text) => self.text(text.as_ref(), true),
            Event::GeneralRef(reference) => self.reference(reference),
            Event::End(_) => self.end(),
            Event::DocType(_) => Err(DecodeError::Invalid("service error XML DTD")),
            Event::Decl(_) if self.root_seen => Err(DecodeError::Invalid(
                "misplaced service error XML declaration",
            )),
            _ => nodes_add(&mut self.nodes, self.limits),
        }
    }

    fn start(&mut self, element: BytesStart<'_>, empty: bool) -> Result<(), DecodeError> {
        self.check_element(&element)?;
        let next = next_depth(self.depth)?;
        depth_check(next, self.limits)?;
        let local = element.name().local_name();
        self.detect_field(local.as_ref(), next, empty)?;
        self.update_root(next, empty, local.as_ref());
        Ok(())
    }

    fn update_root(&mut self, next: usize, empty: bool, local: &str) {
        if self.depth == 0 {
            self.root_seen = true;
            self.root_closed = empty;
        }
        if !empty {
            if self.depth == 1 && matches!(local, "Errors" | "errors") {
                self.errors_wrapper = true;
            }
            if self.error_depth.is_none()
                && (self.depth <= 1 || (self.depth == 2 && self.errors_wrapper))
                && matches!(local, "Error" | "error")
            {
                self.error_depth = Some(next);
            }
            self.depth = next;
        }
    }

    fn check_element(&mut self, element: &BytesStart<'_>) -> Result<(), DecodeError> {
        if self.root_closed {
            return Err(DecodeError::Invalid("multiple service error XML roots"));
        }
        if self.field_depth.is_some() {
            return Err(self.field.invalid());
        }
        nodes_add(&mut self.nodes, self.limits)?;
        for attribute in element.attributes() {
            let attribute = attribute.map_err(syntax)?;
            nodes_add(&mut self.nodes, self.limits)?;
            quick_xml::escape::unescape(attribute.value.as_ref()).map_err(syntax)?;
        }
        Ok(())
    }

    fn detect_field(&mut self, local: &str, next: usize, empty: bool) -> Result<(), DecodeError> {
        let is_field = match self.field {
            Field::Code => matches!(local, "Code" | "code"),
            Field::Message => self.field.matches(local),
        };
        if !is_field || self.error_depth != Some(self.depth) {
            return Ok(());
        }
        if empty || self.found.is_some() {
            return Err(self.field.invalid());
        }
        self.field_depth = Some(next);
        Ok(())
    }

    fn text(&mut self, value: &str, cdata: bool) -> Result<(), DecodeError> {
        nodes_add(&mut self.nodes, self.limits)?;
        if self.depth == 0
            && (cdata
                || !value
                    .bytes()
                    .all(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r')))
        {
            return Err(DecodeError::Invalid("text outside service error XML root"));
        }
        self.append_if_field(value)
    }

    fn reference(&mut self, reference: BytesRef<'_>) -> Result<(), DecodeError> {
        nodes_add(&mut self.nodes, self.limits)?;
        check_reference_depth(self.depth)?;
        let mut bytes = [0; 4];
        let value = resolve_reference(reference, &mut bytes)?;
        self.append_if_field(value)
    }

    fn append_if_field(&mut self, value: &str) -> Result<(), DecodeError> {
        if self.field_depth == Some(self.depth) {
            append_field(&mut self.field_text, value, self.field)?;
        }
        Ok(())
    }
    fn end(&mut self) -> Result<(), DecodeError> {
        nodes_add(&mut self.nodes, self.limits)?;
        self.close_field()?;
        self.close_error();
        if self.depth == 2 {
            self.errors_wrapper = false;
        }
        self.depth = self
            .depth
            .checked_sub(1)
            .ok_or(DecodeError::Invalid("unbalanced service error XML"))?;
        self.root_closed = self.depth == 0;
        Ok(())
    }

    fn close_field(&mut self) -> Result<(), DecodeError> {
        if self.field_depth == Some(self.depth) {
            self.found = Some(match self.field {
                Field::Code => super::normalize_code(&self.field_text)?,
                Field::Message => std::mem::take(&mut self.field_text),
            });
            self.field_depth = None;
        }
        Ok(())
    }

    fn close_error(&mut self) {
        if self.error_depth == Some(self.depth) {
            self.error_depth = None;
        }
    }

    fn finish(self) -> Result<Option<String>, DecodeError> {
        if !self.root_seen || !self.root_closed || self.depth != 0 || self.field_depth.is_some() {
            return Err(DecodeError::Invalid("incomplete service error XML"));
        }
        Ok(self.found)
    }
}

fn next_depth(depth: usize) -> Result<usize, DecodeError> {
    depth
        .checked_add(1)
        .ok_or(DecodeError::Limit("response depth"))
}

fn check_reference_depth(depth: usize) -> Result<(), DecodeError> {
    if depth == 0 {
        Err(DecodeError::Invalid(
            "reference outside service error XML root",
        ))
    } else {
        Ok(())
    }
}

fn resolve_reference<'a>(
    reference: BytesRef<'_>,
    bytes: &'a mut [u8; 4],
) -> Result<&'a str, DecodeError> {
    if let Some(character) = reference.resolve_char_ref().map_err(syntax)? {
        return Ok(character.encode_utf8(bytes));
    }
    quick_xml::escape::resolve_xml_entity(reference.as_ref())
        .ok_or(DecodeError::Invalid("unknown service error XML entity"))
}
fn append_field(target: &mut String, value: &str, field: Field) -> Result<(), DecodeError> {
    let length = target
        .len()
        .checked_add(value.len())
        .ok_or_else(|| field.invalid())?;
    if length > MAX_FIELD_BYTES {
        return Err(field.invalid());
    }
    target.try_reserve(value.len())?;
    target.push_str(value);
    Ok(())
}

fn nodes_add(nodes: &mut usize, limits: DecodeLimits) -> Result<(), DecodeError> {
    *nodes = nodes
        .checked_add(1)
        .ok_or(DecodeError::Limit("response nodes"))?;
    if *nodes > limits.nodes() {
        return Err(DecodeError::Limit("response nodes"));
    }
    Ok(())
}

fn depth_check(depth: usize, limits: DecodeLimits) -> Result<(), DecodeError> {
    if depth > limits.depth() {
        Err(DecodeError::Limit("response depth"))
    } else {
        Ok(())
    }
}

fn syntax<E>(_: E) -> DecodeError {
    DecodeError::Invalid("service error XML syntax")
}
