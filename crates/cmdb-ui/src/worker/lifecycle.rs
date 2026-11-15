use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;

pub(super) struct Shutdown {
    stopped: Arc<AtomicBool>,
    sender: watch::Sender<()>,
}

pub(super) struct Cancellation {
    stopped: Arc<AtomicBool>,
    receiver: watch::Receiver<()>,
}

pub(super) fn shutdown_channel() -> (Shutdown, Cancellation) {
    let stopped = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = watch::channel(());
    let cancellation = Cancellation {
        stopped: Arc::clone(&stopped),
        receiver,
    };
    (Shutdown { stopped, sender }, cancellation)
}

impl Shutdown {
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
        self.sender.send_replace(());
    }
}

impl Cancellation {
    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }

    pub async fn run<T>(&mut self, future: impl Future<Output = T>) -> Option<T> {
        if self.is_stopped() {
            return None;
        }
        tokio::select! {
            biased;
            changed = self.receiver.changed() => {
                if changed.is_err() {
                    tracing::debug!("CMDB worker shutdown owner disconnected");
                }
                None
            }
            output = future => Some(output),
        }
    }
}
