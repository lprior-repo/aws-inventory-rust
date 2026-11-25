use super::{EntryError, GraphBackend, ProjectionSnapshotRead, ResourceKey, tag_pages};
use futures_util::{StreamExt, TryStreamExt, stream};
use inventory_graph::TagRow;
use sha2::{Digest, Sha256};

pub(super) async fn project_tags<G: GraphBackend>(
    snapshot: &mut impl ProjectionSnapshotRead,
    client: &G,
    key: &ResourceKey,
    limit: u16,
) -> Result<(), EntryError> {
    let pages = tag_pages::pages(snapshot, key, limit);
    pages
        .try_fold(client, |client, tags| async move {
            stream::iter(tags)
                .map(Ok::<_, EntryError>)
                .try_fold(client, |client, tag| async move {
                    let identity = tag_identity(key, &tag)?;
                    client
                        .upsert_tag(
                            key,
                            &TagRow {
                                identity,
                                key: tag.key,
                                value: tag.value,
                            },
                        )
                        .await?;
                    Ok(client)
                })
                .await
        })
        .await?;
    Ok(())
}

fn tag_identity(
    key: &ResourceKey,
    tag: &crate::inventory::ProjectionTag,
) -> Result<[u8; 32], EntryError> {
    let mut digest = Sha256::new();
    [
        key.account.as_str(),
        key.region.as_str(),
        &tag.key,
        &tag.value,
    ]
    .into_iter()
    .try_for_each(|value| {
        let length = u64::try_from(value.len()).map_err(|_| super::ProjectionError::Query)?;
        digest.update(length.to_be_bytes());
        digest.update(value.as_bytes());
        Ok::<_, EntryError>(())
    })?;
    Ok(digest.finalize().into())
}

#[cfg(test)]
#[path = "tags/tests.rs"]
mod tests;
