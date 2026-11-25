use super::{SweepError, SweepSummary, run};
use std::{cell::Cell, time::Duration};
use tokio::sync::{Mutex, Notify};

#[tokio::test(start_paused = true)]
async fn given_an_in_flight_page_owns_the_writer_when_a_claim_waits_then_both_complete()
-> Result<(), SweepError> {
    let writer = &Mutex::new(());
    let page_has_writer = &Notify::new();
    let claim_waiting = &Notify::new();
    let next_claim = &Cell::new(0_u8);

    let summary = tokio::time::timeout(
        Duration::from_secs(1),
        run(
            2,
            || true,
            |_| {
                let call = next_claim.get();
                next_claim.set(call.saturating_add(1));
                async move {
                    match call {
                        0 => Ok(vec![0, 1]),
                        1 => {
                            claim_waiting.notify_one();
                            let _turn = writer.lock().await;
                            Ok(vec![2])
                        }
                        _ => Ok(Vec::new()),
                    }
                }
            },
            |lease| async move {
                match lease {
                    0 => {
                        let _turn = writer.lock().await;
                        page_has_writer.notify_one();
                        claim_waiting.notified().await;
                    }
                    1 => page_has_writer.notified().await,
                    _ => {}
                }
                Ok(SweepSummary {
                    resource_writes: 1,
                    ..SweepSummary::default()
                })
            },
            |_| {},
        ),
    )
    .await
    .map_err(|_| SweepError::Operation("claim and page stalled on the same writer"))??;
    assert_eq!(summary.resource_writes, 3);
    Ok(())
}
