use super::super::{DecodeError, DecodeLimits};
use quick_xml::{
    Reader,
    events::{BytesRef, BytesStart, Event},
};

pub(super) fn code(body: &[u8], limits: DecodeLimits) -> Result<Option<String>, DecodeError> {
    if body.is_empty() {
        return Ok(None);
    }
    let source = std::str::from_utf8(body).map_err(syntax)?;
    let mut reader = Reader::from_str(source);
    let mut state = State::new(limits);
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
    error_depth: Option<usize>,
    code_depth: Option<usize>,
    code_text: String,
    found: Option<String>,
}

impl State {
    fn new(limits: DecodeLimits) -> Self {
        Self {
            limits,
            depth: 0,
            nodes: 0,
            root_seen: false,
            root_closed: false,
            error_depth: None,
            code_depth: None,
            code_text: String::new(),
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
        self.detect_code(local.as_ref(), next, empty)?;
        self.update_root(next, empty, local.as_ref());
        Ok(())
    }

    fn update_root(&mut self, next: usize, empty: bool, local: &str) {
        if self.depth == 0 {
            self.root_seen = true;
            self.root_closed = empty;
        }
        if !empty {
            self.depth = next;
            if local == "Error" || local == "error" {
                self.error_depth = Some(next);
            }
        }
    }

    fn check_element(&mut self, element: &BytesStart<'_>) -> Result<(), DecodeError> {
        if self.root_closed {
            return Err(DecodeError::Invalid("multiple service error XML roots"));
        }
        if self.code_depth.is_some() {
            return Err(DecodeError::Invalid("nested service error code"));
        }
        nodes_add(&mut self.nodes, self.limits)?;
        for attribute in element.attributes() {
            let attribute = attribute.map_err(syntax)?;
            nodes_add(&mut self.nodes, self.limits)?;
            quick_xml::escape::unescape(attribute.value.as_ref()).map_err(syntax)?;
        }
        Ok(())
    }

    fn detect_code(&mut self, local: &str, next: usize, empty: bool) -> Result<(), DecodeError> {
        if !matches!(local, "Code" | "code") || self.error_depth != Some(self.depth) {
            return Ok(());
        }
        if empty || self.found.is_some() {
            return Err(DecodeError::Invalid(
                "empty or duplicate service error code",
            ));
        }
        self.code_depth = Some(next);
        self.code_text.clear();
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
        if self.code_depth == Some(self.depth) {
            append_code(&mut self.code_text, value)?;
        }
        Ok(())
    }

    fn reference(&mut self, reference: BytesRef<'_>) -> Result<(), DecodeError> {
        nodes_add(&mut self.nodes, self.limits)?;
        check_reference_depth(self.depth)?;
        let mut bytes = [0; 4];
        let value = resolve_reference(reference, &mut bytes)?;
        self.append_if_code(value)
    }

    fn append_if_code(&mut self, value: &str) -> Result<(), DecodeError> {
        if self.code_depth == Some(self.depth) {
            append_code(&mut self.code_text, value)?;
        }
        Ok(())
    }
    fn end(&mut self) -> Result<(), DecodeError> {
        nodes_add(&mut self.nodes, self.limits)?;
        self.close_code()?;
        self.close_error();
        self.depth = self
            .depth
            .checked_sub(1)
            .ok_or(DecodeError::Invalid("unbalanced service error XML"))?;
        self.root_closed = self.depth == 0;
        Ok(())
    }

    fn close_code(&mut self) -> Result<(), DecodeError> {
        if self.code_depth == Some(self.depth) {
            self.found = Some(super::normalize_code(&self.code_text)?);
            self.code_depth = None;
            self.code_text.clear();
        }
        Ok(())
    }

    fn close_error(&mut self) {
        if self.error_depth == Some(self.depth) {
            self.error_depth = None;
        }
    }

    fn finish(self) -> Result<Option<String>, DecodeError> {
        if !self.root_seen || !self.root_closed || self.depth != 0 || self.code_depth.is_some() {
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
fn append_code(target: &mut String, value: &str) -> Result<(), DecodeError> {
    let length = target
        .len()
        .checked_add(value.len())
        .ok_or(DecodeError::Limit("service error code"))?;
    if length > 256 {
        return Err(DecodeError::Invalid("invalid service error code"));
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
