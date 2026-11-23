use super::*;
use std::slice;

pub(super) struct Candidates<'index, 'a, 'catalog> {
    pub(super) exact: slice::Iter<'index, Entry<'a, 'catalog>>,
    pub(super) staged: slice::Iter<'index, Entry<'a, 'catalog>>,
    pub(super) wildcards: slice::Iter<'index, Consumer<'a, 'catalog>>,
}

impl<'a, 'catalog> Iterator for Candidates<'_, 'a, 'catalog> {
    type Item = &'a CallablePlan<'catalog>;

    fn next(&mut self) -> Option<Self::Item> {
        let consumer = [
            self.exact.as_slice().first().map(|entry| entry.consumer),
            self.staged.as_slice().first().map(|entry| entry.consumer),
            self.wildcards.as_slice().first().copied(),
        ]
        .into_iter()
        .flatten()
        .min_by_key(|consumer| consumer.ordinal)?;
        advance(&mut self.exact, consumer.ordinal);
        advance(&mut self.staged, consumer.ordinal);
        if self
            .wildcards
            .as_slice()
            .first()
            .is_some_and(|entry| entry.ordinal == consumer.ordinal)
        {
            self.wildcards.next();
        }
        Some(consumer.plan)
    }
}

fn advance(entries: &mut slice::Iter<'_, Entry<'_, '_>>, ordinal: usize) {
    if entries
        .as_slice()
        .first()
        .is_some_and(|entry| entry.consumer.ordinal == ordinal)
    {
        entries.next();
    }
}
