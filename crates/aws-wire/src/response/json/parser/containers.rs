use super::*;

impl Engine<'_, '_> {
    pub(super) fn open_object(&mut self, value: Value) -> Result<(), DecodeError> {
        let target = object_target(self.context, value.target)?;
        let depth = nested_depth(value.depth, self.context)?;
        if target.is_some() {
            begin_map(self.builder, value.key)?;
        }
        let object = Object::new(target, depth, MapRole::Nested, self.context)?;
        self.push(Task::Object(object))
    }

    pub(super) fn open_array(&mut self, value: Value) -> Result<(), DecodeError> {
        let child = element_target(self.context, value.target)?;
        let depth = nested_depth(value.depth, self.context)?;
        if value.target.is_some() && child.is_none() {
            return Err(DecodeError::Invalid("missing list element shape"));
        }
        if value.target.is_some() {
            begin_list(self.builder, value.key)?;
        }
        self.push(Task::Array(Array {
            target: value.target,
            depth,
            position: Position::First,
        }))
    }
}
