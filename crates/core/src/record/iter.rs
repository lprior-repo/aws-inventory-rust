//! Shared, statically dispatched traversal, bounded by the container length.
use super::RecordError;
use minicbor::Decoder;

pub(super) fn entries<'a, T>(
    decoder: Decoder<'a>,
    count: u64,
    mut read: impl FnMut(&mut Decoder<'a>) -> Result<T, RecordError>,
) -> impl Iterator<Item = Result<T, RecordError>> {
    (0..count).scan(Some(decoder), move |state, _| {
        let result = read(state.as_mut()?);
        if result.is_err() {
            *state = None;
        }
        Some(result)
    })
}
