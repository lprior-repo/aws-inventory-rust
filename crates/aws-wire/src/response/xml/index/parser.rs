use super::{DecodeError, Document, ResponseContext, Token, validation::*};
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};

pub(super) fn parse(context: ResponseContext<'_>) -> Result<Document<'_>, DecodeError> {
    if context.body.len() > context.limits.body_bytes() {
        return Err(DecodeError::Limit("response body"));
    }
    let mut reader = Reader::from_reader(context.body);
    reader.config_mut().trim_text(false);
    let mut state = State::new(context);
    loop {
        let event = reader
            .read_event()
            .map_err(|_| DecodeError::Invalid("XML document"))?;
        if matches!(event, Event::Eof) {
            break;
        }
        count_node(&mut state.count, context)?;
        state.event(event)?;
    }
    state.finish()
}

struct State<'a> {
    document: Document<'a>,
    stack: Vec<usize>,
    context: ResponseContext<'a>,
    count: usize,
}

impl<'a> State<'a> {
    fn new(context: ResponseContext<'a>) -> Self {
        Self {
            document: Document {
                tokens: Vec::new(),
                parents: Vec::new(),
                mates: Vec::new(),
                root: 0,
                has_root: false,
            },
            stack: Vec::new(),
            context,
            count: 0,
        }
    }

    fn event(&mut self, event: Event<'a>) -> Result<(), DecodeError> {
        match event {
            Event::DocType(_) => Err(DecodeError::Invalid("XML DTD")),
            Event::Start(start) => self.start(start),
            Event::Empty(start) => self.empty(start),
            Event::End(_) => self.end(),
            Event::Text(text) => self.leaf(Token::Text(text)),
            Event::CData(text) => self.leaf(Token::CData(text)),
            Event::GeneralRef(value) => self.leaf(Token::Reference(reference(value)?)),
            _ => self.leaf(Token::Other),
        }
    }

    fn prepare(&mut self, start: &BytesStart<'_>) -> Result<(), DecodeError> {
        count_attributes(start, &mut self.count, self.context)?;
        if self.stack.len() >= self.context.limits.depth() {
            return Err(DecodeError::Limit("response depth"));
        }
        if self.stack.is_empty() {
            if self.document.has_root {
                return Err(DecodeError::Invalid("XML document root"));
            }
            self.document.has_root = true;
            self.document.root = self.document.tokens.len();
        }
        Ok(())
    }

    fn start(&mut self, start: BytesStart<'a>) -> Result<(), DecodeError> {
        self.prepare(&start)?;
        let index = self.document.tokens.len();
        self.stack.try_reserve(1)?;
        self.leaf(Token::Start(start))?;
        self.stack.push(index);
        Ok(())
    }

    fn empty(&mut self, start: BytesStart<'a>) -> Result<(), DecodeError> {
        self.prepare(&start)?;
        self.leaf(Token::Empty(start))
    }

    fn end(&mut self) -> Result<(), DecodeError> {
        let index = self.document.tokens.len();
        let open = self
            .stack
            .pop()
            .ok_or(DecodeError::Invalid("XML nesting"))?;
        self.leaf(Token::Other)?;
        *self
            .document
            .mates
            .get_mut(index)
            .ok_or(DecodeError::Invalid("XML nesting"))? = open;
        *self
            .document
            .mates
            .get_mut(open)
            .ok_or(DecodeError::Invalid("XML nesting"))? = index;
        Ok(())
    }

    fn leaf(&mut self, token: Token<'a>) -> Result<(), DecodeError> {
        let document = &mut self.document;
        push_leaf(
            token,
            &mut document.tokens,
            &mut document.parents,
            &mut document.mates,
            &self.stack,
        )
    }

    fn finish(self) -> Result<Document<'a>, DecodeError> {
        if !self.stack.is_empty()
            || !self.document.has_root
                && self
                    .context
                    .body
                    .iter()
                    .any(|byte| !byte.is_ascii_whitespace())
        {
            return Err(DecodeError::Invalid("XML document root"));
        }
        if self.document.has_root {
            validate_outside_text(
                &self.document.tokens,
                &self.document.parents,
                self.document.root,
            )?;
        }
        Ok(self.document)
    }
}
