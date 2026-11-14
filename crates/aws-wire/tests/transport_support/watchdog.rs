use super::TestError;
use std::{sync::mpsc, time::Duration};

pub async fn watch<F: Future>(future: F) -> Result<F::Output, TestError> {
    let (stop, stopped) = mpsc::sync_channel(1);
    let mut alarm =
        tokio::task::spawn_blocking(move || stopped.recv_timeout(Duration::from_secs(10)));
    tokio::select! {
        output = future => {
            stop.send(())?;
            alarm.await??;
            Ok(output)
        }
        expired = &mut alarm => {
            expired??;
            Err("watchdog unexpectedly signalled".into())
        }
    }
}
