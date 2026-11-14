use super::*;

impl Object {
    pub(super) fn new(
        target: Option<Target>,
        depth: usize,
        role: MapRole,
        context: ResponseContext<'_>,
    ) -> Result<Self, DecodeError> {
        let variants = match target {
            Some(target) if shape_kind(context, target)? == "union" => Variants::EmptyUnion,
            _ => Variants::Structure,
        };
        Ok(Self {
            target,
            depth,
            position: Position::First,
            role,
            variants,
            fields: Vec::new(),
        })
    }

    fn remember(&mut self, key: &Key) -> Result<(), DecodeError> {
        if let Key::Field(id, _, true) = key {
            self.fields.try_reserve(1)?;
            self.fields.push(*id);
        }
        Ok(())
    }
}

impl Variants {
    fn observe(&mut self, key: &Key) -> Result<(), DecodeError> {
        if matches!(key, Key::Ignore) {
            return Ok(());
        }
        match self {
            Self::EmptyUnion => *self = Self::SeenUnion,
            Self::SeenUnion => return Err(DecodeError::Invalid("union has multiple variants")),
            Self::Structure => {}
        }
        Ok(())
    }
}

impl Engine<'_, '_> {
    pub(super) fn object(&mut self, mut object: Object) -> Result<(), DecodeError> {
        let name = match object.position {
            Position::First => self.parser.next_object(),
            Position::Next => self.parser.next_key(),
        }
        .map_err(syntax)?;
        let Some(name) = name else {
            return self.end_object(object);
        };
        let (target, key) = member_target(self.context, object.target, name)?;
        let (target, key) = self.null_child(object.target, target, key)?;
        object.variants.observe(&key)?;
        object.remember(&key)?;
        object.position = Position::Next;
        let value = Value {
            target,
            key,
            depth: object.depth,
        };
        self.push(Task::Object(object))?;
        self.push(Task::Value(value))
    }

    fn null_child(
        &mut self,
        parent: Option<Target>,
        target: Option<Target>,
        key: Key,
    ) -> Result<(Option<Target>, Key), DecodeError> {
        if self.parser.peek().map_err(syntax)? == Peek::Null
            && !schema::preserves_null(self.context, parent)?
        {
            Ok((None, Key::Ignore))
        } else {
            Ok((target, key))
        }
    }

    fn end_object(&mut self, object: Object) -> Result<(), DecodeError> {
        if matches!(object.variants, Variants::EmptyUnion) {
            return Err(DecodeError::Invalid("union has no variant"));
        }
        crate::response::defaults::apply(
            self.builder,
            object.target,
            &object.fields,
            self.context.service,
        )?;
        if matches!(object.role, MapRole::Nested) && object.target.is_some() {
            self.builder.end()?;
        }
        Ok(())
    }
}
