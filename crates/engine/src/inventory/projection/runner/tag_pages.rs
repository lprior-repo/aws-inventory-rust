use super::{EntryError, MAX_REBUILD_PAGES, ProjectionSnapshotRead, ResourceKey, StoreError};
use crate::inventory::ProjectionTag;
use futures_util::{Stream, stream};

pub(super) fn pages<'a>(
    snapshot: &'a mut impl ProjectionSnapshotRead,
    key: &'a ResourceKey,
    limit: u16,
) -> impl Stream<Item = Result<Vec<ProjectionTag>, EntryError>> + 'a {
    stream::try_unfold(
        (snapshot, None, MAX_REBUILD_PAGES, false),
        move |(snapshot, cursor, remaining, done)| async move {
            if done {
                return Ok(None);
            }
            let remaining = remaining_pages(remaining)?;
            let page = snapshot
                .tags_for(key, cursor, limit)
                .await
                .map_err(EntryError::Store)?;
            let done = page.next.is_none();
            Ok::<_, EntryError>(Some((page.tags, (snapshot, page.next, remaining, done))))
        },
    )
}

fn remaining_pages(remaining: usize) -> Result<usize, EntryError> {
    remaining
        .checked_sub(1)
        .ok_or(EntryError::Store(StoreError::Invalid(
            "projection tag page bound exceeded",
        )))
}
