use super::{
    GraphClient, GraphError, MAX_REBUILD_PAGES, ProjectionSnapshot, ResourceKey, StoreError,
};
use futures_util::{StreamExt, TryStreamExt, stream};

/// The snapshot and cursor move together so tag pages cannot mix SQLite revisions.
pub(super) async fn project_tags(
    snapshot: &mut ProjectionSnapshot,
    client: &mut GraphClient,
    key: &ResourceKey,
    limit: u16,
) -> Result<(), GraphError> {
    let pages = stream::try_unfold(
        (snapshot, None, MAX_REBUILD_PAGES, false),
        move |(snapshot, cursor, remaining, done)| async move {
            if done {
                return Ok(None);
            }
            let remaining =
                remaining
                    .checked_sub(1)
                    .ok_or(GraphError::Store(StoreError::Invalid(
                        "projection tag page bound exceeded",
                    )))?;
            let page = snapshot
                .tags_for(key, cursor, limit)
                .await
                .map_err(GraphError::Store)?;
            let done = page.next.is_none();
            Ok::<_, GraphError>(Some((page.tags, (snapshot, page.next, remaining, done))))
        },
    );
    pages
        .try_fold(client, |client, tags| async move {
            stream::iter(tags)
                .map(Ok::<_, GraphError>)
                .try_fold(client, |client, tag| async move {
                    client.tag(key, &tag).await?;
                    Ok(client)
                })
                .await
        })
        .await?;
    Ok(())
}
