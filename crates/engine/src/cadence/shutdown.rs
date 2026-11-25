use tokio_util::sync::CancellationToken;

/// Daemon-scope cancellation: one signal stops profile admission and cancels active work.
#[derive(Clone, Debug, Default)]
pub struct Shutdown {
    token: CancellationToken,
}

impl Shutdown {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request(&self) {
        self.token.cancel();
    }

    #[must_use]
    pub fn is_requested(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Token handed to in-flight collection so the same signal drains it.
    #[must_use]
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    pub async fn requested(&self) {
        self.token.cancelled().await;
    }
}
