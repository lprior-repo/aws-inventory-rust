use super::TestError;
use aws_wire::transport::{Response, TransportError};
use std::{pin::Pin, sync::mpsc, time::Duration};
use tokio::time::Instant;

type SendResult = Result<Response, TransportError>;
// Tokio rounds the deadline wheel and the parked driver's wakeup to milliseconds.
const SCHEDULER_GRANULE: Duration = Duration::from_millis(2);

pub struct Clock {
    _hold: mpsc::SyncSender<()>,
    start: Instant,
}

impl Clock {
    pub fn pause() -> Self {
        let (hold, held) = mpsc::sync_channel::<()>(1);
        // A live blocking task inhibits automatic clock advance during loopback I/O.
        // Dropping the sender also releases the task on assertion failure/unwinding.
        let _task = tokio::task::spawn_blocking(move || {
            let _received = held.recv();
        });
        tokio::time::pause();
        Self {
            _hold: hold,
            start: Instant::now(),
        }
    }

    pub async fn expire<F>(
        &self,
        mut send: Pin<Box<F>>,
        deadline: Duration,
    ) -> Result<SendResult, TestError>
    where
        F: Future<Output = SendResult>,
    {
        let before = deadline.saturating_sub(Duration::from_millis(1));
        tokio::time::advance(before.saturating_sub(self.start.elapsed())).await;
        crate::deadlines::poll_pending(&mut send).await?;
        tokio::time::advance(SCHEDULER_GRANULE).await;
        let result = super::watchdog::watch(send).await?;
        let elapsed = self.start.elapsed();
        assert!(elapsed >= deadline);
        assert!(elapsed < deadline.saturating_add(SCHEDULER_GRANULE));
        Ok(result)
    }
}
