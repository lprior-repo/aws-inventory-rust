use hyper_util::client::legacy::connect::dns::Name;
use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    pin::Pin,
    sync::{Arc, LazyLock},
    task::{Context, Poll},
};
use tokio::sync::Semaphore;
use tower_service::Service;

type Addresses = std::vec::IntoIter<SocketAddr>;
type Resolving = Pin<Box<dyn Future<Output = io::Result<Addresses>> + Send>>;
const MAX_CONCURRENT_LOOKUPS: usize = 32;
static LOOKUPS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(MAX_CONCURRENT_LOOKUPS)));

#[derive(Clone)]
pub(super) struct Resolver {
    slots: Arc<Semaphore>,
}

impl Resolver {
    pub(super) fn new() -> Self {
        Self {
            slots: Arc::clone(&LOOKUPS),
        }
    }
}

impl Service<Name> for Resolver {
    type Response = Addresses;
    type Error = io::Error;
    type Future = Resolving;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: Name) -> Self::Future {
        Box::pin(resolve(name, Arc::clone(&self.slots), lookup))
    }
}

async fn resolve<F>(name: Name, slots: Arc<Semaphore>, lookup: F) -> io::Result<Addresses>
where
    F: FnOnce(Name) -> io::Result<Addresses> + Send + 'static,
{
    let permit = slots.acquire_owned().await.map_err(io::Error::other)?;

    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        lookup(name)
    })
    .await
    .map_err(io::Error::other)?
}

fn lookup(name: Name) -> io::Result<Addresses> {
    (name.as_str(), 0).to_socket_addrs()
}

#[cfg(test)]
#[path = "resolver_tests.rs"]
mod tests;
