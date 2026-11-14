use super::{CompileError, Registry};
use inventory_core::record::{FieldId, Value};

pub(super) fn encode(registry: &Registry) -> Result<(Value, Value), CompileError> {
    let mut names = Vec::new();
    let mut offsets = Vec::new();
    for raw in 1..=registry.len() {
        let id = FieldId::new(u32::try_from(raw).map_err(|_| CompileError::Kind)?)?;
        let name = registry.name(id).ok_or(CompileError::Kind)?;
        let start = u32::try_from(names.len()).map_err(|_| CompileError::Kind)?;
        names.extend_from_slice(name.as_bytes());
        let end = u32::try_from(names.len()).map_err(|_| CompileError::Kind)?;
        offsets.extend_from_slice(&start.to_be_bytes());
        offsets.extend_from_slice(&end.to_be_bytes());
    }
    Ok((Value::Bytes(names), Value::Bytes(offsets)))
}
