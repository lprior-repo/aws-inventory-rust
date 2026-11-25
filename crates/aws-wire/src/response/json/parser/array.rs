use super::*;

impl Engine<'_, '_> {
    pub(super) fn array(&mut self, mut array: Array) -> Result<(), DecodeError> {
        let next = match array.position {
            Position::First => self.parser.next_array(),
            Position::Next => self.parser.array_step(),
        }
        .map_err(syntax)?;
        let Some(next) = next else {
            if array.target.is_some() {
                self.builder.end()?;
            }
            return Ok(());
        };
        let value = self.array_child(&array, next)?;
        array.position = Position::Next;
        self.push(Task::Array(array))?;
        self.push(Task::Value(value))
    }

    fn array_child(&self, array: &Array, next: Peek) -> Result<Value, DecodeError> {
        let target = if next == Peek::Null
            && !self.context.limits.raw_values
            && !schema::preserves_null(self.context, array.target)?
        {
            if self.context.limits.preserve_fields && array.target.is_some() {
                return Err(DecodeError::Invalid(
                    "null collection element has no inventory value",
                ));
            }
            None
        } else {
            element_target(self.context, array.target)?
        };
        Ok(Value {
            target,
            key: Key::List(self.list_format(array.target)?),
            depth: array.depth,
        })
    }

    fn list_format(&self, target: Option<Target>) -> Result<Option<TimestampFormat>, DecodeError> {
        let Some(Target::Shape(index)) = target else {
            return Ok(None);
        };
        let shape = self
            .context
            .service
            .shape(index)?
            .ok_or(DecodeError::Invalid("missing list shape"))?;
        if shape.kind()? == "list" {
            shape.element_wire()?.timestamp_format().map_err(Into::into)
        } else {
            Ok(None)
        }
    }
}
