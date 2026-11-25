use super::*;
use scalar::ScalarContext;

struct WalkState<'a, 'c, 'r> {
    builder: &'a mut RecordBuilder,
    frames: &'a mut Vec<Frame>,
    context: &'c ResponseContext<'r>,
    nodes: &'a mut usize,
}

pub(super) fn walk(
    decoder: &mut Decoder<'_>,
    builder: &mut RecordBuilder,
    frames: &mut Vec<Frame>,
    context: &ResponseContext<'_>,
    nodes: &mut usize,
) -> Result<(), DecodeError> {
    let mut state = WalkState {
        builder,
        frames,
        context,
        nodes,
    };
    while !state.frames.is_empty() {
        if !state.finish_frame(decoder)? {
            state.read_child(decoder)?;
        }
    }
    Ok(())
}

impl WalkState<'_, '_, '_> {
    fn finish_frame(&mut self, decoder: &mut Decoder<'_>) -> Result<bool, DecodeError> {
        let frame = self
            .frames
            .last_mut()
            .ok_or(DecodeError::Invalid("CBOR stack"))?;
        if !frame_done(decoder, frame)? {
            return Ok(false);
        }
        self.validate_union()?;
        self.apply_defaults()?;
        let frame = self
            .frames
            .pop()
            .ok_or(DecodeError::Invalid("CBOR stack"))?;
        if frame.modeled {
            self.builder.end()?;
        }
        Ok(true)
    }

    fn validate_union(&self) -> Result<(), DecodeError> {
        let frame = self
            .frames
            .last()
            .ok_or(DecodeError::Invalid("CBOR stack"))?;
        if !frame.map || !frame.modeled {
            return Ok(());
        }
        let Some(target) = frame.target else {
            return Ok(());
        };
        if target_kind(target, self.context.service)? == "union" && frame.seen.len() != 1 {
            return Err(DecodeError::Invalid("CBOR union must contain one variant"));
        }
        Ok(())
    }

    fn apply_defaults(&mut self) -> Result<(), DecodeError> {
        let frame = self
            .frames
            .last()
            .ok_or(DecodeError::Invalid("CBOR stack"))?;
        crate::response::defaults::apply(
            self.builder,
            frame.target,
            &frame.seen,
            self.context.service,
            self.context.limits,
        )
    }

    fn read_child(&mut self, decoder: &mut Decoder<'_>) -> Result<(), DecodeError> {
        let frame = self
            .frames
            .last_mut()
            .ok_or(DecodeError::Invalid("CBOR stack"))?;
        let (target, key, modeled) = child(decoder, frame, self.context.service)?;
        self.count_child()?;
        if matches!(key, Some(KeyRef::Field(id)) if self.context.limits.excluded_members.contains(id))
        {
            return unknown::parse_unknown(
                decoder,
                self.frames,
                self.nodes,
                self.context.limits.nodes(),
                self.context.limits.depth(),
            );
        }
        if self.consume_null(decoder, modeled, key)? {
            return Ok(());
        }
        self.remember_field(key)?;
        self.parse_value(decoder, target, key, modeled)
    }

    fn count_child(&mut self) -> Result<(), DecodeError> {
        let frame = self
            .frames
            .last()
            .ok_or(DecodeError::Invalid("CBOR stack"))?;
        if frame.map {
            count_node(self.nodes, self.context.limits.nodes())?;
        }
        count_node(self.nodes, self.context.limits.nodes())
    }

    fn consume_null(
        &mut self,
        decoder: &mut Decoder<'_>,
        modeled: bool,
        key: Option<KeyRef<'_>>,
    ) -> Result<bool, DecodeError> {
        if decoder.datatype().map_err(|_| invalid_cbor())? != Type::Null {
            return Ok(false);
        }
        decoder.null().map_err(|_| invalid_cbor())?;
        if modeled && self.context.limits.raw_values {
            self.builder
                .scalar(key, inventory_core::record::ValueRef::Null)?;
            self.remember_field(key)?;
            return Ok(true);
        }
        let frame = self
            .frames
            .last()
            .ok_or(DecodeError::Invalid("CBOR stack"))?;
        if modeled
            && self.context.limits.preserve_fields
            && !preserves_null(frame, self.context.service)?
        {
            return Err(DecodeError::Invalid(
                "null modeled member has no inventory value",
            ));
        }
        if modeled && preserves_null(frame, self.context.service)? {
            self.builder
                .scalar(key, inventory_core::record::ValueRef::Null)?;
        }
        Ok(true)
    }

    fn remember_field(&mut self, key: Option<KeyRef<'_>>) -> Result<(), DecodeError> {
        if let (Some(KeyRef::Field(id)), Some(frame)) = (key, self.frames.last_mut()) {
            frame.seen.try_reserve(1)?;
            frame.seen.push(id);
        }
        Ok(())
    }

    fn parse_value(
        &mut self,
        decoder: &mut Decoder<'_>,
        target: Option<Target>,
        key: Option<KeyRef<'_>>,
        modeled: bool,
    ) -> Result<(), DecodeError> {
        if !modeled {
            return unknown::parse_unknown(
                decoder,
                self.frames,
                self.nodes,
                self.context.limits.nodes(),
                self.context.limits.depth(),
            );
        }
        let kind = target
            .map(|value| target_kind(value, self.context.service))
            .transpose()?;
        self.parse_known(decoder, ScalarContext { target, key, kind })
    }

    fn parse_known(
        &mut self,
        decoder: &mut Decoder<'_>,
        value: ScalarContext<'_>,
    ) -> Result<(), DecodeError> {
        match decoder.datatype().map_err(|_| invalid_cbor())? {
            Type::Map | Type::MapIndef => open_child_map(
                decoder,
                self.builder,
                self.frames,
                value,
                self.context.limits.depth(),
            ),
            Type::Array | Type::ArrayIndef => open_child_list(
                decoder,
                self.builder,
                self.frames,
                value,
                self.context.limits.depth(),
            ),
            Type::Tag => self.tagged(decoder, value),
            _ => scalar::scalar(decoder, self.builder, value.target, value.key, value.kind),
        }
    }

    fn tagged(
        &mut self,
        decoder: &mut Decoder<'_>,
        value: ScalarContext<'_>,
    ) -> Result<(), DecodeError> {
        count_node(self.nodes, self.context.limits.nodes())?;
        scalar::scalar_tag(decoder, self.builder, value)
    }
}
