use super::*;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

fn fake_queue(leases: impl IntoIterator<Item = u32>) -> Rc<RefCell<VecDeque<u32>>> {
    Rc::new(RefCell::new(VecDeque::from_iter(leases)))
}

fn claim_from_queue(queue: &Rc<RefCell<VecDeque<u32>>>, limit: u16) -> Vec<u32> {
    let mut q = queue.borrow_mut();
    let take = std::cmp::min(usize::from(limit), q.len());
    q.drain(0..take).collect()
}

#[tokio::test]
async fn in_flight_stays_at_the_configured_number_until_the_queue_empties() -> Result<(), SweepError>
{
    let queue = fake_queue(0..10);
    let running = Cell::new(0);
    let max_running = Cell::new(0);
    let pages_ran = Cell::new(0);
    let claim_limits = Rc::new(RefCell::new(Vec::new()));
    let claim_limits_for_claim = Rc::clone(&claim_limits);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            3,
            || true,
            {
                let claim_limits = Rc::clone(&claim_limits_for_claim);
                move |limit: u16| {
                    claim_limits.borrow_mut().push(limit);
                    let queue = Rc::clone(&queue);
                    let leases = claim_from_queue(&queue, limit);
                    async move { Ok(leases) }
                }
            },
            {
                let running = &running;
                let max_running = &max_running;
                let pages_ran = &pages_ran;
                move |lease: u32| {
                    let running = running;
                    let max_running = max_running;
                    let pages_ran = pages_ran;
                    async move {
                        running.set(running.get() + 1);
                        max_running.set(max_running.get().max(running.get()));
                        tokio::task::yield_now().await;
                        tokio::task::yield_now().await;
                        running.set(running.get() - 1);
                        pages_ran.set(pages_ran.get() + 1);
                        Ok(SweepSummary {
                            resource_writes: u64::from(lease),
                            ..SweepSummary::default()
                        })
                    }
                }
            },
            |remaining| {
                let _ = remaining;
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    run_result?;
    assert_eq!(pages_ran.get(), 10);
    assert_eq!(max_running.get(), 3);
    let limits = claim_limits.borrow();
    assert_eq!(limits.first().copied(), Some(3));
    for &limit in limits.iter().skip(1) {
        assert!(limit <= 3, "claim limit {} exceeds concurrency", limit);
    }
    Ok(())
}

#[tokio::test]
async fn one_stalled_page_does_not_stop_the_others() -> Result<(), SweepError> {
    let queue = fake_queue(0..10);
    let pages_ran = Cell::new(0);
    let notify = Rc::new(tokio::sync::Notify::new());
    let finished_others = Cell::new(0);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            3,
            || true,
            {
                move |limit: u16| {
                    let queue = Rc::clone(&queue);
                    let leases = claim_from_queue(&queue, limit);
                    async move { Ok(leases) }
                }
            },
            {
                let notify = Rc::clone(&notify);
                let finished_others = &finished_others;
                let pages_ran = &pages_ran;
                move |lease: u32| {
                    let notify = Rc::clone(&notify);
                    let finished_others = finished_others;
                    let pages_ran = pages_ran;
                    async move {
                        if lease == 0 {
                            loop {
                                notify.notified().await;
                                if finished_others.get() >= 9 {
                                    break;
                                }
                            }
                        } else {
                            tokio::task::yield_now().await;
                        }
                        pages_ran.set(pages_ran.get() + 1);
                        if lease != 0 {
                            finished_others.set(finished_others.get() + 1);
                            notify.notify_one();
                        }
                        Ok(SweepSummary {
                            resource_writes: u64::from(lease),
                            ..SweepSummary::default()
                        })
                    }
                }
            },
            |remaining| {
                let _ = remaining;
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    run_result?;
    assert_eq!(pages_ran.get(), 10);
    Ok(())
}

#[tokio::test]
async fn child_work_added_by_a_late_page_is_picked_up() -> Result<(), SweepError> {
    let queue = fake_queue([1]);
    let queue_claim = Rc::clone(&queue);
    let queue_process = Rc::clone(&queue);
    let order = Rc::new(RefCell::new(Vec::new()));
    let order_process = Rc::clone(&order);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            2,
            || true,
            {
                move |limit: u16| {
                    let leases = claim_from_queue(&queue_claim, limit);
                    async move { Ok(leases) }
                }
            },
            {
                move |lease: u32| {
                    let queue = Rc::clone(&queue_process);
                    let order = Rc::clone(&order_process);
                    async move {
                        if lease == 1 {
                            tokio::task::yield_now().await;
                            tokio::task::yield_now().await;
                            tokio::task::yield_now().await;
                            queue.borrow_mut().push_back(2);
                        }
                        order.borrow_mut().push(lease);
                        Ok(SweepSummary::default())
                    }
                }
            },
            |remaining| {
                let _ = remaining;
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    run_result?;
    assert_eq!(order.borrow().as_slice(), &[1, 2]);
    Ok(())
}

#[tokio::test]
async fn cancellation_stops_new_claims() -> Result<(), SweepError> {
    let queue = fake_queue(0..10);
    let queue_claim = Rc::clone(&queue);
    let pages_ran = Cell::new(0);
    let cancelled = Cell::new(false);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            1,
            {
                let cancelled = &cancelled;
                move || !cancelled.get()
            },
            {
                move |limit: u16| {
                    let leases = claim_from_queue(&queue_claim, limit);
                    async move { Ok(leases) }
                }
            },
            {
                let pages_ran = &pages_ran;
                move |_lease: u32| {
                    let pages_ran = pages_ran;
                    async move {
                        pages_ran.set(pages_ran.get() + 1);
                        Ok(SweepSummary::default())
                    }
                }
            },
            {
                let cancelled = &cancelled;
                move |remaining| {
                    if remaining == 0 {
                        cancelled.set(true);
                    }
                }
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    run_result?;
    assert_eq!(pages_ran.get(), 1);
    assert_eq!(queue.borrow().len(), 9);
    Ok(())
}

#[tokio::test]
async fn a_page_error_is_returned_after_the_in_flight_set_drains() -> Result<(), SweepError> {
    let queue = fake_queue(0..10);
    let queue_claim = Rc::clone(&queue);
    let pages_ran = Cell::new(0);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            3,
            || true,
            {
                move |limit: u16| {
                    let leases = claim_from_queue(&queue_claim, limit);
                    async move { Ok(leases) }
                }
            },
            {
                let pages_ran = &pages_ran;
                move |lease: u32| {
                    let pages_ran = pages_ran;
                    async move {
                        if lease == 0 {
                            pages_ran.set(pages_ran.get() + 1);
                            return Err(SweepError::Operation("page failed"));
                        }
                        tokio::task::yield_now().await;
                        tokio::task::yield_now().await;
                        pages_ran.set(pages_ran.get() + 1);
                        if lease == 2 {
                            return Err(SweepError::Operation("later failure"));
                        }
                        Ok(SweepSummary::default())
                    }
                }
            },
            |remaining| {
                let _ = remaining;
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    assert!(matches!(
        run_result,
        Err(SweepError::Operation("page failed"))
    ));
    assert_eq!(pages_ran.get(), 3);
    assert_eq!(queue.borrow().len(), 7);
    Ok(())
}

#[tokio::test]
async fn a_claim_error_is_returned_after_the_in_flight_set_drains() -> Result<(), SweepError> {
    let claim_count = Cell::new(0);
    let pages_ran = Cell::new(0);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            2,
            || true,
            {
                let claim_count = &claim_count;
                move |_limit: u16| {
                    claim_count.set(claim_count.get() + 1);
                    let result = if claim_count.get() == 1 {
                        Ok(vec![0u32, 1])
                    } else {
                        Err(SweepError::Operation("claim failed"))
                    };
                    async move { result }
                }
            },
            {
                let pages_ran = &pages_ran;
                move |lease: u32| {
                    let pages_ran = pages_ran;
                    async move {
                        tokio::task::yield_now().await;
                        pages_ran.set(pages_ran.get() + 1);
                        Ok(SweepSummary {
                            resource_writes: u64::from(lease),
                            ..SweepSummary::default()
                        })
                    }
                }
            },
            |remaining| {
                let _ = remaining;
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    assert!(matches!(
        run_result,
        Err(SweepError::Operation("claim failed"))
    ));
    assert_eq!(pages_ran.get(), 2);
    assert_eq!(claim_count.get(), 2);
    Ok(())
}

#[tokio::test]
async fn one_claim_never_asks_for_more_than_256() -> Result<(), SweepError> {
    let claim_limits = Rc::new(RefCell::new(Vec::new()));
    let claim_limits_for_claim = Rc::clone(&claim_limits);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            1_000,
            || true,
            {
                let claim_limits = Rc::clone(&claim_limits_for_claim);
                move |limit: u16| {
                    claim_limits.borrow_mut().push(limit);
                    async move { Ok(Vec::<u32>::new()) }
                }
            },
            {
                move |lease: u32| {
                    let _ = lease;
                    async move { Ok(SweepSummary::default()) }
                }
            },
            |remaining| {
                let _ = remaining;
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    run_result?;
    assert_eq!(*claim_limits.borrow(), vec![256]);
    Ok(())
}

#[tokio::test]
async fn writes_are_summed_across_pages() -> Result<(), SweepError> {
    let queue = fake_queue([5, 7, 11]);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            2,
            || true,
            {
                move |limit: u16| {
                    let queue = Rc::clone(&queue);
                    let leases = claim_from_queue(&queue, limit);
                    async move { Ok(leases) }
                }
            },
            {
                move |lease: u32| async move {
                    Ok(SweepSummary {
                        rows: 99,
                        scopes: 99,
                        resource_writes: u64::from(lease),
                        ..SweepSummary::default()
                    })
                }
            },
            |remaining| {
                let _ = remaining;
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    // Only writes accumulate; durable totals are applied after the drain.
    assert_eq!(
        run_result?,
        SweepSummary {
            resource_writes: 23,
            ..SweepSummary::default()
        }
    );
    Ok(())
}

#[tokio::test]
async fn zero_concurrency_claims_nothing() -> Result<(), SweepError> {
    let claim_count = Cell::new(0);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            0,
            || true,
            {
                let claim_count = &claim_count;
                move |limit: u16| {
                    let _ = limit;
                    claim_count.set(claim_count.get() + 1);
                    async move { Ok(Vec::<u32>::new()) }
                }
            },
            {
                move |lease: u32| {
                    let _ = lease;
                    async move { Ok(SweepSummary::default()) }
                }
            },
            |remaining| {
                let _ = remaining;
            },
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    run_result?;
    assert_eq!(claim_count.get(), 0);
    Ok(())
}

#[tokio::test]
async fn a_write_counter_overflow_wins_and_later_pages_still_finish() -> Result<(), SweepError> {
    let queue = fake_queue(0..3);
    let pages_ran = Cell::new(0_u32);

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        run(
            3,
            || true,
            move |limit: u16| {
                let leases = claim_from_queue(&queue, limit);
                async move { Ok(leases) }
            },
            |lease: u32| {
                let pages_ran = &pages_ran;
                async move {
                    if lease == 2 {
                        tokio::task::yield_now().await;
                        tokio::task::yield_now().await;
                    }
                    pages_ran.set(pages_ran.get().saturating_add(1));
                    match lease {
                        0 => Ok(SweepSummary {
                            resource_writes: u64::MAX,
                            ..SweepSummary::default()
                        }),
                        1 => Ok(SweepSummary {
                            resource_writes: 1,
                            ..SweepSummary::default()
                        }),
                        _ => Err(SweepError::Operation("later failure")),
                    }
                }
            },
            |_| {},
        ),
    )
    .await;

    let run_result = result.map_err(|_| SweepError::Operation("run timed out"))?;
    assert!(matches!(
        run_result,
        Err(SweepError::Operation("resource counter overflow"))
    ));
    assert_eq!(pages_ran.get(), 3);
    Ok(())
}
