use hickory_resolver::{Hosts, TokioResolver, lookup_ip::LookupIpIntoIter};
use hyper_util::client::legacy::connect::dns::Name;
use std::{
    io,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Semaphore;
use tower_service::Service;

type Addresses = std::iter::Map<LookupIpIntoIter, fn(IpAddr) -> SocketAddr>;
type Resolving = Pin<Box<dyn Future<Output = io::Result<Addresses>> + Send>>;
const MAX_CONCURRENT_LOOKUPS: usize = 32;

#[derive(Clone)]
pub(super) struct Resolver {
    inner: TokioResolver,
    slots: Arc<Semaphore>,
}

impl Resolver {
    pub(super) fn new() -> io::Result<Self> {
        let hosts = Hosts::from_system()?;
        let mut inner = TokioResolver::builder_tokio()
            .map_err(io::Error::other)?
            .build()
            .map_err(io::Error::other)?;
        inner.set_hosts(Arc::new(hosts));
        Ok(Self {
            inner,
            slots: Arc::new(Semaphore::new(MAX_CONCURRENT_LOOKUPS)),
        })
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
        Box::pin(resolve(name, self.inner.clone(), Arc::clone(&self.slots)))
    }
}

async fn resolve(
    name: Name,
    resolver: TokioResolver,
    slots: Arc<Semaphore>,
) -> io::Result<Addresses> {
    let _permit = slots.acquire().await.map_err(io::Error::other)?;
    let addresses = resolver
        .lookup_ip(name.as_str())
        .await
        .map_err(io::Error::other)?;
    if addresses.iter().next().is_none() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "DNS lookup returned no addresses",
        ));
    }
    let address: fn(IpAddr) -> SocketAddr = socket_address;
    Ok(addresses.into_iter().map(address))
}

fn socket_address(address: IpAddr) -> SocketAddr {
    SocketAddr::new(address, 0)
}

#[cfg(test)]
#[path = "resolver_tests.rs"]
mod tests;
