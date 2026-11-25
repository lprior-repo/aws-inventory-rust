use super::{DecodeError, Decoder, Frame, Scanner, Type, depth_check, nodes_add, syntax};

impl Scanner {
    pub(super) fn consume(
        &mut self,
        decoder: &mut Decoder<'_>,
        candidate: bool,
    ) -> Result<(), DecodeError> {
        let mut depth = self.frames.len();
        while decoder.datatype().map_err(syntax)? == Type::Tag {
            depth_check(depth, self.limits)?;
            decoder.tag().map_err(syntax)?;
            nodes_add(&mut self.nodes, self.limits)?;
            depth = depth
                .checked_add(1)
                .ok_or(DecodeError::Limit("response depth"))?;
        }
        if candidate {
            return self.extract_text(decoder);
        }
        self.consume_plain(decoder)
    }

    fn consume_plain(&mut self, decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
        match decoder.datatype().map_err(syntax)? {
            Type::Map | Type::MapIndef => self.map(decoder),
            Type::Array | Type::ArrayIndef => self.array(decoder),
            Type::StringIndef => self.text_chunks(decoder),
            Type::BytesIndef => self.byte_chunks(decoder),
            Type::String => decoder.str().map(|_| ()).map_err(syntax),
            Type::Bytes => decoder.bytes().map(|_| ()).map_err(syntax),
            Type::Break => Err(DecodeError::Invalid("unexpected CBOR break")),
            _ => decoder.skip().map_err(syntax),
        }
    }

    fn extract_text(&mut self, decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
        if decoder.datatype().map_err(syntax)? != Type::String {
            return Err(self.field.invalid());
        }
        self.field
            .record(&mut self.found, decoder.str().map_err(syntax)?)
    }

    fn open(&mut self, frame: Frame) -> Result<(), DecodeError> {
        depth_check(self.frames.len(), self.limits)?;
        self.frames.try_reserve(1)?;
        self.frames.push(frame);
        Ok(())
    }

    fn map(&mut self, decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
        let remaining = decoder.map().map_err(syntax)?;
        self.open(Frame::Map {
            remaining,
            pending: None,
            root: false,
        })
    }

    fn array(&mut self, decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
        let remaining = decoder.array().map_err(syntax)?;
        self.open(Frame::Array { remaining })
    }

    fn text_chunks(&mut self, decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
        for text in decoder.str_iter().map_err(syntax)? {
            nodes_add(&mut self.nodes, self.limits)?;
            text.map_err(syntax)?;
        }
        Ok(())
    }

    fn byte_chunks(&mut self, decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
        for bytes in decoder.bytes_iter().map_err(syntax)? {
            nodes_add(&mut self.nodes, self.limits)?;
            bytes.map_err(syntax)?;
        }
        Ok(())
    }
}
