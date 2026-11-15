use super::{Record, SweepError, ValueRef, tokens};
use inventory_core::record::FieldId;

pub(super) fn select<'a>(
    response: &'a Record,
    path: &[FieldId],
) -> Result<Option<ValueRef<'a>>, SweepError> {
    match tokens::path_value(response, path)? {
        None | Some(ValueRef::Null) => Ok(None),
        Some(ValueRef::List(items)) => items
            .iter()?
            .try_fold(None, |_, item| item.map(Some))
            .map_err(SweepError::from),
        _ => Err(SweepError::Operation("pagination last-item selector requires a list")),
    }
}
