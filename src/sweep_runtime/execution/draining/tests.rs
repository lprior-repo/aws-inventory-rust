use super::*;
use std::cell::Cell;

#[tokio::test]
async fn batch_keeps_the_first_error_and_finishes_later_pages() {
    // Given an admitted page failing before two still-unpolled siblings.
    let completed = Cell::new(0);
    let pages = stream::iter(0..3)
        .map(|index| {
            let completed = &completed;
            async move {
                tokio::task::yield_now().await;
                completed.set(completed.get() + 1);
                match index {
                    0 => Err(SweepError::Api(
                        inventory_engine::limiter::ApiError::Cancelled,
                    )),
                    1 => Ok(SweepSummary {
                        resource_writes: 7,
                        ..SweepSummary::default()
                    }),
                    _ => Err(SweepError::Operation("later failure")),
                }
            }
        })
        .buffer_unordered(1);
    // When completion summaries are reduced without a materialized batch.
    let result = summarize_batch(pages, SweepSummary::default()).await;
    // Then all pages finish, and cancellation is not masked by later errors.
    assert_eq!(completed.get(), 3);
    assert!(matches!(
        result,
        Err(SweepError::Api(
            inventory_engine::limiter::ApiError::Cancelled
        ))
    ));
}

#[tokio::test]
async fn batch_drains_later_pages_when_the_write_counter_overflows() {
    // Given an overflowing write count followed by a later page failure.
    let completed = Cell::new(0);
    let pages = stream::iter([
        Ok(SweepSummary {
            resource_writes: 1,
            ..SweepSummary::default()
        }),
        Err(SweepError::Operation("later failure")),
    ])
    .inspect(|_| completed.set(completed.get() + 1));
    // When the first completion overflows the aggregate.
    let result = summarize_batch(
        pages,
        SweepSummary {
            resource_writes: u64::MAX,
            ..SweepSummary::default()
        },
    )
    .await;
    // Then the sibling is still drained and the earlier overflow wins.
    assert_eq!(completed.get(), 2);
    assert!(matches!(
        result,
        Err(SweepError::Operation("resource counter overflow"))
    ));
}

#[tokio::test]
async fn batch_adds_only_writes_and_preserves_the_existing_summary() -> Result<(), SweepError> {
    // Given summary fields whose durable totals are applied after draining.
    let initial = SweepSummary {
        rows: 11,
        scopes: 3,
        resource_writes: 5,
        ..SweepSummary::default()
    };
    let pages = stream::iter([2, 7]).map(|resource_writes| {
        Ok(SweepSummary {
            rows: 99,
            scopes: 99,
            resource_writes,
            ..SweepSummary::default()
        })
    });
    // When successful page results are folded into the running summary.
    let result = summarize_batch(pages, initial).await?;
    // Then only writes accumulate; all other fields retain their old values.
    assert_eq!(
        result,
        SweepSummary {
            resource_writes: 14,
            ..initial
        }
    );
    Ok(())
}
