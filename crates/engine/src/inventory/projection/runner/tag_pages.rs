use super::{GraphError, MAX_REBUILD_PAGES, ProjectionSnapshotRead, ResourceKey, StoreError};
use crate::inventory::ProjectionTag;
use futures_util::{Stream, stream};

pub(super) fn pages<'a>(
    snapshot: &'a mut impl ProjectionSnapshotRead,
    key: &'a ResourceKey,
    limit: u16,
) -> impl Stream<Item = Result<Vec<ProjectionTag>, GraphError>> + 'a {
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
                .map_err(GraphError::Store)?;
            let done = page.next.is_none();
            Ok::<_, GraphError>(Some((page.tags, (snapshot, page.next, remaining, done))))
        },
    )
}

fn remaining_pages(remaining: usize) -> Result<usize, GraphError> {
    remaining
        .checked_sub(1)
        .ok_or(GraphError::Store(StoreError::Invalid(
            "projection tag page bound exceeded",
        )))
}
