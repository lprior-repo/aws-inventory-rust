use super::{GraphClient, GraphError, ProjectionSnapshotRead, ResourceKey, tag_pages};
use futures_util::{StreamExt, TryStreamExt, stream};

pub(super) async fn project_tags(
    snapshot: &mut impl ProjectionSnapshotRead,
    client: &mut GraphClient,
    key: &ResourceKey,
    limit: u16,
) -> Result<(), GraphError> {
    let pages = tag_pages::pages(snapshot, key, limit);
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
